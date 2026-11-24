// Starlight route middleware (astro.config.mjs routeMiddleware) for the docs, guides and compare
// pages: the link-preview tags move to the top of <head> (right after charset and viewport, where
// WhatsApp, iMessage, Slack, Discord and X read them), each page gets its section's preview card,
// and a TechArticle + BreadcrumbList JSON-LD block (scripts/check-seo.mjs checks both).
import { defineRouteMiddleware } from '@astrojs/starlight/route-data';
import { SITE, ogUrl } from './lib/site.mjs';
import { BRAND, ldJson } from './lib/seo';

type Tag = { tag: string; attrs?: Record<string, string | boolean | undefined>; content?: string };

/** The preview card for a docs path: the Docs card on /docs/, the Guides card under /guides/, the generic docs card elsewhere. */
export const docsCard = (path: string) =>
  path === '/docs/' ? '/og/docs.png' : path.startsWith('/guides/') ? '/og/guides.png' : '/og/docs-page.png';

export const onRequest = defineRouteMiddleware((context) => {
  const route = context.locals.starlightRoute;
  const path = context.url.pathname.endsWith('/') ? context.url.pathname : `${context.url.pathname}/`;
  const url = new URL(path, SITE).href;
  const title = route.entry.data.title;
  const description = route.entry.data.description ?? '';
  const image = ogUrl(docsCard(path));
  const alt = `${title}: ${BRAND} docs`;
  const head = route.head as Tag[];
  const isPreview = (t: Tag) =>
    t.tag === 'meta' && (String(t.attrs?.property ?? '').startsWith('og:') || String(t.attrs?.name ?? '').startsWith('twitter:'));
  const kept = head.filter((t) => !isPreview(t));
  const previews: Tag[] = [
    ...head.filter((t) => isPreview(t) && !/^(og:image|twitter:image|twitter:card)/.test(String(t.attrs?.property ?? t.attrs?.name))),
    ...[['og:image', image], ['og:image:secure_url', image], ['og:image:type', 'image/png'], ['og:image:width', '1200'], ['og:image:height', '630'], ['og:image:alt', alt]]
      .map(([property, content]) => ({ tag: 'meta', attrs: { property, content } })),
    { tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
    { tag: 'meta', attrs: { name: 'twitter:image', content: image } },
    { tag: 'meta', attrs: { name: 'twitter:image:alt', content: alt } },
  ];
  // charset and viewport stay first; the previews follow at once; everything else after them.
  const first = kept.filter((t) => t.tag === 'meta' && (t.attrs?.charset || t.attrs?.name === 'viewport'));
  const rest = kept.filter((t) => !first.includes(t));
  const section = path.startsWith('/guides/') ? ['Guides', '/guides/'] : path.startsWith('/docs/') ? ['Docs', '/docs/'] : null;
  const crumbs: [string, string][] = [[BRAND, '/']];
  if (section && section[1] !== path) crumbs.push(section as [string, string]);
  crumbs.push([path === section?.[1] ? section[0] : title, path]);
  const graph = [
    {
      '@type': 'TechArticle',
      '@id': `${url}#article`,
      headline: title,
      description,
      url,
      mainEntityOfPage: url,
      image,
      inLanguage: 'en',
      isPartOf: { '@id': `${SITE}/#website` },
      author: { '@id': `${SITE}/#organization` },
      publisher: { '@type': 'Organization', '@id': `${SITE}/#organization`, name: BRAND, url: `${SITE}/` },
      ...(route.lastUpdated ? { dateModified: route.lastUpdated.toISOString() } : {}),
    },
    {
      '@type': 'BreadcrumbList',
      itemListElement: crumbs.map(([name, u], i) => ({ '@type': 'ListItem', position: i + 1, name, item: new URL(u, SITE).href })),
    },
  ];
  route.head = [...first, ...previews, ...rest, { tag: 'script', attrs: { type: 'application/ld+json' }, content: ldJson(graph) }] as typeof route.head;
});
