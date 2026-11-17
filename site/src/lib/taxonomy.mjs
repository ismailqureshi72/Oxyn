// The library's grouping, shared by the pages (src/lib/data.ts) and the redirect file written at
// build time (astro.config.mjs). Components follow eight categories; whole app frames are Screens.
// Every older URL (/components/<old-category>/…) keeps working through a redirect.

/** Multi-variant families: one control done many ways, each variant its own page. A family is a
 * category of its own (its pages live at /components/<family>/<slug>/) and gets its own collapsible
 * group in the sidebar. Add a family here when its first component lands. */
export const FAMILIES = [
  { slug: 'thinking-selector', name: 'Thinking Selector', blurb: 'How hard the model thinks: effort sliders, rulers, menus and pickers, each a different take on the same choice.' },
  { slug: 'context-visualizer', name: 'Context Visualizer', blurb: 'How full the context window is and what fills it: bars, fields of dots, breakdowns, stacks and a tide glass.' },
  { slug: 'storage', name: 'Storage', blurb: 'Disk and context usage drawn as dot matrices: capacity, cleanup, waffle and forecast.' },
  { slug: 'attachments', name: 'Attachments', blurb: 'Files, images and links going into a prompt: drop zones, chips with upload progress, prints and lanes.' },
  { slug: 'menu-motion', name: 'Menu Motion', blurb: 'Menus and controls that move like liquid: the gooey menu motion on attach and model menus, a composer, tabs and drops.' },
  { slug: 'theme-selector', name: 'Theme Selector', blurb: 'Backgrounds and texture, theme swatches and font size, plus a split compare, accent and density, a schedule, type, light, icons and comfort.' },
];
function FAMILY_GROUPS() {
  return FAMILIES.map((f) => ({ ...f, family: true }));
}

/** The eight component categories, then the families, in sidebar order. */
export const GROUPS = [
  { slug: 'agents', name: 'Agents', blurb: 'What an agent is doing and what it needs from you: status, tool calls, plans, approvals and checkpoints.' },
  { slug: 'chat', name: 'Chat', blurb: 'The conversation: composer, messages, streaming answers, reasoning and sources.' },
  { slug: 'inputs', name: 'Inputs', blurb: 'Fields, choices, pickers and form flows.' },
  { slug: 'data-display', name: 'Data display', blurb: 'Numbers, meters, charts, badges, avatars and code.' },
  { slug: 'navigation', name: 'Navigation', blurb: 'Tabs, menus, sidebars, search and command menus.' },
  { slug: 'feedback', name: 'Feedback', blurb: 'Loaders, toasts, alerts, skeletons and empty states.' },
  { slug: 'cards', name: 'Cards', blurb: 'Cards, dialogs, sheets and the surfaces other parts sit on.' },
  { slug: 'buttons', name: 'Buttons', blurb: 'Buttons, button groups and actions that change in place.' },
  ...FAMILY_GROUPS(),
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
  ...Object.fromEntries(FAMILIES.map((f) => [f.slug, f.slug])),
};

/** Items whose old category is too broad for one new home. */
export const ITEM_GROUP = {
  'copy-button': 'buttons',
  'code-block': 'data-display',
  'empty-states': 'feedback',
  'notification-center': 'feedback',
  'run-controls': 'agents',
};

/** Pages that were folded into another (plan #32): the old URL lands on the new home. */
export const MOVED = [
  ['/components/chat/model-menu/', '/components/chat/model-picker/'],
  ['/components/composer/model-menu/', '/components/chat/model-picker/'],
  ['/components/chat/run-controls/', '/components/agents/run-controls/'],
  ['/components/feedback/thinking-loader/', '/components/chat/thinking-loader/'],
  ['/components/loaders/thinking-loader/', '/components/chat/thinking-loader/'],
];

/** Whole app frames: these get screen pages (/screens/<slug>/) and a full-viewport view. */
export const SCREENS = ['shell-inset', 'shell-flat', 'shell-three-pane', 'chat-start', 'chat-overview', 'chat-agents', 'chat-double-sidebar', 'chat-progress', 'chat-run', 'folders-overview', 'folders-detail', 'integrations', 'app-shell', 'ai-chat-screens', 'activity-home', 'harness', 'thread-list', 'inbox-sidebar', 'ai-side-panel', 'doc-grid'];

export const groupSlugOf = (d) => ITEM_GROUP[d.slug] ?? OLD_TO_GROUP[d.category] ?? 'navigation';
/** True for a demo that belongs to a multi-variant family. */
export const familyOf = (d) => FAMILIES.find((f) => f.slug === d.category) ?? null;
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
    if (!groupSlugs.has(c.slug)) out.push([`/components/${c.slug}/`, c.slug === 'shells' || c.slug === 'screens' ? '/screens/' : `/components/${OLD_TO_GROUP[c.slug]}/`]);
  }
  for (const m of MOVED) out.push(m);
  return out;
}
