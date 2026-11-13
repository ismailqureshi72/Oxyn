// Build-time data for the site, all generated from the Rust workspace by scripts/gen.mjs.
import demosJson from '../generated/demos.json';
import propsJson from '../generated/props.json';
import specsJson from '../generated/specs.json';
import parityJson from '../generated/parity.json';
import wasmJson from '../generated/wasm.json';
import motionJson from '../generated/motion.json';

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
export const wasm = wasmJson as { js: string | null; wasm: string | null; bytes: number };
export const motion = motionJson as Record<string, any>;

export const demosIn = (cat: string) => demos.filter((d) => d.category === cat);
export const categoryOf = (slug: string) => categories.find((c) => c.slug === slug)!;
export const demoUrl = (d: Demo) => `/components/${d.category}/${d.slug}/`;

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

/** Installation line for a component (the CLI ships later; today it is a git dependency). */
export function installLine(d: Demo): string {
  return `cargo add oxyn-dioxus --git ${REPO}`;
}

/** Items of a parity section from status.csv (affectUI rows by section prefix). */
export function parityItems(section: string) {
  return parity.filter((r) => r.source === 'affectUI' && r.section.startsWith(section));
}
