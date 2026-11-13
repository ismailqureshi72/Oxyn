// Generates everything the site reads from the Rust workspace. Run from `site/`.
//
//   node scripts/gen.mjs          site data only (SSR previews, props, specs, parity, tokens)
//   node scripts/gen.mjs --wasm   also builds the live-demo WASM bundle into public/wasm/
//
// Outputs (all git-ignored): src/generated/*.json, src/generated/*.css, public/wasm/*.
// Needs the Rust toolchain from rust-toolchain.toml (+ wasm32-unknown-unknown and wasm-bindgen-cli
// 0.2.130 for --wasm; wasm-opt is used when it is on PATH).

import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const site = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const root = path.resolve(site, '..');
const gen = path.join(site, 'src/generated');
const wasmOut = path.join(site, 'public/wasm');
const args = new Set(process.argv.slice(2));
fs.mkdirSync(gen, { recursive: true });

const run = (cmd, argv, opts = {}) => {
  console.log(`$ ${cmd} ${argv.join(' ')}`);
  execFileSync(cmd, argv, { stdio: 'inherit', cwd: root, ...opts });
};
const has = (cmd) => {
  try {
    execFileSync(cmd, ['--version'], { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
};

// 1. Demo catalogue + server-rendered previews + component CSS (Rust).
if (!process.env.OXYN_SKIP_CARGO) {
  run('cargo', ['run', '--locked', '-q', '-p', 'oxyn-gallery-web', '--bin', 'site-data', '--', gen]);
}
fs.copyFileSync(path.join(root, 'tokens/dist/oxyn.css'), path.join(gen, 'oxyn-tokens.css'));

// 2. Props tables straight from the Dioxus component signatures and their doc comments.
const srcDir = path.join(root, 'crates/oxyn-dioxus/src');
const walk = (d) =>
  fs.readdirSync(d, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? walk(path.join(d, e.name)) : e.name.endsWith('.rs') ? [path.join(d, e.name)] : [],
  );
const props = {};
for (const file of walk(srcDir)) {
  const src = fs.readFileSync(file, 'utf8');
  const re = /((?:\/\/\/[^\n]*\n)*)#\[component\]\s*pub fn (\w+)\(\s*([\s\S]*?)\)\s*->\s*Element/g;
  for (const m of src.matchAll(re)) {
    const [, docBlock, name, body] = m;
    const list = [];
    let doc = [];
    let attr = null;
    for (const raw of body.split('\n')) {
      const s = raw.trim();
      if (s.startsWith('///')) doc.push(s.slice(3).trim());
      else if (s.startsWith('#[props')) attr = s;
      else {
        const pm = s.match(/^(?:mut )?(\w+)\s*:\s*(.+?),?$/);
        if (!pm) continue;
        let def = null;
        if (attr) {
          const dm = attr.match(/default\s*=\s*(.*)\)\]$/);
          def = dm ? dm[1].replace(/\.to_string\(\)$/, '') : 'Default::default()';
          if (pm[2].startsWith('EventHandler')) def = null;
        }
        const type = pm[2];
        list.push({
          name: pm[1],
          type,
          default: def,
          required: !attr && !type.startsWith('EventHandler') && !type.startsWith('Option<'),
          event: type.startsWith('EventHandler'),
          doc: doc.join(' '),
        });
        doc = [];
        attr = null;
      }
    }
    props[name] = {
      file: path.relative(root, file),
      doc: docBlock.replace(/^\/\/\/ ?/gm, '').trim(),
      props: list,
    };
  }
}
fs.writeFileSync(path.join(gen, 'props.json'), JSON.stringify(props));

// 3. Spec sections shown on component pages (accessibility, edge cases, timing).
const specDir = path.join(root, 'docs/specs');
const specs = {};
for (const f of fs.readdirSync(specDir).filter((f) => f.endsWith('.md'))) {
  const md = fs.readFileSync(path.join(specDir, f), 'utf8');
  const title = (md.match(/^#\s+(.+)$/m) || [, f])[1];
  const sections = {};
  const parts = md.split(/^##\s+/m).slice(1);
  for (const p of parts) {
    const nl = p.indexOf('\n');
    const head = p.slice(0, nl).replace(/^\d+\.\s*/, '').trim();
    sections[head] = p.slice(nl + 1).trim();
  }
  specs[f.replace(/\.md$/, '')] = { title, sections };
}
fs.writeFileSync(path.join(gen, 'specs.json'), JSON.stringify(specs));

// 4. Parity rows (what is built and what is planned), for Screens, Blocks, Illustrations, Motion.
const csv = fs.readFileSync(path.join(root, 'docs/parity/status.csv'), 'utf8');
const rows = [];
const parseLine = (line) => {
  const out = [];
  let cur = '';
  let q = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (q) {
      if (c === '"' && line[i + 1] === '"') {
        cur += '"';
        i++;
      } else if (c === '"') q = false;
      else cur += c;
    } else if (c === '"') q = true;
    else if (c === ',') {
      out.push(cur);
      cur = '';
    } else cur += c;
  }
  out.push(cur);
  return out;
};
const lines = csv.trim().split('\n');
const headers = parseLine(lines[0]);
for (const l of lines.slice(1)) {
  const cells = parseLine(l);
  rows.push(Object.fromEntries(headers.map((h, i) => [h, cells[i] ?? ''])));
}
fs.writeFileSync(path.join(gen, 'parity.json'), JSON.stringify(rows));

// 5. Motion tokens.
fs.copyFileSync(path.join(root, 'tokens/oxyn.motion.json'), path.join(gen, 'motion.json'));

// 6. The live-demo bundle.
const wasmJson = path.join(gen, 'wasm.json');
if (args.has('--wasm')) {
  run('cargo', [
    'build',
    '--locked',
    '--profile',
    'wasm',
    '--target',
    'wasm32-unknown-unknown',
    '-p',
    'oxyn-gallery-web',
    '--lib',
  ]);
  const target = process.env.CARGO_TARGET_DIR || path.join(root, 'target');
  const raw = path.join(target, 'wasm32-unknown-unknown/wasm/oxyn_gallery_web.wasm');
  const tmp = path.join(target, 'site-wasm');
  fs.rmSync(tmp, { recursive: true, force: true });
  run('wasm-bindgen', ['--target', 'web', '--no-typescript', '--out-dir', tmp, '--out-name', 'gallery', raw]);
  const bg = path.join(tmp, 'gallery_bg.wasm');
  if (has('wasm-opt')) {
    // Explicit features: --all-features emits imports Chromium rejects (docs/spike/rust-web.md).
    run('wasm-opt', [
      '-Oz',
      '--enable-bulk-memory',
      '--enable-nontrapping-float-to-int',
      '--enable-sign-ext',
      '--enable-mutable-globals',
      '--enable-reference-types',
      '--enable-multivalue',
      '--strip-debug',
      bg,
      '-o',
      bg,
    ]);
  } else {
    console.warn('wasm-opt not found: shipping the unoptimised bundle');
  }
  fs.rmSync(wasmOut, { recursive: true, force: true });
  fs.mkdirSync(wasmOut, { recursive: true });
  const wasm = fs.readFileSync(bg);
  const hash = createHash('sha256').update(wasm).digest('hex').slice(0, 10);
  const wasmName = `gallery-dioxus.${hash}.wasm`;
  const js = fs.readFileSync(path.join(tmp, 'gallery.js'));
  const jsHash = createHash('sha256').update(js).digest('hex').slice(0, 10);
  const jsName = `gallery-dioxus.${jsHash}.js`;
  fs.writeFileSync(path.join(wasmOut, wasmName), wasm);
  fs.writeFileSync(path.join(wasmOut, jsName), js);
  // wasm-bindgen inline JS snippets, imported relative to the glue file.
  const snippets = path.join(tmp, 'snippets');
  if (fs.existsSync(snippets)) fs.cpSync(snippets, path.join(wasmOut, 'snippets'), { recursive: true });
  fs.writeFileSync(wasmJson, JSON.stringify({ js: `/wasm/${jsName}`, wasm: `/wasm/${wasmName}`, bytes: wasm.length }));
  console.log(`wasm: ${wasmName} (${(wasm.length / 1024).toFixed(0)} KB raw)`);
} else if (!fs.existsSync(wasmJson)) {
  // No bundle: pages render their server-rendered previews and say the live demo is unavailable.
  fs.writeFileSync(wasmJson, JSON.stringify({ js: null, wasm: null, bytes: 0 }));
}
console.log('gen: done');
