import { defineConfig } from 'vite';

export default defineConfig({
  base: './',
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
  },
  optimizeDeps: {
    exclude: ['zots-wasm']
  },
  server: {
    port: 3000,
    open: true,
  }
});
