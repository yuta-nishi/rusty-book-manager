import { redirect } from '@sveltejs/kit';
import { session } from '#lib/session.svelte.js';
import type { LayoutLoad } from './$types';

// Every route below this layout requires an authenticated user. Restoring the
// session here (rather than per page) also guarantees that nested page loads
// which `await parent()` run with the bearer token already in place.
export const load: LayoutLoad = async ({ url }) => {
	await session.restore();
	if (!session.authenticated) {
		const redirectTo = `${url.pathname}${url.search}`;
		throw redirect(307, `/login?redirectTo=${encodeURIComponent(redirectTo)}`);
	}
};
