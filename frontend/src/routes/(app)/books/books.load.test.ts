import { afterEach, beforeEach, describe, expect, it, vi } from 'vite-plus/test';
import { stubResponse } from '#lib/test-utils.js';
import { load } from './+page.js';

const fetchMock = vi.fn();

beforeEach(() => {
	vi.stubGlobal('fetch', fetchMock);
	fetchMock.mockResolvedValue(stubResponse({ total: 0, limit: 20, offset: 0, items: [] }));
});

afterEach(() => {
	fetchMock.mockReset();
});

describe('book list load', () => {
	it('requests the default page when the URL carries no parameters', async () => {
		await load({
			url: new URL('http://localhost/books'),
			parent: async () => ({}),
		} as never);

		expect(String(fetchMock.mock.calls[0][0])).toBe(
			'http://localhost:8080/api/v1/books?limit=20&offset=0',
		);
	});

	it('honours the page requested by the URL', async () => {
		await load({
			url: new URL('http://localhost/books?limit=5&offset=10'),
			parent: async () => ({}),
		} as never);

		expect(String(fetchMock.mock.calls[0][0])).toBe(
			'http://localhost:8080/api/v1/books?limit=5&offset=10',
		);
	});

	it('falls back to the defaults for unparseable or negative values', async () => {
		await load({
			url: new URL('http://localhost/books?limit=abc&offset=-3'),
			parent: async () => ({}),
		} as never);

		expect(String(fetchMock.mock.calls[0][0])).toBe(
			'http://localhost:8080/api/v1/books?limit=20&offset=0',
		);
	});

	it('returns the parsed response', async () => {
		fetchMock.mockResolvedValue(
			stubResponse({
				total: 1,
				limit: 20,
				offset: 0,
				items: [{ id: 'b1', title: 'Rust' }],
			}),
		);

		const result = await load({
			url: new URL('http://localhost/books'),
			parent: async () => ({}),
		} as never);

		expect(result?.books.total).toBe(1);
		expect(result?.books.items[0].title).toBe('Rust');
	});
});
