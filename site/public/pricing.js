// The pricing page (/pricing/) and the thank-you page (/pricing/success/). docs/pricing.md.
// Pricing: monthly/yearly switch, founder spots from /api/billing/plans (the founder rate disappears
// at 100), "Your plan" on the current tier, notes for ?founder=closed, ?billing=active and
// ?from=<slug> (the Pro item a visitor tried to copy), and ?buy=<plan> to carry on to checkout after
// signing in. Success: waits for Polar's webhook to turn the plan on, then offers the way back.
(function () {
  const $ = (s, r = document) => r.querySelector(s);
  const $$ = (s, r = document) => [...r.querySelectorAll(s)];
  const params = new URLSearchParams(location.search);
  const SLUG = /^[a-z0-9-]{1,80}$/;
  const planP = () => window.__oxynPlanP || Promise.resolve(null);

  // ── Thank-you page ────────────────────────────────────────────────────────────────────────
  const done = $('[data-pr-success]');
  if (done) {
    const h = $('[data-pr-done-h]', done), msg = $('[data-pr-done-msg]', done), go = $('[data-pr-done-go]', done);
    let back = null;
    try { back = JSON.parse(localStorage.getItem('oxyn_pro_return') || 'null'); } catch { back = null; }
    if (back && SLUG.test(back.slug || '') && /^\/(screens|components|blocks)\//.test(back.url || '')) {
      go.href = back.url;
      go.textContent = `Back to ${back.name || back.slug}`;
    }
    let tries = 0;
    const tick = async () => {
      tries++;
      let r = null;
      try { r = await fetch('/api/billing/me', { credentials: 'same-origin', headers: { accept: 'application/json' } }); } catch { r = null; }
      const me = r && r.ok ? await r.json() : null;
      if (r && r.status === 401) {
        msg.textContent = 'Payment received. Sign in with the email you paid with to use Pro.';
        go.href = '/login/?next=/account/'; go.textContent = 'Sign in'; go.hidden = false;
        done.dataset.state = 'signin';
        return;
      }
      if (me && me.pro) {
        h.textContent = me.tier === 'team' ? 'Welcome to Oxyn Team' : 'Welcome to Oxyn Pro';
        msg.textContent = me.tier === 'team'
          ? 'Your plan is on. Add your teammates on the Account page; the code of every screen and template is open to you now.'
          : 'Your plan is on. The code of every screen, agent-app template, block and kit is open to you now.';
        go.hidden = false;
        done.dataset.state = 'on';
        try { localStorage.removeItem('oxyn_pro_return'); } catch { /* ignore */ }
        return;
      }
      if (tries < 30) setTimeout(tick, 2000);
      else {
        msg.textContent = 'Payment received, but your plan has not shown up yet. It can take a minute; reload this page or check Your plan. If it still says Free, use the Feedback button and we will sort it out.';
        done.dataset.state = 'slow';
      }
    };
    tick();
    return;
  }

  // ── Pricing page ──────────────────────────────────────────────────────────────────────────
  const page = $('[data-pricing]');
  if (!page) return;
  const note = $('[data-pr-note]');
  const say = (t) => { note.textContent = t; note.hidden = !t; };
  const from = SLUG.test(params.get('from') || '') ? params.get('from') : '';
  let fromName = from.replace(/-/g, ' ').replace(/^./, (c) => c.toUpperCase());
  try {
    const f = JSON.parse(sessionStorage.getItem('oxyn_pro_from') || 'null');
    if (f && f.slug === from && f.name) fromName = f.name;
  } catch { /* ignore */ }

  function setInterval_(iv) {
    page.dataset.interval = iv;
    $$('[data-pr-int]').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.prInt === iv)));
    $$('[data-pr-buy]').forEach((a) => {
      const founder = page.dataset.founder === 'open' && a.dataset['kf' + (iv === 'year' ? 'Year' : 'Month')];
      const key = founder || a.dataset['k' + (iv === 'year' ? 'Year' : 'Month')];
      a.href = '/api/billing/checkout?plan=' + key + (from ? '&from=' + from : '');
      a.dataset.key = key;
    });
  }
  $$('[data-pr-int]').forEach((b, i, all) => {
    b.addEventListener('click', () => setInterval_(b.dataset.prInt));
    b.addEventListener('keydown', (e) => {
      if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return;
      e.preventDefault(); const o = all[1 - i]; o.focus(); o.click();
    });
  });
  // Activity: a pricing view (with the Pro item it came from) and each checkout click.
  try { window.oxynTrack?.('pricing_view', from || null); } catch { /* never block */ }
  $$('[data-pr-buy]').forEach((a) => a.addEventListener('click', () => {
    try { if (/^\/api\/billing\/checkout/.test(a.getAttribute('href') || '')) window.oxynTrack?.('checkout_click', a.dataset.key, null, true); } catch { /* never block */ }
  }));
  // Remember where to lead back to after checkout.
  $$('[data-pr-buy]').forEach((a) => a.addEventListener('click', () => {
    if (!from) return;
    let url = '/screens/' + from + '/';
    try { const f = JSON.parse(sessionStorage.getItem('oxyn_pro_from') || 'null'); if (f && f.slug === from && /^\/(screens|components|blocks)\//.test(f.url)) url = f.url; } catch { /* ignore */ }
    try { localStorage.setItem('oxyn_pro_return', JSON.stringify({ slug: from, name: fromName, url })); } catch { /* ignore */ }
  }));
  setInterval_(params.get('interval') === 'year' ? 'year' : 'month');

  // The founder rate is for new subscribers: someone who had an Oxyn subscription sees the regular price.
  let returning = params.get('founder') === 'used';
  if (returning) { page.dataset.founder = 'closed'; setInterval_(page.dataset.interval); }
  if (params.get('founder') === 'closed') say('The founder rate has just run out: all 100 spots are taken. Pro is still here at the regular price.');
  else if (returning) say('The founder rate is for new subscribers only, and this account has had an Oxyn subscription before. Pro is here at the regular price.');
  else if (params.get('billing') === 'active') say('You already have an active plan. Change it under Billing on your account page.');
  else if (from) say(`The code of ${fromName} is part of Oxyn Pro. Pick a plan; after checkout you can go straight back to it.`);

  fetch('/api/billing/plans', { headers: { accept: 'application/json' } })
    .then((r) => (r.ok ? r.json() : null))
    .then((d) => {
      if (!d || !d.founder) return;
      const f = d.founder;
      page.dataset.founder = f.open && !returning ? 'open' : 'closed';
      const left = $('[data-pr-left]');
      if (left) left.textContent = f.open ? `${f.remaining} of ${f.cap} spots left.` : '';
      setInterval_(page.dataset.interval);
    })
    .catch(() => {});

  planP().then((plan) => {
    if (plan && plan.subscription) {
      returning = true;
      if (page.dataset.founder === 'open') { page.dataset.founder = 'closed'; setInterval_(page.dataset.interval); }
    }
    if (plan) {
      $$('[data-tier]').forEach((t) => {
        const cur = t.dataset.tier === plan.tier;
        t.classList.toggle('pr-on', cur);
        const c = $('[data-pr-cur]', t); if (c) c.hidden = !cur;
      });
      if (plan.pro && plan.has_billing) $$('[data-pr-buy]').forEach((a) => { a.href = '/account/#billing'; $('[data-pr-label]', a).textContent = 'Change plan'; });
    }
    // Back from signing in with a plan picked: go on to checkout once (the URL forgets ?buy).
    const buy = params.get('buy');
    if (buy && /^[a-z-]{4,30}$/.test(buy) && plan && !(plan.pro && plan.via === 'subscription')) {
      params.delete('buy');
      history.replaceState(null, '', location.pathname + (params.toString() ? '?' + params : ''));
      say('Taking you to the secure checkout…');
      location.assign('/api/billing/checkout?plan=' + buy + (from ? '&from=' + from : ''));
    }
  });
})();
