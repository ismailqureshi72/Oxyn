// LOCAL DEVELOPMENT ONLY. Never deployed: wrangler.jsonc's `main` is worker/index.ts, and this
// file is only an entry point when named on the command line:
//
//   npx wrangler dev worker/dev.ts --var DEV_ADMIN_EMAIL:you@example.com      (from site/)
//
// It swaps the Cloudflare Access JWT check for a stub that trusts DEV_ADMIN_EMAIL, and only for
// requests to localhost / 127.0.0.1. CI fails if this stub ever shows up in the deploy bundle
// (see .github/workflows/ci.yml, "admin dev stub is not deployed"). Everything after the Access
// check (admin list, shared password, sessions) is the production code.
import worker from './index';
import { HttpError, type Ctx, type Env } from './shared';
import { makeAdminHandler } from './admin';

const devAdmin = makeAdminHandler(async (req, env) => {
  const host = new URL(req.url).hostname;
  const email = ((env as Env & { DEV_ADMIN_EMAIL?: string }).DEV_ADMIN_EMAIL || '').trim().toLowerCase();
  if (!['localhost', '127.0.0.1'].includes(host) || !email) throw new HttpError(401, 'access_required', 'OXYN_DEV_ACCESS_STUB: set DEV_ADMIN_EMAIL and use localhost.');
  return { email };
});

export default {
  fetch(req: Request, env: Env, ctx: Ctx) {
    const url = new URL(req.url);
    const p = url.pathname;
    if (p === '/admin' || p.startsWith('/admin/') || p === '/api/admin' || p.startsWith('/api/admin/')) return devAdmin(req, env, ctx, url);
    return worker.fetch(req, env, ctx);
  },
};
