-- Oxyn backend core (B1): feedback inbox, rate limits, and the tables the admin panel (B2) and
-- sign-in (L2) build on. Apply with: npx wrangler d1 migrations apply oxyn --remote (from site/).

-- One row per message sent from the Feedback button.
CREATE TABLE feedback (
  id               TEXT PRIMARY KEY,                       -- random UUID
  kind             TEXT NOT NULL CHECK (kind IN ('Bug', 'Idea', 'Question')),
  message          TEXT NOT NULL,
  page_url         TEXT,                                   -- the site page it was sent from
  email            TEXT,                                   -- optional, only for a reply
  ua               TEXT,                                   -- browser user agent, trimmed
  screenshot_key   TEXT,                                   -- R2 key in oxyn-uploads, if any
  created_at       TEXT NOT NULL,                          -- ISO 8601, UTC
  status           TEXT NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'triaged', 'closed')),
  github_issue_url TEXT                                    -- set when the issue was opened
);
CREATE INDEX feedback_status_created ON feedback (status, created_at DESC);
CREATE INDEX feedback_created ON feedback (created_at DESC);

-- Fixed-window rate limit per hashed client IP (the raw IP is never stored).
CREATE TABLE rate_limits (
  key          TEXT NOT NULL,                              -- '<scope>:<sha256(ip + salt)>'
  window_start INTEGER NOT NULL,                           -- unix seconds, start of the window
  count        INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (key, window_start)
);

-- Maintainers allowed into /admin (B2 keeps this in sync with a Cloudflare Access group).
CREATE TABLE admins (
  email      TEXT PRIMARY KEY COLLATE NOCASE,
  added_by   TEXT,
  created_at TEXT NOT NULL
);

-- Small key/value settings (B2: shared admin password hash, feature flags).
CREATE TABLE settings (
  key        TEXT PRIMARY KEY,
  value      TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
