import { api } from '#lib/api/index.js';
import { setAccessToken } from '#lib/api/client.js';
import type { UserResponse } from '#lib/api/types.js';

const TOKEN_STORAGE_KEY = 'rusty-book-manager.access-token';

// Guarded so the module also loads in a server (build) context.
const tokenStorage: Storage | null = typeof localStorage === 'undefined' ? null : localStorage;

/**
 * Authentication state for the SPA.
 *
 * The backend hands out a bearer token in the login response body (rather than
 * a cookie) and enables CORS for the dev server, so the token lives in
 * localStorage and is attached to requests from the browser.
 */
class Session {
	token = $state<string | null>(null);
	user = $state<UserResponse | null>(null);
	/** Set once the persisted token has been validated against the API. */
	ready = $state(false);

	get authenticated(): boolean {
		return this.token !== null && this.user !== null;
	}

	get isAdmin(): boolean {
		return this.user?.role === 'Admin';
	}

	/** Restores the persisted token and verifies it with `GET /users/me`. */
	async restore(): Promise<void> {
		if (this.ready) return;

		const token = tokenStorage?.getItem(TOKEN_STORAGE_KEY) ?? null;
		if (token === null) {
			this.ready = true;
			return;
		}

		setAccessToken(token);
		try {
			this.user = await api.me();
			this.token = token;
		} catch {
			// The token was revoked or has expired.
			this.clear();
		} finally {
			this.ready = true;
		}
	}

	async login(email: string, password: string): Promise<void> {
		const { accessToken } = await api.login({ email, password });
		setAccessToken(accessToken);
		this.token = accessToken;
		tokenStorage?.setItem(TOKEN_STORAGE_KEY, accessToken);
		this.user = await api.me();
	}

	/** Re-fetches the current user, e.g. after their own role changed. */
	async refresh(): Promise<void> {
		try {
			this.user = await api.me();
		} catch {
			this.clear();
		}
	}

	async logout(): Promise<void> {
		try {
			await api.logout();
		} catch {
			// Logging out locally is enough even if the token is already gone.
		}
		this.clear();
	}

	clear(): void {
		setAccessToken(null);
		this.token = null;
		this.user = null;
		tokenStorage?.removeItem(TOKEN_STORAGE_KEY);
	}
}

export const session = new Session();
