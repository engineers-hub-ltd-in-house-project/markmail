import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: "build",
      assets: "build",
      fallback: "index.html",
      precompress: false,
      strict: false,
    }),
    prerender: {
      entries: [
        "/",
        "/auth/login",
        "/auth/register",
        "/campaigns",
        "/subscribers",
        "/templates",
        "/lp",
      ],
    },
  },
};

export default config;
