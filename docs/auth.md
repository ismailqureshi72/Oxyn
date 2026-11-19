# Site accounts (sign in)

People can sign in on the public site with **GitHub** or with an **email and a password**. Accounts
are optional: every free component works without one. They are separate from the maintainers' admin
panel ([admin.md](admin.md)), which keeps its own Cloudflare Access gate and sessions.

Everything runs on the existing Cloudflare Worker (`site/worker/auth.ts`) and D1 database `oxyn`.
No other service is involved.

## Pages

| Page | What it does |
| --- | --- |
| `/login/` | "Continue with GitHub", or email + password. `?next=/some/page/` returns there afterwards. |
| `/signup/` | Same card, creates an account (password at least 10 characters). |
| `/account/` | Email, sign-in methods, member since, **Sign out**, **Sign out of all sessions**. |

All three are static, `noindex`, and left out of the site search. The navbar on every page (landing,
pricing, library, docs) shows **Sign in** when signed out and an avatar menu (email, Account, Sign out)
when signed in. Pages stay fully cacheable: `public/account.js` asks `GET /api/auth/me` from the
browser, and on ordinary pages only when the `oxyn_signed_in` hint cookie says a session may exist,
so signed-out visitors never wake the Worker.

## API (`/api/auth/*`)

| Route | |
| --- | --- |
| `GET /api/auth/me` | `{ok, signed_in, github, user?}`. Always 200; `signed_in:false` when nobody is signed in. `github` says whether GitHub sign-in is configured. |
| `POST /api/auth/signup` | JSON `{email, password}`. 201 + session cookie. |
| `POST /api/auth/login` | JSON `{email, password}`. 200 + session cookie, or 401 `login_failed`. |
| `POST /api/auth/logout` | Deletes this session row and clears the cookies. |
| `POST /api/auth/logout-all` | Deletes every session of the account. |
| `GET /api/auth/github/start` | Redirects to GitHub with a CSRF `state`. **503** `github_unavailable` until the OAuth app is set up. |
| `GET /api/auth/github/callback` | Checks `state`, exchanges the code, signs in, redirects to `next` (default `/account/`). Errors go to `/login/?error=…`. |

Errors are JSON `{ok:false, error:{code, message, field?}}`, like the rest of the API. POSTs must come
from the site itself (Origin or Referer check), and only `application/json` is accepted.

## Security notes

- **Passwords**: PBKDF2-SHA256, 100 000 iterations (the Workers maximum), 16-byte random salt, via
  WebCrypto. Length 10 to 256.
- **Sessions**: a random 256-bit token in `oxyn_session` (HttpOnly, Secure, SameSite=Lax, Path=/,
  30 days). D1 keeps only its SHA-256 (`sessions.token_hash`), so a copy of the table cannot be
  replayed, and signing out is a row delete on the server. `oxyn_signed_in=1` is a readable hint
  with no secret in it.
- **No account enumeration on sign-in**: a wrong password, an unknown email and a GitHub-only
  account all get the same 401 message, and an unknown email still runs PBKDF2 so the timing matches.
  Sign-up with an email that already has an account signs in when the password matches and otherwise
  gets the same generic refusal; without email verification a sign-up form cannot hide entirely that
  an address is taken (see Follow-ups).
- **Rate limits** (fixed windows in `rate_limits`, keyed by a salted hash of the IP or email):
  sign-up 5 per 10 minutes and 20 per day per IP; sign-in 10 per 10 minutes and 100 per day per IP,
  plus 8 per 10 minutes per email; GitHub start 20 per 10 minutes per IP. Over the limit: 429
  `rate_limited` with `Retry-After`.
- **GitHub**: authorization code flow, scopes `read:user user:email`. The `state` is 256 random bits
  in an HttpOnly cookie scoped to `/api/auth/github`, valid 10 minutes, compared in constant time.
  Only a **verified** GitHub email is accepted. If that email already belongs to a password account
  (whose email was never verified), the GitHub login is linked, the password is removed and the
  account's other sessions end, so nobody can pre-register someone else's address and keep a way in.
- `next` only accepts a path on this site (no `//host`, no `/api/`, no `/admin`).

## Turning on "Continue with GitHub"

Until these two Worker secrets exist, the button is shown disabled with "GitHub sign-in coming soon"
and both GitHub routes answer 503. To turn it on (a maintainer with access to the GitHub account or
organisation that should own the app):

1. GitHub → **Settings → Developer settings → OAuth Apps → New OAuth App**
   (for an organisation: the organisation's **Settings → Developer settings → OAuth Apps**).
2. Fill in:
   - **Application name**: `Oxyn`
   - **Homepage URL**: `https://ui.oxyn.workers.dev`
   - **Authorization callback URL**: `https://ui.oxyn.workers.dev/api/auth/github/callback`
   - Leave **Enable Device Flow** off.
3. **Register application**, then **Generate a new client secret**. Copy the Client ID and the secret
   (the secret is shown once).
4. From `site/` (signed in to the Cloudflare account with `wrangler login` or `CLOUDFLARE_API_TOKEN`):

   ```sh
   npx wrangler secret put GITHUB_OAUTH_CLIENT_ID      # paste the Client ID
   npx wrangler secret put GITHUB_OAUTH_CLIENT_SECRET  # paste the client secret
   ```

   Secrets take effect at once; no redeploy is needed. Or set them in the Cloudflare dashboard:
   Workers & Pages → **ui** → Settings → Variables and Secrets (type: Secret).
5. Open `/login/`: the GitHub button is now active. Sign in once to check.

If the site moves to another domain, add that domain's `/api/auth/github/callback` to the OAuth app
(GitHub allows one callback URL per OAuth app; create a second app for a second domain).

## Local development

```sh
cd site
npx wrangler d1 migrations apply oxyn --local
npm run build:site
npx wrangler dev worker/dev.ts --var GITHUB_OAUTH_CLIENT_ID:dev --var GITHUB_OAUTH_CLIENT_SECRET:dev
```

`worker/dev.ts` answers only on `localhost`/`127.0.0.1`. With the two `dev` vars, "Continue with
GitHub" goes to a fake GitHub at `/__dev/github/authorize` that signs in a test user (choose one with
`?as=<base64url JSON {id, login, email, verified, name}>`, or `?deny=1`). Everything after GitHub (the
callback, `state` check, account linking, sessions) is the production code. CI fails if the fake
GitHub or the admin Access stub is ever in the deploy bundle (`.github/workflows/ci.yml`, "dev stubs
are not deployed").

## Data

Migration `site/migrations/0003_users.sql` (additive): `users`, `oauth_accounts`, `sessions`.
Apply remotely with `npx wrangler d1 migrations apply oxyn --remote` before deploying a Worker that
uses it. Deleting a user row cascades to its sessions and linked GitHub account.

## Follow-ups (not built yet)

- **Email verification and password reset**: both need sending email, which needs a mail service.
  Until then: an email account cannot reset a forgotten password (signing in with GitHub on the same
  verified email takes the account over and works as a reset), and sign-up can reveal that an
  address is taken to someone who tries it.
- Changing the email or password from `/account/`, deleting an account from the UI, more providers.
- Admin view of site accounts in `/admin`.
