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
      // Already signed in: the sign-in pages have nothing to do, so go on.
      if (authCard) { location.replace(nextPath()); return; }
    } else showSignedOut();
    setupCard(me);
    fillAccount(me);
  }
  boot();
})();
