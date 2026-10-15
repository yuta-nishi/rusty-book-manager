/**
 * Builds a `Response` for stubbing `fetch` in tests.
 *
 * The backend answers with either a JSON body or no body at all (errors carry
 * only a status code), so both cases need to be reproducible.
 */
export function stubResponse(body?: unknown, status = 200): Response {
	if (body === undefined) {
		return new Response(null, { status });
	}
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'content-type': 'application/json' },
	});
}

/** A user as returned by `GET /api/v1/users/me`. */
export const testUser = {
	id: '2bd5c35c7a8943cf9863cc65a6f23110',
	name: 'Eleazar Fig',
	email: 'eleazar.fig@example.com',
	role: 'Admin' as const,
};
