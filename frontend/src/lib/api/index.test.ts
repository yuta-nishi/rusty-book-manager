import { afterEach, beforeEach, describe, expect, it, vi } from 'vite-plus/test';
import { api } from '#lib/api/index.js';
import { stubResponse, testUser } from '#lib/test-utils.js';

const fetchMock = vi.fn();

beforeEach(() => {
	vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => {
	fetchMock.mockReset();
});

// The backend mounts the auth routes at `/auth`, *not* under `/api/v1`, while
// everything else lives under `/api/v1`. Getting this wrong answers 404.
describe('auth endpoints', () => {
	it('posts credentials to /auth/login', async () => {
		fetchMock.mockResolvedValue(stubResponse({ userId: testUser.id, accessToken: 'token-1' }));

		await api.login({ email: testUser.email, password: 'password' });

		expect(String(fetchMock.mock.calls[0][0])).toBe('http://localhost:8080/auth/login');
	});

	it('posts to /auth/logout', async () => {
		fetchMock.mockResolvedValue(stubResponse(undefined, 204));

		await api.logout();

		expect(String(fetchMock.mock.calls[0][0])).toBe('http://localhost:8080/auth/logout');
	});
});
