// Build-time data for the site, all generated from the Rust workspace by scripts/gen.mjs.
import demosJson from '../generated/demos.json';
import propsJson from '../generated/props.json';
import specsJson from '../generated/specs.json';
import parityJson from '../generated/parity.json';
import { publicText, publicMarkdown } from './public-text.mjs';
import wasmJson from '../generated/wasm.json';
import motionJson from '../generated/motion.json';
import { GROUPS, groupSlugOf, isScreen, isAgentApp, isArcScreen, isProComponent, isFree, isBlock, blockGroupOf, BLOCK_GROUPS, ARC_SCREENS, pageUrl } from './taxonomy.mjs';

export type Control = {
  key: string;
  label: string;
  kind: 'select' | 'toggle' | 'range' | 'text';
  default: string;
  extra: { options?: string[]; min?: number; max?: number; step?: number };
};
export type Variant = { name: string; note: string; set: [string, string][]; code: string; html: string };
export type Demo = {
  slug: string;
  name: string;
  category: string;
  summary: string;
  dioxus: string;
  gpui: string;
  spec: string;
  source: string;
  controls: Control[];
  variants: Variant[];
  keyboard: [string, string][];
  a11y: string[];
  responsive: string[];
  height: number;
  live: boolean;
  code: string;
  html: string;
};
export type Category = { slug: string; name: string; blurb: string };
export type Prop = {
  name: string;
  type: string;
  default: string | null;
  required: boolean;
  event: boolean;
  doc: string;
};

export const REPO = 'https://github.com/ismailqureshi72/Oxyn';
export const categories = demosJson.categories as Category[];
export const demos = demosJson.demos as Demo[];
export const props = propsJson as Record<string, { file: string; doc: string; props: Prop[] }>;
export const specs = specsJson as Record<string, { title: string; sections: Record<string, string> }>;
export const parity = parityJson as Record<string, string>[];
// Parity names: a demo that rebuilds an affectUI item takes affectUI's own name ("Chat Composer",
// "Comment Thread"), so the sidebar and pages read like the reference. Its tier there does not
// matter here: Oxyn's own split is below (isPro).
const norm = (s: string) => s.toLowerCase().replace(/[^a-z0-9]+/g, ' ').trim();
const affect = new Map<string, Record<string, string>>();
for (const r of parity) if (r.source === 'affectUI' && !affect.has(norm(r.item))) affect.set(norm(r.item), r);
for (const d of demos) {
  const r = affect.get(norm(d.name));
  if (r) d.name = r.item;
  // Public copy: no reference-library names or plan numbers on a page (src/lib/public-text.mjs).
  d.summary = publicText(d.summary);
  d.a11y = (d.a11y ?? []).map(publicText).filter(Boolean);
  for (const v of d.variants) if (v.note) v.note = publicText(v.note);
}
for (const p of Object.values(propsJson as Record<string, { doc: string; props: { doc: string }[] }>)) {
  p.doc = publicText(p.doc);
  for (const x of p.props) x.doc = publicText(x.doc);
}
for (const s of Object.values(specsJson as Record<string, { sections: Record<string, string> }>))
  for (const k of Object.keys(s.sections)) s.sections[k] = publicMarkdown(s.sections[k]);
export const wasm = wasmJson as { js: string | null; wasm: string | null; bytes: number };
export const motion = motionJson as Record<string, any>;

export const demosIn = (cat: string) => demos.filter((d) => d.category === cat);
export const categoryOf = (slug: string) => categories.find((c) => c.slug === slug)!;
export const demoUrl = (d: Demo) => pageUrl(d) as string;

/** Section of a spec whose heading starts with one of `names`. */
export function specSection(spec: string, ...names: string[]): string | null {
  const s = specs[spec];
  if (!s) return null;
  for (const n of names) {
    const key = Object.keys(s.sections).find((k) => k.toLowerCase().startsWith(n.toLowerCase()));
    if (key) return s.sections[key];
  }
  return null;
}

const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

function inline(s: string): string {
  let out = esc(s);
  out = out.replace(/`([^`]+)`/g, '<code>$1</code>');
  out = out.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  out = out.replace(/\[([^\]]+)\]\((https?:[^)]+)\)/g, '<a href="$2">$1</a>');
  // Relative links point into the repo's docs; keep the text only.
  out = out.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '$1');
  return out;
}

/** A small Markdown renderer for spec excerpts: paragraphs, lists, tables and code. */
export function md(src: string | null): string {
  if (!src) return '';
  const lines = src.split('\n');
  const html: string[] = [];
  let i = 0;
  while (i < lines.length) {
    const l = lines[i];
    if (/^```/.test(l)) {
      const buf: string[] = [];
      i++;
      while (i < lines.length && !/^```/.test(lines[i])) buf.push(lines[i++]);
      i++;
      html.push(`<pre><code>${esc(buf.join('\n'))}</code></pre>`);
    } else if (/^\|/.test(l)) {
      const rows: string[][] = [];
      while (i < lines.length && /^\|/.test(lines[i])) {
        const cells = lines[i].replace(/^\||\|$/g, '').split('|').map((c) => c.trim());
        if (!cells.every((c) => /^:?-+:?$/.test(c))) rows.push(cells);
        i++;
      }
      const [head, ...body] = rows;
      html.push(
        `<div class="table-wrap"><table><thead><tr>${head.map((c) => `<th>${inline(c)}</th>`).join('')}</tr></thead><tbody>${body
          .map((r) => `<tr>${r.map((c) => `<td>${inline(c)}</td>`).join('')}</tr>`)
          .join('')}</tbody></table></div>`,
      );
    } else if (/^\s*[-*] /.test(l)) {
      const items: string[] = [];
      while (i < lines.length && (/^\s*[-*] /.test(lines[i]) || /^\s{2,}\S/.test(lines[i]))) {
        if (/^\s*[-*] /.test(lines[i])) items.push(lines[i].replace(/^\s*[-*] /, ''));
        else items[items.length - 1] += ' ' + lines[i].trim();
        i++;
      }
      html.push(`<ul>${items.map((t) => `<li>${inline(t)}</li>`).join('')}</ul>`);
    } else if (/^###\s/.test(l)) {
      html.push(`<h4>${inline(l.replace(/^###\s*/, ''))}</h4>`);
      i++;
    } else if (l.trim() === '') {
      i++;
    } else {
      const buf: string[] = [];
      while (i < lines.length && lines[i].trim() !== '' && !/^(\||```|\s*[-*] |###)/.test(lines[i])) buf.push(lines[i++]);
      html.push(`<p>${inline(buf.join(' '))}</p>`);
    }
  }
  return html.join('\n');
}

/** Rust source → HTML with light token classes (keywords, strings, comments, numbers). */
export function highlightRust(code: string): string {
  const re =
    /(\/\/[^\n]*)|("(?:\\.|[^"\\])*")|\b(let|mut|move|fn|use|pub|struct|impl|match|if|else|for|in|return|Some|None|true|false|vec|rsx)\b|\b(\d[\d_]*(?:\.\d+)?)\b|\b([A-Z][A-Za-z0-9]*)\b/g;
  let out = '';
  let last = 0;
  for (const m of code.matchAll(re)) {
    out += esc(code.slice(last, m.index));
    const [t, com, str, kw, num, ty] = m;
    const cls = com ? 'c' : str ? 's' : kw ? 'k' : num ? 'n' : ty ? 't' : '';
    out += `<span class="tk-${cls}">${esc(t)}</span>`;
    last = (m.index ?? 0) + t.length;
  }
  return out + esc(code.slice(last));
}

/** The stylesheet constant a component's source file ships. */
export function cssConst(source: string): string {
  if (source.startsWith('primitives/')) return 'PRIMITIVES_CSS';
  if (source.startsWith('composer/')) return 'COMPOSER_CSS';
  if (source.startsWith('run/')) return 'RUN_CSS';
  if (source === 'approval.rs' || source === 'checkpoint.rs') return 'HITL_CSS';
  if (source === 'status_text.rs') return 'STATUS_TEXT_CSS';
  if (source === 'glyph_loader.rs') return 'GLYPH_LOADER_CSS';
  return 'STATUS_INDICATOR_CSS';
}

/** The CLI line that installs an item (docs/cli.md). Pro items need Oxyn Pro to install. */
export function installLine(d: Demo): string {
  return `oxyn add ${d.slug}`;
}

/** Installs the `oxyn` CLI from the repository (no crates.io release yet). */
export const CLI_INSTALL = `cargo install --git ${REPO} oxyn-cli`;


// ── Library v2 (plan #29): eight categories + screens, affectUI-style pages ─────────────────
export type Group = { slug: string; name: string; blurb: string; family?: boolean };
export const groups = GROUPS as Group[];
export const screens = demos.filter((d) => isScreen(d));
export const components = demos.filter((d) => !isScreen(d) && !isBlock(d));
/** Blocks: larger compositions with their own section (/blocks/<group>/<slug>/), all Pro. */
export const blocks = demos.filter((d) => isBlock(d));
export const blockGroups = BLOCK_GROUPS.map((g) => ({ ...g, items: blocks.filter((d) => d.category === g.cat) }));
export { isBlock, blockGroupOf };
/** Agent apps: the screens of whole desktop agent apps, their own category after Screens. */
export const agentApps = screens.filter((d) => isAgentApp(d));
/** Arc screens: work-app screens on the Arc frame, their own category after Agent apps. */
export const arcScreens = ARC_SCREENS.map((s) => screens.find((d) => d.slug === s)!).filter(Boolean);
export const appScreens = screens.filter((d) => !isAgentApp(d) && !isArcScreen(d));
export const groupOf = (d: Demo) => groups.find((g) => g.slug === groupSlugOf(d))!;
// Within a category, the agent-specific parts come first (approvals before status text).
const CAT_ORDER = ['approvals', 'tools', 'checkpoints-git', 'composer', 'chat', 'status-text'];
const rank = (d: Demo) => { const i = CAT_ORDER.indexOf(d.category); return i < 0 ? CAT_ORDER.length : i; };
export const inGroup = (g: string) => components.filter((d) => groupSlugOf(d) === g).sort((a, b) => rank(a) - rank(b));
export { isScreen };
/** Newest components: the ones the latest parity plan that built components added (docs/parity/status.csv).
 *  A plan that only built screens is skipped, so the promo strip and /components/new/ never go empty. */
const builtRows = parity.filter((r) => r.status.startsWith('built'));
const planNo = (r: { plan_pr?: string }) => parseInt((r.plan_pr || '#0').slice(1)) || 0;
const specsOf = (n: number) => new Set(builtRows.filter((r) => planNo(r) === n).flatMap((r) => (r.oxyn_spec || '').split(',').map((x) => x.trim())));
const plansDesc = [...new Set(builtRows.map(planNo))].filter((n) => n > 0).sort((a, b) => b - a);
const inPlan = (n: number) => { const sp = specsOf(n); return components.filter((d) => sp.has(d.spec) || sp.has(d.slug)); };
export const newDemos = (() => { for (const n of plansDesc) { const v = inPlan(n); if (v.length) return v; } return [] as typeof components; })();
export const motionDemos = ['text-shimmer', 'verb-cycler', 'text-reveal', 'number-ticker', 'glyph-loader'].map((s) => demos.find((d) => d.slug === s)!).filter(Boolean);
/** Screens composed on the site from several components (no single Rust demo). */
export const composedScreens = [{ name: 'Agent workspace', href: '/screens/agent-workspace/', summary: 'A complete AI agent app: threads, the agent thread and the work pane.' }];
export const counts = {
  components: components.length,
  proComponents: components.filter((d) => !isFree(d)).length,
  free: demos.filter((d) => isFree(d)).length,
  screens: screens.length + composedScreens.length,
  agentApps: agentApps.length,
  arcScreens: arcScreens.length,
  blocks: blocks.length,
  illustrations: 0,
  motion: motionDemos.length,
  variants: demos.reduce((n, d) => n + d.variants.length, 0),
};
/** Oxyn Pro (docs/pricing.md): everything outside the Free tier (src/lib/free-tier.json) is Pro:
 *  every screen, agent-app template, block and AI special component, and every component not in
 *  the Free list. Pro gets the bolt marker and a locked code panel for accounts without Pro.
 *  Previews stay open to everyone. */
export const isPro = (d: Demo) => !isFree(d);
// Pro code never reaches a static page: some server-rendered previews carry the playground's code
// panel, so it is cut from every Pro demo's markup here (Pro accounts get the code from the Worker).
const stripCode = (h: string) => h.replace(/<div class="oxd-code">[\s\S]*?<\/pre><\/div>/g, '');
for (const d of demos) {
  if (!isPro(d)) continue;
  d.html = stripCode(d.html);
  for (const v of d.variants) v.html = stripCode(v.html);
}
/** Plan prices shown on the site, in US dollars before tax (Polar adds tax at checkout). Keep in
 *  step with PLANS in worker/billing.ts. */
export const PRICES = {
  pro: { month: 19.99, year: 199.99 },
  founder: { month: 12.99, year: 124.45, cap: 100 },
  team: { month: 44.99, year: 449, people: 4, seats: 3 }, // up to 4 people: the owner plus 3 teammates
};

const firstPara = (s: string | null) => {
  if (!s) return '';
  const para = s.split(/\n\s*\n/)[0];
  return para.split('\n').filter((l) => !/^\s*[-*] /.test(l)).join(' ').trim();
};
const plain = (s: string) => s.replace(/\*\*([^*]+)\*\*/g, '$1').replace(/`([^`]+)`/g, '$1').replace(/\[([^\]]+)\]\([^)]+\)/g, '$1').replace(/\s*\([^)]*§[^)]*\)/g, '');

/** "When to use": the spec's Purpose, split into short points. */
export function whenToUse(d: Demo): string[] {
  const p = plain(firstPara(specSection(d.spec, 'Purpose')));
  const sentences = (p || d.summary).split(/(?<=[.!?])\s+(?=[A-Z])/).map((x) => x.trim()).filter(Boolean);
  return sentences.slice(0, 4);
}

/** Related components named in the spec's Related section; same-category neighbours fill up. */
export function related(d: Demo, n = 4): Demo[] {
  const sec = specSection(d.spec, 'Related') ?? '';
  const names = [...sec.matchAll(/([a-z0-9-]+)\.md/g)].map((m) => m[1]);
  const out: Demo[] = [];
  for (const nm of names) {
    const hit = demos.find((x) => x.slug !== d.slug && (x.spec === nm || x.slug === nm || 'composer-' + x.slug === nm));
    if (hit && !out.includes(hit)) out.push(hit);
    if (out.length >= n) return out;
  }
  const pool = isScreen(d) ? screens : isBlock(d) ? blocks : inGroup(groupSlugOf(d));
  for (const x of pool) {
    if (x.slug !== d.slug && !out.includes(x)) out.push(x);
    if (out.length >= n) break;
  }
  return out;
}

/** "When not to use": the jobs that a related component does directly (from their summaries). */
export function whenNotToUse(d: Demo): { text: string; demo: Demo }[] {
  return related(d, 3).map((r) => ({ text: r.summary.split(/[.:–;]/)[0].trim(), demo: r }));
}

/** Keys of a keyboard row as chips: "Shift + Enter" → ["Shift", "Enter"]. Alternatives split on " / ". */
export function keyChips(k: string): string[][] {
  return k.split(/\s+\/\s+|,\s+/).map((alt) => alt.split(/\s*\+\s*/).map((x) => x.trim()).filter(Boolean));
}

/** Notes for coding agents, built only from what the component ships. */
export function notesForAi(d: Demo, propNames: string[]): string[] {
  const comps = d.dioxus.split(',').map((s) => s.trim());
  const out = [
    `Import ${comps.map((c) => '`' + c + '`').join(', ')} from \`oxyn_dioxus\` and include \`${cssConst(d.source)}\` plus the token stylesheet once.`,
    d.gpui ? `A GPUI version exists: \`${d.gpui}\` in \`oxyn_gpui\`.` : 'There is no GPUI version yet; use the Dioxus component.',
  ];
  if (propNames.length) out.push(`Props to set first: ${propNames.slice(0, 6).map((p) => '`' + p + '`').join(', ')}.`);
  if (d.live) out.push('It reads a clock (`now`) for live timers; pass one clock for the whole page rather than one per component.');
  out.push('Keep its states and timings as they ship; restyle it through the CSS variables rather than by hand.');
  return out;
}

export const GUIDES = [
  { href: '/guides/agent-activity/', title: 'Show what your agent is doing', blurb: 'Status, tool calls and plans while a run is going.', for: ['agents', 'feedback'] },
  { href: '/guides/approvals/', title: 'Ask before acting', blurb: 'Approval cards, risk tiers, countdowns and what happens after.', for: ['agents', 'cards'] },
  { href: '/guides/composer/', title: 'Build a composer', blurb: 'Send and stop, queue and steer, models, effort and attachments.', for: ['chat', 'inputs', 'buttons'] },
];
export const DOC_CARDS = [
  { href: '/docs/installation/', title: 'Installation', blurb: 'Add the crates and the stylesheets.' },
  { href: '/docs/theming/', title: 'Theming and tokens', blurb: 'Light, dark and high-contrast from one token set.' },
  { href: '/docs/accessibility/', title: 'Accessibility', blurb: 'Labels, focus and announcements across the library.' },
];
export function guidesFor(d: Demo) {
  const g = isScreen(d) ? 'chat' : groupSlugOf(d);
  const hits = GUIDES.filter((x) => x.for.includes(g));
  return hits.length ? hits : DOC_CARDS;
}

/** The library in one line, from the data (never hand-written numbers). */
export const countsText = () =>
  `${counts.free} free components, ${counts.proComponents} Pro components, ${counts.screens} screens and ${counts.blocks} blocks`;
export const endLine = () =>
  `Oxyn · ${counts.free} free components · ${counts.proComponents} Pro components · ${counts.screens} screens · ${counts.blocks} blocks · Dioxus and GPUI`;
