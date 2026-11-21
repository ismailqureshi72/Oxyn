// Scores every component by the amount of work in it and prints the top N as the Free tier
// candidates (docs/pricing.md, "Choosing the Free set"). Run from `site/` after `npm run gen`:
//
//   node scripts/score-free.mjs [N=74]          prints the ranking
//   node scripts/score-free.mjs 74 --write      also rewrites src/lib/free-tier.json
//
// Only components are candidates: screens, agent apps, blocks and the AI special components are
// Pro by definition. The list it writes is the single source of truth for Free: the site (isPro),
// the Worker (plan_for gating of /api/pro/source), the registry builder and docs/parity/status.csv
// all read it. Re-running it is a deliberate pricing change, never part of a build.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { SCREENS, PRO_COMPONENTS, BLOCK_GROUPS } from '../src/lib/taxonomy.mjs';

const site = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const root = path.resolve(site, '..');
const gen = path.join(site, 'src/generated');
const N = Number(process.argv[2] || 74);
const { demos } = JSON.parse(fs.readFileSync(path.join(gen, 'demos.json'), 'utf8'));
const specs = JSON.parse(fs.readFileSync(path.join(gen, 'specs.json'), 'utf8'));
const props = JSON.parse(fs.readFileSync(path.join(gen, 'props.json'), 'utf8'));
const csv = fs.readFileSync(path.join(root, 'docs/parity/status.csv'), 'utf8');

const blockCats = new Set(BLOCK_GROUPS.map((g) => g.cat));
const candidates = demos.filter((d) => !SCREENS.includes(d.slug) && !PRO_COMPONENTS.includes(d.slug) && !blockCats.has(d.category));

const scored = candidates.map((d) => {
  const spec = specs[d.spec] || { sections: {} };
  const specChars = Object.values(spec.sections).join('').length;
  const p = props[d.dioxus]?.props?.length ?? 0;
  const builtRows = csv.split('\n').filter((l) => l.includes(d.slug) && /,built/.test(l)).length;
  const parts = {
    variants: Math.min(d.variants?.length ?? 0, 12) * 3,
    controls: Math.min(d.controls?.length ?? 0, 12) * 2,
    gpui: d.gpui ? 8 : 0,
    spec: Math.min(Object.keys(spec.sections).length, 12) + Math.min(specChars / 1500, 8),
    a11y: Math.min((d.a11y?.length ?? 0) + (d.keyboard?.length ?? 0) + (d.responsive?.length ?? 0), 12) * 0.5,
    props: Math.min(p, 16) * 0.5,
    parity: Math.min(builtRows, 3) * 2,
  };
  const score = Object.values(parts).reduce((a, b) => a + b, 0);
  return { slug: d.slug, name: d.name, category: d.category, score: Math.round(score * 10) / 10, parts };
});
scored.sort((a, b) => b.score - a.score || a.slug.localeCompare(b.slug));
// The Free set must be closed: a free component that renders another component needs that one free
// too. The registry index (public/r/index.json from `npm run gen`) has each item's dependencies; an
// item joins with the deps it pulls in, in score order, while the set stays within N.
const idxPath = path.join(site, 'public/r/index.json');
const deps = {};
if (fs.existsSync(idxPath)) {
  for (const e of JSON.parse(fs.readFileSync(idxPath, 'utf8')).items) deps[e.slug] = [...new Set(Object.values(e.oxyn_deps || {}).flat())].filter((d) => d !== 'theme');
} else console.warn('no public/r/index.json: picking without dependency closure');
const isCandidate = new Set(candidates.map((c) => c.slug));
const need = (slug, acc = new Set()) => {
  for (const d of deps[slug] || []) if (!acc.has(d)) { acc.add(d); need(d, acc); }
  return acc;
};
const chosen = new Set();
const pulledBy = {};
for (const s of scored) {
  if (chosen.has(s.slug)) continue;
  const req = [...need(s.slug)].filter((d) => !chosen.has(d));
  if (req.some((d) => !isCandidate.has(d))) continue; // renders a screen / block / AI component
  if (chosen.size + 1 + req.length > N) continue;
  chosen.add(s.slug);
  for (const d of req) { chosen.add(d); pulledBy[d] = s.slug; }
}
const top = scored.filter((s) => chosen.has(s.slug));
for (const [i, s] of scored.entries()) {
  if (i < N + 15 || chosen.has(s.slug)) console.log(`${String(i + 1).padStart(3)} ${s.score.toFixed(1).padStart(5)} ${chosen.has(s.slug) ? 'FREE' : '    '} ${s.slug} (${s.category})${pulledBy[s.slug] ? ` <- needed by ${pulledBy[s.slug]}` : ''}`);
}
console.log(`${candidates.length} candidates; free = ${top.length} (top by score, closed under dependencies, at most ${N})`);
if (process.argv.includes('--write')) {
  const out = {
    about: 'The Free tier: the only items whose code is free (site, CLI registry, Worker). Everything else is Oxyn Pro. Chosen by site/scripts/score-free.mjs; see docs/pricing.md.',
    method: 'score = 3*variants(<=12) + 2*controls(<=12) + 8 if a GPUI build exists + spec sections(<=12) + spec length/1500 chars(<=8) + 0.5*(a11y+keyboard+responsive notes, <=12) + 0.5*props(<=16) + 2*built parity rows(<=3); components only; taken in score order together with the components they render (so the set is closed), up to N.',
    free: top.map((s) => s.slug).sort(),
    scores: Object.fromEntries(top.map((s) => [s.slug, s.score])),
  };
  fs.writeFileSync(path.join(site, 'src/lib/free-tier.json'), JSON.stringify(out, null, 2) + '\n');
  console.log(`wrote src/lib/free-tier.json (${top.length} free)`);
}
