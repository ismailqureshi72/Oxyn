// @ts-check
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { redirects } from './src/lib/taxonomy.mjs';
import { SITE } from './src/lib/site.mjs';

// Moved URLs (plan #29 regrouped 21 categories into 8 + Screens): the pages also ship as
// meta-refresh stubs, and the Worker answers them with a 301 from dist/_redirects.
const movedUrls = {
  name: 'oxyn-redirects',
  hooks: {
    'astro:build:done': ({ dir }) => {
      const gen = path.resolve(path.dirname(fileURLToPath(import.meta.url)), 'src/generated/demos.json');
      if (!fs.existsSync(gen)) return;
      const { demos, categories } = JSON.parse(fs.readFileSync(gen, 'utf8'));
      const renames = JSON.parse(fs.readFileSync(path.resolve(path.dirname(gen), '../lib/renames.json'), 'utf8')).renames;
      const lines = redirects(demos, categories, renames).map(([from, to]) => `${from} ${to} 301`);
      fs.writeFileSync(new URL('_redirects', dir), lines.join('\n') + '\n');
    },
  },
};

// sitemap.xml (the one robots.txt names): every indexable page in the build, with its images
// (og:image and same-site <img>) as image-sitemap entries. Pages marked noindex (sign-in, account,
// full-screen views) and moved-page stubs are left out. Starlight's own sitemap-index.xml stays.
const sitemap = {
  name: 'oxyn-sitemap',
  hooks: {
    'astro:build:done': ({ dir }) => {
      const root = fileURLToPath(dir);
      const pages = [];
      const walk = (d) => {
        for (const e of fs.readdirSync(d, { withFileTypes: true })) {
          const f = path.join(d, e.name);
          if (e.isDirectory()) walk(f);
          else if (e.name.endsWith('.html')) pages.push(f);
        }
      };
      walk(root);
      const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
      const rows = [];
      for (const f of pages) {
        const html = fs.readFileSync(f, 'utf8');
        if (/<meta[^>]+name="robots"[^>]+noindex/i.test(html) || /http-equiv="refresh"/i.test(html)) continue;
        let rel = path.relative(root, f).split(path.sep).join('/');
        if (rel === '404.html' || rel.endsWith('/404.html')) continue;
        rel = rel === 'index.html' ? '' : rel.endsWith('/index.html') ? rel.slice(0, -'index.html'.length) : rel;
        const loc = `${SITE}/${rel}`;
        const imgs = new Set();
        const og = /<meta[^>]+property="og:image"[^>]+content="([^"]+)"/i.exec(html);
        if (og) imgs.add(og[1].replace(/&amp;/g, '&'));
        for (const m of html.matchAll(/<img[^>]+src="(\/[^"/][^"]*\.(?:png|jpe?g|webp))"/gi)) imgs.add(SITE + m[1]);
        rows.push(`<url><loc>${esc(loc)}</loc>${[...imgs].map((i) => `<image:image><image:loc>${esc(i)}</image:loc></image:image>`).join('')}</url>`);
      }
      rows.sort();
      fs.writeFileSync(
        path.join(root, 'sitemap.xml'),
        `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1">\n${rows.join('\n')}\n</urlset>\n`,
      );
    },
  },
};

// Static site: Astro pages for the component gallery, Starlight for docs and guides.
// Search is the site's own ⌘K palette over /search-index.json (src/pages/search-index.json.ts),
// so Starlight's Pagefind run is off and its header is replaced by the site nav.
export default defineConfig({
  site: SITE,
  output: 'static',
  trailingSlash: 'ignore',
  integrations: [
    movedUrls,
    sitemap,
    starlight({
      title: 'Oxyn UI',
      titleDelimiter: '–',
      favicon: '/favicon.svg',
      head: [
        { tag: 'link', attrs: { rel: 'icon', href: '/favicon.ico', sizes: '48x48' } },
        { tag: 'link', attrs: { rel: 'icon', href: '/favicon-96.png', type: 'image/png', sizes: '96x96' } },
        { tag: 'link', attrs: { rel: 'icon', href: '/favicon-32.png', type: 'image/png', sizes: '32x32' } },
        { tag: 'link', attrs: { rel: 'apple-touch-icon', href: '/apple-touch-icon.png', sizes: '180x180' } },
        // The same link-preview card as the site pages (src/components/v2/Head.astro).
        ...[
          ['og:image', `${SITE}/og.png?v=2`], ['og:image:secure_url', `${SITE}/og.png?v=2`], ['og:image:type', 'image/png'],
          ['og:image:width', '1200'], ['og:image:height', '630'], ['og:image:alt', 'Oxyn UI: Rust UI components for AI agents'],
        ].map(([property, content]) => ({ tag: 'meta', attrs: { property, content } })),
        { tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
        { tag: 'meta', attrs: { name: 'twitter:image', content: `${SITE}/og.png?v=2` } },
      ],
      description: 'Oxyn UI docs: Rust UI components for AI agents, for Dioxus and GPUI.',
      pagefind: false,
      tableOfContents: false,
      disable404Route: true,
      customCss: ['./src/generated/oxyn-tokens.css', './src/generated/oxyn-components.css', './src/styles/starlight.css'],
      components: {
        Header: './src/components/StarlightHeader.astro',
        ThemeSelect: './src/components/StarlightThemeSelect.astro',
        PageTitle: './src/components/StarlightPageTitle.astro',
      },
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/ismailqureshi72/Oxyn' }],
      sidebar: [
        {
          label: 'Docs',
          items: [
            { label: 'Introduction', link: '/docs/' },
            { label: 'Installation', link: '/docs/installation/' },
            { label: 'CLI', link: '/docs/cli/' },
            { label: 'MCP server', link: '/docs/mcp/' },
            { label: 'Agent state', link: '/docs/agent-state/' },
            { label: 'Theming and tokens', link: '/docs/theming/' },
            { label: 'Motion', link: '/docs/motion/' },
            { label: 'Accessibility', link: '/docs/accessibility/' },
            { label: 'How the live demos work', link: '/docs/live-demos/' },
          ],
        },
        {
          label: 'Guides',
          items: [
            { label: 'All guides', link: '/guides/' },
            { label: 'Build an AI agent UI in Rust', link: '/guides/build-an-ai-agent-ui-in-rust/' },
            { label: 'Show what your agent is doing', link: '/guides/agent-activity/' },
            { label: 'Ask before acting', link: '/guides/approvals/' },
            { label: 'Build a composer', link: '/guides/composer/' },
          ],
        },
        { label: 'Compare', link: '/compare/' },
      ],
    }),
  ],
});
