# Site accounts (sign in)

There is **one sign-in for everyone**, visitors and maintainers alike: **Continue with Google**,
**Continue with GitHub**, or an **email and a password**. Accounts are optional: every free component
works without one. A maintainer is just an account whose email Google or GitHub verified and that is
on the admin list; they get an **Admin** item in the account menu that opens `/admin`
([admin.md](admin.md)).

Everything runs on the existing Cloudflare Worker (`site/worker/auth.ts`) and D1 database `oxyn`.
No other service is involved.

## Pages

| Page | What it does |
| --- | --- |
| `/login/` | "Continue with Google", "Continue with GitHub", or email + password. `?next=/some/page/` returns there afterwards. |
| `/signup/` | Same card, creates an account (password at least 10 characters). |
| `/account/` | Email, sign-in methods, member since, **Sign out**, **Sign out of all sessions**. |

All three are static, `noindex`, and left out of the site search. The navbar on every page (landing,
pricing, library, docs) shows **Sign in** when signed out and an avatar menu (email, Account, **Admin** for admins, Sign out)
when signed in. Pages stay fully cacheable: `public/account.js` asks `GET /api/auth/me` from the
browser, and on ordinary pages only when the `oxyn_signed_in` hint cookie says a session may exist,
so signed-out visitors never wake the Worker.

## API (`/api/auth/*`)

| Route | |
| --- | --- |
| `GET /api/auth/me` | `{ok, signed_in, github, google, is_admin, user?}`. Always 200; `signed_in:false` when nobody is signed in. `github` / `google` say whether that sign-in is configured. `is_admin` is true for a verified admin (the panel opens); `user.admin_needs_verification` is true for an admin email that has only signed in with a password. |
| `POST /api/auth/signup` | JSON `{email, password}`. 201 + session cookie. |
| `POST /api/auth/login` | JSON `{email, password}`. 200 + session cookie, or 401 `login_failed`. |
| `POST /api/auth/logout` | Deletes this session row and clears the cookies. |
| `POST /api/auth/logout-all` | Deletes every session of the account. |
| `GET /api/auth/github/start` | Redirects to GitHub with a CSRF `state`. **503** `github_unavailable` until the OAuth app is set up. |
| `GET /api/auth/github/callback` | Checks `state`, exchanges the code, signs in, redirects to `next` (default `/account/`). Errors go to `/login/?error=…`. |
| `GET /api/auth/google/start` | Redirects to Google with `state`, `nonce` and a PKCE S256 challenge. **503** `google_unavailable` until the OAuth client is set up. |
| `GET /api/auth/google/callback` | Checks `state`, exchanges the code with the PKCE verifier, checks the ID token, signs in, redirects to `next`. Errors go to `/login/?error=google_…`. |

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
  plus 8 per 10 minutes per email; GitHub start and Google start 20 per 10 minutes per IP each. Over the limit: 429
  `rate_limited` with `Retry-After`.
- **GitHub**: authorization code flow, scopes `read:user user:email`. The `state` is 256 random bits
  in an HttpOnly cookie scoped to `/api/auth/github`, valid 10 minutes, compared in constant time.
  Only a **verified** GitHub email is accepted. If that email already belongs to a password account
  (whose email was never verified), the GitHub login is linked, the password is removed and the
  account's other sessions end, so nobody can pre-register someone else's address and keep a way in.
- **Google**: OpenID Connect authorization code flow, scopes `openid email profile`, with PKCE
  (S256, a 64-character verifier) and a `nonce`. `state`, `nonce`, verifier and `next` live in one
  HttpOnly cookie scoped to `/api/auth/google`, valid 10 minutes; `state` is compared in constant
  time. The callback exchanges the code (client secret + verifier) and reads the ID token the token
  endpoint returns directly over TLS (OpenID Connect Core 3.1.3.7 lets the client rely on TLS
  instead of the token's signature in this case). It checks `iss` (`https://accounts.google.com`),
  `aud` (our client id, and `azp` when there are several audiences), `exp`, `iat` and the `nonce`,
  and accepts only `email_verified: true`. The account is keyed by Google's `sub` and linked by
  verified email exactly like GitHub (below).
- **Linking** (Google and GitHub): a provider identity signs in to the account it is already linked
  to, else to the account with the same verified email, else creates one. Linking to an account that
  was only ever an unverified email + password removes that password and ends its other sessions,
  so nobody can pre-register someone else's address and keep a way in. A provider sign-in sets
  `users.email_verified = 1`, which is what admits an admin email to `/admin`.
- `next` only accepts a path on this site (no `//host`, no `/api/`). `/admin` is allowed, so the
  panel's sign-in redirect comes back to it.

## Turning on "Continue with Google"

Until these two Worker secrets exist, the Google button is shown disabled ("Google sign-in coming
soon") and both Google routes answer 503. To turn it on (someone with access to a Google Cloud
project that should own the client, for example a new project named `Oxyn`):

1. https://console.cloud.google.com → pick or create the project.
2. **APIs & Services → OAuth consent screen** (in the newer console: **Google Auth Platform →
   Branding / Audience / Data access**):
   - User type **External**; app name `Oxyn`; support email and developer contact: a maintainer's
     address; app home page `https://oxynui.com`; authorised domains `oxynui.com` and
     `oxyn.workers.dev` (Google may ask for `workers.dev`).
   - Scopes: `openid`, `.../auth/userinfo.email`, `.../auth/userinfo.profile` (non-sensitive, so no
     Google review is needed).
   - **Publish app** (status *In production*). While it is in *Testing*, only the test users listed
     there can sign in.
3. **APIs & Services → Credentials → Create credentials → OAuth client ID**:
   - Application type **Web application**, name `Oxyn site`.
   - **Authorised JavaScript origins**: `https://oxynui.com` (and `https://ui.oxyn.workers.dev`,
     the original address)
   - **Authorised redirect URIs**: `https://oxynui.com/api/auth/google/callback` (and
     `https://ui.oxyn.workers.dev/api/auth/google/callback`)
   - **Create**, then copy the Client ID and the Client secret.
4. From `site/` (signed in to the Cloudflare account with `wrangler login` or `CLOUDFLARE_API_TOKEN`):

   ```sh
   npx wrangler secret put GOOGLE_OAUTH_CLIENT_ID      # paste the Client ID (…apps.googleusercontent.com)
   npx wrangler secret put GOOGLE_OAUTH_CLIENT_SECRET  # paste the client secret
   ```

   Secrets take effect at once; no redeploy is needed. Or set them in the Cloudflare dashboard:
   Workers & Pages → **ui** → Settings → Variables and Secrets (type: Secret).
5. Open `/login/`: "Continue with Google" is now active. Sign in once to check; an admin then sees
   **Admin** in the account menu.

If the site moves to another domain, add that domain's `/api/auth/google/callback` (and origin) to
the same OAuth client.

## Turning on "Continue with GitHub"

Until these two Worker secrets exist, the button is shown disabled with "GitHub sign-in coming soon"
and both GitHub routes answer 503. To turn it on (a maintainer with access to the GitHub account or
organisation that should own the app):

1. GitHub → **Settings → Developer settings → OAuth Apps → New OAuth App**
   (for an organisation: the organisation's **Settings → Developer settings → OAuth Apps**).
2. Fill in:
   - **Application name**: `Oxyn`
   - **Homepage URL**: `https://oxynui.com`
   - **Authorization callback URL**: `https://oxynui.com/api/auth/github/callback`
     (an OAuth App holds one callback URL; the old `https://ui.oxyn.workers.dev/...` one is not
     needed, since sign-in pages on that host redirect to https://oxynui.com)
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
npx wrangler dev worker/dev.ts \
  --var GOOGLE_OAUTH_CLIENT_ID:dev --var GOOGLE_OAUTH_CLIENT_SECRET:dev \
  --var GITHUB_OAUTH_CLIENT_ID:dev --var GITHUB_OAUTH_CLIENT_SECRET:dev
```

`worker/dev.ts` answers only on `localhost`/`127.0.0.1`. With those vars, "Continue with Google"
goes to a fake Google at `/__dev/google/authorize` that signs in a test user (choose one with
`?as=<base64url JSON {sub, email, email_verified, name}>`, or `?deny=1`). Its token endpoint checks
the PKCE verifier and returns an unsigned ID token; `as` can also set `iss`, `aud`, `nonce` or `exp`
to check that the callback refuses a wrong one. "Continue with GitHub" goes to a fake GitHub at
`/__dev/github/authorize` (`?as=<base64url JSON {id, login, email, verified, name}>`, `?deny=1`).
Everything after the provider (the callback, `state` / `nonce` / PKCE checks, account linking,
sessions, the admin panel) is the production code. CI fails if either fake provider is ever in the
deploy bundle (`.github/workflows/ci.yml`, "dev stubs are not deployed").

## Data

Migration `site/migrations/0003_users.sql` (additive): `users`, `oauth_accounts`, `sessions`.
`0004_unified_admin.sql` removes the old admin-only password and sessions (see [admin.md](admin.md)).
Apply remotely with `npx wrangler d1 migrations apply oxyn --remote` before deploying a Worker that
uses it. Deleting a user row cascades to its sessions and linked Google and GitHub accounts.

## Follow-ups (not built yet)

- **Email verification and password reset**: both need sending email, which needs a mail service.
  Until then: an email account cannot reset a forgotten password (signing in with Google or GitHub
  on the same verified email takes the account over and works as a reset), sign-up can reveal that
  an address is taken to someone who tries it, and an admin email needs one Google or GitHub sign-in
  before the admin panel opens.
- Changing the email or password from `/account/`, deleting an account from the UI.
- Admin view of site accounts in `/admin`.
