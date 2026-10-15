import { redirect } from '@sveltejs/kit';
import { api } from '#lib/api/index.js';
import { session } from '#lib/session.svelte.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, parent }) => {
	await parent();

	const book = await api.getBook(params.bookId);
	// Only the owner may edit (the API answers 404 for anyone else).
	if (book.owner.id !== session.user?.id) {
		throw redirect(307, `/books/${params.bookId}`);
	}

	return { book };
};
