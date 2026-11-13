// @ts-check
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';

// Static site: Astro pages for the component gallery, Starlight for docs and guides.
// Search is one Pagefind index over every page (built after `astro build`, see package.json),
// so Starlight's own Pagefind run is off and its header is replaced by the site nav.
export default defineConfig({
  site: 'https://ui.oxyn.workers.dev',
  output: 'static',
  trailingSlash: 'ignore',
  integrations: [
    starlight({
      title: 'Oxyn',
      description: 'Agent-UI components for Rust: GPUI and Dioxus.',
      pagefind: false,
      disable404Route: true,
      customCss: ['./src/generated/oxyn-tokens.css', './src/generated/oxyn-components.css', './src/styles/starlight.css'],
      components: {
        Header: './src/components/StarlightHeader.astro',
        ThemeSelect: './src/components/StarlightThemeSelect.astro',
        PageTitle: './src/components/StarlightPageTitle.astro',
      },
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/ismailqureshi72/Oxyn' }],
      editLink: { baseUrl: 'https://github.com/ismailqureshi72/Oxyn/edit/main/site/' },
      sidebar: [
        {
          label: 'Docs',
          items: [
            { label: 'Introduction', link: '/docs/' },
            { label: 'Installation', link: '/docs/installation/' },
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
