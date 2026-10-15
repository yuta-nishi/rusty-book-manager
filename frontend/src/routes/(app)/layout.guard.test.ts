import { afterEach, beforeEach, describe, expect, it, vi } from 'vite-plus/test';
import { session } from '#lib/session.svelte.js';
import { stubResponse, testUser } from '#lib/test-utils.js';
import { load } from './+layout.js';

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

describe('authenticated layout guard', () => {
	it('redirects a logged-out visitor to the login page, preserving the target', async () => {
		const error = await Promise.resolve(
			load({ url: new URL('http://localhost/books?offset=20') } as never),
		).catch((cause: unknown) => cause);

		expect(error).toMatchObject({
			status: 307,
			location: '/login?redirectTo=%2Fbooks%3Foffset%3D20',
		});
	});

	it('lets an authenticated user through', async () => {
		localStorage.setItem(TOKEN_KEY, 'stored-token');
		fetchMock.mockResolvedValue(stubResponse(testUser));

		await expect(
			load({ url: new URL('http://localhost/books') } as never),
		).resolves.toBeUndefined();
		expect(session.user?.email).toBe(testUser.email);
	});
});
