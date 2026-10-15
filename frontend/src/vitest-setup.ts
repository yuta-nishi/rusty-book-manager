import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/svelte';
import { afterEach } from 'vite-plus/test';
import { publicApiBaseUrl } from './env.js';

// Testing Library only registers its own cleanup when Vitest runs with
// `globals: true`, so it is registered here instead.
afterEach(cleanup);

// Vitest does not boot a SvelteKit server, so the module SvelteKit generates
// for `$app/env/public` (`.svelte-kit/generated/dev/env/public/client.js`) has
// no backing global. This mirrors what SvelteKit's own service-worker bootstrap
// does (`.svelte-kit/generated/dev/env/service-worker.js`).
globalThis.__sveltekit_dev = {
	env: {
		PUBLIC_API_BASE_URL: publicApiBaseUrl(process.env.PUBLIC_API_BASE_URL),
	},
};
