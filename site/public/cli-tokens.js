// Account page (src/pages/account.astro): this month's Pro items from the plan account.js loads
// (/api/billing/me → usage), and the CLI / MCP tokens (/api/cli/tokens, site/worker/pro.ts). A new
// token is shown once; only its hash is kept.
(function () {
  const $ = (s, r = document) => r.querySelector(s);
  const box = $('[data-cli]');
  if (!box) return;
  const list = $('[data-tok-list]', box), form = $('[data-tok-form]', box), err = $('[data-tok-err]', box);
  const fresh = $('[data-tok-new]', box), val = $('[data-tok-value]', box), copy = $('[data-tok-copy]', box);
  const day = (s) => { const d = new Date(s); return isNaN(d) ? '' : d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' }); };
  const say = (t) => { err.textContent = t || ''; err.hidden = !t; };

  // ── Usage ──
  function usage(plan) {
    const u = plan && plan.usage, el = $('[data-usage]');
    if (!el) return;
    el.hidden = !u || u.plan === 'free';
    if (!u || u.plan === 'free') return;
    for (const k of ['components', 'screens']) {
      const x = u[k];
      $(`[data-usage-n="${k}"]`, el).textContent = x.limit == null ? `${x.used} opened` : `${x.used} of ${x.limit}`;
      const bar = $(`[data-usage-bar="${k}"]`, el);
      const pct = x.limit == null ? 0 : Math.min(100, Math.round((x.used / Math.max(1, x.limit)) * 100));
      bar.style.width = pct + '%';
      if (x.limit != null && x.used >= x.limit) bar.dataset.full = ''; else delete bar.dataset.full;
    }
    $('[data-usage-pool]', el).textContent = u.pooled ? 'shared by your Team' : u.unlimited ? 'no limit (admin)' : '';
    $('[data-usage-line]', el).textContent = u.unlimited
      ? 'Admins have no monthly limit.'
      : `Each Pro item counts once a month, on the site, in the CLI or the MCP server. Downloading one again is free. Resets on ${day(u.resets_at)}.`;
  }
  if (window.__oxynPlan !== undefined) usage(window.__oxynPlan);
  document.addEventListener('oxyn:plan', (e) => usage(e.detail));

  // ── Tokens ──
  async function call(method, path, body) {
    const init = { method, credentials: 'same-origin', headers: { accept: 'application/json' } };
    if (body) { init.headers['content-type'] = 'application/json'; init.body = JSON.stringify(body); }
    try {
      const r = await fetch(path, init);
      let d = {};
      try { d = await r.json(); } catch { /* empty */ }
      return { ok: r.ok, status: r.status, data: d };
    } catch { return { ok: false, status: 0, data: { error: { message: 'Could not reach Oxyn. Check your connection and try again.' } } }; }
  }
  async function load() {
    const r = await call('GET', '/api/cli/tokens');
    if (r.status === 401) return; // signed out: account.js shows the sign-in card
    if (!r.ok) { say(r.data.error?.message || 'Could not load your tokens.'); return; }
    list.textContent = '';
    for (const t of r.data.tokens || []) {
      const li = document.createElement('li');
      const meta = document.createElement('span'); meta.className = 'x-tok-meta';
      const name = document.createElement('b'); name.textContent = `${t.name} ····${t.hint}`;
      const when = document.createElement('small');
      when.textContent = `Created ${day(t.created_at)} · ${t.last_used_at ? `last used ${day(t.last_used_at)}` : 'never used'}`;
      meta.append(name, when);
      const rm = document.createElement('button'); rm.type = 'button'; rm.className = 'x-pill pill-grey pill-sm'; rm.textContent = 'Revoke';
      rm.setAttribute('aria-label', `Revoke ${t.name}`);
      rm.addEventListener('click', async () => {
        if (!confirm(`Revoke "${t.name}"? The CLI or MCP server using it stops getting Pro items.`)) return;
        rm.disabled = true;
        const d = await call('DELETE', '/api/cli/tokens/' + encodeURIComponent(t.id));
        if (d.ok) load(); else { say(d.data.error?.message || 'Could not revoke that token.'); rm.disabled = false; }
      });
      li.append(meta, rm);
      list.append(li);
    }
  }
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    say('');
    const b = form.querySelector('button'); b.disabled = true;
    const r = await call('POST', '/api/cli/tokens', { name: form.elements.name.value.trim() || 'CLI token' });
    b.disabled = false;
    if (!r.ok) { say(r.data.error?.message || 'Could not create a token.'); return; }
    form.reset();
    val.textContent = r.data.token;
    fresh.hidden = false;
    copy.textContent = 'Copy';
    load();
  });
  copy.addEventListener('click', async () => {
    try { await navigator.clipboard.writeText(val.textContent); copy.textContent = 'Copied'; }
    catch { const s = getSelection(); const rg = document.createRange(); rg.selectNodeContents(val); s.removeAllRanges(); s.addRange(rg); }
  });
  if (/(?:^|;\s*)oxyn_signed_in=1/.test(document.cookie)) load();
})();
