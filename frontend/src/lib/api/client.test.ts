import { afterEach, beforeEach, describe, expect, it, vi } from 'vite-plus/test';
import { ApiError, apiFetch, messageForStatus, setAccessToken } from '#lib/api/client.js';
import { stubResponse } from '#lib/test-utils.js';

const fetchMock = vi.fn();

beforeEach(() => {
	vi.stubGlobal('fetch', fetchMock);
	setAccessToken(null);
});

afterEach(() => {
	fetchMock.mockReset();
});

describe('apiFetch', () => {
	it('builds the URL from the configured base and skips undefined query values', async () => {
		fetchMock.mockResolvedValue(stubResponse({ items: [] }));

		await apiFetch('/api/v1/books', { query: { limit: 20, offset: undefined, isbn: 'a b' } });

		const [url, init] = fetchMock.mock.calls[0];
		expect(String(url)).toBe('http://localhost:8080/api/v1/books?limit=20&isbn=a+b');
		expect(init.method).toBe('GET');
		expect(init.body).toBeUndefined();
	});

	it('serializes the body and sends the bearer token once one is set', async () => {
		fetchMock.mockResolvedValue(stubResponse(undefined, 201));
		setAccessToken('token-1');

		await apiFetch('/api/v1/books', { method: 'POST', body: { title: 'T' } });

		const [, init] = fetchMock.mock.calls[0];
		expect(init.method).toBe('POST');
		expect(init.body).toBe('{"title":"T"}');
		expect(init.headers.get('content-type')).toBe('application/json');
		expect(init.headers.get('authorization')).toBe('Bearer token-1');
	});

	it('omits the authorization header while logged out', async () => {
		fetchMock.mockResolvedValue(stubResponse({ items: [] }));

		await apiFetch('/api/v1/books');

		const [, init] = fetchMock.mock.calls[0];
		expect(init.headers.has('authorization')).toBe(false);
	});

	it('parses a JSON body', async () => {
		fetchMock.mockResolvedValue(stubResponse({ total: 1, items: [{ id: 'b' }] }));

		await expect(apiFetch('/api/v1/books')).resolves.toEqual({
			total: 1,
			items: [{ id: 'b' }],
		});
	});

	it('returns undefined for 204 and for an empty body', async () => {
		fetchMock.mockResolvedValueOnce(stubResponse(undefined, 204));
		fetchMock.mockResolvedValueOnce(stubResponse(undefined, 200));

		await expect(apiFetch('/auth/logout', { method: 'POST' })).resolves.toBeUndefined();
		await expect(apiFetch('/api/v1/books/1', { method: 'DELETE' })).resolves.toBeUndefined();
	});

	it('throws an ApiError carrying the status and a message derived from it', async () => {
		fetchMock.mockResolvedValue(stubResponse(undefined, 422));

		const error = await apiFetch('/x').catch((cause: unknown) => cause);

		expect(error).toBeInstanceOf(ApiError);
		expect((error as ApiError).status).toBe(422);
		expect((error as ApiError).message).toBe('現在の状態では処理できません。');
	});

	it('uses the caller-provided message for statuses whose meaning is endpoint specific', async () => {
		fetchMock.mockResolvedValue(stubResponse(undefined, 403));

		const error = await apiFetch('/x', {
			errorMessage: () => 'カスタムメッセージ',
		}).catch((cause: unknown) => cause);

		expect((error as ApiError).message).toBe('カスタムメッセージ');
	});
});

describe('messageForStatus', () => {
	it('maps the statuses the backend can return', () => {
		expect(messageForStatus(400)).toContain('不正');
		expect(messageForStatus(401)).toContain('ログイン');
		expect(messageForStatus(403)).toContain('権限');
		expect(messageForStatus(404)).toContain('見つかりません');
		expect(messageForStatus(422)).toContain('処理できません');
		expect(messageForStatus(500)).toContain('サーバー');
	});

	it('falls back to the status code for unmapped statuses', () => {
		expect(messageForStatus(418)).toBe('予期しないエラーが発生しました (HTTP 418)');
	});
});
