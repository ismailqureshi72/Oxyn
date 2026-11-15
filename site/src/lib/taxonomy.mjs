// The library's grouping, shared by the pages (src/lib/data.ts) and the redirect file written at
// build time (astro.config.mjs). Components follow eight categories; whole app frames are Screens.
// Every older URL (/components/<old-category>/…) keeps working through a redirect.

/** The eight component categories, in sidebar order. */
export const GROUPS = [
  { slug: 'agents', name: 'Agents', blurb: 'What an agent is doing and what it needs from you: status, tool calls, plans, approvals and checkpoints.' },
  { slug: 'chat', name: 'Chat', blurb: 'The conversation: composer, messages, streaming answers, reasoning and sources.' },
  { slug: 'inputs', name: 'Inputs', blurb: 'Fields, choices, pickers and form flows.' },
  { slug: 'data-display', name: 'Data display', blurb: 'Numbers, meters, charts, badges, avatars and code.' },
  { slug: 'navigation', name: 'Navigation', blurb: 'Tabs, menus, sidebars, search and command menus.' },
  { slug: 'feedback', name: 'Feedback', blurb: 'Loaders, toasts, alerts, skeletons and empty states.' },
  { slug: 'cards', name: 'Cards', blurb: 'Cards, dialogs, sheets and the surfaces other parts sit on.' },
  { slug: 'buttons', name: 'Buttons', blurb: 'Buttons, button groups and actions that change in place.' },
];

/** Old category (plan #24–#28) → new category. `shells` is decided per item below. */
export const OLD_TO_GROUP = {
  composer: 'chat',
  loaders: 'feedback',
  'status-text': 'agents',
  tools: 'agents',
  approvals: 'agents',
  'checkpoints-git': 'agents',
  buttons: 'buttons',
  inputs: 'inputs',
  forms: 'inputs',
  pickers: 'inputs',
  'data-display': 'data-display',
  overlays: 'navigation',
  surfaces: 'cards',
  feedback: 'feedback',
  search: 'navigation',
  chat: 'chat',
  navigation: 'navigation',
  choice: 'inputs',
  chrome: 'navigation',
  shells: 'navigation',
};

/** Items whose old category is too broad for one new home. */
export const ITEM_GROUP = {
  'copy-button': 'buttons',
  'code-block': 'data-display',
  'empty-states': 'feedback',
  'notification-center': 'feedback',
};

/** Whole app frames: these get screen pages (/screens/<slug>/) and a full-viewport view. */
export const SCREENS = ['app-shell', 'ai-chat-screens', 'activity-home', 'harness', 'thread-list', 'inbox-sidebar', 'ai-side-panel', 'doc-grid'];

export const groupSlugOf = (d) => ITEM_GROUP[d.slug] ?? OLD_TO_GROUP[d.category] ?? 'navigation';
export const isScreen = (d) => SCREENS.includes(d.slug);
export const pageUrl = (d) => (isScreen(d) ? `/screens/${d.slug}/` : `/components/${groupSlugOf(d)}/${d.slug}/`);
export const legacyUrl = (d) => `/components/${d.category}/${d.slug}/`;

/** [from, to] pairs for every URL that moved. */
export function redirects(demos, oldCategories) {
  const out = [];
  for (const d of demos) {
    const from = legacyUrl(d);
    const to = pageUrl(d);
    if (from !== to) out.push([from, to]);
  }
  const groupSlugs = new Set(GROUPS.map((g) => g.slug));
  for (const c of oldCategories) {
    if (!groupSlugs.has(c.slug)) out.push([`/components/${c.slug}/`, c.slug === 'shells' ? '/screens/' : `/components/${OLD_TO_GROUP[c.slug]}/`]);
  }
  return out;
}
