import staticAdapter from "@sveltejs/adapter-static"
import { vitePreprocess } from "@sveltejs/kit/vite"
import preprocess from "svelte-preprocess"

/** @type {import('@sveltejs/kit').Config} */
export default {
  // Consult https://kit.svelte.dev/docs/integrations#preprocessors
  // for more information about preprocessors
  preprocess: [
    // vitePreprocess(),
    preprocess(),
  ],

  kit: {
    //adapter: adapter(),
    adapter: staticAdapter(),
  },
}
