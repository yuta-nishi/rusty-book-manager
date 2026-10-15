import adapter from '@sveltejs/adapter-node';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite-plus';

// Vitest runs the app under Node, but the code targets the browser, so the
// `browser` entry points of every package must be resolved while testing.
const isVitest = Boolean(process.env.VITEST);

export default defineConfig({
	plugins: [
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true,
			},

			// The app is self-hosted, so it is built for a Node server
			// (`vp build && node build`).
			adapter: adapter(),
		}),
	],

	resolve: isVitest ? { conditions: ['browser'] } : {},

	lint: {
		ignorePatterns: ['.svelte-kit/**', 'build/**'],
		options: {
			typeAware: true,
			typeCheck: true,
		},
	},

	fmt: {
		useTabs: true,
		singleQuote: true,
		// `.svelte` files are formatted through a bundled Prettier plugin.
		svelte: true,
		ignorePatterns: ['.svelte-kit/**', 'build/**', 'pnpm-lock.yaml'],
	},

	test: {
		include: ['src/**/*.{test,spec}.ts'],
		environment: 'jsdom',
		setupFiles: ['./src/vitest-setup.ts'],
		restoreMocks: true,
		unstubGlobals: true,
	},
});
