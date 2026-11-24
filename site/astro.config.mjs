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

// Static site: Astro pages for the component gallery, Starlight for docs and guides.
// Search is the site's own ⌘K palette over /search-index.json (src/pages/search-index.json.ts),
// so Starlight's Pagefind run is off and its header is replaced by the site nav.
export default defineConfig({
  site: SITE,
  output: 'static',
  trailingSlash: 'ignore',
  integrations: [
    movedUrls,
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
