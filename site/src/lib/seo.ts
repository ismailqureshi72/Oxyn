// Search and answer-engine facts in one place: the brand names, page titles, breadcrumbs and the
// schema.org JSON-LD the pages carry (Organization, WebSite, SoftwareApplication, FAQPage,
// BreadcrumbList, ItemList). Every number comes from the data (src/lib/data.ts), never by hand.
import fs from 'node:fs';
import path from 'node:path';
import { SITE } from './site.mjs';
import { REPO, PRICES, counts, groups, blockGroups, CLI_INSTALL } from './data';

/** The product name everywhere (titles, JSON-LD, footer). "Oxyn" is the short name. */
export const BRAND = 'Oxyn UI';
export const BRAND_SHORT = 'Oxyn';
/** " – Oxyn UI" after every page title but the landing's. */
export const titleOf = (title: string, home = false) => (home ? title : `${title} – ${BRAND}`);

/** Published crates (crates.io, docs.rs): the entity's sameAs, with the repository. */
export const CRATES = ['oxyn-cli', 'oxyn-mcp', 'oxyn-core', 'oxyn-tokens', 'oxyn-registry-schema'];
export const SAME_AS = [REPO, ...CRATES.map((c) => `https://crates.io/crates/${c}`), 'https://docs.rs/oxyn-core'];

/** The workspace version (Cargo.toml [workspace.package]), read when the site is built. */
const version = (() => {
  try {
    const toml = fs.readFileSync(path.resolve(process.cwd(), '../Cargo.toml'), 'utf8');
    return /\[workspace\.package\][^[]*?version\s*=\s*"([^"]+)"/.exec(toml)?.[1] ?? null;
  } catch {
    return null;
  }
})();

const abs = (p: string) => new URL(p, SITE).href;
const ORG_ID = `${SITE}/#organization`;
const SITE_ID = `${SITE}/#website`;
const APP_ID = `${SITE}/#software`;

/** The one-paragraph definition of Oxyn UI (landing, /about/, llms.txt, JSON-LD description). */
export const ENTITY = `${BRAND} (often called ${BRAND_SHORT}) is an open-source Rust UI component library for AI agent interfaces: chat and composer, streaming answers, tool calls, plans, approvals, checkpoints and live runs, plus the everyday inputs, navigation and data display around them. Components are written for Dioxus (web, desktop and mobile) and GPUI (native desktop), share their models and state machines in the oxyn-core crate, and install as source you own with the oxyn CLI (${CLI_INSTALL}) or through the oxyn-mcp server for coding agents. ${counts.free} components are free under the MIT licence; ${BRAND} Pro adds ${counts.proComponents} more components, ${counts.screens} screens and ${counts.blocks} blocks.`;

/** Landing meta description: 150-155 characters, starts with the brand. */
export const HOME_DESCRIPTION = `Oxyn UI is a Rust UI component library for AI agents: chat, tool calls, approvals and live runs for Dioxus and GPUI apps. ${counts.free} free components, MIT licensed.`;

const usd = (n: number) => n.toFixed(2);

export function organization() {
  return {
    '@type': 'Organization',
    '@id': ORG_ID,
    name: BRAND,
    alternateName: BRAND_SHORT,
    url: `${SITE}/`,
    logo: { '@type': 'ImageObject', url: abs('/brand/icon-512.png'), width: 512, height: 512 },
    image: abs('/og.png'),
    description: ENTITY,
    sameAs: SAME_AS,
    founder: [
      { '@type': 'Person', name: 'Khubaib Nazeer', url: 'https://github.com/Khubaib7-del' },
      { '@type': 'Person', name: 'Ismail Qureshi', url: 'https://github.com/ismailqureshi72' },
    ],
  };
}

export function website() {
  return {
    '@type': 'WebSite',
    '@id': SITE_ID,
    name: BRAND,
    alternateName: [BRAND_SHORT, 'oxynui.com'],
    url: `${SITE}/`,
    inLanguage: 'en',
    publisher: { '@id': ORG_ID },
    // The site search: /?q=<terms> opens the search palette with the terms (public/site.js).
    potentialAction: {
      '@type': 'SearchAction',
      target: { '@type': 'EntryPoint', urlTemplate: `${SITE}/?q={search_term_string}` },
      'query-input': 'required name=search_term_string',
    },
  };
}

export function softwareApplication() {
  const offer = (name: string, price: number, unit?: 'MON' | 'ANN') => ({
    '@type': 'Offer',
    name,
    price: usd(price),
    priceCurrency: 'USD',
    url: abs('/pricing/'),
    availability: 'https://schema.org/InStock',
    ...(unit ? { priceSpecification: { '@type': 'UnitPriceSpecification', price: usd(price), priceCurrency: 'USD', billingDuration: 1, unitCode: unit } } : {}),
  });
  return {
    '@type': 'SoftwareApplication',
    '@id': APP_ID,
    name: BRAND,
    alternateName: BRAND_SHORT,
    url: `${SITE}/`,
    description: ENTITY,
    applicationCategory: 'DeveloperApplication',
    applicationSubCategory: 'UI component library',
    operatingSystem: 'Windows, macOS, Linux, Web (WebAssembly)',
    ...(version ? { softwareVersion: version } : {}),
    downloadUrl: 'https://crates.io/crates/oxyn-cli',
    installUrl: abs('/docs/installation/'),
    softwareHelp: abs('/docs/'),
    license: `${REPO}/blob/main/LICENSE`,
    isAccessibleForFree: true,
    image: abs('/og.png'),
    screenshot: abs('/og/components.png'),
    keywords: 'Rust UI components, Dioxus components, GPUI components, AI agent UI, chat UI, tool call UI, MCP',
    publisher: { '@id': ORG_ID },
    sameAs: SAME_AS,
    offers: [
      offer('Free', 0),
      offer('Pro, monthly', PRICES.pro.month, 'MON'),
      offer('Pro, yearly', PRICES.pro.year, 'ANN'),
      offer('Team, monthly', PRICES.team.month, 'MON'),
      offer('Team, yearly', PRICES.team.year, 'ANN'),
    ],
  };
}

/** FAQPage from question and plain-text answer pairs that are also on the page. */
export function faqPage(qa: [string, string][]) {
  return {
    '@type': 'FAQPage',
    mainEntity: qa.map(([q, a]) => ({ '@type': 'Question', name: q, acceptedAnswer: { '@type': 'Answer', text: a } })),
  };
}

export function itemList(name: string, items: { name: string; url: string }[]) {
  return {
    '@type': 'ItemList',
    name,
    numberOfItems: items.length,
    itemListElement: items.map((it, i) => ({ '@type': 'ListItem', position: i + 1, name: it.name, url: abs(it.url) })),
  };
}

// Breadcrumbs from the path: each segment's name from the sections and categories, the last one
// from the page's own title.
const SECTION: Record<string, string> = {
  components: 'Components', screens: 'Screens', blocks: 'Blocks', pricing: 'Pricing', docs: 'Docs', guides: 'Guides',
  about: 'About', dioxus: 'Dioxus', gpui: 'GPUI', motion: 'Motion', studio: 'Studio', illustrations: 'Illustrations', compare: 'Compare',
};
const segName = (parts: string[], i: number) => {
  const s = parts[i];
  if (i === 0) return SECTION[s] ?? s;
  if (parts[0] === 'components') return groups.find((g) => g.slug === s)?.name ?? s;
  if (parts[0] === 'blocks') return blockGroups.find((g) => g.slug === s)?.name ?? s;
  return s.replace(/-/g, ' ').replace(/^./, (c) => c.toUpperCase());
};
export function breadcrumbs(pathname: string, name: string) {
  const parts = pathname.split('/').filter(Boolean);
  if (!parts.length) return null;
  const items: [string, string][] = [[BRAND, '/']];
  parts.forEach((_, i) => {
    // Block groups have no page of their own: their crumb is the group's section on /blocks/.
    const url = parts[0] === 'blocks' && i === 1 && parts.length > 2 ? `/blocks/#${parts[1]}` : `/${parts.slice(0, i + 1).join('/')}/`;
    items.push([i === parts.length - 1 ? name : segName(parts, i), url]);
  });
  return {
    '@type': 'BreadcrumbList',
    itemListElement: items.map(([n, u], i) => ({ '@type': 'ListItem', position: i + 1, name: n, item: abs(u) })),
  };
}

/** One <script type="application/ld+json"> body: a @graph, with "<" escaped so it never ends the tag. */
export const ldJson = (nodes: object[]) =>
  JSON.stringify({ '@context': 'https://schema.org', '@graph': nodes }).replace(/</g, '\\u003c');
