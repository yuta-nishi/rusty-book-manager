import { redirect } from '@sveltejs/kit';
import { api } from '#lib/api/index.js';
import { session } from '#lib/session.svelte.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ parent }) => {
	await parent();

	// User management is admin-only; everyone else sees the book list.
	if (!session.isAdmin) throw redirect(307, '/books');

	return { users: await api.listUsers() };
};
