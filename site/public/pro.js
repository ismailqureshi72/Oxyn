// Oxyn Pro items (screens, agent-app templates; docs/pricing.md). Loaded only on Pro item pages.
// The page itself carries no Pro code. With an active plan (public/account.js resolves it), this
// fetches the code from /api/pro/source/<slug> (the Worker checks the plan and this month's Pro
// items again) and fills the Code tab, Usage and Examples; an item not opened this month shows
// Open code first. Without one, every way of taking the code leads to pricing with
// ?from=<slug>: the Copy buttons, Ctrl/Cmd+C in a locked code area, and copying the Pro CLI command.
(function () {
  if (window.__oxynPro) return;
  window.__oxynPro = true;
  const wrap = document.querySelector('.pvwrap[data-pro-slug]');
  if (!wrap) return;
  const slug = wrap.dataset.proSlug;
  const name = document.currentScript?.dataset.proName || slug;
  const hint = /(?:^|;\s*)oxyn_signed_in=1/.test(document.cookie);
  const planP = window.__oxynPlanP || Promise.resolve(null);
  let plan = window.__oxynPlan;
  let src = null; // { code, variants: [{ name, code }] } once fetched

  const esc = (t) => t.replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' })[c]);
  // Same token classes as the build-time highlighter (src/lib/data.ts highlightRust).
  function highlight(code) {
    const re = /(\/\/[^\n]*)|("(?:\\.|[^"\\])*")|\b(let|mut|move|fn|use|pub|struct|impl|match|if|else|for|in|return|Some|None|true|false|vec|rsx)\b|\b(\d[\d_]*(?:\.\d+)?)\b|\b([A-Z][A-Za-z0-9]*)\b/g;
    let out = '', last = 0;
    for (const m of code.matchAll(re)) {
      out += esc(code.slice(last, m.index));
      const [t, com, str, kw, num, ty] = m;
      out += `<span class="tk-${com ? 'c' : str ? 's' : kw ? 'k' : num ? 'n' : ty ? 't' : ''}">${esc(t)}</span>`;
      last = m.index + t.length;
    }
    return out + esc(code.slice(last));
  }

  function toPricing(how) {
    // The conversion signal for the admin Activity tab (sent before the page changes).
    try { window.oxynTrack?.('pro_copy', slug, how || 'copy', true); } catch { /* never block */ }
    try { sessionStorage.setItem('oxyn_pro_from', JSON.stringify({ slug, name, url: location.pathname })); } catch { /* private mode */ }
    location.assign('/pricing/?from=' + encodeURIComponent(slug));
  }
  const isPro = () => !!(plan && plan.pro);

  function fill() {
    if (!src) return;
    const blocks = document.querySelectorAll('[data-pro-code]');
    blocks.forEach((b) => {
      const k = b.dataset.proCode;
      const code = k === 'usage' ? src.code : src.variants?.[Number(k)]?.code;
      if (code == null) return;
      const pre = b.querySelector('pre');
      pre.querySelector('code').innerHTML = highlight(code);
      pre.hidden = false;
      b.querySelector('[data-pro-lock]')?.remove();
      b.querySelector('.pill-copy')?.removeAttribute('hidden');
    });
    // The Code tab: the playground leaves the code out on Pro items, so show the usage code there.
    const lock = wrap.querySelector('.pv > [data-pro-lock]');
    if (lock) {
      const box = document.createElement('div');
      box.className = 'oxd-code pro-open';
      box.innerHTML = '<div class="oxd-ch"><span>Dioxus</span></div><pre><code class="language-rust"></code></pre>';
      box.querySelector('code').innerHTML = highlight(src.code);
      lock.replaceWith(box);
    }
    wrap.classList.add('pro-unlocked');
  }

  // Pro plans open a set number of Pro items a month (site/worker/pro.ts): the code shows once this
  // account (or its Team) opened the item this month. Opening it here, with Open code or a Copy
  // button, counts like `oxyn add` in the CLI; opening it again is free.
  let gate = null; // { code: 'unlock_required' | 'quota_exceeded', message, usage } while closed
  const resetDay = (u) => { const d = new Date(u?.resets_at); return isNaN(d) ? '' : d.toLocaleDateString(undefined, { month: 'long', day: 'numeric' }); };
  function paintGate() {
    document.querySelectorAll('[data-pro-lock]').forEach((lock) => {
      const t = lock.querySelector('[data-pro-lock-title]'), m = lock.querySelector('[data-pro-lock-msg]'), go = lock.querySelector('[data-pro-lock-go]');
      if (!gate) return;
      const u = gate.usage;
      if (gate.code === 'quota_exceeded') {
        if (t) t.textContent = 'This month\'s Pro items are used';
        if (m) m.textContent = gate.message;
        if (go) { go.hidden = true; }
        return;
      }
      if (t) t.textContent = 'Open the code';
      if (m) m.textContent = u && !u.unlimited
        ? `Opening it counts toward this month's Pro items: ${u.summary.replace(/, resets.*$/, '')}. Opening it again is free.${resetDay(u) ? ` Resets on ${resetDay(u)}.` : ''}`
        : 'Opening it counts toward this month\'s Pro items. Opening it again is free.';
      if (go) {
        go.hidden = false;
        go.dataset.proOpen = '1';
        go.setAttribute('href', '#');
        go.lastChild.textContent = 'Open code';
      }
    });
  }
  async function load(open) {
    if (!isPro() || src) return;
    try {
      const r = await fetch('/api/pro/source/' + encodeURIComponent(slug), open
        ? { method: 'POST', credentials: 'same-origin', headers: { accept: 'application/json' } }
        : { credentials: 'same-origin', headers: { accept: 'application/json' } });
      const d = await r.json().catch(() => ({}));
      if (!r.ok) {
        if (d.error && (d.error.code === 'unlock_required' || d.error.code === 'quota_exceeded')) { gate = { code: d.error.code, message: d.error.message, usage: d.usage }; paintGate(); }
        return;
      }
      gate = null;
      src = d;
      fill();
    } catch { /* stays locked */ }
  }
  // Open code (the lock's button for a Pro account).
  document.addEventListener('click', (e) => {
    const go = e.target.closest?.('[data-pro-lock-go][data-pro-open]');
    if (!go || !isPro()) return;
    e.preventDefault();
    go.setAttribute('aria-busy', 'true');
    load(true).finally(() => go.removeAttribute('aria-busy'));
  });

  planP.then((p) => { plan = p; load(); });
  document.addEventListener('oxyn:plan', (e) => { plan = e.detail; load(); });

  // Anything that would copy Pro code (or the Pro CLI command) without a plan goes to pricing.
  const gated = (el) => !!el && !!el.closest('.pvwrap[data-pro-slug] [data-copy-from], .pvwrap[data-pro-slug] .oxd-copy, [data-pro-cli], [data-pro-code] .pill-copy');
  const inLocked = (node) => {
    const el = node && (node.nodeType === 1 ? node : node.parentElement);
    return !!el && !!el.closest('[data-pro-lock], [data-pro-code], .pvwrap[data-pro-slug] .pv[data-view="code"], .install-pill');
  };
  async function decide(e, go, how) {
    if (isPro()) {
      // A Pro account copying code it has not opened this month: open it first (counts once).
      if (src || !gate || gate.code !== 'unlock_required') return;
      e.preventDefault();
      e.stopImmediatePropagation();
      await load(true);
      if (src) go?.();
      return;
    }
    e.preventDefault();
    e.stopImmediatePropagation();
    if (plan === undefined && hint) {
      // Plan not known yet (the page just loaded): wait for it, then copy or go.
      plan = await planP;
      if (isPro()) { await load(); go?.(); return; }
    }
    toPricing(how);
  }
  window.addEventListener('click', (e) => {
    const b = e.target.closest?.('button, a');
    if (!gated(b)) return;
    decide(e, () => b.click(), b.closest('[data-pro-cli]') ? 'cli' : 'copy');
  }, true);
  document.addEventListener('keydown', (e) => {
    if (!(e.ctrlKey || e.metaKey) || e.key.toLowerCase() !== 'c' || isPro()) return;
    const sel = document.getSelection();
    if (inLocked(document.activeElement) || (sel && !sel.isCollapsed && (inLocked(sel.anchorNode) || inLocked(sel.focusNode)))) decide(e, null, 'keys');
  }, true);
  document.addEventListener('copy', (e) => {
    if (isPro()) return;
    const sel = document.getSelection();
    if (sel && (inLocked(sel.anchorNode) || inLocked(sel.focusNode))) { e.preventDefault(); toPricing('menu'); }
  }, true);
})();
