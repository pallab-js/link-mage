import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({
			fallback: 'index.html'
		}),
		// SvelteKit hashes its inline bootstrap script and emits the CSP as a
		// <meta> tag in the built HTML, so `script-src 'unsafe-inline'` can go
		// away from tauri.conf.json entirely.
		csp: {
			mode: 'hash',
			directives: {
				'default-src': ["'self'"],
				'script-src': ["'self'"],
				// Inline `style="…"` attributes are used for dynamic colors.
				'style-src': ["'self'", "'unsafe-inline'"],
				'img-src': ["'self'", 'data:', 'blob:'],
				'font-src': ["'self'", 'data:'],
				'connect-src': ["'self'", 'http://localhost:*']
			}
		}
	}
};

export default config;
