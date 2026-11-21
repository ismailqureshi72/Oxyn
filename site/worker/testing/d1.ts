// An in-memory D1 for the Worker tests: node:sqlite with every migration in site/migrations
// applied, behind the small D1 surface the Worker uses (prepare/bind/first/all/run, batch).
import { DatabaseSync } from 'node:sqlite';
import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { D1, D1Stmt } from '../shared.ts';

const norm = (v: unknown[]) => v.map((x) => (x === undefined ? null : typeof x === 'boolean' ? (x ? 1 : 0) : x)) as (string | number | null)[];

export function memoryD1(): D1 & { raw: DatabaseSync } {
  const db = new DatabaseSync(':memory:');
  const dir = fileURLToPath(new URL('../../migrations/', import.meta.url));
  for (const f of readdirSync(dir).filter((f) => f.endsWith('.sql')).sort()) db.exec(readFileSync(dir + f, 'utf8'));
  const stmt = (q: string, args: unknown[] = []): D1Stmt => ({
    bind: (...v: unknown[]) => stmt(q, v),
    first: async <T>() => ((db.prepare(q).get(...norm(args)) as T | undefined) ?? null),
    all: async <T>() => ({ results: db.prepare(q).all(...norm(args)) as T[], meta: {} }),
    run: async () => {
      const p = db.prepare(q);
      if (/^\s*(SELECT|WITH)\b/i.test(q) || /\bRETURNING\b/i.test(q)) {
        const rows = p.all(...norm(args));
        return { results: rows, meta: { changes: rows.length } };
      }
      const r = p.run(...norm(args));
      return { results: [], meta: { changes: Number(r.changes) } };
    },
  });
  return {
    raw: db,
    prepare: (q: string) => stmt(q),
    batch: async (list: D1Stmt[]) => {
      const out = [];
      for (const s of list) out.push(await s.run());
      return out;
    },
  };
}
