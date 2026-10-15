import { api } from '#lib/api/index.js';
import type { PageLoad } from './$types';

const DEFAULT_LIMIT = 20;

function positiveInt(value: string | null, fallback: number): number {
	if (value === null) return fallback;
	const parsed = Number(value);
	return Number.isInteger(parsed) && parsed >= 0 ? parsed : fallback;
}

export const load: PageLoad = async ({ url, parent }) => {
	await parent();

	const limit = positiveInt(url.searchParams.get('limit'), DEFAULT_LIMIT);
	const offset = positiveInt(url.searchParams.get('offset'), 0);

	return { books: await api.listBooks({ limit, offset }) };
};
