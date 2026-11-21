-- Pro registry (P3): CLI tokens, the `oxyn login` device flow, and the monthly Pro unlock caps.
-- Additive only, so it is safe to apply before or after the Worker that uses it. Apply with:
--   npx wrangler d1 migrations apply oxyn --remote   (from site/)

-- CLI tokens: made on the account page (or by `oxyn login`), sent as `Authorization: Bearer`.
-- Only the SHA-256 of a token is stored; the token itself is shown once. Revoking keeps the row.
CREATE TABLE cli_tokens (
  id           TEXT PRIMARY KEY,                           -- random UUID
  user_id      TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  token_hash   TEXT NOT NULL UNIQUE,                       -- hex SHA-256 of the token
  name         TEXT NOT NULL,                              -- "laptop", "CI", "oxyn login on <host>"
  hint         TEXT NOT NULL,                              -- last 4 characters, to tell tokens apart
  created_at   TEXT NOT NULL,                              -- ISO 8601, UTC
  last_used_at TEXT,
  revoked_at   TEXT
);
CREATE INDEX cli_tokens_user ON cli_tokens (user_id);

-- `oxyn login`: the CLI starts a request, the signed-in browser approves it at /cli/auth/?code=…,
-- the CLI polls with its secret device code and gets a new token once. Rows live 10 minutes.
CREATE TABLE cli_auth_requests (
  device_hash TEXT PRIMARY KEY,                            -- hex SHA-256 of the CLI's device code
  user_code   TEXT NOT NULL UNIQUE,                        -- short code shown in the CLI and the URL
  client_name TEXT,                                        -- host name the CLI sent, for the token name
  user_id     TEXT REFERENCES users (id) ON DELETE CASCADE,-- set when approved
  created_at  INTEGER NOT NULL,                            -- unix seconds
  expires_at  INTEGER NOT NULL,                            -- unix seconds
  denied      INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX cli_auth_expires ON cli_auth_requests (expires_at);

-- Pro unlocks: one row per Pro item a quota pool opened in a billing month. A pool is one account
-- (Pro, Founder, admins) or one Team (shared by its people). Downloading the same item again in the
-- same period finds its row and counts nothing. An admin reset stamps reset_at on the period's rows,
-- which frees them; the rows stay as the record.
CREATE TABLE pro_unlocks (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  pool         TEXT NOT NULL,                              -- user:<id>, team:<subscription id>, grant:<owner grant id>
  period_start TEXT NOT NULL,                              -- ISO 8601, UTC: the billing month it counts in
  slug         TEXT NOT NULL,
  kind         TEXT NOT NULL CHECK (kind IN ('component', 'screen')), -- blocks and apps count as screens
  user_id      TEXT,                                       -- who unlocked it
  via          TEXT NOT NULL,                              -- cli, mcp, site
  created_at   TEXT NOT NULL,
  reset_at     TEXT,                                       -- set by an admin reset
  reset_by     TEXT
);
CREATE UNIQUE INDEX pro_unlocks_live ON pro_unlocks (pool, period_start, slug) WHERE reset_at IS NULL;
CREATE INDEX pro_unlocks_pool ON pro_unlocks (pool, period_start, kind);
CREATE INDEX pro_unlocks_user ON pro_unlocks (user_id, created_at);

-- The start of the subscription's current period, from Polar's webhook (current_period_start). The
-- billing month of a quota starts here; without it the calendar month (UTC) is used.
ALTER TABLE subscriptions ADD COLUMN current_period_start TEXT;
