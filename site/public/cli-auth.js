// /cli/auth/?code=XXXX-XXXX (src/pages/cli/auth.astro): approve or decline an `oxyn login` request
// for the signed-in account (GET /api/cli/auth/request, POST /api/cli/auth/approve, pro.ts).
(function () {
  const $ = (s, r = document) => r.querySelector(s);
  const card = $('[data-cliauth]');
  if (!card) return;
  const msg = $('[data-cliauth-msg]', card), codeEl = $('[data-cliauth-code]', card), detail = $('[data-cliauth-detail]', card);
  const acts = $('[data-cliauth-actions]', card), signin = $('[data-cliauth-signin]', card);
  const code = (new URLSearchParams(location.search).get('code') || '').toUpperCase().trim();
  const done = (text, state) => { msg.textContent = text; card.dataset.state = state; card.removeAttribute('aria-busy'); acts.hidden = true; };
  const json = async (r) => { try { return await r.json(); } catch { return {}; } };

  async function boot() {
    if (!/^[A-Z]{4}-[A-Z]{4}$/.test(code)) { done('This link has no sign-in code. Run oxyn login in your terminal and open the link it prints.', 'error'); return; }
    codeEl.textContent = code;
    codeEl.hidden = false;
    let r;
    try { r = await fetch('/api/cli/auth/request?code=' + encodeURIComponent(code), { credentials: 'same-origin', headers: { accept: 'application/json' } }); }
    catch { done('Could not reach Oxyn. Check your connection and reload.', 'error'); return; }
    if (r.status === 401) {
      done('Sign in to Oxyn first, then approve this code.', 'signin');
      $('[data-cliauth-login]', card).href = '/login/?next=' + encodeURIComponent(location.pathname + location.search);
      signin.hidden = false;
      return;
    }
    const d = await json(r);
    if (!r.ok) { done(d.error?.message || 'This sign-in request is not valid.', 'error'); return; }
    if (d.approved || d.denied) { done(d.approved ? 'This code was already approved. You can close this tab.' : 'This code was declined.', 'done'); return; }
    card.dataset.state = 'ask';
    card.removeAttribute('aria-busy');
    msg.textContent = 'Does your terminal show this code?';
    detail.textContent = d.client ? `Requested by oxyn login on ${d.client}.` : 'Requested by oxyn login.';
    detail.hidden = false;
    acts.hidden = false;
  }

  async function answer(approve, btn) {
    btn.disabled = true;
    let r;
    try {
      r = await fetch('/api/cli/auth/approve', { method: 'POST', credentials: 'same-origin', headers: { 'content-type': 'application/json', accept: 'application/json' }, body: JSON.stringify({ code, approve }) });
    } catch { btn.disabled = false; msg.textContent = 'Could not reach Oxyn. Please try again.'; return; }
    const d = await json(r);
    if (!r.ok) { done(d.error?.message || 'That did not work. Run oxyn login again.', 'error'); return; }
    detail.hidden = true;
    done(approve ? 'Approved. Your terminal is signed in; you can close this tab.' : 'Declined. Nothing was signed in.', 'done');
  }
  $('[data-cliauth-ok]', card).addEventListener('click', (e) => answer(true, e.currentTarget));
  $('[data-cliauth-no]', card).addEventListener('click', (e) => answer(false, e.currentTarget));
  boot();
})();
