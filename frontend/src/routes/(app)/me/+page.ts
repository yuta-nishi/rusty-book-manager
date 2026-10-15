import { api } from '#lib/api/index.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ parent }) => {
	await parent();

	const [user, checkouts] = await Promise.all([api.me(), api.myCheckouts()]);
	return { user, checkouts };
};
