// Admin & access (G2): the Users, Grants and Activity tabs of the admin panel. Routed from
// admin.ts after its checks (signed in, provider-verified email on the admin list, same Origin for
// changes, per-admin write limit). See docs/admin.md.
//
//   GET    /api/admin/users                 ?q= &plan= &sort=created|active|email|plan &dir=asc|desc &page= &size= &tz= &format=csv
//   GET    /api/admin/users/<id>            one account: plan and where it comes from, grants, subscriptions
//   POST   /api/admin/users/<id>/plan       {plan: free|pro|team, founder, members[], reason, expires_at, confirm, acknowledge_paid}
//   POST   /api/admin/users/<id>/usage/reset frees this month's Pro items (P3 caps) of the account's pool
//   GET    /api/admin/grants                ?status=active|pending|revoked|expired|all &q=
//   POST   /api/admin/grants/check          {plan, emails[] | owner + members[]}: per-line report, writes nothing
//   POST   /api/admin/grants                {plan, founder, emails[] | owner + members[], reason, expires_at, confirm, ...}
//   POST   /api/admin/grants/<id>/revoke    soft revoke (a Team owner's grant takes its members with it)
//   POST   /api/admin/grants/<id>/restore   undo a revoke (the panel offers it for 30 s, and later from History)
//   GET    /api/admin/activity              ?days=7|30|90 &tz=: the dashboard numbers
//
// Grants are never deleted. Every add, revoke, restore and plan change goes to audit_log with the
// admin's email and the state before and after. The browser checks the same things first (instant
// feedback), but only these checks decide.

import { HttpError, json, type Ctx, type Env } from './shared';
import { ACTIVE, POLAR, billingSummary } from './billing';
import { checkEmail, type EmailCheck } from './emailcheck';
import { EVENTS, prune } from './events';
import { audit } from './admin';
import { adminResetUsage, adminUsage } from './pro';

export const ACCESS = {
  bulkMax: 200, // emails in one bulk grant
  bulkAck: 20, // above this, the admin also ticks "I checked this list"
  hourlyGrants: { window: 3600, max: 300 }, // grant emails per admin per hour
  reasonMin: 3,
  reasonMax: 300,
  usersCap: 20_000, // users read for one list (filters and sort run on these)
  pageSize: 50,
  polarDashboard: 'https://polar.sh/dashboard/oxyn',
};
const ACTIVE_SQL = ACTIVE.map((s) => `'${s}'`).join(', ');
const iso = () => new Date().toISOString();
const nowS = () => Math.floor(Date.now() / 1000);

// ── Plans as the panel shows them ───────────────────────────────────────────────────────────────
export type PlanKey = 'free' | 'pro' | 'founder' | 'team_owner' | 'team_member' | 'granted';
export const PLAN_LABEL: Record<PlanKey, string> = {
  free: 'Free', pro: 'Pro', founder: 'Founder', team_owner: 'Team owner', team_member: 'Team member', granted: 'Granted',
};
export interface PlanView {
  key: PlanKey;
  label: string; // e.g. "Granted · Team owner"
  tier: 'free' | 'pro' | 'team';
  source: 'polar' | 'seat' | 'grant' | null;
  expires_at: string | null; // grant end, or the Polar period end when it is canceling
  founder: boolean;
  owner: string | null; // Team owner's email, for a member
  grant_id: number | null;
  pending: string | null; // a grant that is not in effect yet, and why
  paid: boolean; // a live Polar subscription or a seat on one: "Manage in Polar"
}
interface Raw {
  verified: number;
  sub: string | null; // "plan|founder|cancel|period_end"
  seat_owner: string | null;
  grant: string | null; // "id|plan|parent|founder|expires|owner_email"
}
const rank = (t: string) => (t === 'team' ? 2 : t === 'pro' ? 1 : 0);

export function planView(r: Raw): PlanView {
  const base: PlanView = { key: 'free', label: 'Free', tier: 'free', source: null, expires_at: null, founder: false, owner: null, grant_id: null, pending: null, paid: false };
  const opts: PlanView[] = [];
  if (r.sub) {
    const [plan, founder, cancel, end] = r.sub.split('|');
    const key: PlanKey = plan === 'team' ? 'team_owner' : founder === '1' ? 'founder' : 'pro';
    opts.push({ ...base, key, label: PLAN_LABEL[key], tier: plan === 'team' ? 'team' : 'pro', source: 'polar', founder: founder === '1', expires_at: cancel === '1' ? end || null : null, paid: true });
  }
  if (r.seat_owner != null) opts.push({ ...base, key: 'team_member', label: 'Team member', tier: 'team', source: 'seat', owner: r.seat_owner || null, paid: true });
  let grant: PlanView | null = null;
  if (r.grant) {
    const [id, plan, parent, founder, expires, owner] = r.grant.split('|');
    const sub: PlanKey = plan === 'team' ? (parent ? 'team_member' : 'team_owner') : founder === '1' ? 'founder' : 'pro';
    grant = { ...base, key: 'granted', label: `Granted · ${PLAN_LABEL[sub]}`, tier: plan === 'team' ? 'team' : 'pro', source: 'grant', founder: founder === '1', expires_at: expires || null, owner: owner || null, grant_id: Number(id) };
    if (r.verified) opts.push(grant);
  }
  // Highest tier wins; on a tie the paid plan is shown (it is what the person pays for).
  opts.sort((a, b) => rank(b.tier) - rank(a.tier) || (a.paid === b.paid ? 0 : a.paid ? -1 : 1));
  const top = opts[0] ?? base;
  if (grant && !r.verified) return { ...top, grant_id: grant.grant_id, pending: `${grant.label} waits until this email is verified with Google or GitHub.` };
  return top;
}

const PLAN_SQL = (email: string, verified: string, id: string) => `
  (SELECT s.plan || '|' || s.founder || '|' || s.cancel_at_period_end || '|' || COALESCE(s.current_period_end, '')
     FROM licences l JOIN subscriptions s ON s.id = l.subscription_id
    WHERE (l.user_id = ${id} OR (${verified} = 1 AND l.email = ${email})) AND s.status IN (${ACTIVE_SQL}) AND s.ended_at IS NULL
    ORDER BY (s.plan = 'team') DESC, s.created_at DESC LIMIT 1) AS sub,
  (SELECT COALESCE((SELECT email FROM licences WHERE subscription_id = s.id), '')
     FROM team_seats t JOIN subscriptions s ON s.id = t.subscription_id
    WHERE ${verified} = 1 AND t.email = ${email} AND t.removed_at IS NULL AND s.plan = 'team' AND s.status IN (${ACTIVE_SQL}) AND s.ended_at IS NULL
    LIMIT 1) AS seat_owner,
  (SELECT g.id || '|' || g.plan || '|' || COALESCE(g.parent_id, '') || '|' || g.founder || '|' || COALESCE(g.expires_at, '') || '|' ||
          COALESCE((SELECT p.email FROM grants p WHERE p.id = g.parent_id), '')
     FROM grants g WHERE g.email = ${email} AND g.revoked_at IS NULL AND (g.expires_at IS NULL OR g.expires_at > ?1)
    ORDER BY (g.plan = 'team') DESC, g.created_at DESC LIMIT 1) AS "grant"`;

// ── Users ──────────────────────────────────────────────────────────────────────────────────────
interface UserRow extends Raw {
  id: string; email: string; name: string | null; created_at: string; last_login_at: string | null;
  has_password: number; providers: string | null; last_event: number | null;
}
const tzOf = (url: URL) => {
  const n = Number(url.searchParams.get('tz') || 0);
  return Number.isFinite(n) ? Math.max(-840, Math.min(840, Math.round(n))) : 0; // minutes east of UTC
};
/** Start of "today" in the admin's time zone, as an ISO instant. */
const todayStart = (tz: number) => {
  const local = Date.now() + tz * 60_000;
  return new Date(local - (local % 86_400_000) - tz * 60_000);
};

function userOut(u: UserRow) {
  const providers = (u.providers ? u.providers.split(',') : []).filter(Boolean).sort();
  if (u.has_password) providers.push('password');
  const lastEvent = u.last_event ? new Date(u.last_event * 1000).toISOString() : null;
  const last = [u.last_login_at, lastEvent].filter(Boolean).sort().pop() ?? null;
  return { id: u.id, email: u.email, name: u.name, verified: !!u.verified, providers, created_at: u.created_at, last_active: last, plan: planView(u) };
}
type UserOut = ReturnType<typeof userOut>;

async function allUsers(env: Env, where = '', bind: unknown[] = []): Promise<UserOut[]> {
  const r = await env.DB.prepare(
    `SELECT u.id, u.email, u.name, u.email_verified AS verified, u.created_at, u.last_login_at, u.password IS NOT NULL AS has_password,
       (SELECT group_concat(provider) FROM oauth_accounts o WHERE o.user_id = u.id) AS providers,
       (SELECT max(at) FROM events e WHERE e.user_id = u.id) AS last_event,
       ${PLAN_SQL('u.email', 'u.email_verified', 'u.id')}
     FROM users u ${where} ORDER BY u.created_at DESC LIMIT ${ACCESS.usersCap}`,
  ).bind(iso(), ...bind).all<UserRow>();
  return (r.results ?? []).map(userOut);
}

const csvCell = (v: unknown) => {
  let s = v == null ? '' : String(v);
  if (/^[=+\-@\t\r]/.test(s)) s = "'" + s; // no formulas when opened in a spreadsheet
  return /[",\n\r]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s;
};

export async function listUsers(env: Env, url: URL): Promise<Response> {
  const q = url.searchParams;
  const tz = tzOf(url);
  const text = (q.get('q') || '').trim().toLowerCase().slice(0, 100);
  const plan = q.get('plan') || '';
  if (plan && !(plan in PLAN_LABEL) && plan !== 'pending') throw new HttpError(400, 'invalid_plan', 'Unknown plan filter.');
  const sort = q.get('sort') || 'created';
  if (!['created', 'active', 'email', 'plan'].includes(sort)) throw new HttpError(400, 'invalid_sort', 'Unknown sort.');
  const dir = q.get('dir') === 'asc' ? 1 : -1;
  const size = Math.max(10, Math.min(200, Number(q.get('size')) || ACCESS.pageSize));

  const all = await allUsers(env);
  const t0 = todayStart(tz).toISOString();
  const ago = (d: number) => new Date(Date.now() - d * 86_400_000).toISOString();
  const between = (a: string, b: string) => all.filter((u) => u.created_at >= a && u.created_at < b).length;
  const counts: Record<string, number> = { free: 0, pro: 0, founder: 0, team_owner: 0, team_member: 0, granted: 0, pending: 0 };
  for (const u of all) { counts[u.plan.key]++; if (u.plan.pending) counts.pending++; }
  const summary = {
    total: all.length,
    capped: all.length >= ACCESS.usersCap,
    verified: all.filter((u) => u.verified).length,
    today: all.filter((u) => u.created_at >= t0).length,
    yesterday: between(new Date(Date.parse(t0) - 86_400_000).toISOString(), t0),
    d7: between(ago(7), iso() + '~'), prev7: between(ago(14), ago(7)),
    d30: between(ago(30), iso() + '~'), prev30: between(ago(60), ago(30)),
    plans: counts,
  };

  let rows = all.filter((u) => (!text || u.email.includes(text) || (u.name || '').toLowerCase().includes(text)) &&
    (!plan || (plan === 'pending' ? !!u.plan.pending : u.plan.key === plan)));
  const order = ['free', 'pro', 'founder', 'granted', 'team_member', 'team_owner'];
  const key = (u: UserOut): string | number =>
    sort === 'email' ? u.email : sort === 'active' ? u.last_active || '' : sort === 'plan' ? order.indexOf(u.plan.key) * 10 + rank(u.plan.tier) : u.created_at;
  rows = rows.sort((a, b) => { const x = key(a), y = key(b); return (x < y ? -1 : x > y ? 1 : 0) * dir || (a.email < b.email ? -1 : 1); });

  if (q.get('format') === 'csv') {
    const head = ['email', 'name', 'sign_in', 'verified', 'signed_up_utc', 'last_active_utc', 'plan', 'plan_source', 'plan_expires_utc', 'founder', 'team_owner', 'pending'];
    const lines = [head.join(',')].concat(rows.map((u) => [
      u.email, u.name, u.providers.join(' '), u.verified ? 'yes' : 'no', u.created_at, u.last_active, u.plan.label, u.plan.source, u.plan.expires_at,
      u.plan.founder ? 'yes' : '', u.plan.owner, u.plan.pending,
    ].map(csvCell).join(',')));
    return new Response(lines.join('\r\n') + '\r\n', {
      headers: {
        'content-type': 'text/csv; charset=utf-8',
        'content-disposition': `attachment; filename="oxyn-users-${iso().slice(0, 10)}.csv"`,
        'cache-control': 'no-store', 'x-content-type-options': 'nosniff', 'x-robots-tag': 'noindex',
      },
    });
  }
  const pages = Math.max(1, Math.ceil(rows.length / size));
  const page = Math.max(1, Math.min(pages, Number(q.get('page')) || 1));
  return json({ ok: true, summary, total: rows.length, page, pages, size, items: rows.slice((page - 1) * size, page * size) });
}

async function oneUser(env: Env, id: string) {
  const [u] = await allUsers(env, 'WHERE u.id = ?2', [id]);
  if (!u) throw new HttpError(404, 'not_found', 'That account no longer exists.');
  return u;
}

export async function getUser(env: Env, id: string): Promise<Response> {
  const u = await oneUser(env, id);
  const [g, s] = await env.DB.batch([
    env.DB.prepare('SELECT g.*, (SELECT p.email FROM grants p WHERE p.id = g.parent_id) AS owner_email FROM grants g WHERE g.email = ?1 ORDER BY g.created_at DESC LIMIT 50').bind(u.email),
    env.DB.prepare(
      'SELECT s.id, s.plan, s.interval, s.founder, s.status, s.amount, s.cancel_at_period_end, s.current_period_end, s.created_at, s.ended_at ' +
        'FROM licences l JOIN subscriptions s ON s.id = l.subscription_id WHERE l.user_id = ?1 OR (?3 = 1 AND l.email = ?2) ORDER BY s.created_at DESC LIMIT 20',
    ).bind(u.id, u.email, u.verified ? 1 : 0),
  ]);
  let usage = null;
  try { usage = await adminUsage(env, { id: u.id, email: u.email, email_verified: u.verified ? 1 : 0 }); } catch (e) { console.error('admin usage', e); }
  return json({ ok: true, user: u, grants: g.results ?? [], subscriptions: s.results ?? [], usage, polar_dashboard: ACCESS.polarDashboard, people_limit: POLAR.people });
}

/** POST /api/admin/users/<id>/usage/reset: frees this month's Pro items of the account's pool (its
 *  Team's for a Team member: everyone on it gets the count back). Audited. */
export async function resetUsage(env: Env, admin: string, id: string): Promise<Response> {
  const u = await oneUser(env, id);
  const r = await adminResetUsage(env, admin, { id: u.id, email: u.email, email_verified: u.verified ? 1 : 0 });
  await audit(env, admin, 'usage.reset', u.email, r);
  return json({ ok: true, ...r });
}

// ── Grants: checks ─────────────────────────────────────────────────────────────────────────────
type Role = 'single' | 'owner' | 'member';
export interface LineReport extends EmailCheck {
  line: number;
  role: Role;
  status: 'valid' | 'invalid' | 'typo' | 'duplicate' | 'paid';
  account: 'verified' | 'unverified' | 'none' | null;
  warnings: string[];
}

interface Entry { raw: string; role: Role }

/** Splits a bulk paste into entries: one per line, also split on commas, semicolons and spaces. */
export function splitBulk(text: string): string[] {
  return String(text || '').split(/[\n\r,;\t]+|\s{2,}|(?<=@\S*)\s+(?=\S+@)/).map((s) => s.trim()).filter(Boolean);
}

async function checkSet(env: Env, entries: Entry[]): Promise<LineReport[]> {
  const reps: LineReport[] = entries.map((e, i) => ({ ...checkEmail(e.raw), line: i + 1, role: e.role, status: 'valid', account: null, warnings: [] }));
  const seen = new Map<string, number>();
  for (const r of reps) {
    if (r.code === 'empty' || r.code === 'invalid' || r.code === 'too_long') { r.status = 'invalid'; continue; }
    if (seen.has(r.email)) { r.status = 'duplicate'; r.message = `Listed twice (line ${seen.get(r.email)}).`; continue; }
    seen.set(r.email, r.line);
    if (r.code === 'typo') r.status = 'typo';
  }
  const emails = [...seen.keys()];
  if (!emails.length) return reps;
  const now = iso();
  const info = new Map<string, { grant: string | null; paid: string | null; verified: number | null }>();
  // One query per 50 emails (D1 binds at most 100 values).
  for (let i = 0; i < emails.length; i += 50) {
    const chunk = emails.slice(i, i + 50);
    const marks = chunk.map((_, k) => `?${k + 2}`).join(', ');
    const r = await env.DB.prepare(
      `SELECT e.value AS email,
         (SELECT g.plan || CASE WHEN g.parent_id IS NULL THEN '' ELSE ' member' END FROM grants g
           WHERE g.email = e.value AND g.revoked_at IS NULL AND (g.expires_at IS NULL OR g.expires_at > ?1) LIMIT 1) AS "grant",
         (SELECT s.plan FROM licences l JOIN subscriptions s ON s.id = l.subscription_id
           WHERE (l.email = e.value OR l.user_id = (SELECT id FROM users WHERE email = e.value)) AND s.status IN (${ACTIVE_SQL}) AND s.ended_at IS NULL LIMIT 1) AS paid,
         (SELECT 'seat' FROM team_seats t JOIN subscriptions s ON s.id = t.subscription_id
           WHERE t.email = e.value AND t.removed_at IS NULL AND s.status IN (${ACTIVE_SQL}) AND s.ended_at IS NULL LIMIT 1) AS seat,
         (SELECT email_verified FROM users WHERE email = e.value) AS verified
       FROM json_each(json_array(${marks})) e`,
    ).bind(now, ...chunk).all<{ email: string; grant: string | null; paid: string | null; seat: string | null; verified: number | null }>();
    for (const x of r.results ?? []) info.set(x.email, { grant: x.grant, paid: x.paid ? `a paid ${x.paid === 'team' ? 'Team' : 'Pro'} subscription` : x.seat ? 'a seat on a paid Team' : null, verified: x.verified });
  }
  for (const r of reps) {
    if (r.status === 'invalid' || r.status === 'duplicate') continue;
    const x = info.get(r.email);
    if (!x) continue;
    r.account = x.verified == null ? 'none' : x.verified ? 'verified' : 'unverified';
    if (!x.grant) {
      if (r.account === 'none') r.warnings.push('No account yet: the grant waits and turns on when they sign in with Google or GitHub on this email.');
      if (r.account === 'unverified') r.warnings.push('Password-only account: the grant turns on once they sign in with Google or GitHub on this email.');
    }
    if (x.grant) { r.status = 'duplicate'; r.message = `Already has an active grant (${x.grant === 'team' ? 'Team owner' : x.grant === 'team member' ? 'Team member' : 'Pro'}). Revoke it first or change the plan from Users.`; continue; }
    if (x.paid && r.status === 'valid') { r.status = 'paid'; r.message = `Already has ${x.paid}. A grant on top is usually not needed.`; }
  }
  return reps;
}

function entriesFrom(b: Record<string, unknown>): { plan: 'pro' | 'team'; entries: Entry[] } {
  const plan = b.plan === 'team' ? 'team' : b.plan === 'pro' ? 'pro' : null;
  if (!plan) throw new HttpError(400, 'invalid_plan', 'Choose Pro or Team.', 'plan');
  if (plan === 'team') {
    const members = Array.isArray(b.members) ? b.members.map(String).filter((m) => m.trim()) : [];
    if (members.length > POLAR.people - 1) throw new HttpError(400, 'too_many_members', `A Team is up to ${POLAR.people} people: the owner and ${POLAR.people - 1} teammates.`, 'members');
    return { plan, entries: [{ raw: String(b.owner ?? ''), role: 'owner' }, ...members.map((m) => ({ raw: m, role: 'member' as Role }))] };
  }
  let list: string[] = Array.isArray(b.emails) ? b.emails.map(String) : typeof b.text === 'string' ? splitBulk(b.text) : [];
  list = list.filter((x) => x.trim());
  if (!list.length) throw new HttpError(400, 'no_emails', 'Enter at least one email address.', 'emails');
  if (list.length > ACCESS.bulkMax) throw new HttpError(400, 'too_many', `Up to ${ACCESS.bulkMax} emails at a time.`, 'emails');
  return { plan, entries: list.map((raw) => ({ raw, role: 'single' as Role })) };
}

const summarise = (reps: LineReport[]) => ({
  total: reps.length,
  valid: reps.filter((r) => r.status === 'valid').length,
  invalid: reps.filter((r) => r.status === 'invalid').length,
  duplicate: reps.filter((r) => r.status === 'duplicate').length,
  typo: reps.filter((r) => r.status === 'typo').length,
  paid: reps.filter((r) => r.status === 'paid').length,
});

export async function checkGrants(env: Env, b: Record<string, unknown>): Promise<Response> {
  const { plan, entries } = entriesFrom(b);
  const lines = await checkSet(env, entries);
  return json({ ok: true, plan, lines, summary: summarise(lines) });
}

// ── Grants: writes ─────────────────────────────────────────────────────────────────────────────
function reasonOf(b: Record<string, unknown>) {
  const reason = typeof b.reason === 'string' ? b.reason.trim().replace(/\s+/g, ' ') : '';
  if (reason.length < ACCESS.reasonMin) throw new HttpError(400, 'reason_required', 'Write why this grant is given (a few words).', 'reason');
  if (reason.length > ACCESS.reasonMax) throw new HttpError(400, 'reason_too_long', `Keep the reason under ${ACCESS.reasonMax} characters.`, 'reason');
  return reason;
}
function expiryOf(b: Record<string, unknown>) {
  if (b.expires_at == null || b.expires_at === '') return null;
  const t = Date.parse(String(b.expires_at));
  if (!Number.isFinite(t)) throw new HttpError(400, 'invalid_expiry', 'That end date could not be read.', 'expires_at');
  if (t <= Date.now() + 60_000) throw new HttpError(400, 'expiry_in_past', 'The end date must be in the future.', 'expires_at');
  if (t > Date.now() + 5 * 366 * 86_400_000) throw new HttpError(400, 'expiry_too_far', 'Pick an end date within five years, or leave it empty for no end.', 'expires_at');
  return new Date(t).toISOString();
}

async function grantLimit(env: Env, admin: string, n: number) {
  const t = nowS();
  const w = ACCESS.hourlyGrants;
  const r = await env.DB.prepare(
    'INSERT INTO rate_limits (key, window_start, count) VALUES (?1, ?2, ?3) ON CONFLICT (key, window_start) DO UPDATE SET count = count + ?3 RETURNING count',
  ).bind(`grants:${admin}`, t - (t % w.window), n).first<{ count: number }>();
  if (r && r.count > w.max) throw new HttpError(429, 'rate_limited', `More than ${w.max} grants in an hour. Wait a while and try again.`);
}

interface GrantOpts { reason: string; expires: string | null; founder: boolean }

/** Validates a full set again and writes it. Refuses (409, with the report) unless every line is
 *  valid, or a typo the admin chose to keep, or a paid account the admin acknowledged. */
async function writeGrants(env: Env, admin: string, plan: 'pro' | 'team', entries: Entry[], o: GrantOpts, b: Record<string, unknown>, action = 'grant.add') {
  const lines = await checkSet(env, entries);
  const keep = new Set(Array.isArray(b.keep_typos) ? b.keep_typos.map((x) => String(x).trim().toLowerCase()) : []);
  const bad = lines.filter((l) => l.status === 'invalid' || l.status === 'duplicate' || (l.status === 'typo' && !keep.has(l.email)) || (l.status === 'paid' && b.acknowledge_paid !== true));
  if (bad.length) {
    const first = bad[0];
    const code = first.status === 'typo' ? 'typo_suspected' : first.status === 'paid' ? 'paid_subscription' : first.status === 'duplicate' ? 'duplicate' : 'invalid_email';
    return json({ ok: false, error: { code, message: `${first.email || first.input}: ${first.message}`, field: 'emails' }, lines, summary: summarise(lines) }, first.status === 'invalid' ? 400 : 409);
  }
  const n = lines.length;
  // The typed confirmation: the email for one grant (and for a Team, its owner), the count for a bulk grant.
  const confirm = String(b.confirm ?? '').trim().toLowerCase();
  const want = plan === 'team' || n === 1 ? lines[0].email : String(n);
  if (confirm !== want) throw new HttpError(400, 'confirm_mismatch', plan === 'team' || n === 1 ? `Type ${lines[0].email} to confirm.` : `Type ${n} to confirm.`, 'confirm');
  if (n > ACCESS.bulkAck && b.bulk_ack !== true) throw new HttpError(400, 'bulk_ack_required', `For more than ${ACCESS.bulkAck} emails, tick that you checked the list.`, 'bulk_ack');
  await grantLimit(env, admin, n);
  const now = iso();
  const ids: number[] = [];
  let parent: number | null = null;
  for (const l of lines) {
    const seats = plan === 'team' && l.role === 'owner' ? POLAR.people : 1;
    const r: { id: number } | null = await env.DB.prepare(
      'INSERT INTO grants (email, plan, seats, expires_at, reason, granted_by, created_at, parent_id, founder, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?7) RETURNING id',
    ).bind(l.email, plan, seats, o.expires, o.reason, admin, now, l.role === 'member' ? parent : null, o.founder && plan === 'pro' ? 1 : 0).first<{ id: number }>();
    const id: number = Number(r?.id);
    ids.push(id);
    if (l.role === 'owner') parent = id;
    await audit(env, admin, action, l.email, {
      grant: id, before: null,
      after: { plan: plan === 'team' ? (l.role === 'member' ? 'team_member' : 'team_owner') : o.founder ? 'founder' : 'pro', expires_at: o.expires, reason: o.reason, ...(l.role === 'member' ? { owner: lines[0].email } : {}) },
    });
  }
  return json({ ok: true, ids, count: ids.length, lines, summary: summarise(lines) }, 201);
}

export async function addGrants(env: Env, admin: string, b: Record<string, unknown>): Promise<Response> {
  const { plan, entries } = entriesFrom(b);
  const o = { reason: reasonOf(b), expires: expiryOf(b), founder: b.founder === true };
  return writeGrants(env, admin, plan, entries, o, b);
}

interface GrantRow { id: number; email: string; plan: string; parent_id: number | null; founder: number; expires_at: string | null; revoked_at: string | null; revoked_by: string | null; reason: string | null }

async function grantById(env: Env, id: number) {
  const g = await env.DB.prepare('SELECT * FROM grants WHERE id = ?1').bind(id).first<GrantRow>();
  if (!g) throw new HttpError(404, 'not_found', 'That grant does not exist.');
  return g;
}
const grantKey = (g: GrantRow) => (g.plan === 'team' ? (g.parent_id ? 'team_member' : 'team_owner') : g.founder ? 'founder' : 'pro');

/** Soft revoke: sets revoked_at / revoked_by. A Team owner's grant takes its active members along. */
export async function revokeGrant(env: Env, admin: string, id: number, why = ''): Promise<Response> {
  const g = await grantById(env, id);
  if (g.revoked_at) throw new HttpError(409, 'already_revoked', 'That grant is already revoked.');
  const at = iso();
  const r = await env.DB.prepare(
    'UPDATE grants SET revoked_at = ?2, revoked_by = ?3, updated_at = ?2 WHERE (id = ?1 OR parent_id = ?1) AND revoked_at IS NULL RETURNING id, email',
  ).bind(id, at, admin).all<{ id: number; email: string }>();
  const rows = r.results ?? [];
  for (const x of rows) await audit(env, admin, 'grant.revoke', x.email, { grant: x.id, before: { plan: x.id === id ? grantKey(g) : 'team_member', active: true }, after: { active: false }, ...(why ? { reason: why.slice(0, 200) } : {}) });
  return json({ ok: true, ids: rows.map((x) => x.id), revoked_at: at });
}

/** Undo a revoke: the grant (and the members revoked with it) come back as they were. */
export async function restoreGrant(env: Env, admin: string, id: number): Promise<Response> {
  const g = await grantById(env, id);
  if (!g.revoked_at) throw new HttpError(409, 'not_revoked', 'That grant is active already.');
  if (g.parent_id) {
    const p = await grantById(env, g.parent_id);
    if (p.revoked_at) throw new HttpError(409, 'owner_revoked', `Restore the Team owner's grant (${p.email}) first; its members come back with it.`);
    const n = await env.DB.prepare('SELECT COUNT(*) AS n FROM grants WHERE parent_id = ?1 AND revoked_at IS NULL').bind(g.parent_id).first<{ n: number }>();
    if (Number(n?.n ?? 0) >= POLAR.people - 1) throw new HttpError(409, 'seats_full', `That Team already has ${POLAR.people - 1} teammates.`);
  }
  const group = (await env.DB.prepare('SELECT * FROM grants WHERE (id = ?1 OR (parent_id = ?1 AND revoked_at = ?2))').bind(id, g.revoked_at).all<GrantRow>()).results ?? [];
  const now = iso();
  // No double grants: an email that got another active grant since then is left out (the owner must not be).
  const clash = new Set<string>();
  for (const x of group) {
    const other = await env.DB.prepare('SELECT id FROM grants WHERE email = ?1 AND id != ?2 AND revoked_at IS NULL AND (expires_at IS NULL OR expires_at > ?3) LIMIT 1').bind(x.email, x.id, now).first();
    if (other) clash.add(x.email);
  }
  if (clash.has(g.email)) throw new HttpError(409, 'duplicate', `${g.email} has another active grant now. Revoke that one first.`);
  const back = group.filter((x) => !clash.has(x.email));
  for (const x of back) {
    await env.DB.prepare('UPDATE grants SET revoked_at = NULL, revoked_by = NULL, updated_at = ?2 WHERE id = ?1').bind(x.id, now).run();
    await audit(env, admin, 'grant.restore', x.email, { grant: x.id, before: { active: false, revoked_at: x.revoked_at, revoked_by: x.revoked_by }, after: { plan: grantKey(x), active: true } });
  }
  const expired = !!g.expires_at && g.expires_at <= now;
  return json({ ok: true, ids: back.map((x) => x.id), skipped: [...clash], expired });
}

export async function listGrants(env: Env, url: URL): Promise<Response> {
  const status = url.searchParams.get('status') || 'active';
  if (!['active', 'pending', 'revoked', 'expired', 'all'].includes(status)) throw new HttpError(400, 'invalid_status', 'Unknown status.');
  const text = (url.searchParams.get('q') || '').trim().toLowerCase().slice(0, 100);
  const now = iso();
  const r = await env.DB.prepare(
    `SELECT g.id, g.email, g.plan, g.seats, g.parent_id, g.founder, g.expires_at, g.reason, g.granted_by, g.created_at, g.revoked_at, g.revoked_by,
       (SELECT email_verified FROM users u WHERE u.email = g.email) AS verified,
       (SELECT id FROM users u WHERE u.email = g.email) AS user_id,
       (SELECT s.plan FROM licences l JOIN subscriptions s ON s.id = l.subscription_id
         WHERE l.email = g.email AND s.status IN (${ACTIVE_SQL}) AND s.ended_at IS NULL LIMIT 1) AS paid
     FROM grants g ORDER BY g.created_at DESC, g.id LIMIT 5000`,
  ).all<GrantRow & { seats: number; granted_by: string; created_at: string; verified: number | null; user_id: string | null; paid: string | null }>();
  const rows = (r.results ?? []).map((g) => {
    const st = g.revoked_at ? 'revoked' : g.expires_at && g.expires_at <= now ? 'expired' : g.verified ? 'active' : 'pending';
    return { ...g, key: grantKey(g), label: PLAN_LABEL[grantKey(g) as PlanKey], status: st, account: g.verified == null ? 'none' : g.verified ? 'verified' : 'unverified', members: [] as unknown[] };
  });
  const byId = new Map(rows.map((g) => [g.id, g]));
  for (const g of rows) if (g.parent_id && byId.has(g.parent_id)) byId.get(g.parent_id)!.members.push(g);
  const counts = { active: 0, pending: 0, revoked: 0, expired: 0 };
  for (const g of rows) counts[g.status as keyof typeof counts]++;
  // Members are listed under their owner unless a search or status picks them out on their own.
  const want = (g: (typeof rows)[number]) =>
    (status === 'all' || g.status === status || (status === 'active' && g.status === 'pending')) &&
    (!text || g.email.includes(text) || (g.reason || '').toLowerCase().includes(text) || g.members.some((m) => (m as { email: string }).email.includes(text)));
  const items = rows.filter((g) => want(g) && (!g.parent_id || !byId.has(g.parent_id) || !want(byId.get(g.parent_id)!)));
  return json({ ok: true, items: items.slice(0, 1000), counts, now });
}

// ── Change a user's plan ───────────────────────────────────────────────────────────────────────
export async function changePlan(env: Env, admin: string, id: string, b: Record<string, unknown>): Promise<Response> {
  const u = await oneUser(env, id);
  const target = String(b.plan || '');
  if (!['free', 'pro', 'team'].includes(target)) throw new HttpError(400, 'invalid_plan', 'Choose Free, Pro or Team.', 'plan');
  const confirm = String(b.confirm ?? '').trim().toLowerCase();
  if (confirm !== u.email) throw new HttpError(400, 'confirm_mismatch', `Type ${u.email} to confirm.`, 'confirm');
  const before = u.plan;
  const live = (await env.DB.prepare('SELECT id, parent_id FROM grants WHERE email = ?1 AND revoked_at IS NULL').bind(u.email).all<{ id: number; parent_id: number | null }>()).results ?? [];
  if (target === 'free') {
    if (before.source === 'polar' || before.source === 'seat')
      throw new HttpError(409, before.source === 'seat' ? 'paid_seat' : 'paid_subscription',
        before.source === 'seat' ? `This plan is a seat on ${before.owner || 'a paid Team'}: the Team owner removes it in their account, or cancel the Team in Polar.` : 'This plan is a paid Polar subscription. Cancel or refund it in Polar; it cannot be removed here.');
    if (!live.length) throw new HttpError(409, 'already_free', 'This account has no grant to remove.');
  }
  const reason = reasonOf(b);
  const expires = target === 'free' ? null : expiryOf(b);
  // Out with the old grant (soft, restorable), in with the new one.
  const revoked: number[] = [];
  if (target === 'free' || live.length) {
    const at = iso();
    for (const g of live) {
      const r = await env.DB.prepare('UPDATE grants SET revoked_at = ?2, revoked_by = ?3, updated_at = ?2 WHERE (id = ?1 OR parent_id = ?1) AND revoked_at IS NULL RETURNING id')
        .bind(g.id, at, admin).all<{ id: number }>();
      revoked.push(...(r.results ?? []).map((x) => x.id));
    }
  }
  if (target === 'free') {
    await audit(env, admin, 'plan.change', u.email, { user: u.id, before: before.label, after: 'Free', revoked_grants: revoked, reason });
    return json({ ok: true, revoked });
  }
  const members = target === 'team' && Array.isArray(b.members) ? b.members.map(String).filter((m) => m.trim()) : [];
  const entries: Entry[] = target === 'team' ? [{ raw: u.email, role: 'owner' }, ...members.map((m) => ({ raw: m, role: 'member' as Role }))] : [{ raw: u.email, role: 'single' }];
  if (members.length > POLAR.people - 1) throw new HttpError(400, 'too_many_members', `A Team is up to ${POLAR.people} people: the owner and ${POLAR.people - 1} teammates.`, 'members');
  // Put the old grant back when the new one is refused: the change did not go through.
  const undo = async () => {
    if (revoked.length) await env.DB.prepare(`UPDATE grants SET revoked_at = NULL, revoked_by = NULL WHERE id IN (${revoked.map(() => '?').join(',')})`).bind(...revoked).run();
  };
  let res: Response;
  try {
    res = await writeGrants(env, admin, target as 'pro' | 'team', entries, { reason, expires, founder: b.founder === true }, { ...b, confirm: u.email }, 'plan.grant');
  } catch (e) { await undo(); throw e; }
  if (!res.ok) { await undo(); return res; }
  const after = await oneUser(env, id);
  await audit(env, admin, 'plan.change', u.email, { user: u.id, before: before.label, after: after.plan.label, revoked_grants: revoked, reason });
  return json({ ok: true, revoked, plan: after.plan }, 201);
}

// ── Activity ───────────────────────────────────────────────────────────────────────────────────
export async function activity(env: Env, ctx: Ctx, url: URL): Promise<Response> {
  const days = [7, 30, 90].includes(Number(url.searchParams.get('days'))) ? Number(url.searchParams.get('days')) : 30;
  const tz = tzOf(url);
  const t = nowS();
  const since = t - days * 86_400;
  const sinceIso = new Date(since * 1000).toISOString();
  const today = Math.floor(todayStart(tz).getTime() / 1000);
  const off = `${tz >= 0 ? '+' : ''}${tz} minutes`;
  // Days of the chart, in the admin's time zone, oldest first.
  const dayKeys: string[] = [];
  for (let i = days - 1; i >= 0; i--) dayKeys.push(new Date((today - i * 86_400 + tz * 60) * 1000).toISOString().slice(0, 10));
  const chartStart = today - (days - 1) * 86_400;
  const q = (sql: string, ...v: unknown[]) => env.DB.prepare(sql).bind(...v);
  const res = await env.DB.batch([
    q('SELECT COUNT(DISTINCT user_id) AS n FROM events WHERE user_id IS NOT NULL AND at >= ?1', t - 86_400), // 0 DAU
    q('SELECT COUNT(DISTINCT user_id) AS n FROM events WHERE user_id IS NOT NULL AND at >= ?1', t - 7 * 86_400), // 1 WAU
    q('SELECT COUNT(DISTINCT user_id) AS n FROM events WHERE user_id IS NOT NULL AND at >= ?1', t - 30 * 86_400), // 2 MAU
    q("SELECT COUNT(DISTINCT visitor) AS v, COUNT(*) AS n FROM events WHERE type = 'view' AND at >= ?1", today), // 3 today
    q(`SELECT date(created_at, '${off}') AS d, COUNT(*) AS n FROM users WHERE created_at >= ?1 GROUP BY d`, new Date(chartStart * 1000).toISOString()), // 4 signups/day
    q(`SELECT date(at, 'unixepoch', '${off}') AS d, COUNT(*) AS n, COUNT(DISTINCT visitor) AS v FROM events WHERE type = 'view' AND at >= ?1 GROUP BY d`, chartStart), // 5 views/day
    q("SELECT item, COUNT(*) AS n FROM events WHERE type = 'signin' AND at >= ?1 GROUP BY item ORDER BY n DESC", since), // 6 sign-ins
    q("SELECT item, MAX(path) AS path, COUNT(*) AS n, COUNT(DISTINCT visitor) AS v FROM events WHERE type = 'view' AND item IS NOT NULL AND at >= ?1 GROUP BY item ORDER BY n DESC LIMIT 12", since), // 7 top
    q(`SELECT item, COUNT(*) AS n, SUM(detail = 'copy') AS copy, SUM(detail = 'keys') AS keys, SUM(detail = 'cli') AS cli, SUM(detail = 'menu') AS menu,
         (SELECT COUNT(*) FROM events v WHERE v.type = 'view' AND v.item = e.item AND v.at >= ?1) AS views
       FROM events e WHERE type = 'pro_copy' AND at >= ?1 GROUP BY item ORDER BY n DESC LIMIT 15`, since), // 8 pro copy
    q("SELECT COUNT(*) AS n, COUNT(DISTINCT visitor) AS v FROM events WHERE type = 'pricing_view' AND at >= ?1", since), // 9 pricing
    q("SELECT item, COUNT(*) AS n FROM events WHERE type = 'pricing_view' AND item IS NOT NULL AND at >= ?1 GROUP BY item ORDER BY n DESC LIMIT 5", since), // 10 pricing from
    q("SELECT item, COUNT(*) AS n FROM events WHERE type = 'checkout_click' AND at >= ?1 GROUP BY item ORDER BY n DESC", since), // 11 checkouts started
    q('SELECT plan, founder, COUNT(*) AS n FROM subscriptions WHERE created_at >= ?1 GROUP BY plan, founder', sinceIso), // 12 paid
    q(`SELECT COUNT(*) AS n FROM subscriptions WHERE (status IN ('canceled', 'revoked', 'unpaid') OR ended_at IS NOT NULL) AND updated_at >= ?1`, new Date((t - 30 * 86_400) * 1000).toISOString()), // 13 churn 30d
    q(`SELECT COUNT(*) AS n FROM subscriptions WHERE status IN (${ACTIVE_SQL}) AND ended_at IS NULL AND cancel_at_period_end = 1`), // 14 canceling
    q(`SELECT COUNT(*) AS n FROM grants WHERE revoked_at IS NULL AND (expires_at IS NULL OR expires_at > ?1)`, iso()), // 15 grants
    q('SELECT COUNT(*) AS n FROM users'), // 16 users
    q("SELECT COUNT(*) AS n, MIN(at) AS first FROM events"), // 17 events kept
  ]);
  const rows = <T>(i: number) => (res[i].results ?? []) as T[];
  const one = <T>(i: number) => (rows<T>(i)[0] ?? {}) as T;
  const n = (i: number) => Number(one<{ n: number }>(i).n ?? 0);
  const signups = new Map(rows<{ d: string; n: number }>(4).map((x) => [x.d, x.n]));
  const views = new Map(rows<{ d: string; n: number; v: number }>(5).map((x) => [x.d, x]));
  const bill = await billingSummary(env);
  const churned = n(13);
  const startedBy = rows<{ item: string; n: number }>(11);
  const paidBy = rows<{ plan: string; founder: number; n: number }>(12);
  if (Math.random() < 0.2) ctx.waitUntil(prune(env));
  return json({
    ok: true,
    days, tz,
    generated_at: iso(),
    active: { dau: n(0), wau: n(1), mau: n(2) },
    today: { visitors: Number(one<{ v: number }>(3).v ?? 0), views: Number(one<{ n: number }>(3).n ?? 0) },
    series: dayKeys.map((d) => ({ day: d, signups: signups.get(d) ?? 0, views: views.get(d)?.n ?? 0, visitors: views.get(d)?.v ?? 0 })),
    signins: rows<{ item: string; n: number }>(6).map((x) => ({ provider: x.item || 'unknown', n: x.n })),
    top: rows<{ item: string; path: string; n: number; v: number }>(7).map((x) => ({ item: x.item, path: x.path, kind: x.path.startsWith('/screens/') ? 'screen' : 'component', views: x.n, visitors: x.v })),
    pro_copy: rows<{ item: string; n: number; copy: number; keys: number; cli: number; menu: number; views: number }>(8),
    pricing: { views: Number(one<{ n: number }>(9).n ?? 0), visitors: Number(one<{ v: number }>(9).v ?? 0), from: rows<{ item: string; n: number }>(10) },
    checkouts: { started: startedBy.reduce((a, x) => a + x.n, 0), started_by_plan: startedBy, paid: paidBy.reduce((a, x) => a + x.n, 0), paid_by_plan: paidBy },
    subs: { active: bill.counts.active, pro: bill.counts.pro - bill.counts.founder, founder: bill.counts.founder, team: bill.counts.team, canceling: n(14), grants: n(15) },
    mrr_cents: bill.mrr_cents,
    arr_cents: bill.arr_cents,
    churn: { canceled_30d: churned, rate: bill.counts.active + churned ? churned / (bill.counts.active + churned) : 0 },
    users: n(16),
    retention_days: EVENTS.retentionDays,
    events_kept: n(17),
  });
}
