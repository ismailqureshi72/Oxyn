-- Oxyn Pro and Team (G1): subscriptions sold through Polar, the licences they grant, Team seats and
-- the founder-rate counter. Polar is the merchant of record and the source of truth; these tables
-- are the Worker's copy, written only by the signed Polar webhook (site/worker/billing.ts).
-- Additive only, so it is safe to apply before the Worker that uses it. Apply with:
--   npx wrangler d1 migrations apply oxyn --remote   (from site/)

-- One row per Polar subscription, the latest state Polar sent.
CREATE TABLE subscriptions (
  id                   TEXT PRIMARY KEY,                   -- Polar subscription id
  user_id              TEXT REFERENCES users (id) ON DELETE SET NULL, -- our account, once known
  customer_id          TEXT NOT NULL,                      -- Polar customer id
  customer_email       TEXT NOT NULL COLLATE NOCASE,       -- the email the buyer paid with
  product_id           TEXT NOT NULL,                      -- Polar product id
  plan                 TEXT NOT NULL CHECK (plan IN ('pro', 'team')),
  interval             TEXT NOT NULL CHECK (interval IN ('month', 'year')),
  founder              INTEGER NOT NULL DEFAULT 0,         -- 1 = founder rate (first 100)
  status               TEXT NOT NULL,                      -- Polar status: active, trialing, past_due, canceled, ...
  amount               INTEGER NOT NULL DEFAULT 0,         -- per interval, in cents, before tax
  currency             TEXT NOT NULL DEFAULT 'usd',
  cancel_at_period_end INTEGER NOT NULL DEFAULT 0,
  current_period_end   TEXT,                               -- ISO 8601, UTC
  started_at           TEXT,
  ended_at             TEXT,                               -- set when access ended (revoked)
  checkout_id          TEXT,                               -- the Polar checkout that created it
  polar_modified_at    TEXT,                               -- Polar's modified_at; older events are ignored
  created_at           TEXT NOT NULL,
  updated_at           TEXT NOT NULL
);
CREATE INDEX subscriptions_user ON subscriptions (user_id);
CREATE INDEX subscriptions_email ON subscriptions (customer_email);
CREATE INDEX subscriptions_status ON subscriptions (status);
CREATE INDEX subscriptions_checkout ON subscriptions (checkout_id);

-- The licence a subscription grants its buyer: Pro for one developer, or the Team owner. Access
-- follows the subscription's status; the row stays as the record. Matched by account id, or by
-- email when the account's email was verified by Google or GitHub.
CREATE TABLE licences (
  id              TEXT PRIMARY KEY,                        -- random UUID (the future CLI licence id)
  subscription_id TEXT NOT NULL UNIQUE REFERENCES subscriptions (id) ON DELETE CASCADE,
  user_id         TEXT REFERENCES users (id) ON DELETE SET NULL,
  email           TEXT NOT NULL COLLATE NOCASE,
  plan            TEXT NOT NULL CHECK (plan IN ('pro', 'team')),
  created_at      TEXT NOT NULL,
  revoked_at      TEXT                                     -- set when Polar revoked the subscription
);
CREATE INDEX licences_user ON licences (user_id);
CREATE INDEX licences_email ON licences (email);

-- Team seats: the owner of a Team subscription invites up to four emails. A seat opens Pro for the
-- account with that email once Google or GitHub verified it.
CREATE TABLE team_seats (
  subscription_id TEXT NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
  email           TEXT NOT NULL COLLATE NOCASE,
  invited_by      TEXT NOT NULL,                           -- owner's account id
  created_at      TEXT NOT NULL,
  removed_at      TEXT,
  PRIMARY KEY (subscription_id, email)
);
CREATE INDEX team_seats_email ON team_seats (email);

-- Founder-rate counter: one row per founder subscription that became active. The site stops
-- offering the founder rate at 100 rows, and the Worker archives the two founder products in Polar.
CREATE TABLE founder_claims (
  subscription_id TEXT PRIMARY KEY,
  email           TEXT,
  claimed_at      TEXT NOT NULL
);

-- Checkouts that finished, with the account that started them (Polar sends our account id back as
-- the checkout's reference_id). Lets a subscription event find its account.
CREATE TABLE checkout_refs (
  checkout_id TEXT PRIMARY KEY,
  user_id     TEXT NOT NULL,
  email       TEXT,
  created_at  TEXT NOT NULL
);

-- Webhook deliveries already handled (Standard Webhooks `webhook-id`), so a retry is a no-op.
CREATE TABLE webhook_events (
  id          TEXT PRIMARY KEY,
  type        TEXT NOT NULL,
  received_at TEXT NOT NULL
);
CREATE INDEX webhook_events_received ON webhook_events (received_at);
