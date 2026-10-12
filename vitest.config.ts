import { defineConfig } from 'vitest/config';

// Plain-TS module tests only (stores, utils). Component tests would need
// @testing-library/svelte — add when a component needs regression coverage.
export default defineConfig({
  test: {
    include: ['src/**/*.{test,spec}.ts'],
    environment: 'node',
  },
});
