// The ⌘K palette's index, written once at build time to /search-index.json: every component,
// screen, agent app, docs page and library page with its name, url, category and a short line,
// plus keywords (slug words, family, old category, aliases, docs headings) for partial matches.
// The palette (public/site.js) fetches it once, keeps it in memory and ranks on the client.
import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';
import { components, screens, groups, groupOf, demoUrl, agentApps, arcScreens, composedScreens, type Demo } from '../lib/data';
import { AGENT_APPS, familyOf } from '../lib/taxonomy.mjs';

// Words people type for a part that its own name does not contain.
const ALIASES: Record<string, string> = {
  'approval-card': 'permission allow deny confirm human in the loop hitl ask before acting',
  'approval-pager': 'permissions batch allow',
  'tool-call': 'function call tools',
  'checkpoint-lanes': 'git branch rewind fork restore timeline',
  'checkpoint-tree': 'git branch fork rewind',
  'checkpoint-marker': 'restore rewind',
  'thinking-loader': 'spinner loading working',
  'chat-composer': 'prompt input message box textarea',
  'model-picker': 'model menu llm select',
  'command-menu': 'command palette cmdk search',
  'agent-trail': 'steps timeline progress run',
  'code-block': 'syntax highlight snippet',
  'copy-button': 'clipboard',
  'empty-states': 'blank nothing here',
  'notification-center': 'notifications alerts inbox bell',
};

type Row = { t: string; u: string; g: string; k: string; d: string; w: string };
const clip = (s: string, n = 96) => {
  const one = (s || '').replace(/\s+/g, ' ').replace(/[`*_]/g, '').trim();
  if (one.length <= n) return one;
  const cut = one.slice(0, n);
  return cut.slice(0, cut.lastIndexOf(' ')).replace(/[,;:.–-]+$/, '') + '…';
};
const words = (...xs: (string | undefined)[]) =>
  [...new Set(xs.filter(Boolean).join(' ').toLowerCase().replace(/[^a-z0-9]+/g, ' ').trim().split(' '))].join(' ');

const appOf = (d: Demo) => AGENT_APPS.find((a) => a.screens.includes(d.slug));
// Screen summaries open with their breadcrumb ("Agent apps / Taskdeck: …"); the row shows it already.
const lede = (s: string) => {
  const t = (s || '').replace(/^[^:]{0,60} \/ [^:]{0,60}:\s*/, '');
  return t.charAt(0).toUpperCase() + t.slice(1);
};

export const GET: APIRoute = async () => {
  const rows: Row[] = [];
  for (const d of components) {
    const g = groupOf(d);
    const fam = familyOf(d);
    rows.push({ t: d.name, u: demoUrl(d), g: 'c', k: g?.name ?? '', d: clip(d.summary), w: words(d.slug, d.category, fam?.name, ALIASES[d.slug]) });
  }
  const apps = new Set(agentApps.map((d) => d.slug));
  const arc = new Set(arcScreens.map((d) => d.slug));
  for (const d of screens) {
    const app = appOf(d);
    rows.push({
      t: d.name,
      u: demoUrl(d),
      g: apps.has(d.slug) ? 'a' : 's',
      k: app ? app.name : arc.has(d.slug) ? 'Arc screen' : 'Screen',
      d: clip(lede(d.summary)),
      w: words(d.slug, app?.name, app ? 'agent app' : 'screen', ALIASES[d.slug]),
    });
  }
  for (const s of composedScreens) rows.push({ t: s.name, u: s.href, g: 's', k: 'Screen', d: clip(s.summary), w: words('screen app') });
  for (const g of groups) rows.push({ t: g.name, u: `/components/${g.slug}/`, g: 'p', k: g.family ? 'Family' : 'Category', d: clip(g.blurb), w: words(g.slug, 'category') });
  const pages: [string, string, string][] = [
    ['Components', '/components/', 'Every component in the library.'],
    ['New components', '/components/new/', 'The latest additions to the library.'],
    ['Screens', '/screens/', 'Whole app frames built from Oxyn components.'],
    ['Agent apps', '/screens/#agent-apps', 'Whole desktop agent apps, in the patterns those apps share.'],
    ['Arc screens', '/screens/#arc-screens', 'Work-app screens on the Arc frame.'],
    ['Blocks', '/blocks/', 'Larger sections you drop into a page.'],
    ['Illustrations', '/illustrations/', 'Animated spot illustrations for empty states and onboarding.'],
    ['Motion', '/motion/', 'Durations, easings and springs as tokens.'],
    ['Pricing', '/pricing/', 'The core is free and MIT licensed.'],
  ];
  for (const [t, u, d] of pages) rows.push({ t, u, g: 'p', k: 'Page', d, w: words(u) });
  const docs = await getCollection('docs');
  for (const e of docs) {
    const heads = [...(e.body ?? '').matchAll(/^#{2,3}\s+(.+)$/gm)].map((m) => m[1]).join(' ');
    const id = e.id.replace(/\/index$/, '');
    rows.push({
      t: e.data.title,
      u: `/${id}/`,
      g: 'd',
      k: id.startsWith('guides') ? 'Guide' : 'Docs',
      d: clip(e.data.description ?? ''),
      w: words(id, heads),
    });
  }
  return new Response(JSON.stringify({ v: 1, items: rows }), { headers: { 'Content-Type': 'application/json' } });
};
