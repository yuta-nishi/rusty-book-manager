// The app is a client-side SPA: the Rust backend enables CORS for the dev
// server and returns a bearer token from `/auth/login`, so all data fetching
// happens in the browser.
export const ssr = false;
