import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	build: {
		// Cytoscape + cose-bilkent together are ~500 kB minified; raise the
		// threshold to suppress the noise warning. The canvas chunk is only
		// loaded when the graph page is mounted, so startup is unaffected.
		chunkSizeWarningLimit: 700,
	},
});
