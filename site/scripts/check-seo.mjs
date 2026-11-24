// After `astro build`: the search and answer-engine basics hold in site/dist (CI, site job).
// Every JSON-LD block parses, every indexable page has one canonical, a title and a description,
// the landing names Oxyn UI in its title and H1, and robots.txt, sitemap.xml and llms.txt are there.
import fs from 'node:fs';
import path from 'node:path';

const dist = path.resolve(process.argv[2] ?? 'dist');
const errors = [];
const fail = (m) => errors.push(m);
const pages = [];
const walk = (d) => { for (const e of fs.readdirSync(d, { withFileTypes: true })) { const f = path.join(d, e.name); if (e.isDirectory()) walk(f); else if (e.name.endsWith('.html')) pages.push(f); } };
walk(dist);

let blocks = 0;
for (const f of pages) {
  const html = fs.readFileSync(f, 'utf8');
  const rel = path.relative(dist, f);
  for (const m of html.matchAll(/<script[^>]*type="application\/ld\+json"[^>]*>([\s\S]*?)<\/script>/g)) {
    blocks++;
    try { JSON.parse(m[1]); } catch (e) { fail(`${rel}: JSON-LD does not parse (${e.message})`); }
  }
  if (/name="robots"[^>]+noindex/.test(html) || /http-equiv="refresh"/i.test(html) || rel.endsWith('404.html')) continue;
  const canon = html.match(/<link[^>]+rel="canonical"[^>]*>/g) ?? [];
  if (canon.length !== 1) fail(`${rel}: ${canon.length} canonical links`);
  else if (!/href="https:\/\/oxynui\.com\//.test(canon[0])) fail(`${rel}: canonical is not https://oxynui.com/…`);
  if (!/<title>[^<]+<\/title>/.test(html)) fail(`${rel}: no <title>`);
  if (!/<meta[^>]+name="description"[^>]+content="[^"]+"/.test(html)) fail(`${rel}: no meta description`);
}

const home = fs.readFileSync(path.join(dist, 'index.html'), 'utf8');
const title = /<title>([^<]+)<\/title>/.exec(home)?.[1] ?? '';
if (!title.startsWith('Oxyn UI') || title.length > 60) fail(`landing title "${title}" must start with Oxyn UI and stay under 60 characters`);
const h1 = /<h1[^>]*>([\s\S]*?)<\/h1>/.exec(home)?.[1].replace(/<[^>]+>/g, '') ?? '';
if (!h1.includes('Oxyn UI')) fail(`landing H1 "${h1}" does not name Oxyn UI`);
const desc = /<meta name="description" content="([^"]+)"/.exec(home)?.[1] ?? '';
if (!desc.startsWith('Oxyn UI is') || desc.length < 140 || desc.length > 160) fail(`landing description (${desc.length} chars) must start "Oxyn UI is" and be about 150-155 characters`);
for (const t of ['Organization', 'WebSite', 'SoftwareApplication', 'FAQPage']) if (!home.includes(`"@type":"${t}"`)) fail(`landing JSON-LD has no ${t}`);

const robots = fs.readFileSync(path.join(dist, 'robots.txt'), 'utf8');
if (!robots.includes('Sitemap: https://oxynui.com/sitemap.xml')) fail('robots.txt does not name sitemap.xml');
const sm = fs.readFileSync(path.join(dist, 'sitemap.xml'), 'utf8');
const urls = (sm.match(/<loc>/g) ?? []).length;
if (urls < 100) fail(`sitemap.xml lists only ${urls} pages`);
if (/\/(login|signup|account)\//.test(sm)) fail('sitemap.xml lists a noindex page');
const llms = fs.readFileSync(path.join(dist, 'llms.txt'), 'utf8');
if (!llms.startsWith('# Oxyn UI')) fail('llms.txt does not start with "# Oxyn UI"');
if (!fs.existsSync(path.join(dist, 'llms-full.txt'))) fail('no llms-full.txt');

console.log(`${pages.length} pages, ${blocks} JSON-LD blocks, ${urls} sitemap URLs, landing title "${title}" (${title.length}), description ${desc.length} chars`);
if (errors.length) {
  for (const e of errors.slice(0, 50)) console.log(`::error::${e}`);
  if (errors.length > 50) console.log(`… and ${errors.length - 50} more`);
  process.exit(1);
}
