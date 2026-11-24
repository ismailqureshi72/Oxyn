// /llms.txt and /llms-full.txt (llmstxt.org): what answer engines and coding agents read to learn
// what Oxyn UI is, how to install it and where everything lives. Built from the data and the docs
// collection, so counts, prices and pages stay in step with the site. Public pages only.
import { getCollection } from 'astro:content';
import { SITE } from './site.mjs';
import { counts, components, screens, blocks, groups, inGroup, demoUrl, isPro, PRICES, REPO, CLI_INSTALL } from './data';
import { BRAND, BRAND_SHORT, ENTITY, CRATES } from './seo';

const u = (p: string) => new URL(p, SITE).href;
const usd = (n: number) => `$${Number.isInteger(n) ? n : n.toFixed(2)}`;

async function docs() {
  const all = await getCollection('docs');
  const id = (e: (typeof all)[number]) => e.id.replace(/\/index$/, '');
  return all.map((e) => ({ title: e.data.title, description: e.data.description ?? '', url: u(`/${id(e)}/`), body: e.body ?? '' })).sort((a, b) => a.url.localeCompare(b.url));
}

function head(): string[] {
  return [
    `# ${BRAND}`,
    '',
    `> ${BRAND} (often called ${BRAND_SHORT}) is an open-source Rust UI component library for AI agent interfaces, for Dioxus (web) and GPUI (desktop). Site: ${SITE}`,
    '',
    ENTITY,
    '',
    '## Install',
    '',
    `- CLI: \`${CLI_INSTALL}\`, then \`oxyn init\` in a Dioxus or GPUI project and \`oxyn add <component>\`. Source lands in \`src/oxyn/\`.`,
    '- MCP server for coding agents: `cargo install oxyn-mcp`, then e.g. `claude mcp add --transport stdio oxyn -- oxyn-mcp`.',
    `- Free registry JSON: ${u('/r/index.json')}`,
    '',
    '## Pricing',
    '',
    `- Free: ${counts.free} components, the docs, the CLI and the MCP server (MIT licence).`,
    `- Pro: ${usd(PRICES.pro.month)}/month or ${usd(PRICES.pro.year)}/year; every other component (${counts.proComponents}), ${counts.screens} screens and ${counts.blocks} blocks. Intro price for the first ${PRICES.founder.cap} new subscribers: ${usd(PRICES.founder.month)} first month or ${usd(PRICES.founder.year)} first year.`,
    `- Team: up to ${PRICES.team.people} people, ${usd(PRICES.team.month)}/month or ${usd(PRICES.team.year)}/year.`,
    `- Details: ${u('/pricing/')}`,
    '',
  ];
}

function links(): string[] {
  return [
    '## Links',
    '',
    `- [About ${BRAND}](${u('/about/')}): what it is, how it works, who makes it`,
    `- [Components](${u('/components/')}): ${counts.components} components, ${counts.free} free`,
    `- [Screens](${u('/screens/')}): ${counts.screens} whole agent app screens`,
    `- [Blocks](${u('/blocks/')}): ${counts.blocks} agent and AI surfaces`,
    `- [Dioxus components](${u('/dioxus/')}) and [GPUI components](${u('/gpui/')})`,
    `- [Pricing](${u('/pricing/')})`,
    `- [GitHub](${REPO})`,
    ...CRATES.map((c) => `- [${c} on crates.io](https://crates.io/crates/${c})`),
    `- [API docs on docs.rs](https://docs.rs/oxyn-core)`,
    '',
  ];
}

export async function llmsTxt(): Promise<string> {
  const d = await docs();
  const cats = groups.map((g) => `- [${g.name}](${u(`/components/${g.slug}/`)}): ${g.blurb}`);
  return [
    ...head(),
    '## Docs',
    '',
    ...d.map((x) => `- [${x.title}](${x.url})${x.description ? `: ${x.description}` : ''}`),
    '',
    '## Component categories',
    '',
    ...cats,
    '',
    ...links(),
    '## Optional',
    '',
    `- [Full text for LLMs](${u('/llms-full.txt')}): every docs page and the whole component list in one file`,
    '',
  ].join('\n');
}

const clean = (md: string) =>
  md
    .replace(/^---[\s\S]*?---\n/, '')
    .replace(/<nav[\s\S]*?<\/nav>/g, '')
    .replace(/<img[^>]*>/g, '')
    .replace(/<\/?span[^>]*>/g, '')
    .replace(/\]\(\//g, `](${SITE}/`)
    .trim();

export async function llmsFullTxt(): Promise<string> {
  const d = await docs();
  const item = (x: (typeof components)[number]) => `- [${x.name}](${u(demoUrl(x))})${isPro(x) ? ' (Pro)' : ' (free)'}: ${x.summary}`;
  return [
    ...head(),
    ...links(),
    '## Components by category',
    '',
    ...groups.flatMap((g) => [`### ${g.name}`, '', g.blurb, '', ...inGroup(g.slug).map(item), '']),
    '## Screens (Pro)',
    '',
    ...screens.map((x) => `- [${x.name}](${u(demoUrl(x))}): ${x.summary}`),
    '',
    '## Blocks (Pro)',
    '',
    ...blocks.map((x) => `- [${x.name}](${u(demoUrl(x))}): ${x.summary}`),
    '',
    '## Docs and guides',
    '',
    ...d.flatMap((x) => [`### ${x.title}`, '', `Source: ${x.url}`, '', clean(x.body), '']),
  ].join('\n');
}
