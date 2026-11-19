-- Oxyn site accounts (L2): people who sign in on the public site with GitHub or an email and a
-- password. Separate from the maintainers' admin panel (admins / admin_sessions in 0002).
-- Additive only, so it is safe to apply before the Worker that uses it. Apply with:
--   npx wrangler d1 migrations apply oxyn --remote   (from site/)

CREATE TABLE users (
  id             TEXT PRIMARY KEY,                         -- random UUID
  email          TEXT NOT NULL UNIQUE COLLATE NOCASE,      -- lower-cased on the way in
  email_verified INTEGER NOT NULL DEFAULT 0,               -- 1 = a provider (GitHub) vouched for it
  name           TEXT,                                     -- display name from GitHub, if any
  avatar_url     TEXT,                                     -- GitHub avatar, if any
  password       TEXT,                                     -- JSON {alg,iter,salt,hash}; NULL = no password
  created_at     TEXT NOT NULL,                            -- ISO 8601, UTC
  last_login_at  TEXT
);

-- One row per linked sign-in provider account (today only 'github').
CREATE TABLE oauth_accounts (
  provider    TEXT NOT NULL,
  provider_id TEXT NOT NULL,                               -- GitHub numeric user id, as text
  user_id     TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  login       TEXT,                                        -- GitHub username at last sign-in
  created_at  TEXT NOT NULL,
  PRIMARY KEY (provider, provider_id)
);
CREATE INDEX oauth_accounts_user ON oauth_accounts (user_id);

-- Signed-in sessions. The cookie carries a random 256-bit token; only its SHA-256 is stored, so a
-- leaked table cannot be replayed. Signing out deletes the row.
CREATE TABLE sessions (
  token_hash TEXT PRIMARY KEY,                             -- hex SHA-256 of the cookie token
  user_id    TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  provider   TEXT NOT NULL,                                -- 'password' or 'github'
  created_at TEXT NOT NULL,
  expires_at INTEGER NOT NULL                              -- unix seconds
);
CREATE INDEX sessions_user ON sessions (user_id);
CREATE INDEX sessions_expires ON sessions (expires_at);
