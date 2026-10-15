import { defineEnvVars } from '@sveltejs/kit/env';

/**
 * Resolves the base URL of the Rust backend API.
 *
 * Exported so tests can resolve it exactly like SvelteKit does.
 */
export const publicApiBaseUrl = (value: string | undefined): string =>
	(value ?? 'http://localhost:8080').replace(/\/+$/, '');

/**
 * Environment variables used by the frontend.
 *
 * Values are read from `.env` / `.env.local` in development and from the
 * process environment in production. See `.env.example` for the defaults.
 */
export const variables = defineEnvVars({
	/** Base URL of the Rust backend API. */
	PUBLIC_API_BASE_URL: {
		public: true,
		description: 'Base URL of the rusty-book-manager API (e.g. http://localhost:8080)',
		schema: publicApiBaseUrl,
	},
});
