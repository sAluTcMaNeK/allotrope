import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  // Permet d'utiliser TypeScript dans les fichiers .svelte
  preprocess: vitePreprocess(),
};
