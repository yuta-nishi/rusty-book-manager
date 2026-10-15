import { api } from '#lib/api/index.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ parent }) => {
	await parent();
	return { checkouts: await api.listCheckouts() };
};
