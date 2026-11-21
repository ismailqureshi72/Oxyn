// The library's grouping, shared by the pages (src/lib/data.ts) and the redirect file written at
// build time (astro.config.mjs). Components follow eight categories; whole app frames are Screens.
// Every older URL (/components/<old-category>/…) keeps working through a redirect.

import FREE_TIER from './free-tier.json' with { type: 'json' };

/** The Free tier (docs/pricing.md): the only items whose code is free. The single source of truth,
 * read by the site, the Worker (plan_for gating), the registry builder and status.csv. Every item
 * not in it, including every new one, is Oxyn Pro. */
export const FREE = new Set(FREE_TIER.free);
export const isFree = (d) => FREE.has(typeof d === 'string' ? d : d.slug);

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
  { slug: 'ai', name: 'AI', blurb: 'Pro AI components: run timelines, model cards, workflows, image generation, speech, meeting notes, replies, handoff, redaction, costs, keys, stats, translation, summaries, connectors, enrichment and routing.' },
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
  'ai-special': 'ai',
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
export const SCREENS = ['shell-inset', 'shell-flat', 'shell-three-pane', 'chat-start', 'chat-overview', 'chat-agents', 'chat-double-sidebar', 'chat-progress', 'chat-run', 'folders-overview', 'folders-detail', 'integrations', 'kanban', 'crm', 'analytics', 'workflow', 'settings', 'settings-profile', 'settings-preferences', 'settings-notifications', 'settings-privacy', 'settings-general', 'settings-members', 'settings-models', 'settings-security', 'settings-billing', 'settings-onboarding', 'onboarding-website', 'onboarding-brand', 'onboarding-tone', 'onboarding-ready', 'taskdeck-start', 'taskdeck-task', 'taskdeck-board', 'taskdeck-plugins', 'taskdeck-scheduled', 'taskdeck-sites', 'command-center', 'session-studio', 'fleet-window', 'workspace-coder', 'command-board', 'threadbar', 'tabstack', 'repo-desk', 'preview-desk', 'hangar', 'halo-chat', 'pilot-panel', 'courier', 'metrics-dashboard', 'kanban-board', 'project-board', 'gantt-timeline', 'team-directory', 'week-calendar', 'month-calendar', 'inbox-triage', 'support-conversation', 'help-center', 'market-terminal', 'product-list', 'product-detail', 'invoice-studio', 'sign-in-split', 'admin-overview', 'audit-log', 'status-board', 'api-keys-webhooks', 'members-roles', 'billing-usage', 'agent-tool-calls', 'agent-orchestration', 'agent-thread', 'inbox', 'agent-inbox-board', 'agent-tasks', 'agent-activity', 'agent-memory', 'agent-agents', 'agent-skills', 'agent-schedules', 'agent-usage', 'agent-permissions', 'agent-run-detail', 'evals', 'app-shell', 'ai-chat-screens', 'activity-home', 'harness', 'thread-list', 'inbox-sidebar', 'ai-side-panel', 'doc-grid'];

export const groupSlugOf = (d) => ITEM_GROUP[d.slug] ?? OLD_TO_GROUP[d.category] ?? 'navigation';
/** True for a demo that belongs to a multi-variant family. */
export const familyOf = (d) => FAMILIES.find((f) => f.slug === d.category) ?? null;
export const isScreen = (d) => SCREENS.includes(d.slug);
/** Pro components (docs/pricing.md): the AI special components (C1d). Like screens, the preview is
 * free and the code needs Pro. */
export const PRO_COMPONENTS = ['agent-timeline', 'model-card', 'ai-workflow', 'image-generation', 'speech-player', 'meeting-summary', 'reply-draft', 'human-handoff', 'pii-redaction', 'llm-cost-calculator', 'provider-keys', 'generation-stats', 'ai-translate', 'tldr-toggle', 'connector-setup', 'enrich-column', 'ticket-router', 'ask-your-data', 'agent-doc-cursor', 'provider-status', 'training-run', 'tone-tuner', 'ai-mascot', 'image-reveal', 'ai-spark', 'pixel-mascot', 'ghost-mascot', 'cube-mascot', 'upgrade-prompt', 'referral-card', 'trial-banner', 'live-visitors', 'social-proof-toast'];
export const isProComponent = (d) => PRO_COMPONENTS.includes(d.slug);
/** Agent apps: whole desktop agent apps (our own designs in the patterns those apps share). They are
 * screens too (pages at /screens/<slug>/), listed in their own category after Screens. */
export const AGENT_APPS = [{ slug: 'taskdeck', name: 'Taskdeck', blurb: 'A task-runner agent app: composer, running task with steps and artifacts, a task board, plugins & skills, scheduled tasks and deployed sites.', screens: ['taskdeck-start', 'taskdeck-task', 'taskdeck-board', 'taskdeck-plugins', 'taskdeck-scheduled', 'taskdeck-sites'] }, { slug: 'command-center', name: 'Command Center', blurb: 'A coding-agent app: threads by project, a running thread, a review pane of hunks to accept or revert, Local / Worktree / Cloud and a task queue.', screens: ['command-center'] }, { slug: 'session-studio', name: 'Session Studio', blurb: 'A coding-agent app: sessions, read / edit / bash tool blocks, permission prompts, plan mode, a todo list and a diff pane.', screens: ['session-studio'] }, { slug: 'fleet-window', name: 'Fleet Window', blurb: 'A coding-agent app: parallel agents on their own worktrees, one prompt on several models compared side by side, plan mode and a changes pane.', screens: ['fleet-window'] }, { slug: 'workspace-coder', name: 'Workspace Coder', blurb: 'An agent app with its own computer: progress and tool log in the chat, a workspace of files, terminal, browser and a preview to publish.', screens: ['workspace-coder'] }, { slug: 'command-board', name: 'Command Board', blurb: 'An autonomous-agent app: sessions from tickets and chat, questions with quick answers, a pull request with checks and a workspace timeline you can scrub.', screens: ['command-board'] }, { slug: 'threadbar', name: 'Threadbar', blurb: 'An editor agent panel: threads by project with terminal threads, tool calls and an edit to keep or reject, profiles and outside agents, Files and Git.', screens: ['threadbar'] }, { slug: 'tabstack', name: 'Tabstack', blurb: 'An agent-first terminal: vertical tabs with branch and PR, command blocks, a command waiting on Run / Edit / Deny, agent runs and review.', screens: ['tabstack'] }, { slug: 'repo-desk', name: 'Repo Desk', blurb: 'A chat-first app on a code host: an Inbox of issues and PRs, projects with goals, sessions on their own branches and shared canvases.', screens: ['repo-desk'] }, { slug: 'preview-desk', name: 'Preview Desk', blurb: 'Agent work as previews: sessions on their own machines, the live site with Design mode comments, a doc and a diff with line comments.', screens: ['preview-desk'] }, { slug: 'hangar', name: 'Hangar', blurb: 'An agent manager: conversations by workspace with an Inbox, a task list and artifacts, a plan to comment on line by line, and Editor and Browser surfaces.', screens: ['hangar'] }, { slug: 'halo-chat', name: 'Halo Chat', blurb: 'An assistant app: helpers and pinned chats, tool cards for search, code and tables, Sources and Canvas, a mini chat and a research plan.', screens: ['halo-chat'] }, { slug: 'pilot-panel', name: 'Pilot Panel', blurb: 'An editor agent panel: Plan and Act, commands that wait on approval, a checkpoint after every change to compare or restore, tokens and cost.', screens: ['pilot-panel'] }, { slug: 'courier', name: 'Courier', blurb: 'A thread-first coding agent: running threads, an edits card per turn, suggested replies, model and effort, and a pod with Desktop, Ports and Changes.', screens: ['courier'] }];
export const isAgentApp = (d) => AGENT_APPS.some((a) => a.screens.includes(d.slug));
/** Arc screens: work-app screens on the Arc frame (248 px sidebar with a context card, an account row
 * with the email, an inset page and a right card). Screens too, listed in their own category. */
export const ARC_SCREENS = ['metrics-dashboard', 'kanban-board', 'project-board', 'gantt-timeline', 'team-directory', 'week-calendar', 'month-calendar', 'inbox-triage', 'support-conversation', 'help-center', 'market-terminal', 'product-list', 'product-detail', 'invoice-studio', 'sign-in-split', 'admin-overview', 'audit-log', 'status-board', 'api-keys-webhooks', 'members-roles', 'billing-usage'];
export const isArcScreen = (d) => ARC_SCREENS.includes(d.slug);
/** Blocks (docs/pricing.md: Pro): larger compositions with their own section, /blocks/<group>/<slug>/.
 * A block's demo category is `blocks-<group>`. */
export const BLOCK_GROUPS = [
  { slug: 'agents', cat: 'blocks-agents', name: 'Agents', blurb: 'Whole agent surfaces: a live session with a team of agents, mission control and its timeline, a run inspector, background coding agents, routines, MCP servers, a knowledge base, an eval suite, an agent builder and an agent gallery.' },
  { slug: 'ai', cat: 'blocks-ai', name: 'AI', blurb: 'AI surfaces for products: a KPI card that explains its own change, a blind model arena, a prompt studio, assistant memory, a catch-up digest, a feature launch card, a setup wizard, sales call insights and lead scoring.' },
];
export const isBlock = (d) => BLOCK_GROUPS.some((g) => g.cat === d.category);
export const blockGroupOf = (d) => BLOCK_GROUPS.find((g) => g.cat === d.category) ?? null;
export const pageUrl = (d) => (isScreen(d) ? `/screens/${d.slug}/` : isBlock(d) ? `/blocks/${blockGroupOf(d).slug}/${d.slug}/` : `/components/${groupSlugOf(d)}/${d.slug}/`);
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
    if (!groupSlugs.has(c.slug)) out.push([`/components/${c.slug}/`, c.slug === 'shells' || c.slug === 'screens' ? '/screens/' : c.slug.startsWith('blocks-') ? '/blocks/' : `/components/${OLD_TO_GROUP[c.slug]}/`]);
  }
  for (const m of MOVED) out.push(m);
  return out;
}
