// The site's canonical origin, in one place: Astro's `site` (canonical, Open Graph, sitemap), the
// registry index (`homepage`, item `url`s, written by scripts/gen.mjs) and the docs links on item
// pages. The Worker redirects www.oxynui.com and the old ui.oxyn.workers.dev here (worker/index.ts);
// keep `SITE_URL` in wrangler.jsonc and the Rust defaults (oxyn-cli, oxyn-mcp, oxyn-codegen) in step.
export const SITE = 'https://oxynui.com';
