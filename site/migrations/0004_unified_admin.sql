-- One sign-in for the whole site (A1): the admin panel now uses the normal site session of an
-- account whose email Google or GitHub verified, plus the D1 `admins` list. Cloudflare Access, the
-- shared admin password and the admin-only sessions are gone. Safe to apply before or after the
-- Worker that uses it (the old Worker only loses its shared password, behind Access). Apply with:
--   npx wrangler d1 migrations apply oxyn --remote   (from site/)

-- The shared admin password hash, the admin-session HMAC key and the Access sync error.
DELETE FROM settings WHERE key IN ('admin_password', 'admin_session_key', 'access_sync_error');

-- Admin-only sessions are no longer read. The table stays (empty) so this stays additive.
DELETE FROM admin_sessions;

-- users.email_verified (0003) is now what admits an admin: 1 only after a Google or GitHub
-- sign-in on that email. Make sure every account with a linked provider is marked.
UPDATE users SET email_verified = 1
  WHERE email_verified = 0 AND id IN (SELECT user_id FROM oauth_accounts);
