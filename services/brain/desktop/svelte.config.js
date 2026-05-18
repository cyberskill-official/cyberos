import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/vite-plugin-svelte').SvelteConfig} */
export default {
  preprocess: vitePreprocess(),
  compilerOptions: {
    // Svelte 5 runes mode is opt-in per-file via `<script lang="ts">` + `$state`;
    // we leave the default ("undefined") so Svelte 5 auto-detects mode.
  },
};
