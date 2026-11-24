// Site accounts in the browser (site/worker/auth.ts, docs/auth.md). Loaded once per page by the
// navbar account control (src/components/v2/AccountMenu.astro). Pages are static and cached; this
// asks GET /api/auth/me only when the oxyn_signed_in hint cookie says a session may exist, or on
// the sign-in, sign-up and account pages themselves.
(function () {
  if (window.__oxynAccount) return;
  window.__oxynAccount = true;

  const $ = (s, r = document) => r.querySelector(s);
  const $$ = (s, r = document) => [...r.querySelectorAll(s)];
  const hint = /(?:^|;\s*)oxyn_signed_in=1/.test(document.cookie);
  const authCard = $('[data-auth]');
  const account = $('[data-account]');
  const params = new URLSearchParams(location.search);

  /** A same-site path to return to after signing in; never the sign-in pages or the API. */
  function nextPath() {
    const n = params.get('next') || '';
    if (n.startsWith('/') && !n.startsWith('//') && !n.startsWith('/\\') && !/^\/(api|login|signup)(\/|$)/.test(n) && n.length < 512) return n;
    return '/account/';
  }

  async function api(path, body) {
    const init = body === undefined
      ? { credentials: 'same-origin', headers: { accept: 'application/json' } }
      : { method: 'POST', credentials: 'same-origin', headers: { 'content-type': 'application/json', accept: 'application/json' }, body: JSON.stringify(body) };
    let res, data = null;
    try { res = await fetch(path, init); } catch { return { ok: false, status: 0, data: { error: { message: 'Could not reach Oxyn. Check your connection and try again.' } } }; }
    try { data = await res.json(); } catch { data = null; }
    return { ok: res.ok, status: res.status, data: data || {} };
  }

  function initials(u) {
    const src = (u.name || u.email || '?').trim();
    const parts = src.split(/[\s._-]+/).filter(Boolean);
    return ((parts[0] || '?')[0] + (u.name && parts[1] ? parts[1][0] : '')).toUpperCase();
  }
  function paintAvatar(el, u) {
    el.textContent = '';
    if (u.avatar_url && /^https:\/\//.test(u.avatar_url)) {
      const img = new Image();
      img.alt = '';
      img.decoding = 'async';
      img.referrerPolicy = 'no-referrer';
      img.src = u.avatar_url + (u.avatar_url.includes('?') ? '&' : '?') + 's=96';
      img.onerror = () => { el.textContent = initials(u); };
      el.appendChild(img);
    } else el.textContent = initials(u);
  }

  // ── Navbar control ───────────────────────────────────────────────────────────────────────────
  const acct = $('[data-acct]');
  const btn = acct && $('[data-acct-btn]', acct);
  const menu = acct && $('[data-acct-menu]', acct);
  const items = () => (menu ? $$('[role="menuitem"]', menu).filter((x) => !x.hidden) : []);

  function openMenu(focusFirst) {
    menu.hidden = false;
    btn.setAttribute('aria-expanded', 'true');
    if (focusFirst) items()[0]?.focus();
  }
  function closeMenu(refocus) {
    if (!menu || menu.hidden) return;
    menu.hidden = true;
    btn.setAttribute('aria-expanded', 'false');
    if (refocus) btn.focus();
  }
  if (btn && menu) {
    btn.addEventListener('click', () => (menu.hidden ? openMenu(false) : closeMenu(false)));
    btn.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowDown' || e.key === 'Enter' || e.key === ' ') { e.preventDefault(); openMenu(true); }
    });
    menu.addEventListener('keydown', (e) => {
      const list = items();
      const i = list.indexOf(document.activeElement);
      if (e.key === 'ArrowDown') { e.preventDefault(); list[(i + 1) % list.length].focus(); }
      else if (e.key === 'ArrowUp') { e.preventDefault(); list[(i - 1 + list.length) % list.length].focus(); }
      else if (e.key === 'Home') { e.preventDefault(); list[0].focus(); }
      else if (e.key === 'End') { e.preventDefault(); list[list.length - 1].focus(); }
      else if (e.key === 'Tab') closeMenu(false);
    });
    document.addEventListener('click', (e) => { if (!acct.contains(e.target)) closeMenu(false); });
    document.addEventListener('keydown', (e) => { if (e.key === 'Escape' && !menu.hidden) { e.preventDefault(); closeMenu(true); } });
  }

  function showSignedIn(u, me) {
    // Admin: maintainers whose email Google or GitHub verified, plus admin emails that still need
    // that check (the panel explains it). Everyone else never sees the link.
    $$('[data-acct-admin]').forEach((a) => { a.hidden = !(me.is_admin || u.admin_needs_verification); });
    if (!acct) return;
    acct.dataset.state = 'in';
    $('[data-acct-signin]', acct).hidden = true;
    btn.hidden = false;
    btn.setAttribute('aria-label', `Account menu, signed in as ${u.email}`);
    $$('[data-acct-av]').forEach((el) => paintAvatar(el, u));
    const nm = $('[data-acct-name]', acct);
    nm.textContent = u.name || u.email.split('@')[0];
    $('[data-acct-email]', acct).textContent = u.email;
  }
  function showSignedOut() {
    if (!acct) return;
    acct.dataset.state = 'out';
    closeMenu(false);
    btn.hidden = true;
    const a = $('[data-acct-signin]', acct);
    a.hidden = false;
    // Come back to this page after signing in (not from the sign-in pages themselves).
    if (!/^\/(login|signup)\/?$/.test(location.pathname)) a.href = '/login/?next=' + encodeURIComponent(location.pathname + location.search);
  }

  async function signOut(all) {
    const r = await api(all ? '/api/auth/logout-all' : '/api/auth/logout', {});
    if (!r.ok && all) {
      const err = $('[data-account-err]');
      if (err) { err.textContent = r.data.error?.message || 'Could not sign out. Try again.'; err.hidden = false; }
      return;
    }
    if (location.pathname.startsWith('/account')) location.assign('/login/');
    else location.reload();
  }
  $$('[data-acct-signout]').forEach((b) => b.addEventListener('click', () => signOut(false)));
  $('[data-account-all]')?.addEventListener('click', () => signOut(true));

  // ── Plan (docs/pricing.md) ─────────────────────────────────────────────────────────────────────
  // Signed in: GET /api/billing/me once per page. The answer goes on <html data-plan="free|pro|team">
  // and window.__oxynPlanP (a promise; null when signed out), and fires "oxyn:plan" for the page
  // scripts (public/pro.js, public/pricing.js).
  let planResolve;
  window.__oxynPlanP = new Promise((r) => { planResolve = r; });
  const TIER = { free: 'Free', pro: 'Pro', team: 'Team' };
  function setPlan(plan) {
    window.__oxynPlan = plan;
    document.documentElement.dataset.plan = plan ? plan.tier : 'none';
    if (plan && plan.pro) document.documentElement.dataset.pro = '1'; else delete document.documentElement.dataset.pro;
    // Admins (maintainers) have everything open: no locks, no upgrade prompts, "Admin" as the plan.
    if (plan && plan.admin) document.documentElement.dataset.admin = '1'; else delete document.documentElement.dataset.admin;
    // Remembered for the next page's first paint (Head.astro), so Get Pro does not flash for members.
    try { if (plan && plan.pro) localStorage.setItem('oxyn-plan-hint', plan.admin ? 'admin' : 'pro'); else localStorage.removeItem('oxyn-plan-hint'); } catch {}
    planResolve(plan);
    document.dispatchEvent(new CustomEvent('oxyn:plan', { detail: plan }));
    const label = (p) => (p ? (p.admin ? 'Admin' : TIER[p.tier] || 'Free') : 'Free');
    $$('[data-acct-plan]').forEach((el) => { el.textContent = label(plan); el.dataset.tier = plan ? (plan.admin ? 'admin' : plan.tier) : 'free'; });
    $$('[data-acct-billing]').forEach((a) => { a.hidden = !(plan && plan.has_billing && plan.via !== 'seat'); });
    fillPlan(plan);
  }
  const fmtDay = (s) => { const d = new Date(s); return isNaN(d) ? '' : d.toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' }); };
  const money = (c, cur) => { try { return (c / 100).toLocaleString(undefined, { style: 'currency', currency: (cur || 'usd').toUpperCase() }); } catch { return '$' + (c / 100).toFixed(2); } };
  function fillPlan(plan) {
    const box = $('[data-plan]');
    if (!box || !plan) return;
    const chip = $('[data-plan-chip]', box), line = $('[data-plan-line]', box), note = $('[data-plan-note]', box);
    const up = $('[data-plan-upgrade]', box);
    const s = plan.subscription;
    chip.textContent = plan.admin ? 'Admin' : TIER[plan.tier] || 'Free';
    chip.dataset.tier = plan.admin ? 'admin' : plan.tier;
    up.hidden = plan.pro;
    let text = line.dataset.freeLine || 'The free components and primitives, MIT licensed.';
    if (plan.admin) {
      text = 'Admin: every component, screen and block is open to you, whatever the plan.';
    } else if (plan.via === 'subscription' && s) {
      // Founder rate = intro price: the first month (or year), then the regular Pro price.
      const what = s.founder
        ? `Pro at the founder rate, ${money(s.amount, s.currency)} for your first ${s.interval}`
        : `${s.plan === 'team' ? 'Team' : 'Pro'}, ${money(s.amount, s.currency)} a ${s.interval}`;
      const when = s.current_period_end ? fmtDay(s.current_period_end) : '';
      const renew = s.founder && s.renews_at_amount
        ? ` Renews${when ? ` on ${when}` : ''} at the regular ${money(s.renews_at_amount, s.currency)} a ${s.interval}.`
        : when ? ` Renews on ${when}.` : '';
      text = s.cancel_at_period_end ? `${what}. Cancelled: Pro stays open until ${when || 'the end of the period'}.`
        : s.status === 'past_due' ? `${what}. The last payment did not go through; update your card under Billing below.`
        : `${what}.${renew}`;
    } else if (plan.via === 'seat' && plan.seat) {
      text = `Pro through a Team plan (seat from ${plan.seat.owner_email}).`;
    } else if (plan.via === 'grant' && plan.grant) {
      text = `${plan.grant.plan === 'team' ? 'Team' : 'Pro'}, given by the Oxyn team${plan.grant.expires_at ? ` until ${fmtDay(plan.grant.expires_at)}` : ''}.`;
    } else if (s && !s.active) {
      text = `Your ${s.plan === 'team' ? 'Team' : 'Pro'} subscription has ended. Code you installed keeps working; subscribe again to get the Pro code and updates.`;
    }
    line.textContent = text;
    cancelUi(box, plan);
    loadBilling(plan);
    const p = new URLSearchParams(location.search);
    if (p.get('billing') === 'active') { note.textContent = 'You already have an active plan. Change it under Billing below.'; note.hidden = false; }
    else if (!plan.pro && !plan.email_verified && plan.tier === 'free') { note.textContent = 'Got a Team seat? Sign in once with Google or GitHub on this email to verify it, and the seat opens.'; note.hidden = false; }
    // Team owner: seats
    const seats = $('[data-seats]', box);
    if (!seats) return;
    seats.hidden = !plan.seats;
    if (!plan.seats) return;
    const list = $('[data-seat-list]', seats);
    $('[data-seat-count]', seats).textContent = `${plan.seats.length} of ${plan.seat_limit} teammates`;
    list.textContent = '';
    for (const st of plan.seats) {
      const li = document.createElement('li');
      const em = document.createElement('span'); em.textContent = st.email;
      const rm = document.createElement('button'); rm.type = 'button'; rm.className = 'x-pill pill-grey pill-sm'; rm.textContent = 'Remove';
      rm.setAttribute('aria-label', `Remove ${st.email}`);
      rm.addEventListener('click', async () => {
        rm.disabled = true;
        const r = await apiDelete('/api/billing/seats/' + encodeURIComponent(st.email));
        if (r.ok) loadPlan(); else { seatErr(r.data.error?.message || 'Could not remove that seat.'); rm.disabled = false; }
      });
      li.append(em, rm); list.append(li);
    }
    const form = $('[data-seat-form]', seats);
    form.hidden = plan.seats.length >= plan.seat_limit;
  }
  // Cancel at the period end / resume, for the account's own live subscription (not a Team seat,
  // not a grant). Polar keeps Pro open until the period end and refunds nothing.
  function cancelUi(box, plan) {
    const s = plan.subscription;
    const own = !!(s && s.active); // its own live subscription (a seat or grant is not)
    const cancelB = $('[data-plan-cancel]', box), resumeB = $('[data-plan-resume]', box), confirm = $('[data-plan-confirm]', box);
    if (!cancelB || !resumeB || !confirm) return;
    const when = s && s.current_period_end ? fmtDay(s.current_period_end) : '';
    cancelB.hidden = !own || s.cancel_at_period_end;
    resumeB.hidden = !own || !s.cancel_at_period_end;
    confirm.hidden = true;
    planErr('');
    if (!own) return;
    if (s.cancel_at_period_end) {
      $('[data-plan-line]', box).textContent = `Cancels on ${when || 'the last day of this period'}. Pro stays open until then, and the plan will not renew.`;
    }
    $('[data-plan-confirm-msg]', confirm).textContent = `Your plan stays active until ${when || 'the end of this billing period'}, then it won't renew. Nothing is refunded, and you can resume any time before then.`;
    cancelB.onclick = () => { confirm.hidden = false; cancelB.hidden = true; $('[data-plan-confirm-no]', confirm).focus(); };
    $('[data-plan-confirm-no]', confirm).onclick = () => { confirm.hidden = true; cancelB.hidden = false; cancelB.focus(); };
    const send = async (btn, path) => {
      btn.disabled = true; planErr('');
      const r = await api(path, {});
      btn.disabled = false;
      if (r.ok) { await loadPlan(); const f = $(path.endsWith('cancel') ? '[data-plan-resume]' : '[data-plan-cancel]', box); if (f && !f.hidden) f.focus(); }
      else planErr((r.data.error && r.data.error.message) || 'That did not work. Please try again.');
    };
    $('[data-plan-confirm-yes]', confirm).onclick = (e) => send(e.currentTarget, '/api/billing/cancel');
    resumeB.onclick = (e) => send(e.currentTarget, '/api/billing/resume');
  }
  // ── Billing (account page, #billing): read live from Polar through the Worker (worker/billing.ts).
  // Plan switches, payments and receipts happen here; only "Update card" opens Polar's card page.
  const PLAN_NAME = { pro: 'Pro', team: 'Team' };
  const per = (i) => (i === 'year' ? 'a year' : 'a month');
  const planName = (p) => `${p.founder ? 'Pro (founder rate)' : PLAN_NAME[p.plan] || 'Pro'} ${p.interval === 'year' ? 'yearly' : 'monthly'}`;
  let billSeq = 0;
  async function loadBilling(plan) {
    const box = $('[data-bill]');
    if (!box) return;
    const show = !!(plan && (plan.has_billing || (plan.subscription && plan.via === 'subscription')));
    box.hidden = !show;
    $$('[data-plan-manage]').forEach((a) => { a.hidden = !show; });
    if (!show) return;
    const seq = ++billSeq;
    const [r, o] = await Promise.all([api('/api/billing/subscription'), api('/api/billing/invoices')]);
    if (seq !== billSeq) return;
    $('[data-bill-load]', box).hidden = true;
    billErr(r.ok ? '' : (r.data.error && r.data.error.message) || 'Billing could not be loaded. Please try again.');
    if (r.ok) fillBilling(box, r.data);
    fillOrders(box, o.ok ? o.data.items : null, o.ok ? '' : (o.data.error && o.data.error.message));
    const p = new URLSearchParams(location.search);
    if (p.get('billing') === 'card_unavailable') billErr('The card page could not be opened right now. Please try again in a minute.');
  }
  function fillBilling(box, d) {
    const s = d.subscription;
    const sum = $('[data-bill-sum]', box), pend = $('[data-bill-pending]', box), sw = $('[data-bill-switch]', box);
    sum.hidden = !s; pend.hidden = true; sw.hidden = true;
    if (!s) return;
    $('[data-bill-plan]', box).textContent = planName(s.plan) + (s.status === 'past_due' ? ' · payment failed' : s.cancel_at_period_end ? ' · cancels at the period end' : '');
    $('[data-bill-price]', box).textContent = s.plan.founder
      ? `${money(s.amount, s.currency)} for your first ${s.plan.interval} (founder intro price)`
      : `${money(s.amount, s.currency)} ${per(s.plan.interval)}`;
    const end = fmtDay(s.current_period_end);
    let next = '';
    if (s.cancel_at_period_end) next = `None. The plan ends on ${end}.`;
    else if (s.next) {
      next = `${money(s.next.cents, s.currency)} on ${fmtDay(s.next.date)}`;
      if (s.next.founder_ends && s.next.plan.key !== s.plan.key) next += `: the founder price ends and ${planName(s.next.plan)} starts at the regular price`;
      else if (s.next.plan.key !== s.plan.key) next += ` for ${planName(s.next.plan)}`;
    }
    $('[data-bill-next]', box).textContent = next || '—';
    const c = d.card;
    $('[data-bill-card]', box).textContent = c ? `${c.brand.charAt(0).toUpperCase() + c.brand.slice(1)} ending in ${c.last4}, expires ${String(c.exp_month).padStart(2, '0')}/${String(c.exp_year).slice(-2)}` : 'The card you paid with';
    $('[data-bill-card-up]', box).hidden = false;
    $('[data-bill-card-note]', box).hidden = false;
    // A switch the owner scheduled for the renewal (not the founder price ending, which shows above).
    if (s.pending && !s.pending.founder_renewal) {
      pend.hidden = false;
      $('[data-bill-pending-msg]', box).textContent = `Switching to ${planName(s.pending.plan)} on ${fmtDay(s.pending.applies_at)}. Nothing changes before then.`;
      const keep = $('[data-bill-keep]', box);
      keep.hidden = false;
      keep.onclick = async () => {
        keep.disabled = true; billErr('');
        const r = await api('/api/billing/keep', {});
        keep.disabled = false;
        if (r.ok) { await loadPlan(); } else billErr((r.data.error && r.data.error.message) || 'That did not work. Please try again.');
      };
    }
    const why = $('[data-bill-why]', box), opts = $('[data-bill-opts]', box);
    sw.hidden = false; opts.textContent = ''; why.hidden = true;
    $('[data-bill-confirm]', box).hidden = true;
    if (s.blocked) {
      why.hidden = false;
      why.textContent = s.blocked === 'cancelling' ? 'Your plan is set to cancel. Resume it above to change plan.'
        : s.blocked === 'past_due' ? 'The last payment did not go through. Update your card, then change plan.'
        : 'This plan cannot be changed right now.';
      return;
    }
    for (const op of s.options) {
      const b = document.createElement('button');
      b.type = 'button'; b.className = 'x-bill-opt'; b.setAttribute('aria-pressed', 'false');
      const t = document.createElement('b'); t.textContent = planName(op);
      const pr = document.createElement('span'); pr.textContent = `${money(op.cents, s.currency)} ${per(op.interval)}`;
      const w = document.createElement('em'); w.textContent = op.scheduled ? 'Your plan from the renewal' : op.when === 'now' ? 'Starts now' : 'Starts at your renewal';
      b.append(t, pr, w);
      b.disabled = !!op.scheduled;
      b.addEventListener('click', () => confirmSwitch(box, op, b));
      opts.append(b);
    }
  }
  async function confirmSwitch(box, op, btn) {
    const cf = $('[data-bill-confirm]', box), msg = $('[data-bill-confirm-msg]', box);
    $$('.x-bill-opt', box).forEach((x) => x.setAttribute('aria-pressed', String(x === btn)));
    billErr('');
    btn.disabled = true;
    const r = await api('/api/billing/preview?plan=' + encodeURIComponent(op.key));
    btn.disabled = false;
    if (!r.ok) { billErr((r.data.error && r.data.error.message) || 'That plan cannot be chosen right now.'); return; }
    const p = r.data.preview;
    $('[data-bill-confirm-h]', box).textContent = `Switch to ${planName(p.to)}?`;
    const later = `${money(p.next_charge.cents, p.currency)} ${per(p.to.interval)} from ${fmtDay(p.next_charge.date)}`;
    msg.textContent = p.when === 'now'
      ? `Team starts now. You are charged about ${money(p.charge_now_cents, p.currency)} today: the difference for the rest of this period (plus any tax, on Polar's invoice), to the card on file. Then ${later}.`
      : `Nothing is charged now. You keep ${planName(p.from)} until ${fmtDay(p.next_charge.date)}, then ${later}.${p.seats_end ? ' Your teammates lose their Team seats on that day.' : ''}`;
    cf.hidden = false;
    const yes = $('[data-bill-confirm-yes]', box), no = $('[data-bill-confirm-no]', box);
    yes.textContent = p.when === 'now' ? `Pay and switch` : 'Schedule the switch';
    yes.focus();
    no.onclick = () => { cf.hidden = true; btn.setAttribute('aria-pressed', 'false'); btn.focus(); };
    yes.onclick = async () => {
      yes.disabled = true; billErr('');
      const c = await api('/api/billing/change', { plan: op.key });
      yes.disabled = false;
      if (c.ok) { cf.hidden = true; await loadPlan(); $('[data-bill]')?.scrollIntoView({ block: 'start' }); }
      else billErr((c.data.error && c.data.error.message) || 'That did not work. Please try again.');
    };
  }
  const REASON = { subscription_create: 'New subscription', subscription_cycle: 'Renewal', subscription_update: 'Plan change', purchase: 'Purchase' };
  function fillOrders(box, items, err) {
    const wrap = $('[data-bill-orders]', box), list = $('[data-bill-list]', box), empty = $('[data-bill-empty]', box);
    wrap.hidden = false; list.textContent = '';
    if (!items) { empty.hidden = false; empty.textContent = err || 'Payments could not be loaded.'; return; }
    empty.hidden = items.length > 0; empty.textContent = 'No payments yet.';
    for (const o of items) {
      const li = document.createElement('li');
      const row = document.createElement('span'); row.className = 'x-bill-row';
      const top = document.createElement('span'); top.textContent = `${fmtDay(o.created_at)} · ${money(o.total_cents, o.currency)}`;
      const sm = document.createElement('small');
      sm.textContent = [o.description, REASON[o.billing_reason] || '', o.status === 'paid' ? '' : o.status.replace('_', ' '), o.invoice_number || ''].filter(Boolean).join(' · ');
      row.append(top, sm);
      const btns = document.createElement('span'); btns.className = 'x-bill-dl-btns';
      for (const kind of o.receipt ? ['receipt', 'invoice'] : []) {
        const b = document.createElement('button'); b.type = 'button'; b.className = 'x-pill pill-grey pill-sm';
        b.textContent = kind === 'receipt' ? 'Receipt' : 'Invoice';
        b.setAttribute('aria-label', `${b.textContent} PDF for ${fmtDay(o.created_at)}`);
        b.addEventListener('click', () => download(b, o.id, kind));
        btns.append(b);
      }
      li.append(row, btns); list.append(li);
    }
  }
  async function download(btn, id, kind, tries = 0) {
    btn.disabled = true; billErr('');
    const r = await api(`/api/billing/invoices/${id}/${kind}`);
    if (r.ok && r.data.url) { btn.disabled = false; location.href = r.data.url; return; }
    if (r.status === 202 && tries < 5) { setTimeout(() => download(btn, id, kind, tries + 1), 2500); return; }
    btn.disabled = false;
    billErr((r.data.error && r.data.error.message) || r.data.message || 'That PDF is not ready yet. Please try again in a moment.');
  }
  function billErr(t) { const e = $('[data-bill-err]'); if (e) { e.textContent = t || ''; e.hidden = !t; } }

  function planErr(t) { const e = $('[data-plan-err]'); if (e) { e.textContent = t || ''; e.hidden = !t; } }
  function seatErr(t) { const e = $('[data-seat-err]'); if (e) { e.textContent = t || ''; e.hidden = !t; } }
  async function apiDelete(path) {
    try { const r = await fetch(path, { method: 'DELETE', credentials: 'same-origin', headers: { accept: 'application/json' } }); let d = {}; try { d = await r.json(); } catch { /* empty */ } return { ok: r.ok, data: d }; }
    catch { return { ok: false, data: { error: { message: 'Could not reach Oxyn. Check your connection and try again.' } } }; }
  }
  async function loadPlan() {
    const r = await api('/api/billing/me');
    setPlan(r.ok ? r.data : null);
  }
  const seatForm = $('[data-seat-form]');
  if (seatForm) seatForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    const em = seatForm.elements.email.value.trim();
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/.test(em)) { seatErr('Enter a valid email address.'); return; }
    seatErr('');
    const b = seatForm.querySelector('button'); b.disabled = true;
    const r = await api('/api/billing/seats', { email: em });
    b.disabled = false;
    if (r.ok) { seatForm.reset(); loadPlan(); } else seatErr(r.data.error?.message || 'Could not add that seat.');
  });

  // ── Sign-in / sign-up card ───────────────────────────────────────────────────────────────────
  const GH_ERRORS = {
    google_state: 'That Google sign-in expired or came from another tab. Please try again.',
    google_denied: 'Google sign-in was cancelled.',
    google_email: 'Google did not confirm an email address for that account. Use another account, or your email here.',
    google_failed: 'Google sign-in did not work this time. Please try again.',
    github_state: 'That GitHub sign-in expired or came from another tab. Please try again.',
    github_denied: 'GitHub sign-in was cancelled.',
    github_email: 'Your GitHub account has no verified email address. Add one on GitHub, or use your email here.',
    github_failed: 'GitHub sign-in did not work this time. Please try again.',
  };
  function setupCard(me) {
    if (!authCard) return;
    const form = $('[data-auth-form]', authCard);
    const err = $('[data-auth-err]', authCard);
    const go = $('[data-auth-go]', authCard);
    const gh = $('[data-auth-gh]', authCard);
    const gg = $('[data-auth-gg]', authCard);
    const ghNote = $('[data-auth-ghnote]', authCard);
    const mode = authCard.dataset.auth;
    const next = nextPath();
    const email = form.elements.email;
    const pw = form.elements.password;
    const label = go.textContent;

    $$('[data-auth-keep-next]', authCard).forEach((a) => { if (params.get('next')) a.href += '?next=' + encodeURIComponent(next); });
    const showErr = (t, field) => {
      err.textContent = t || ''; err.hidden = !t;
      [email, pw].forEach((f) => f.removeAttribute('aria-invalid'));
      if (field === 'email') email.setAttribute('aria-invalid', 'true');
      if (field === 'password') pw.setAttribute('aria-invalid', 'true');
    };
    const code = params.get('error');
    if (code && GH_ERRORS[code]) showErr(GH_ERRORS[code]);

    // Turn on each provider the server reports as set up; the note names whatever is still off.
    const enable = (btn, path) => {
      btn.removeAttribute('aria-disabled');
      btn.removeAttribute('tabindex');
      btn.removeAttribute('aria-describedby');
      btn.href = path + '?next=' + encodeURIComponent(next);
    };
    const ggOn = !!(me && me.google), ghOn = !!(me && me.github);
    if (ggOn) enable(gg, '/api/auth/google/start');
    if (ghOn) enable(gh, '/api/auth/github/start');
    const off = [!ggOn && 'Google', !ghOn && 'GitHub'].filter(Boolean);
    ghNote.textContent = off.length ? off.join(' and ') + ' sign-in coming soon' : '';
    ghNote.hidden = !off.length;
    [gg, gh].forEach((b) => b.addEventListener('click', (e) => { if (b.getAttribute('aria-disabled') === 'true') e.preventDefault(); }));

    const eye = $('[data-auth-eye]', authCard);
    eye.addEventListener('click', () => {
      const show = pw.type === 'password';
      pw.type = show ? 'text' : 'password';
      eye.setAttribute('aria-pressed', String(show));
      eye.setAttribute('aria-label', show ? 'Hide password' : 'Show password');
      eye.textContent = show ? 'Hide' : 'Show';
    });

    let busy = false;
    form.addEventListener('submit', async (e) => {
      e.preventDefault();
      if (busy) return;
      const em = email.value.trim();
      if (!em || !/^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/.test(em)) { showErr('Enter a valid email address.', 'email'); email.focus(); return; }
      if (!pw.value) { showErr('Enter your password.', 'password'); pw.focus(); return; }
      if (mode === 'signup' && pw.value.length < 10) { showErr('Use at least 10 characters.', 'password'); pw.focus(); return; }
      busy = true; go.disabled = true; go.textContent = mode === 'login' ? 'Signing in…' : 'Creating account…';
      showErr('');
      const r = await api(mode === 'login' ? '/api/auth/login' : '/api/auth/signup', { email: em, password: pw.value });
      if (r.ok) { location.assign(next); return; }
      busy = false; go.disabled = false; go.textContent = label;
      const er = r.data.error || {};
      showErr(er.message || 'Something went wrong. Please try again.', er.field);
      (er.field === 'email' ? email : pw).focus();
    });
  }

  // ── Account page ─────────────────────────────────────────────────────────────────────────────
  const NAMES = { password: 'Email and password', github: 'GitHub', google: 'Google' };
  function fillAccount(me) {
    if (!account) return;
    account.removeAttribute('aria-busy');
    $('[data-account-loading]', account).hidden = true;
    if (!me || !me.signed_in) {
      account.dataset.state = 'out';
      $('[data-account-out]', account).hidden = false;
      return;
    }
    const u = me.user;
    account.dataset.state = 'in';
    $('[data-account-in]', account).hidden = false;
    $('[data-account-name]', account).textContent = u.name || 'Your account';
    $('[data-account-email]', account).textContent = u.email;
    $('[data-account-email2]', account).textContent = u.email;
    $('[data-account-providers]', account).textContent = (u.providers.length ? u.providers : [u.session_provider]).map((p) => NAMES[p] || p).join(', ');
    const d = new Date(u.created_at);
    $('[data-account-created]', account).textContent = isNaN(d) ? u.created_at : d.toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' });
  }

  async function boot() {
    const needed = hint || authCard || account;
    let me = null;
    if (needed) {
      const r = await api('/api/auth/me');
      if (r.ok) me = r.data;
    }
    if (me && me.signed_in) {
      showSignedIn(me.user, me);
      loadPlan();
      // Already signed in: the sign-in pages have nothing to do, so go on.
      if (authCard) { location.replace(nextPath()); return; }
    } else { showSignedOut(); setPlan(null); }
    setupCard(me);
    fillAccount(me);
  }
  boot();
})();
