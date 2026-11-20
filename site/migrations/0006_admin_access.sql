-- Admin & access (G2): grants written from the admin panel (Pro, Team with members, founder-rate
-- flag, soft revoke and restore), and a small privacy-friendly event log for the Activity tab.
-- Additive only, so it is safe to apply before or after the Worker that uses it. Apply with:
--   npx wrangler d1 migrations apply oxyn --remote   (from site/)

-- grants (0005) gains:
--   parent_id   a Team grant is one owner row (plan 'team', seats 4) plus one row per teammate
--               (plan 'team', seats 1, parent_id = the owner row). Revoking the owner revokes them too.
--   founder     1 = show the founder rate on this grant. Display only: nothing is charged.
--   revoked_by  admin email that revoked it; revoked_at (0005) keeps the time. Never hard-deleted:
--   restoring clears both, and every change is in audit_log.
ALTER TABLE grants ADD COLUMN parent_id INTEGER REFERENCES grants (id);
ALTER TABLE grants ADD COLUMN founder INTEGER NOT NULL DEFAULT 0;
ALTER TABLE grants ADD COLUMN revoked_by TEXT;
ALTER TABLE grants ADD COLUMN updated_at TEXT;
CREATE INDEX grants_parent ON grants (parent_id);
CREATE INDEX grants_live ON grants (revoked_at, email);

-- Site events (POST /api/events from the site's own pages, plus sign-ins recorded by the Worker).
-- No IP, no user agent, no email: `visitor` is a salted hash of IP + user agent + the UTC day, so it
-- changes every day and cannot follow anyone across days; `user_id` is set only for a signed-in
-- account and is taken from its session, never from the request body. Rows older than 90 days are
-- deleted by the Worker.
CREATE TABLE events (
  id      INTEGER PRIMARY KEY AUTOINCREMENT,
  at      INTEGER NOT NULL,                                -- unix seconds
  type    TEXT NOT NULL,                                   -- view, pricing_view, pro_copy, checkout_click, signin
  path    TEXT,                                            -- the page path, no query string
  item    TEXT,                                            -- component / screen slug, plan key or provider
  detail  TEXT,                                            -- short: how a Pro copy happened (copy, keys, cli)
  user_id TEXT,
  visitor TEXT                                             -- daily-rotating hash, 16 hex chars
);
CREATE INDEX events_at ON events (at);
CREATE INDEX events_type_at ON events (type, at);
CREATE INDEX events_user_at ON events (user_id, at);
