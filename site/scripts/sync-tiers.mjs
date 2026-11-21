// Keeps the `oxyn_tier` column of docs/parity/status.csv in step with the Free list
// (src/lib/free-tier.json, the single source of truth; docs/pricing.md). Run from `site/` after
// `npm run gen` (it needs src/generated/demos.json for the slugs that exist):
//
//   node scripts/sync-tiers.mjs           rewrites the column
//   node scripts/sync-tiers.mjs --check   exits 1 when the column is stale (CI)
//
// A row's tier comes from the Oxyn items it maps to (the slugs in `oxyn_spec`): Free when all of
// them are free, Pro when none is, Mixed otherwise; empty when it maps to no built item. The
// `tier` column stays what it is: the source library's own tier.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { FREE } from '../src/lib/taxonomy.mjs';

const site = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const csvPath = path.resolve(site, '../docs/parity/status.csv');
const { demos } = JSON.parse(fs.readFileSync(path.join(site, 'src/generated/demos.json'), 'utf8'));
const slugs = new Set(demos.map((d) => d.slug));

const parse = (line) => {
  const out = [];
  let cur = '', q = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (q) {
      if (c === '"' && line[i + 1] === '"') { cur += '"'; i++; }
      else if (c === '"') q = false;
      else cur += c;
    } else if (c === '"') q = true;
    else if (c === ',') { out.push(cur); cur = ''; }
    else cur += c;
  }
  out.push(cur);
  return out;
};
const quote = (v) => (/[",\n]/.test(v) ? `"${v.replace(/"/g, '""')}"` : v);

const raw = fs.readFileSync(csvPath, 'utf8');
const lines = raw.replace(/\n$/, '').split('\n');
const head = parse(lines[0]);
let col = head.indexOf('oxyn_tier');
const rows = lines.slice(1).map(parse);
if (col < 0) {
  col = head.indexOf('tier') + 1;
  head.splice(col, 0, 'oxyn_tier');
  rows.forEach((r) => r.splice(col, 0, ''));
}
const specCol = head.indexOf('oxyn_spec');
for (const r of rows) {
  // `a (note, more), b` → [a, b]: drop parentheticals, then split.
  const spec = (r[specCol] || '').replace(/\([^)]*\)/g, '');
  const mine = spec.split(/[,;]/).map((s) => s.trim().split(/\s+/)[0]).filter((s) => slugs.has(s));
  const free = mine.filter((s) => FREE.has(s)).length;
  r[col] = !mine.length ? '' : free === mine.length ? 'Free' : free === 0 ? 'Pro' : 'Mixed';
}
const out = [head, ...rows].map((r) => r.map(quote).join(',')).join('\n') + '\n';
if (process.argv.includes('--check')) {
  if (out !== raw) {
    console.error('docs/parity/status.csv: oxyn_tier is out of date with src/lib/free-tier.json; run `node scripts/sync-tiers.mjs` in site/');
    process.exit(1);
  }
  console.log('status.csv tiers match the Free list');
} else {
  fs.writeFileSync(csvPath, out);
  const n = (t) => rows.filter((r) => r[col] === t).length;
  console.log(`status.csv: ${n('Free')} Free, ${n('Mixed')} Mixed, ${n('Pro')} Pro, ${rows.length - n('Free') - n('Mixed') - n('Pro')} unmapped`);
}
