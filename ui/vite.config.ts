import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// Chemins relatifs : le coeur Rust sert `dist/` depuis un scheme local, pas depuis une racine http.
export default defineConfig({
  base: './',
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  plugins: [react(), tailwindcss()],
  build: {
    // Deux pages : l'interface, et le menu contextuel servi au-dessus du contenu.
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL('./index.html', import.meta.url)),
        menu: fileURLToPath(new URL('./menu.html', import.meta.url)),
        pages: fileURLToPath(new URL('./pages.html', import.meta.url)),
      },
    },
    outDir: 'dist',
    emptyOutDir: true,
    target: 'chrome120',
    assetsInlineLimit: 0,
  },
  server: { port: 4310, strictPort: true },
})
