// Server-rendered previews the landing does not inline. Hidden tab panels (the Screens, Blocks and
// Motion grids, the laptop's other screens, the other compare tabs) would put ~400 KB of markup
// into the first HTML for things nobody sees until they pick a tab. Their markup is written to
// /ssr/<key>.htm instead (pages/ssr/[key].htm.ts) and <oxyn-demo data-ssr> fetches it the first
// time the preview comes near the screen (public/oxyn-demo.js), before the live WASM mount.
// One list for both sides, so every lazy preview has its fragment.
import { demos, screens, blocks, type Demo } from './data';

const find = (s: string) => demos.find((d) => d.slug === s)!;

/** The landing's library tabs: [key, label, slugs | null (coming soon)]. The first is inlined. */
export const homeTabSets: [string, string, string[] | null][] = [
  ['components', 'Components', ['approval-card', 'tool-call', 'agent-plan', 'model-picker', 'thinking-pill', 'context-meter']],
  ['screens', 'Screens', screens.slice(0, 6).map((d) => d.slug)],
  ['blocks', 'Blocks', blocks.slice(0, 6).map((d) => d.slug)],
  ['illustrations', 'Illustrations', null],
  ['motion', 'Motion', ['text-shimmer', 'verb-cycler', 'text-reveal', 'elapsed-timer', 'text-morph', 'slot-text']],
];

/** The laptop's screen tabs: [slug, label, icon]. The first is inlined. */
export const lapTabs = [
  ['chat-agents', 'Agent chat', 'chat'],
  ['agent-orchestration', 'Orchestration', 'branch'],
  ['chat-run', 'Agent run', 'play'],
  ['kanban', 'Kanban', 'kanban'],
  ['workflow', 'Workflow', 'flow'],
  ['folders-overview', 'Folders', 'folder'],
] as const;

/** The compare tabs: [key, label, demo slug]. The first is inlined. */
export const compareTabSlugs = [
  ['approval', 'Approval', 'approval-card'],
  ['tools', 'Tool calls', 'tool-call'],
  ['chat', 'Agent chat', 'reasoning-block'],
] as const;

/** The fragment name for a preview: `<slug>` for the whole-screen markup (variant -1), else `<slug>.<variant>`. */
export const ssrKey = (slug: string, variant: number) => (variant < 0 ? slug : `${slug}.${variant}`);
export const ssrUrl = (slug: string, variant: number) => `/ssr/${ssrKey(slug, variant)}.htm`;

/** Every lazy preview on the landing: its fragment name and markup. */
export function lazySsr(): { key: string; html: string }[] {
  const out = new Map<string, string>();
  const add = (d: Demo | undefined, variant: number) => {
    if (!d) return;
    out.set(ssrKey(d.slug, variant), variant < 0 ? d.html : d.variants[variant].html);
  };
  for (const [, , slugs] of homeTabSets.slice(1)) for (const s of slugs ?? []) add(find(s), 0);
  for (const [s] of lapTabs.slice(1)) add(demos.find((d) => d.slug === s), -1);
  for (const [, , s] of compareTabSlugs.slice(1)) add(find(s), 0);
  return [...out].map(([key, html]) => ({ key, html }));
}
