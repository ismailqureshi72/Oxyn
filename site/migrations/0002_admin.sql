-- Oxyn admin panel (B2): maintainer list state, sessions, audit log. Additive only, so it is safe to
-- apply before or after the Worker that uses it. Apply with:
--   npx wrangler d1 migrations apply oxyn --remote   (from site/)

-- admins (from 0001) is the list the Worker trusts. Cloudflare Access is the first gate; the Worker
-- also requires the Access email to be an active row here, so a removal takes effect at once even
-- while the Access group still lists the email.
ALTER TABLE admins ADD COLUMN access_synced INTEGER NOT NULL DEFAULT 1;   -- 0 = Access group not updated yet
ALTER TABLE admins ADD COLUMN removed_at TEXT;                            -- set = removed, kept until Access is synced

-- The two maintainers the Access group "oxyn-admins" was created with.
INSERT OR IGNORE INTO admins (email, added_by, created_at, access_synced) VALUES
  ('m.ismailqureshi62@gmail.com', 'setup', '2026-11-20T00:00:00.000Z', 1),
  ('khubaibnazeer8@gmail.com', 'setup', '2026-11-20T00:00:00.000Z', 1);

-- One row per signed-in admin session (the cookie carries the id, signed with HMAC-SHA256).
CREATE TABLE admin_sessions (
  id         TEXT PRIMARY KEY,                             -- random, 32 bytes base64url
  email      TEXT NOT NULL COLLATE NOCASE,                 -- the Access email that signed in
  pw_version INTEGER NOT NULL,                             -- password version; a change ends old sessions
  created_at TEXT NOT NULL,
  expires_at INTEGER NOT NULL                              -- unix seconds
);
CREATE INDEX admin_sessions_expires ON admin_sessions (expires_at);

-- Every admin action: sign-ins, failed passwords, status changes, deletes, admin list, settings.
CREATE TABLE audit_log (
  id     INTEGER PRIMARY KEY AUTOINCREMENT,
  at     TEXT NOT NULL,                                    -- ISO 8601, UTC
  actor  TEXT NOT NULL,                                    -- Access email
  action TEXT NOT NULL,                                    -- e.g. 'feedback.status', 'admin.add'
  target TEXT,                                             -- feedback id, email, flag key
  detail TEXT                                              -- short JSON or text, never a password
);
CREATE INDEX audit_log_at ON audit_log (at DESC);

-- Content flags edited in Settings (key 'flag:<name>', value JSON {"enabled": bool, "value": text}).
-- Public read: GET /api/flags returns the enabled ones.
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES
  ('flag:promo_strip', '{"enabled":false,"value":""}', '2026-11-20T00:00:00.000Z');
