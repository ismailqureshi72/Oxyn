// Node module hooks for the Worker tests (see register.mjs).
import { existsSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

export async function resolve(spec, ctx, next) {
  if ((spec.startsWith('./') || spec.startsWith('../')) && !/\.(ts|mjs|js|json|html|css)$/.test(spec)) {
    const ts = new URL(spec + '.ts', ctx.parentURL);
    if (existsSync(fileURLToPath(ts))) spec += '.ts';
  }
  return next(spec, ctx);
}

export async function load(url, ctx, next) {
  if (/\/admin-ui\/[^/]+\.(html|css|js)$/.test(url)) {
    const text = await readFile(fileURLToPath(url), 'utf8');
    return { format: 'module', source: `export default ${JSON.stringify(text)};`, shortCircuit: true };
  }
  if (url.startsWith('file:') && url.endsWith('.json')) {
    const text = await readFile(fileURLToPath(url), 'utf8');
    return { format: 'module', source: `export default ${text};`, shortCircuit: true };
  }
  return next(url, ctx);
}
