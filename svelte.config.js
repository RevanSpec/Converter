import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  // TypeScript dans les balises <script lang="ts"> des composants
  preprocess: vitePreprocess(),
};
