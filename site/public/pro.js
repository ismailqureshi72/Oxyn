// Oxyn Pro items (screens, agent-app templates; docs/pricing.md). Loaded only on Pro item pages.
// The page itself carries no Pro code. With an active plan (public/account.js resolves it), this
// fetches the code from /api/pro/source/<slug> (the Worker checks the plan again) and fills the
// Code tab, Usage and Examples. Without one, every way of taking the code leads to pricing with
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

  function toPricing() {
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

  async function load() {
    if (!isPro() || src) return;
    try {
      const r = await fetch('/api/pro/source/' + encodeURIComponent(slug), { credentials: 'same-origin', headers: { accept: 'application/json' } });
      if (!r.ok) return;
      src = await r.json();
      fill();
    } catch { /* stays locked */ }
  }

  planP.then((p) => { plan = p; load(); });
  document.addEventListener('oxyn:plan', (e) => { plan = e.detail; load(); });

  // Anything that would copy Pro code (or the Pro CLI command) without a plan goes to pricing.
  const gated = (el) => !!el && !!el.closest('.pvwrap[data-pro-slug] [data-copy-from], .pvwrap[data-pro-slug] .oxd-copy, [data-pro-cli], [data-pro-code] .pill-copy');
  const inLocked = (node) => {
    const el = node && (node.nodeType === 1 ? node : node.parentElement);
    return !!el && !!el.closest('[data-pro-lock], [data-pro-code], .pvwrap[data-pro-slug] .pv[data-view="code"], .install-pill');
  };
  async function decide(e, go) {
    if (isPro()) return; // let the normal copy run
    e.preventDefault();
    e.stopImmediatePropagation();
    if (plan === undefined && hint) {
      // Plan not known yet (the page just loaded): wait for it, then copy or go.
      plan = await planP;
      if (isPro()) { await load(); go?.(); return; }
    }
    toPricing();
  }
  window.addEventListener('click', (e) => {
    const b = e.target.closest?.('button, a');
    if (!gated(b)) return;
    decide(e, () => b.click());
  }, true);
  document.addEventListener('keydown', (e) => {
    if (!(e.ctrlKey || e.metaKey) || e.key.toLowerCase() !== 'c' || isPro()) return;
    const sel = document.getSelection();
    if (inLocked(document.activeElement) || (sel && !sel.isCollapsed && (inLocked(sel.anchorNode) || inLocked(sel.focusNode)))) decide(e);
  }, true);
  document.addEventListener('copy', (e) => {
    if (isPro()) return;
    const sel = document.getSelection();
    if (sel && (inLocked(sel.anchorNode) || inLocked(sel.focusNode))) { e.preventDefault(); toPricing(); }
  }, true);
})();
