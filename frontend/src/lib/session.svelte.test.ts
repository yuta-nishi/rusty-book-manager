import { afterEach, beforeEach, describe, expect, it, vi } from 'vite-plus/test';
import { session } from '#lib/session.svelte.js';
import { stubResponse, testUser } from '#lib/test-utils.js';

const TOKEN_KEY = 'rusty-book-manager.access-token';
const fetchMock = vi.fn();

beforeEach(() => {
	localStorage.clear();
	session.clear();
	session.ready = false;
	vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => {
	fetchMock.mockReset();
});

describe('restore', () => {
	it('stays logged out without touching the API when no token is stored', async () => {
		await session.restore();

		expect(session.authenticated).toBe(false);
		expect(session.ready).toBe(true);
		expect(fetchMock).not.toHaveBeenCalled();
	});

	it('loads the user for a stored token and sends it as a bearer token', async () => {
		localStorage.setItem(TOKEN_KEY, 'stored-token');
		fetchMock.mockResolvedValue(stubResponse(testUser));

		await session.restore();

		expect(session.authenticated).toBe(true);
		expect(session.user?.name).toBe('Eleazar Fig');
		expect(session.isAdmin).toBe(true);
		const [url, init] = fetchMock.mock.calls[0];
		expect(String(url)).toBe('http://localhost:8080/api/v1/users/me');
		expect(init.headers.get('authorization')).toBe('Bearer stored-token');
	});

	it('forgets a revoked token', async () => {
		localStorage.setItem(TOKEN_KEY, 'revoked-token');
		fetchMock.mockResolvedValue(stubResponse(undefined, 401));

		await session.restore();

		expect(session.authenticated).toBe(false);
		expect(localStorage.getItem(TOKEN_KEY)).toBeNull();
		expect(session.ready).toBe(true);
	});

	it('only checks the API once', async () => {
		localStorage.setItem(TOKEN_KEY, 'stored-token');
		fetchMock.mockResolvedValue(stubResponse(testUser));

		await session.restore();
		await session.restore();

		expect(fetchMock).toHaveBeenCalledTimes(1);
	});
});

describe('login', () => {
	it('persists the token and resolves the current user', async () => {
		fetchMock.mockResolvedValueOnce(
			stubResponse({ userId: testUser.id, accessToken: 'fresh-token' }),
		);
		fetchMock.mockResolvedValueOnce(stubResponse(testUser));

		await session.login(testUser.email, 'password');

		expect(localStorage.getItem(TOKEN_KEY)).toBe('fresh-token');
		expect(session.authenticated).toBe(true);
		expect(session.user?.email).toBe(testUser.email);
	});
});

describe('logout', () => {
	it('clears the local session', async () => {
		localStorage.setItem(TOKEN_KEY, 'stored-token');
		fetchMock.mockResolvedValue(stubResponse(undefined, 204));

		await session.logout();

		expect(session.authenticated).toBe(false);
		expect(localStorage.getItem(TOKEN_KEY)).toBeNull();
	});

	it('still clears the local session when the API call fails', async () => {
		localStorage.setItem(TOKEN_KEY, 'stored-token');
		fetchMock.mockResolvedValue(stubResponse(undefined, 500));

		await session.logout();

		expect(session.authenticated).toBe(false);
		expect(localStorage.getItem(TOKEN_KEY)).toBeNull();
	});
});

describe('refresh', () => {
	it('picks up a changed role', async () => {
		localStorage.setItem(TOKEN_KEY, 'stored-token');
		fetchMock.mockResolvedValueOnce(stubResponse(testUser));
		await session.restore();
		expect(session.isAdmin).toBe(true);

		fetchMock.mockResolvedValueOnce(stubResponse({ ...testUser, role: 'User' }));
		await session.refresh();

		expect(session.isAdmin).toBe(false);
	});

	it('logs out when the token is no longer valid', async () => {
		localStorage.setItem(TOKEN_KEY, 'stored-token');
		fetchMock.mockResolvedValueOnce(stubResponse(testUser));
		await session.restore();

		fetchMock.mockResolvedValueOnce(stubResponse(undefined, 403));
		await session.refresh();

		expect(session.authenticated).toBe(false);
		expect(localStorage.getItem(TOKEN_KEY)).toBeNull();
	});
});
