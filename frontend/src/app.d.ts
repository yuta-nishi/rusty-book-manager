// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}

	// Set by SvelteKit's dev server and service-worker bootstrap
	// (`.svelte-kit/generated/*/env/service-worker.js`). Unit tests run without
	// a SvelteKit server, so `src/vitest-setup.ts` provides it there.
	var __sveltekit_dev: { env: Record<string, string | undefined> } | undefined;
}

export {};
