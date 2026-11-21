// @ts-check
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { redirects } from './src/lib/taxonomy.mjs';

// Moved URLs (plan #29 regrouped 21 categories into 8 + Screens): the pages also ship as
// meta-refresh stubs, and the Worker answers them with a 301 from dist/_redirects.
const movedUrls = {
  name: 'oxyn-redirects',
  hooks: {
    'astro:build:done': ({ dir }) => {
      const gen = path.resolve(path.dirname(fileURLToPath(import.meta.url)), 'src/generated/demos.json');
      if (!fs.existsSync(gen)) return;
      const { demos, categories } = JSON.parse(fs.readFileSync(gen, 'utf8'));
      const lines = redirects(demos, categories).map(([from, to]) => `${from} ${to} 301`);
      fs.writeFileSync(new URL('_redirects', dir), lines.join('\n') + '\n');
    },
  },
};

// Static site: Astro pages for the component gallery, Starlight for docs and guides.
// Search is the site's own ⌘K palette over /search-index.json (src/pages/search-index.json.ts),
// so Starlight's Pagefind run is off and its header is replaced by the site nav.
export default defineConfig({
  site: 'https://ui.oxyn.workers.dev',
  output: 'static',
  trailingSlash: 'ignore',
  integrations: [
    movedUrls,
    starlight({
      title: 'Oxyn',
      favicon: '/favicon.svg',
      head: [
        { tag: 'link', attrs: { rel: 'icon', href: '/favicon-32.png', type: 'image/png', sizes: '32x32' } },
        { tag: 'link', attrs: { rel: 'apple-touch-icon', href: '/apple-touch-icon.png', sizes: '180x180' } },
      ],
      description: 'Agent-UI components for Rust: GPUI and Dioxus.',
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
