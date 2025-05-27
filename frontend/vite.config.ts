import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { TanStackRouterVite } from '@tanstack/router-vite-plugin'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), TanStackRouterVite()],
  appType: 'spa',
  build: {
    outDir: '../dist',
  },
  server: {
    proxy: {
      '/api': 'http://0.0.0.0:10000',
    },
  },
})
