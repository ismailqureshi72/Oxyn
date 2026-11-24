// The site's canonical origin, in one place: Astro's `site` (canonical, Open Graph, sitemap), the
// registry index (`homepage`, item `url`s, written by scripts/gen.mjs) and the docs links on item
// pages. The Worker redirects www.oxynui.com and the old ui.oxyn.workers.dev here (worker/index.ts);
// keep `SITE_URL` in wrangler.jsonc and the Rust defaults (oxyn-cli, oxyn-mcp, oxyn-codegen) in step.
export const SITE = 'https://oxynui.com';

/** Link-preview card version: every og:image / twitter:image URL carries ?v=OG_V, so apps that
 *  cached an older card fetch the new one. Bump it whenever public/og.png or public/og/* change. */
export const OG_V = 3;
/** The absolute, versioned URL of a link-preview card under public/ ('/og.png', '/og/docs.png'). */
export const ogUrl = (p) => `${SITE}${p}?v=${OG_V}`;
