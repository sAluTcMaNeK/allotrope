import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// @ts-expect-error process est une variable globale de Node.js
const host = process.env.TAURI_DEV_HOST;

// Configuration Vite adaptée à Tauri : https://v2.tauri.app/start/frontend/vite/
export default defineConfig(async () => ({
  plugins: [svelte()],

  // Ne pas effacer la console, pour garder visibles les erreurs Rust
  clearScreen: false,
  server: {
    // Tauri attend un port fixe ; on échoue si le port est déjà pris
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      // Vite n'a pas à surveiller le code Rust
      ignored: ["**/src-tauri/**"],
    },
  },
}));
