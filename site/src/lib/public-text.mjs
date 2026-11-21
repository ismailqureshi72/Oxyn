// Public copy only: strips the names of the reference libraries a demo was studied from and the
// internal plan numbers from text that reaches a page, the search index or the CLI registry
// (demo summaries, variant notes, spec sections, prop docs). The Rust sources and specs keep their
// own notes; this runs on what the site prints. Used by src/lib/data.ts and scripts/gen.mjs.

const REF = String.raw`(?:affect ?UI|ui ?arc|bencho|kshv|Beautiful ?UI|Arc UI|Arc(?=['’]s\b)|Arc(?= (?:screens?|demo app|frame|gaps?|parity|reference|version|style|spec|ships|has|does|uses|draws|shows|keeps|puts|look)\b)|[Pp]lan #\d+|parity (?:plan|tracker|item|row|section)s?)`;
const REF_RE = new RegExp(REF, 'i');

/** Exact rewrites first, so a breadcrumb or a lead-in keeps reading well. */
const REWRITES = [
  [/\bArc screens \/ /g, 'Work screens / '],
  [/\bArc demo app \/ /g, 'Admin app / '],
  [/\bArc screens\b/g, 'Work screens'],
  [/\b(Clean|Streaming|Default|Classic), as affect ?UI ships it:\s*/gi, '$1: '],
  [/\bAs affect ?UI ships it:\s*/gi, ''],
  [/\bOxyn extra \(beyond affect ?UI\):\s*(?:the )?(?:plan #\d+ )?/gi, 'Extended: the '],
  [/\bOxyn-only extra, with no affect ?UI counterpart:\s*/gi, ''],
  [/\s*Kept as our own frame beside the affect ?UI-matched shells\./gi, ''],
  [/\baffect ?UI style:\s*/gi, ''],
  [/\bBeautiful ?UI['’]s [^:.]*? ported:\s*/gi, ''],
  [/\baffect ?UI['’]s menu:\s*/gi, 'Menu: '],
  [/\s*\(Arc start \/ end\)/g, ' (start / end)'],
  [/\s*[–—-]\s*Arc UI['’]s [^.]*?(?=\.|$)/g, ''],
  [/\b(?:the )?plan #\d+ /gi, 'the '],
];

const cap = (s) => s.replace(/^(\s*)([a-z])/, (_, w, c) => w + c.toUpperCase());

/** One line or paragraph of public copy. */
export function publicText(s) {
  if (typeof s !== 'string' || !s) return s;
  if (!REF_RE.test(s) && !/affect ?UI|Arc screens|plan #\d/i.test(s)) return s;
  let t = s;
  for (const [re, to] of REWRITES) t = t.replace(re, to);
  // Parentheticals that only cite a reference or a plan: "(Beautiful UI, MIT)", "(plan #23)".
  t = t.replace(new RegExp(String.raw`\s*\((?:[^()]|\([^()]*\))*?${REF}(?:[^()]|\([^()]*\))*?\)`, 'gi'), '');
  t = t.replace(/\bArc['’]s (\w)/g, (_, c) => 'A ' + c);
  // Whole sentences that still name one.
  if (REF_RE.test(t)) {
    t = t
      .split(/(?<=[.!?])\s+/)
      .filter((x) => !REF_RE.test(x))
      .join(' ');
  }
  t = t.replace(/\s+([,.;:])/g, '$1').replace(/ {2,}/g, ' ').replace(/^\s*[–—-]\s*/, '');
  return cap(t.trim()) || '';
}

/** Markdown (spec sections): line by line, keeping list markers and table rows. */
export function publicMarkdown(md) {
  if (typeof md !== 'string' || !md) return md;
  return md
    .split('\n')
    .map((line) => {
      if (!REF_RE.test(line) && !/affect ?UI|Arc screens|plan #\d/i.test(line)) return line;
      const m = line.match(/^(\s*(?:[-*+]|\d+\.|>|\|)?\s*)(.*)$/);
      const body = publicText(m[2]);
      return body ? m[1] + body : null;
    })
    .filter((x) => x !== null)
    .join('\n');
}

/** True when text still names a reference library or a plan number (used by checks). */
export const leaks = (s) => typeof s === 'string' && REF_RE.test(s);
