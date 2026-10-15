import { PUBLIC_API_BASE_URL } from '$app/env/public';

/**
 * Error thrown for any non-2xx API response.
 *
 * The backend responds with a bare status code and an empty body, so the
 * message is derived from the status code (callers may override it for
 * statuses whose meaning depends on the endpoint, e.g. 403 on login).
 */
export class ApiError extends Error {
	readonly status: number;

	constructor(status: number, message: string) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
	}
}

const STATUS_MESSAGES: Record<number, string> = {
	400: 'リクエストが不正です。入力内容を確認してください。',
	401: '認証情報がありません。ログインしてください。',
	403: 'この操作を行う権限がありません。',
	404: '対象のデータが見つかりません。',
	422: '現在の状態では処理できません。',
	500: 'サーバーでエラーが発生しました。時間をおいて再度お試しください。',
};

export function messageForStatus(status: number): string {
	return STATUS_MESSAGES[status] ?? `予期しないエラーが発生しました (HTTP ${status})`;
}

// The access token is held in memory; `session` is the only writer, and it
// restores the token from localStorage on startup.
let accessToken: string | null = null;

export function setAccessToken(token: string | null): void {
	accessToken = token;
}

export interface RequestOptions {
	method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
	body?: unknown;
	query?: Record<string, string | number | boolean | undefined>;
	/** Overrides the message used for non-2xx responses. */
	errorMessage?: (status: number) => string;
}

export async function apiFetch<T>(path: string, options: RequestOptions = {}): Promise<T> {
	const url = new URL(`${PUBLIC_API_BASE_URL}${path}`);
	for (const [key, value] of Object.entries(options.query ?? {})) {
		if (value !== undefined) url.searchParams.set(key, String(value));
	}

	const headers = new Headers({ accept: 'application/json' });
	if (options.body !== undefined) headers.set('content-type', 'application/json');
	if (accessToken !== null) headers.set('authorization', `Bearer ${accessToken}`);

	const response = await fetch(url, {
		method: options.method ?? 'GET',
		headers,
		body: options.body === undefined ? undefined : JSON.stringify(options.body),
	});

	if (!response.ok) {
		const message = options.errorMessage?.(response.status) ?? messageForStatus(response.status);
		throw new ApiError(response.status, message);
	}

	// Successful responses are either 204 (no content) or JSON.
	if (response.status === 204) return undefined as T;
	const text = await response.text();
	return (text === '' ? undefined : JSON.parse(text)) as T;
}
