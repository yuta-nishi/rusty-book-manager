import { api } from '#lib/api/index.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, parent }) => {
	await parent();

	const [book, history] = await Promise.all([
		api.getBook(params.bookId),
		api.bookCheckoutHistory(params.bookId),
	]);

	return { book, history };
};
