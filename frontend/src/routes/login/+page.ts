import { redirect } from '@sveltejs/kit';
import { session } from '#lib/session.svelte.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	await session.restore();
	if (session.authenticated) throw redirect(307, '/books');
	return {};
};
