// Oxyn admin panel UI. Served by the Worker only to a signed-in, provider-verified admin (admin.ts).
// No framework, no innerHTML for data: every value from the API goes in as text.
(() => {
  'use strict';
  const root = document.documentElement;
  const saved = localStorage.getItem('starlight-theme');
  const prefersDark = window.matchMedia && matchMedia('(prefers-color-scheme: dark)').matches;
  root.dataset.theme = saved === 'dark' || saved === 'light' ? saved : prefersDark ? 'dark' : 'light';

  const $ = (s, el = document) => el.querySelector(s);
  /** h('div', {class: 'x', onclick: fn}, 'text', child) */
  function h(tag, attrs, ...kids) {
    const el = document.createElement(tag);
    for (const [k, v] of Object.entries(attrs || {})) {
      if (v == null || v === false) continue;
      if (k.startsWith('on')) el.addEventListener(k.slice(2), v);
      else if (k === 'class') el.className = v;
      else if (k === 'text') el.textContent = v;
      else el.setAttribute(k, v === true ? '' : String(v));
    }
    for (const c of kids.flat()) if (c != null && c !== false) el.append(c instanceof Node ? c : document.createTextNode(String(c)));
    return el;
  }

  /** el.replaceChildren without null/false kids (replaceChildren would print "null"). */
  const fill = (el, ...kids) => el.replaceChildren(...kids.flat().filter((k) => k != null && k !== false));

  class ApiError extends Error {
    constructor(status, code, message, field) { super(message); this.status = status; this.code = code; this.field = field; }
  }
  async function api(path, opts = {}) {
    const init = { method: opts.method || 'GET', credentials: 'same-origin', headers: {} };
    if (opts.body !== undefined) { init.headers['content-type'] = 'application/json'; init.body = JSON.stringify(opts.body); }
    let r;
    try { r = await fetch('/api/admin' + path, init); }
    catch { throw new ApiError(0, 'network', 'Could not reach the server. Check your connection and reload the page.'); }
    let data = null;
    try { data = await r.json(); } catch { /* not JSON */ }
    if (!r.ok || !data || data.ok === false) {
      const e = (data && data.error) || {};
      // Signed out or no longer an admin: the page itself shows why (sign-in or refusal).
      if (['signed_out', 'not_an_admin', 'verify_email'].includes(e.code)) { location.reload(); }
      throw new ApiError(r.status, e.code || 'http_' + r.status, e.message || `Request failed (${r.status}).`, e.field);
    }
    return data;
  }

  const main = () => $('#main');
  let toastT = 0;
  function toast(msg) {
    const t = $('#toast');
    t.textContent = msg; t.classList.add('on');
    clearTimeout(toastT); toastT = setTimeout(() => t.classList.remove('on'), 3200);
  }
  function confirmDlg(title, msg, okLabel) {
    const dlg = $('#confirm');
    $('#confirm-t').textContent = title; $('#confirm-m').textContent = msg; $('#confirm-ok').textContent = okLabel || 'Delete';
    return new Promise((resolve) => {
      const done = (v) => { dlg.close(); cleanup(); resolve(v); };
      const ok = () => done(true), no = () => done(false), cancel = (e) => { e.preventDefault(); done(false); };
      const cleanup = () => { $('#confirm-ok').removeEventListener('click', ok); $('#confirm-no').removeEventListener('click', no); dlg.removeEventListener('cancel', cancel); };
      $('#confirm-ok').addEventListener('click', ok); $('#confirm-no').addEventListener('click', no); dlg.addEventListener('cancel', cancel);
      dlg.showModal(); $('#confirm-no').focus();
    });
  }
  const fmtDate = (s) => {
    if (!s) return '';
    const d = new Date(s);
    return isNaN(d) ? s : d.toLocaleString(undefined, { year: 'numeric', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
  };
  const ago = (s) => {
    const m = Math.round((Date.now() - new Date(s).getTime()) / 60000);
    if (!isFinite(m)) return '';
    if (m < 1) return 'just now';
    if (m < 60) return m + ' min ago';
    if (m < 1440) return Math.round(m / 60) + ' h ago';
    if (m < 10080) return Math.round(m / 1440) + ' d ago';
    return new Date(s).toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  };
  const cap = (s) => (s ? s[0].toUpperCase() + s.slice(1) : s);
  function setBusy(btn, busy, label) {
    if (!btn) return;
    if (busy) { btn.dataset.label = btn.textContent; btn.textContent = label || 'Working…'; btn.disabled = true; btn.setAttribute('aria-busy', 'true'); }
    else { btn.textContent = btn.dataset.label || btn.textContent; btn.disabled = false; btn.removeAttribute('aria-busy'); }
  }
  /** Shows an API error on the form field it names, else in the form's error line. */
  function showErr(form, e) {
    form.querySelectorAll('[aria-invalid]').forEach((x) => x.removeAttribute('aria-invalid'));
    const line = form.querySelector('.err.form');
    form.querySelectorAll('.err').forEach((x) => (x.textContent = ''));
    if (!e) return;
    const input = e.field && form.querySelector(`[name="${e.field}"]`);
    if (input) {
      input.setAttribute('aria-invalid', 'true');
      const er = form.querySelector(`#${input.id}-err`);
      if (er) er.textContent = e.message; else if (line) line.textContent = e.message;
      input.focus();
    } else if (line) line.textContent = e.message;
  }
  function field(id, label, attrs, hint) {
    return h('div', { class: 'field' },
      h('label', { for: id }, label),
      h('input', { class: 'input', id, name: attrs.name || id, 'aria-describedby': `${id}-err${hint ? ` ${id}-hint` : ''}`, ...attrs }),
      hint ? h('span', { class: 'hint', id: `${id}-hint` }, hint) : null,
      h('span', { class: 'err', id: `${id}-err` }));
  }

  // ── Theme + chrome ─────────────────────────────────────────────────────────────────
  function syncThemeBtn() {
    const b = $('#theme'); const dark = root.dataset.theme === 'dark';
    b.setAttribute('aria-label', dark ? 'Switch to light theme' : 'Switch to dark theme');
  }
  let me = null;
  const state = { tab: 'activity', kind: '', status: 'new', q: '', sel: null, items: [], next: null, cameFrom: null };

  async function start() {
    $('#tabs').hidden = true; $('#logout').hidden = true;
    try { me = await api('/me'); }
    catch (e) { main().replaceChildren(h('div', { class: 'auth' }, h('div', { class: 'card' }, h('h1', {}, 'Admin unavailable'), h('p', {}, e.message)))); return; }
    me.signed_in = true;
    $('#who').textContent = me.email;
    $('#tabs').hidden = false; $('#logout').hidden = false;
    route();
  }

  // ── Tabs + routing ─────────────────────────────────────────────────────────────────
  const TABS = ['activity', 'users', 'grants', 'inbox', 'billing', 'admins', 'settings', 'audit'];
  function route() {
    const [tab, id] = location.hash.replace(/^#/, '').split('/');
    const prevTab = state.tab;
    state.tab = TABS.includes(tab) ? tab : 'activity';
    const prev = { tab: state.tab, sel: state.sel };
    if (TABS.includes(tab) ? tab === 'inbox' : true) state.sel = id || null;
    document.querySelectorAll('#tabs [role=tab]').forEach((b) => {
      const on = b.dataset.tab === state.tab;
      b.setAttribute('aria-selected', String(on)); b.tabIndex = on ? 0 : -1;
    });
    main().setAttribute('aria-labelledby', 'tab-' + state.tab);
    // Inbox already on screen: only the selection changed, keep the list (and focus) as it is.
    const wrap = $('.inbox');
    if (state.tab === 'inbox' && prevTab === 'inbox' && wrap && prev.sel !== state.sel) {
      wrap.classList.toggle('open', !!state.sel);
      if (state.sel) loadDetail(state.sel);
      else {
        $('#detail').replaceChildren(h('p', { class: 'empty' }, 'Select a message to read it.'));
        document.querySelectorAll('#list .item').forEach((b) => b.setAttribute('aria-current', 'false'));
        const back = prev.sel && document.querySelector(`#list .item[data-id="${CSS.escape(prev.sel)}"]`);
        if (back) back.focus();
      }
      return;
    }
    if (state.tab !== 'users' || !id) window.scrollTo(0, 0);
    ({ activity: renderActivity, users: () => renderUsers(id), grants: renderGrants, inbox: renderInbox, billing: renderBilling, admins: renderAdmins, settings: renderSettings, audit: renderAudit })[state.tab]();
  }
  function go(hash) { if (location.hash !== '#' + hash) location.hash = hash; else route(); }

  // ── Inbox ──────────────────────────────────────────────────────────────────────────
  function seg(label, opts, cur, onPick) {
    const g = h('div', { class: 'seg', role: 'radiogroup', 'aria-label': label });
    const btns = opts.map(([v, l]) => h('button', { type: 'button', role: 'radio', 'aria-checked': String(v === cur), tabindex: v === cur ? 0 : -1, onclick: () => onPick(v) }, l));
    g.addEventListener('keydown', (e) => {
      const i = btns.indexOf(document.activeElement);
      if (i < 0) return;
      const n = e.key === 'ArrowRight' || e.key === 'ArrowDown' ? 1 : e.key === 'ArrowLeft' || e.key === 'ArrowUp' ? -1 : 0;
      if (!n) return;
      e.preventDefault();
      const j = (i + n + btns.length) % btns.length; btns[j].focus(); btns[j].click();
    });
    g.append(...btns);
    return g;
  }
  async function renderInbox() {
    document.title = 'Inbox · Oxyn admin';
    const search = h('input', { class: 'input search', type: 'search', placeholder: 'Search messages and emails', 'aria-label': 'Search feedback', value: state.q });
    let st = 0;
    search.addEventListener('input', () => { clearTimeout(st); st = setTimeout(() => { state.q = search.value; loadList(); }, 250); });
    const tools = h('div', { class: 'tools' },
      seg('Kind', [['', 'All kinds'], ['Bug', 'Bug'], ['Idea', 'Idea'], ['Question', 'Question']], state.kind, (v) => { state.kind = v; renderInbox(); }),
      seg('Status', [['new', 'New'], ['triaged', 'Triaged'], ['closed', 'Closed'], ['', 'All']], state.status, (v) => { state.status = v; renderInbox(); }),
      search);
    const list = h('div', { class: 'card list', id: 'list' }, h('p', { class: 'empty' }, 'Loading…'));
    const detail = h('section', { class: 'card detail', id: 'detail', 'aria-label': 'Feedback detail' }, h('p', { class: 'empty' }, 'Select a message to read it.'));
    const wrap = h('div', { class: 'inbox' + (state.sel ? ' open' : '') }, list, detail);
    main().replaceChildren(h('div', { class: 'page' }, h('div', { class: 'head' }, h('h1', {}, 'Feedback inbox')), tools, wrap));
    await loadList();
    if (state.sel) loadDetail(state.sel);
  }
  async function loadList(append) {
    const list = $('#list'); if (!list) return;
    const p = new URLSearchParams();
    if (state.kind) p.set('kind', state.kind);
    if (state.status) p.set('status', state.status);
    if (state.q.trim()) p.set('q', state.q.trim());
    if (append && state.next) p.set('before', state.next);
    let r;
    try { r = await api('/feedback?' + p); }
    catch (e) { list.replaceChildren(h('p', { class: 'empty' }, e.message)); return; }
    state.items = append ? state.items.concat(r.items) : r.items; state.next = r.next;
    $('#newcount').textContent = r.counts && r.counts.new ? String(r.counts.new) : '';
    $('#newcount').setAttribute('aria-label', r.counts && r.counts.new ? `${r.counts.new} new` : '');
    if (!state.items.length) { list.replaceChildren(h('p', { class: 'empty' }, state.q || state.kind || state.status ? 'Nothing matches these filters.' : 'No feedback yet.')); return; }
    const ul = h('ul', { 'aria-label': 'Feedback messages' }, state.items.map((it) =>
      h('li', {}, h('button', { type: 'button', class: 'item' + (it.status === 'new' ? ' unread' : ''), 'aria-current': String(it.id === state.sel), 'data-id': it.id, onclick: () => go('inbox/' + it.id) },
        h('span', { class: 'top' }, h('span', { class: 'chip' + (it.kind === 'Bug' ? ' solid' : '') }, it.kind), it.status !== 'new' ? h('span', { class: 'chip outline' }, cap(it.status)) : null,
          it.has_screenshot ? h('span', { class: 'chip', title: 'Has a screenshot' }, 'Screenshot') : null,
          it.github_issue_url ? h('span', { class: 'chip', title: 'Sent to GitHub' }, 'GitHub') : null,
          h('span', { class: 'when' }, h('time', { datetime: it.created_at, title: fmtDate(it.created_at) }, ago(it.created_at)))),
        h('span', { class: 'msg' }, it.message),
        h('span', { class: 'meta' }, h('span', {}, it.page_url || 'unknown page'), it.email ? h('span', {}, it.email) : null)))));
    // Up/down arrows move through the list.
    ul.addEventListener('keydown', (e) => {
      if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return;
      const all = [...ul.querySelectorAll('.item')]; const i = all.indexOf(document.activeElement);
      if (i < 0) return;
      e.preventDefault(); const n = all[Math.max(0, Math.min(all.length - 1, i + (e.key === 'ArrowDown' ? 1 : -1)))]; n.focus();
    });
    list.replaceChildren(ul);
    if (state.next) list.append(h('div', { class: 'more' }, h('button', { class: 'btn sm', type: 'button', onclick: () => loadList(true) }, 'Load more')));
  }
  async function loadDetail(id) {
    const box = $('#detail'); if (!box) return;
    box.replaceChildren(h('p', { class: 'empty' }, 'Loading…'));
    let it;
    try { it = (await api('/feedback/' + encodeURIComponent(id))).item; }
    catch (e) { box.replaceChildren(h('button', { class: 'btn ghost sm back', type: 'button', onclick: () => go('inbox') }, '← Back to inbox'), h('p', { class: 'empty' }, e.message)); return; }
    const page = it.page_url ? h('a', { href: it.page_url, target: '_blank', rel: 'noopener noreferrer' }, it.page_url) : 'unknown';
    const status = seg('Status', [['new', 'New'], ['triaged', 'Triaged'], ['closed', 'Closed']], it.status, async (v) => {
      if (v === it.status) return;
      try { await api('/feedback/' + it.id, { method: 'PATCH', body: { status: v } }); toast(`Marked ${v}.`); it.status = v; loadList(); loadDetail(it.id); }
      catch (e) { toast(e.message); }
    });
    const gh = it.github_issue_url
      ? h('a', { class: 'btn sm', href: it.github_issue_url, target: '_blank', rel: 'noopener noreferrer' }, 'Open GitHub issue')
      : h('button', { class: 'btn sm', type: 'button', disabled: !me.github, 'aria-describedby': me.github ? null : 'gh-note', onclick: async (e) => {
          const b = e.currentTarget; setBusy(b, true, 'Sending…');
          try { await api('/feedback/' + it.id + '/github', { method: 'POST', body: {} }); toast('Issue opened on GitHub.'); loadList(); loadDetail(it.id); }
          catch (er) { setBusy(b, false); toast(er.message); }
        } }, 'Send to GitHub');
    const del = h('button', { class: 'btn sm danger', type: 'button', onclick: async () => {
      if (!(await confirmDlg('Delete this feedback?', 'The message and its screenshot are deleted for good.', 'Delete'))) return;
      try { await api('/feedback/' + it.id, { method: 'DELETE' }); toast('Deleted.'); state.sel = null; go('inbox'); }
      catch (e) { toast(e.message); }
    } }, 'Delete');
    fill(box,
      h('button', { class: 'btn ghost sm back', type: 'button', onclick: () => go('inbox') }, '← Back to inbox'),
      h('div', { class: 'dtop' }, h('span', { class: 'chip' + (it.kind === 'Bug' ? ' solid' : '') }, it.kind), h('span', { class: 'chip outline' }, cap(it.status)),
        h('span', { class: 'muted' }, h('time', { datetime: it.created_at }, fmtDate(it.created_at)))),
      h('h2', { class: 'sr', id: 'detail-t' }, `${it.kind} feedback`),
      h('p', { class: 'msgfull' }, it.message),
      h('dl', { class: 'kv' },
        h('dt', {}, 'Page'), h('dd', {}, page),
        h('dt', {}, 'Reply to'), h('dd', {}, it.email ? h('a', { href: 'mailto:' + it.email }, it.email) : 'no email given'),
        h('dt', {}, 'Browser'), h('dd', { class: 'mono' }, it.ua || 'unknown'),
        h('dt', {}, 'GitHub'), h('dd', {}, it.github_issue_url ? h('a', { href: it.github_issue_url, target: '_blank', rel: 'noopener noreferrer' }, it.github_issue_url) : 'not sent'),
        h('dt', {}, 'ID'), h('dd', { class: 'mono' }, it.id)),
      it.screenshot_key ? h('a', { href: `/api/admin/feedback/${it.id}/screenshot`, target: '_blank', rel: 'noopener', 'aria-label': 'Open the screenshot full size' },
        h('img', { class: 'shot', src: `/api/admin/feedback/${it.id}/screenshot`, alt: 'Screenshot attached to this feedback', loading: 'lazy' })) : null,
      h('div', { class: 'actions' },
        h('div', { class: 'row' }, h('span', { class: 'lbl', id: 'st-l' }, 'Status'), status),
        h('div', { class: 'row' }, gh, del),
        me.github || it.github_issue_url ? null : h('p', { class: 'hint', id: 'gh-note' }, 'Send to GitHub is off: the GITHUB_TOKEN Worker secret is not set (docs/admin.md).')));
    document.querySelectorAll('#list .item').forEach((b) => b.setAttribute('aria-current', String(b.dataset.id === it.id)));
    if (matchMedia('(max-width:1023px)').matches) box.querySelector('.back').focus();
  }

  // ── Admins ─────────────────────────────────────────────────────────────────────────
  async function renderAdmins() {
    document.title = 'Admins · Oxyn admin';
    const pageEl = h('div', { class: 'page' }, h('div', { class: 'head' }, h('h1', {}, 'Admins')), h('p', { class: 'empty' }, 'Loading…'));
    main().replaceChildren(pageEl);
    let r;
    try { r = await api('/admins'); } catch (e) { pageEl.lastChild.textContent = e.message; return; }
    const active = r.items.filter((a) => !a.removed_at);
    const form = h('form', { class: 'addrow', novalidate: true },
      field('add-email', 'Add an admin', { type: 'email', name: 'email', autocomplete: 'off', placeholder: 'name@example.com', required: true }),
      h('button', { class: 'btn primary', type: 'submit' }, 'Add admin'),
      h('span', { class: 'err form', role: 'alert' }));
    form.addEventListener('submit', async (ev) => {
      ev.preventDefault();
      const btn = form.querySelector('button[type=submit]'); const v = form.email.value.trim();
      if (!/^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/.test(v)) return showErr(form, { field: 'email', message: 'That email address does not look right.' });
      setBusy(btn, true, 'Adding…');
      try { await api('/admins', { method: 'POST', body: { email: v } }); toast(`${v} added.`); renderAdmins(); }
      catch (e) { setBusy(btn, false); showErr(form, e); }
    });
    const lis = r.items.map((a) => {
      const self = a.email.toLowerCase() === r.me.toLowerCase();
      const state = a.verified ? h('span', { class: 'chip' }, 'Verified') : h('span', { class: 'chip outline', title: 'Signs in with Google or GitHub on this email to open the panel' }, 'Not verified yet');
      const last = active.length <= 1;
      const rm = h('button', { class: 'btn sm danger', type: 'button', disabled: last, title: last ? 'Keep at least one admin' : null, 'aria-label': `Remove ${a.email}`, onclick: async () => {
        let q = '';
        if (self) {
          // Removing yourself locks you out at once: type your own email (the server asks for it too).
          const c = await typedConfirm({ title: 'Remove yourself as an admin?', lines: ['You lose access to the admin panel right away. Another admin has to add you back.'], expect: a.email, okLabel: 'Remove me', danger: true });
          if (!c.ok) return;
          q = '?confirm=' + encodeURIComponent(c.value);
        } else if (!(await confirmDlg(`Remove ${a.email}?`, `${a.email} will no longer be able to use the admin panel.`, 'Remove'))) return;
        try { const x = await api('/admins/' + encodeURIComponent(a.email) + q, { method: 'DELETE' }); toast('Removed.'); if (x.self) location.reload(); else renderAdmins(); }
        catch (e) { toast(e.message); }
      } }, 'Remove');
      return h('li', {}, h('span', { class: 'em' }, a.email), self ? h('span', { class: 'chip solid' }, 'You') : null, state, h('span', { class: 'r' }, rm),
        h('span', { class: 'sub' }, `Added ${fmtDate(a.created_at)}${a.added_by ? ' by ' + a.added_by : ''}`));
    });
    fill(pageEl,
      h('div', { class: 'head' }, h('h1', {}, 'Admins'), h('span', { class: 'muted' }, `${active.length} active`)),
      h('div', { class: 'stack' },
        h('div', { class: 'card' }, h('ul', { class: 'alist', 'aria-label': 'Admins' }, lis)),
        h('div', { class: 'card section' }, form,
          h('p', { class: 'hint muted addhint' }, 'They open the panel by signing in to Oxyn with Google or GitHub on this email. An email and password account on it is not enough until Google or GitHub has verified the address.'))));
  }

  // ── Billing (Polar subscriptions, written by the signed webhook; docs/pricing.md) ──────────────
  const usd = (c) => '$' + (c / 100).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  async function renderBilling() {
    document.title = 'Billing · Oxyn admin';
    const box = h('div', { class: 'stack' }, h('div', { class: 'card' }, h('p', { class: 'empty' }, 'Loading…')));
    main().replaceChildren(h('div', { class: 'page' }, h('div', { class: 'head' }, h('h1', {}, 'Billing'), h('span', { class: 'muted' }, 'Subscriptions from Polar')), box));
    let r;
    try { r = await api('/billing'); } catch (e) { box.replaceChildren(h('div', { class: 'card' }, h('p', { class: 'empty' }, e.message))); return; }
    const c = r.counts, f = r.founder;
    const stat = (k, v, sub) => h('div', { class: 'stat card' }, h('span', { class: 'k' }, k), h('b', {}, v), sub ? h('small', {}, sub) : null);
    const notes = [];
    if (!r.webhook_ready) notes.push('The POLAR_WEBHOOK_SECRET Worker secret is not set: Polar webhooks are refused and nothing is recorded.');
    if (!r.api_token) notes.push('No POLAR_ACCESS_TOKEN Worker secret: Manage billing opens Polar’s portal sign-in instead of a pre-signed session, and the founder products cannot be archived automatically.');
    if (f.archive && f.archive !== 'done') notes.push(`Founder products in Polar: ${f.archive}.`);
    const rows = r.items.map((s) => h('tr', {},
      h('td', { class: 'u' }, s.customer_email, s.linked ? null : h('span', { class: 'chip outline', title: 'Not linked to a site account yet' }, 'no account')),
      h('td', {}, cap(s.plan) + (s.founder ? ' · founder' : ''), s.seats ? ` · ${s.seats} seats` : ''),
      h('td', {}, s.interval === 'year' ? 'Yearly' : 'Monthly'),
      h('td', {}, usd(s.amount)),
      h('td', {}, h('span', { class: 'chip' + (s.active ? ' solid' : '') }, s.cancel_at_period_end && s.active ? 'canceling' : s.status)),
      h('td', { class: 't' }, s.current_period_end ? fmtDate(s.current_period_end) : '—'),
      h('td', { class: 't' }, h('time', { datetime: s.created_at, title: fmtDate(s.created_at) }, ago(s.created_at)))));
    box.replaceChildren(
      h('div', { class: 'stats' },
        stat('MRR', usd(r.mrr_cents), `${usd(r.arr_cents)} a year`),
        stat('Active', String(c.active), `${c.pro} Pro · ${c.team} Team`),
        stat('Founder spots', `${f.claimed} / ${f.cap}`, f.open ? `${f.remaining} left` : 'closed'),
        stat('Canceling', String(c.canceling), `${c.ended} ended`)),
      notes.length ? h('div', { class: 'note' }, h('div', {}, ...notes.map((n) => h('p', { class: 'nline' }, n)))) : null,
      h('div', { class: 'card' }, r.items.length ? h('div', { class: 'twrap' }, h('table', { class: 'audit subs' },
        h('caption', { class: 'sr' }, 'Subscribers, newest first'),
        h('thead', {}, h('tr', {}, ['Email', 'Plan', 'Billing', 'Amount', 'Status', 'Renews / ends', 'Since'].map((t) => h('th', { scope: 'col' }, t)))),
        h('tbody', {}, rows))) : h('p', { class: 'empty' }, 'No subscribers yet.')));
  }

  // ── Settings ───────────────────────────────────────────────────────────────────────
  async function renderSettings() {
    document.title = 'Settings · Oxyn admin';
    const flagsBox = h('div', { class: 'flags', id: 'flags' }, h('p', { class: 'empty' }, 'Loading…'));
    const add = h('form', { class: 'addrow', novalidate: true },
      field('flag-key', 'New flag', { name: 'key', autocomplete: 'off', placeholder: 'promo_strip', required: true }),
      field('flag-value', 'Value', { name: 'value', autocomplete: 'off', placeholder: 'Text (optional)' }),
      h('button', { class: 'btn', type: 'submit' }, 'Add flag'),
      h('span', { class: 'err form', role: 'alert' }));
    add.addEventListener('submit', async (ev) => {
      ev.preventDefault();
      const k = add.key.value.trim();
      if (!/^[a-z0-9][a-z0-9_.-]{0,47}$/.test(k)) return showErr(add, { field: 'key', message: 'Use a-z, 0-9, _ . - (up to 48).' });
      try { await api('/flags/' + encodeURIComponent(k), { method: 'PUT', body: { enabled: false, value: add.value.value } }); add.reset(); showErr(add, null); toast(`Flag ${k} added (off).`); loadFlags(); }
      catch (e) { showErr(add, e); }
    });
    main().replaceChildren(h('div', { class: 'page' }, h('div', { class: 'head' }, h('h1', {}, 'Settings')),
      h('div', { class: 'stack' },
        h('section', { class: 'card', 'aria-labelledby': 'fl-t' },
          h('div', { class: 'section fhead' }, h('h2', { id: 'fl-t' }, 'Content and flags'),
            h('p', {}, 'Simple switches and texts the site can read from /api/flags (only flags that are on are public).')),
          flagsBox,
          h('div', { class: 'section fadd' }, add)))));
    loadFlags();
  }
  async function loadFlags() {
    const box = $('#flags'); if (!box) return;
    let r;
    try { r = await api('/flags'); } catch (e) { box.replaceChildren(h('p', { class: 'empty' }, e.message)); return; }
    if (!r.flags.length) { box.replaceChildren(h('p', { class: 'empty' }, 'No flags yet.')); return; }
    box.replaceChildren(...r.flags.map((f) => {
      const id = 'fv-' + f.key.replace(/[^a-z0-9]/g, '-');
      const sw = h('button', { class: 'sw', type: 'button', role: 'switch', 'aria-checked': String(f.enabled), 'aria-label': `${f.key} on` });
      const val = h('input', { class: 'input v', id, value: f.value, 'aria-label': `${f.key} value`, maxlength: 500, placeholder: 'No text' });
      const save = async (enabled) => {
        try { await api('/flags/' + encodeURIComponent(f.key), { method: 'PUT', body: { enabled, value: val.value } }); f.enabled = enabled; sw.setAttribute('aria-checked', String(enabled)); toast(`${f.key} saved (${enabled ? 'on' : 'off'}).`); }
        catch (e) { toast(e.message); }
      };
      sw.addEventListener('click', () => save(!f.enabled));
      val.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); save(f.enabled); } });
      return h('div', { class: 'flag' }, sw, h('span', { class: 'k' }, f.key), val,
        h('span', { class: 'row' }, h('button', { class: 'btn sm', type: 'button', onclick: () => save(f.enabled) }, 'Save'),
          h('button', { class: 'btn sm ghost', type: 'button', 'aria-label': `Delete flag ${f.key}`, onclick: async () => {
            if (!(await confirmDlg(`Delete flag ${f.key}?`, 'The site stops seeing it at once.', 'Delete'))) return;
            try { await api('/flags/' + encodeURIComponent(f.key), { method: 'DELETE' }); toast('Flag deleted.'); loadFlags(); } catch (e) { toast(e.message); }
          } }, 'Delete')));
    }));
  }

  // ── Audit log ──────────────────────────────────────────────────────────────────────
  const ACTIONS = {
    'password.setup': 'Set the shared password', 'password.change': 'Changed the password', 'password.change_fail': 'Wrong current password',
    login: 'Signed in (old admin password)', 'login.fail': 'Wrong password (old admin password)', logout: 'Logged out',
    'feedback.status': 'Changed status', 'feedback.delete': 'Deleted feedback', 'feedback.github': 'Sent to GitHub',
    'admin.add': 'Added admin', 'admin.remove': 'Removed admin', 'access.sync': 'Synced Access group',
    'flag.set': 'Saved flag', 'flag.delete': 'Deleted flag',
  };
  async function renderAudit() {
    document.title = 'Audit log · Oxyn admin';
    const box = h('div', { class: 'card' }, h('p', { class: 'empty' }, 'Loading…'));
    main().replaceChildren(h('div', { class: 'page' }, h('div', { class: 'head' }, h('h1', {}, 'Audit log'), h('span', { class: 'muted' }, 'Latest 100 actions')), box));
    let r;
    try { r = await api('/audit'); } catch (e) { box.replaceChildren(h('p', { class: 'empty' }, e.message)); return; }
    if (!r.items.length) { box.replaceChildren(h('p', { class: 'empty' }, 'Nothing yet.')); return; }
    box.replaceChildren(h('table', { class: 'audit' },
      h('caption', { class: 'sr' }, 'Admin actions, newest first'),
      h('thead', {}, h('tr', {}, h('th', { scope: 'col' }, 'Action'), h('th', { scope: 'col' }, 'Who'), h('th', { scope: 'col' }, 'Target'), h('th', { scope: 'col' }, 'When'))),
      h('tbody', {}, r.items.map((a) => h('tr', {},
        h('td', { class: 'a' }, ACTIONS[a.action] || a.action),
        h('td', { class: 'u' }, a.actor),
        h('td', { class: 'g' + (a.target || a.detail ? '' : ' none') }, [a.target, a.detail].filter(Boolean).join(' · ') || '—'),
        h('td', { class: 't' }, h('time', { datetime: a.at, title: fmtDate(a.at) }, ago(a.at))))))));
  }

  // ── Email checks (mirror of worker/emailcheck.ts; the server checks again and decides) ──────────
  const COMMON_DOMAINS = ['gmail.com', 'googlemail.com', 'yahoo.com', 'hotmail.com', 'outlook.com', 'live.com', 'icloud.com', 'me.com',
    'proton.me', 'protonmail.com', 'aol.com', 'msn.com', 'gmx.com', 'mail.com', 'zoho.com', 'yandex.com', 'hey.com'];
  const DOMAIN_TYPOS = {
    'gmial.com': 'gmail.com', 'gamil.com': 'gmail.com', 'gmai.com': 'gmail.com', 'gmal.com': 'gmail.com', 'gmaill.com': 'gmail.com',
    'gnail.com': 'gmail.com', 'gmali.com': 'gmail.com', 'gmail.co': 'gmail.com', 'gmail.cm': 'gmail.com', 'gmail.con': 'gmail.com',
    'gmail.om': 'gmail.com', 'gmail.comm': 'gmail.com', 'gmail.cmo': 'gmail.com', 'gmail.ocm': 'gmail.com', 'gmail.vom': 'gmail.com',
    'gmail.xom': 'gmail.com', 'gmail.cpm': 'gmail.com', 'gmaik.com': 'gmail.com', 'gmeil.com': 'gmail.com', 'gimail.com': 'gmail.com',
    'hotmial.com': 'hotmail.com', 'hotmal.com': 'hotmail.com', 'hotmai.com': 'hotmail.com', 'hotnail.com': 'hotmail.com',
    'hotmail.co': 'hotmail.com', 'hotmail.con': 'hotmail.com', 'hotmali.com': 'hotmail.com', 'homail.com': 'hotmail.com',
    'yahooo.com': 'yahoo.com', 'yaho.com': 'yahoo.com', 'yhaoo.com': 'yahoo.com', 'yahoo.co': 'yahoo.com', 'yahoo.con': 'yahoo.com',
    'yahho.com': 'yahoo.com', 'tahoo.com': 'yahoo.com',
    'outlok.com': 'outlook.com', 'outllok.com': 'outlook.com', 'outloo.com': 'outlook.com', 'outlook.co': 'outlook.com',
    'outlook.con': 'outlook.com', 'outlookk.com': 'outlook.com', 'otlook.com': 'outlook.com',
    'iclod.com': 'icloud.com', 'icoud.com': 'icloud.com', 'icloud.co': 'icloud.com', 'icloud.con': 'icloud.com', 'iclud.com': 'icloud.com',
    'protonmail.co': 'protonmail.com', 'protonmal.com': 'protonmail.com', 'proton.m': 'proton.me',
  };
  const BAD_TLDS = { con: 'com', cmo: 'com', ocm: 'com', vom: 'com', xom: 'com', comm: 'com', cm: 'com', om: 'com' };
  function lev(a, b) {
    const d = Array.from({ length: a.length + 1 }, (_, i) => Array.from({ length: b.length + 1 }, (_, j) => (i === 0 ? j : j === 0 ? i : 0)));
    for (let i = 1; i <= a.length; i++) {
      for (let j = 1; j <= b.length; j++) {
        d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
        if (i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1]) d[i][j] = Math.min(d[i][j], d[i - 2][j - 2] + 1);
      }
    }
    return d[a.length][b.length];
  }
  function suggestDomain(domain) {
    if (COMMON_DOMAINS.includes(domain)) return null;
    if (DOMAIN_TYPOS[domain]) return DOMAIN_TYPOS[domain];
    const dot = domain.lastIndexOf('.');
    const tld = domain.slice(dot + 1);
    if (BAD_TLDS[tld]) return domain.slice(0, dot + 1) + BAD_TLDS[tld];
    let best = null, bestD = 3;
    for (const c of COMMON_DOMAINS) { const x = lev(domain, c); if (x < bestD) { bestD = x; best = c; } }
    return best && (bestD === 1 || (bestD === 2 && domain.length >= 9)) ? best : null;
  }
  const LOCAL = /^[a-z0-9!#$%&'*+/=?^_`{|}~.-]{1,64}$/;
  const LABEL = /^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$/;
  function checkEmail(raw) {
    const input = String(raw == null ? '' : raw);
    let email = input.trim();
    const notes = [];
    const angle = email.match(/<([^<>]+)>\s*$/);
    if (angle) email = angle[1].trim();
    email = email.replace(/^mailto:/i, '');
    const lowered = email.toLowerCase();
    if (lowered !== input.trim() && lowered) notes.push(`Read as ${lowered}.`);
    email = lowered;
    const out = (ok, code, message, suggestion) => ({ input, email, ok, code, message, ...(suggestion ? { suggestion } : {}), notes });
    if (!email) return out(false, 'empty', 'Enter an email address.');
    if (email.length > 254) return out(false, 'too_long', 'Email addresses are at most 254 characters.');
    const at = email.split('@');
    if (at.length !== 2) return out(false, 'invalid', at.length < 2 ? 'This is missing the @ and the domain.' : 'An email address has exactly one @.');
    const [local, domain] = at;
    if (!local || !LOCAL.test(local) || local.startsWith('.') || local.endsWith('.') || local.includes('..')) return out(false, 'invalid', 'The part before the @ does not look right.');
    const labels = domain.split('.');
    if (labels.length < 2 || !labels.every((l) => LABEL.test(l)) || !/^[a-z]{2,24}$/.test(labels[labels.length - 1])) return out(false, 'invalid', 'The domain after the @ does not look right.');
    if (domain === 'gmail.com' || domain === 'googlemail.com') {
      if (local.includes('.') || local.includes('+')) notes.push('Gmail ignores dots and anything after +, but this grant matches only this exact address: the person must sign in with exactly this email.');
    } else if (local.includes('+')) notes.push('Has a +tag: the grant matches only this exact address.');
    const sug = suggestDomain(domain);
    if (sug) return out(false, 'typo', `Did you mean ${local}@${sug}?`, `${local}@${sug}`);
    return out(true, 'ok', '');
  }
  const splitBulk = (text) => String(text || '').split(/[\n\r,;\t]+|\s{2,}|(?<=@\S*)\s+(?=\S+@)/).map((s) => s.trim()).filter(Boolean);
  window.__oxynEmailCheck = { checkEmail, splitBulk }; // for the local test that compares it with the server

  // ── Shared bits for the new tabs ─────────────────────────────────────────────────────────────
  const TZ = (() => { try { return Intl.DateTimeFormat().resolvedOptions().timeZone || ''; } catch { return ''; } })();
  const tzMin = () => -new Date().getTimezoneOffset();
  const tzName = () => {
    try {
      const p = new Intl.DateTimeFormat(undefined, { timeZoneName: 'short' }).formatToParts(new Date()).find((x) => x.type === 'timeZoneName');
      return (p && p.value) || TZ;
    } catch { return TZ; }
  };
  const fmtDay = (s) => { if (!s) return '—'; const d = new Date(s); return isNaN(d) ? s : d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' }); };
  const num = (n) => Number(n || 0).toLocaleString();
  const pct = (a, b) => (b ? Math.round((a / b) * 1000) / 10 + '%' : '—');
  function trend(cur, prev) {
    if (!prev && !cur) return 'no change';
    if (!prev) return `up from 0`;
    const d = Math.round(((cur - prev) / prev) * 100);
    return d === 0 ? 'same as before' : `${d > 0 ? '+' : ''}${d}% vs the period before`;
  }
  const statCard = (k, v, sub, cls) => h('div', { class: 'stat card' + (cls ? ' ' + cls : '') }, h('span', { class: 'k' }, k), h('b', {}, v), sub ? h('small', {}, sub) : null);
  function errorBox(msg, retry) {
    return h('div', { class: 'empty errbox', role: 'alert' }, h('p', {}, msg), retry ? h('button', { class: 'btn sm', type: 'button', onclick: retry }, 'Try again') : null);
  }
  /** Admin sub-view Back: history back when the list is the previous entry, else the list itself. */
  function backBtn(listHash, label) {
    return h('button', { class: 'btn ghost sm subback', type: 'button', 'aria-label': 'Go back', onclick: () => {
      if (state.cameFrom === listHash && history.length > 1) history.back(); else go(listHash);
    } }, h('span', { 'aria-hidden': 'true' }, '←'), ' ', label || 'Back');
  }
  const PROVIDER = { google: 'Google', github: 'GitHub', password: 'Email' };
  const prov = (p) => PROVIDER[p] || cap(p);
  const PLAN_CHIP = { free: '', pro: 'solid', founder: 'solid', team_owner: 'solid', team_member: 'solid', granted: 'grant' };
  function planCell(p) {
    const src = p.source === 'polar' ? 'Paid via Polar' : p.source === 'seat' ? `Seat on ${p.owner || 'a paid Team'}` : p.source === 'grant'
      ? (p.owner ? `Grant · Team of ${p.owner}` : 'Grant') : '';
    return h('div', { class: 'plancell' },
      h('span', { class: 'chip ' + (PLAN_CHIP[p.key] || '') }, p.label),
      p.founder && p.key !== 'founder' ? h('span', { class: 'chip outline', title: 'Founder rate shown (display only)' }, 'Founder') : null,
      src || p.expires_at ? h('small', { class: 'src' }, [src, p.expires_at ? `${p.source === 'polar' ? 'ends' : 'until'} ${fmtDay(p.expires_at)}` : ''].filter(Boolean).join(' · ')) : null,
      p.pending ? h('small', { class: 'src pend' }, p.pending) : null);
  }

  /** A confirm dialog where the admin types a value (an email, or a count) before the action runs. */
  function typedConfirm({ title, lines = [], expect, okLabel = 'Confirm', danger = false, checkbox = null }) {
    return new Promise((resolve) => {
      const inp = h('input', { class: 'input', id: 'tc-in', autocomplete: 'off', spellcheck: 'false', 'aria-describedby': 'tc-hint tc-err' });
      const cb = checkbox ? h('input', { type: 'checkbox', id: 'tc-cb' }) : null;
      const ok = h('button', { class: 'btn ' + (danger ? 'danger' : 'primary'), type: 'button', disabled: true }, okLabel);
      const no = h('button', { class: 'btn', type: 'button' }, 'Cancel');
      const err = h('span', { class: 'err', id: 'tc-err' });
      const dlg = h('dialog', { class: 'dlg wide', 'aria-labelledby': 'tc-t' }, h('div', { class: 'dlg-in' },
        h('h2', { id: 'tc-t' }, title),
        ...lines.map((l) => (l instanceof Node ? l : h('p', {}, l))),
        h('div', { class: 'field' },
          h('label', { for: 'tc-in' }, 'Type ', h('b', { class: 'mono' }, expect), ' to confirm'),
          inp, h('span', { class: 'hint', id: 'tc-hint' }, 'This guards against granting or removing access by mistake.'), err),
        cb ? h('label', { class: 'cbrow', for: 'tc-cb' }, cb, ' ', checkbox) : null,
        h('div', { class: 'row end' }, no, ok)));
      const valid = () => inp.value.trim().toLowerCase() === String(expect).toLowerCase() && (!cb || cb.checked);
      const sync = () => { ok.disabled = !valid(); err.textContent = ''; };
      inp.addEventListener('input', sync); cb && cb.addEventListener('change', sync);
      inp.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); if (valid()) done(true); else err.textContent = inp.value ? 'That does not match yet.' : 'Type it to confirm.'; } });
      // Pasting is allowed, but the value still has to match exactly.
      const done = (v) => { dlg.close(); dlg.remove(); resolve(v ? { ok: true, value: inp.value.trim().toLowerCase() } : { ok: false }); };
      ok.addEventListener('click', () => valid() && done(true));
      no.addEventListener('click', () => done(false));
      dlg.addEventListener('cancel', (e) => { e.preventDefault(); done(false); });
      document.body.append(dlg); dlg.showModal(); inp.focus();
    });
  }

  /** A toast with Undo, open for `secs` seconds (revoke). */
  let undoT = 0, undoTick = 0;
  function undoToast(msg, onUndo, secs = 30) {
    const box = $('#undo');
    clearTimeout(undoT); clearInterval(undoTick);
    let left = secs;
    const btn = h('button', { class: 'btn sm undo-b', type: 'button' }, `Undo (${left}s)`);
    const close = h('button', { class: 'ib undo-x', type: 'button', 'aria-label': 'Dismiss' }, '✕');
    const hide = () => { clearTimeout(undoT); clearInterval(undoTick); box.classList.remove('on'); box.replaceChildren(); };
    btn.addEventListener('click', async () => { setBusy(btn, true, 'Undoing…'); try { await onUndo(); hide(); } catch (e) { setBusy(btn, false); toast(e.message); } });
    close.addEventListener('click', hide);
    box.replaceChildren(h('span', { class: 'undo-m' }, msg), btn, close);
    box.classList.add('on');
    undoTick = setInterval(() => { left--; if (left > 0) btn.textContent = `Undo (${left}s)`; }, 1000);
    undoT = setTimeout(hide, secs * 1000);
  }
  // ── Users ──────────────────────────────────────────────────────────────────────────────────
  const U = { q: '', plan: '', sort: 'created', dir: 'desc', page: 1 };
  const PLAN_OPTS = [['', 'All plans'], ['free', 'Free'], ['pro', 'Pro'], ['founder', 'Founder'], ['team_owner', 'Team owner'], ['team_member', 'Team member'], ['granted', 'Granted'], ['pending', 'Grant waiting for sign-in']];
  const SORT_OPTS = [['created:desc', 'Newest first'], ['created:asc', 'Oldest first'], ['active:desc', 'Last active'], ['email:asc', 'Email A–Z'], ['plan:desc', 'Plan']];
  const select = (label, id, opts, cur, onPick) => {
    const s = h('select', { class: 'input sel', id, 'aria-label': label }, opts.map(([v, l]) => h('option', { value: v, selected: v === cur }, l)));
    s.addEventListener('change', () => onPick(s.value));
    return s;
  };
  const usersQuery = (extra = {}) => {
    const p = new URLSearchParams({ sort: U.sort, dir: U.dir, page: String(U.page), tz: String(tzMin()), ...extra });
    if (U.q.trim()) p.set('q', U.q.trim());
    if (U.plan) p.set('plan', U.plan);
    return p;
  };
  async function renderUsers(id) {
    if (id) return renderUser(id);
    document.title = 'Users · Oxyn admin';
    const search = h('input', { class: 'input search', type: 'search', placeholder: 'Search email or name', 'aria-label': 'Search users', value: U.q });
    let st = 0;
    search.addEventListener('input', () => { clearTimeout(st); st = setTimeout(() => { U.q = search.value; U.page = 1; loadUsers(); }, 250); });
    const csv = h('a', { class: 'btn sm', id: 'csv', href: '#', download: '' }, 'Export CSV');
    const tools = h('div', { class: 'tools' }, search,
      select('Filter by plan', 'u-plan', PLAN_OPTS, U.plan, (v) => { U.plan = v; U.page = 1; loadUsers(); }),
      select('Sort', 'u-sort', SORT_OPTS, `${U.sort}:${U.dir}`, (v) => { [U.sort, U.dir] = v.split(':'); U.page = 1; loadUsers(); }),
      h('span', { class: 'grow' }), csv);
    main().replaceChildren(h('div', { class: 'page' },
      h('div', { class: 'head' }, h('h1', {}, 'Users'), h('span', { class: 'muted' }, `Times in ${tzName()}`)),
      h('div', { class: 'stats', id: 'u-stats' }, ['Total signups', 'Today', 'Last 7 days', 'Last 30 days'].map((k) => statCard(k, '…'))),
      tools,
      h('div', { class: 'card', id: 'u-list', 'aria-live': 'polite' }, h('p', { class: 'empty' }, 'Loading…')),
      h('div', { class: 'pager', id: 'u-pager' })));
    loadUsers();
  }
  async function loadUsers() {
    const box = $('#u-list'); if (!box) return;
    const csv = $('#csv'); if (csv) csv.href = '/api/admin/users?' + usersQuery({ format: 'csv' });
    box.setAttribute('aria-busy', 'true');
    let r;
    try { r = await api('/users?' + usersQuery()); }
    catch (e) { box.removeAttribute('aria-busy'); box.replaceChildren(errorBox(e.message, loadUsers)); return; }
    box.removeAttribute('aria-busy');
    const s = r.summary;
    fill($('#u-stats'),
      statCard('Total signups', num(s.total), `${num(s.verified)} verified with Google or GitHub`),
      statCard('Today', num(s.today), `${num(s.yesterday)} yesterday`),
      statCard('Last 7 days', num(s.d7), trend(s.d7, s.prev7)),
      statCard('Last 30 days', num(s.d30), trend(s.d30, s.prev30)));
    if (!r.items.length) {
      box.replaceChildren(h('p', { class: 'empty' }, s.total ? 'No user matches this search or filter.' : 'Nobody has signed up yet.'));
      $('#u-pager').replaceChildren();
      return;
    }
    const rows = r.items.map((u) => h('tr', { tabindex: 0, 'data-id': u.id, onclick: () => openUser(u.id), onkeydown: (e) => { if (e.key === 'Enter') openUser(u.id); } },
      h('td', { class: 'who2' }, h('a', { href: '#users/' + u.id, class: 'em', onclick: (e) => { e.preventDefault(); openUser(u.id); } }, u.email), u.name ? h('small', {}, u.name) : null),
      h('td', { class: 'prov' }, u.providers.length ? u.providers.map((p) => h('span', { class: 'chip', title: p === 'password' ? 'Email and password' : null }, prov(p))) : h('span', { class: 'muted' }, '—')),
      h('td', { class: 'ver' }, u.verified ? h('span', { class: 'chip ok' }, 'Verified') : h('span', { class: 'chip outline' }, 'No')),
      h('td', { class: 't' }, h('time', { datetime: u.created_at, title: fmtDate(u.created_at) }, fmtDay(u.created_at))),
      h('td', { class: 't' }, u.last_active ? h('time', { datetime: u.last_active, title: fmtDate(u.last_active) }, ago(u.last_active)) : '—'),
      h('td', { class: 'pl' }, planCell(u.plan))));
    box.replaceChildren(h('div', { class: 'twrap' }, h('table', { class: 'audit users' },
      h('caption', { class: 'sr' }, 'Users'),
      h('thead', {}, h('tr', {}, ['User', 'Sign-in', 'Verified', 'Signed up', 'Last active', 'Plan'].map((t) => h('th', { scope: 'col' }, t)))),
      h('tbody', {}, rows))));
    const from = (r.page - 1) * r.size + 1, to = Math.min(r.total, r.page * r.size);
    fill($('#u-pager'),
      h('span', { class: 'muted' }, `${num(from)}–${num(to)} of ${num(r.total)}${r.total !== s.total ? ` (${num(s.total)} in all)` : ''}`),
      h('span', { class: 'grow' }),
      h('button', { class: 'btn sm', type: 'button', disabled: r.page <= 1, onclick: () => { U.page = r.page - 1; loadUsers(); } }, '← Previous'),
      h('span', { class: 'muted' }, `Page ${r.page} of ${r.pages}`),
      h('button', { class: 'btn sm', type: 'button', disabled: r.page >= r.pages, onclick: () => { U.page = r.page + 1; loadUsers(); } }, 'Next →'));
  }
  function openUser(id) { state.cameFrom = 'users'; go('users/' + id); }

  async function renderUser(id) {
    document.title = 'User · Oxyn admin';
    const pageEl = h('div', { class: 'page narrow' }, backBtn('users', 'All users'), h('div', { class: 'card' }, h('p', { class: 'empty' }, 'Loading…')));
    main().replaceChildren(pageEl);
    let r;
    try { r = await api('/users/' + encodeURIComponent(id)); }
    catch (e) { pageEl.lastChild.replaceChildren(errorBox(e.message, () => renderUser(id))); return; }
    const u = r.user, p = u.plan;
    const paid = p.source === 'polar' || p.source === 'seat';
    const grants = r.grants.map((g) => h('tr', {},
      h('td', {}, g.plan === 'team' ? (g.parent_id ? `Team member (of ${g.owner_email})` : 'Team owner') : g.founder ? 'Pro · founder' : 'Pro'),
      h('td', {}, h('span', { class: 'chip ' + (g.revoked_at ? 'outline' : 'ok') }, g.revoked_at ? 'Revoked' : g.expires_at && new Date(g.expires_at) < new Date() ? 'Expired' : 'Active')),
      h('td', { class: 't' }, g.expires_at ? fmtDay(g.expires_at) : 'No end'),
      h('td', {}, g.reason || '—'),
      h('td', { class: 't' }, `${g.granted_by} · ${fmtDay(g.created_at)}`),
      h('td', {}, g.revoked_at
        ? h('button', { class: 'btn sm', type: 'button', onclick: async (e) => { const b = e.currentTarget; setBusy(b, true, 'Restoring…'); try { await api(`/grants/${g.id}/restore`, { method: 'POST', body: {} }); toast('Grant restored.'); renderUser(id); } catch (er) { setBusy(b, false); toast(er.message); } } }, 'Restore')
        : h('button', { class: 'btn sm danger', type: 'button', onclick: () => revokeFlow(g, () => renderUser(id)) }, 'Revoke'))));
    const subs = r.subscriptions.map((s) => h('tr', {},
      h('td', {}, cap(s.plan) + (s.founder ? ' · founder' : '')), h('td', {}, s.interval === 'year' ? 'Yearly' : 'Monthly'),
      h('td', {}, h('span', { class: 'chip' }, s.cancel_at_period_end ? 'canceling' : s.status)), h('td', { class: 't' }, fmtDay(s.current_period_end)), h('td', { class: 't' }, fmtDay(s.created_at))));
    fill(pageEl,
      backBtn('users', 'All users'),
      h('div', { class: 'head' }, h('h1', { class: 'em' }, u.email), u.name ? h('span', { class: 'muted' }, u.name) : null),
      h('div', { class: 'stack' },
        h('div', { class: 'card section' },
          h('dl', { class: 'kv' },
            h('dt', {}, 'Plan'), h('dd', {}, planCell(p)),
            h('dt', {}, 'Sign-in'), h('dd', {}, u.providers.length ? u.providers.map((x) => (x === 'password' ? 'Email and password' : prov(x))).join(', ') : '—'),
            h('dt', {}, 'Verified'), h('dd', {}, u.verified ? 'Yes, by Google or GitHub' : 'No: grants wait until they sign in with Google or GitHub'),
            h('dt', {}, 'Signed up'), h('dd', {}, fmtDate(u.created_at)),
            h('dt', {}, 'Last active'), h('dd', {}, u.last_active ? fmtDate(u.last_active) : '—'),
            h('dt', {}, 'Account id'), h('dd', { class: 'mono' }, u.id)),
          paid ? h('div', { class: 'note' }, h('div', {},
            h('p', { class: 'nline' }, p.source === 'seat' ? `Pro comes from a seat on ${p.owner || 'a paid Team'}. The Team owner manages seats in their account.` : 'Pro comes from a paid Polar subscription. Cancel, refund or change it in Polar; it cannot be removed here.'),
            h('a', { class: 'btn sm', href: r.polar_dashboard, target: '_blank', rel: 'noopener noreferrer' }, 'Manage in Polar ↗'))) : null),
        planForm(u, r),
        h('section', { class: 'card', 'aria-labelledby': 'ug-t' }, h('div', { class: 'section fhead' }, h('h2', { id: 'ug-t' }, 'Grants for this email')),
          grants.length ? h('div', { class: 'twrap' }, h('table', { class: 'audit gtab' }, h('thead', {}, h('tr', {}, ['Plan', 'Status', 'Ends', 'Reason', 'By', ''].map((t) => h('th', { scope: 'col' }, t)))), h('tbody', {}, grants)))
            : h('p', { class: 'empty' }, 'No grants yet.')),
        subs.length ? h('section', { class: 'card', 'aria-labelledby': 'us-t' }, h('div', { class: 'section fhead' }, h('h2', { id: 'us-t' }, 'Polar subscriptions')),
          h('div', { class: 'twrap' }, h('table', { class: 'audit' }, h('thead', {}, h('tr', {}, ['Plan', 'Billing', 'Status', 'Renews / ends', 'Since'].map((t) => h('th', { scope: 'col' }, t)))), h('tbody', {}, subs)))) : null));
    pageEl.querySelector('.subback').focus();
  }

  /** Change one account's plan: Free (removes its grant), Pro (optionally with the founder flag) or Team. */
  function planForm(u, r) {
    const p = u.plan;
    const paid = p.source === 'polar' || p.source === 'seat';
    let plan = p.tier === 'team' && p.source === 'grant' && !p.owner ? 'team' : p.source === 'grant' ? 'pro' : 'pro';
    const form = h('form', { class: 'section pform', novalidate: true });
    const draw = () => {
      const segEl = seg('New plan', [['free', 'Free'], ['pro', 'Pro'], ['team', 'Team']], plan, (v) => { plan = v; draw(); });
      const members = plan === 'team' ? h('fieldset', { class: 'members' }, h('legend', {}, `Teammates (up to ${r.people_limit - 1}; ${r.people_limit} people with the owner)`),
        [1, 2, 3].map((i) => emailField(`pm-${i}`, `Teammate ${i}`, 'members'))) : null;
      fill(form,
        h('h2', { id: 'pf-t' }, 'Change plan'),
        h('p', { class: 'muted' }, 'Gives the plan as a grant (no payment). Free removes the grant. Every change is in the audit log.'),
        h('div', { class: 'row' }, h('span', { class: 'lbl' }, 'Plan'), segEl),
        plan === 'free' && paid ? h('p', { class: 'err' }, 'This account pays through Polar: Free cannot be set here. Use Manage in Polar.') : null,
        plan === 'pro' ? h('label', { class: 'cbrow', for: 'pf-founder' }, h('input', { type: 'checkbox', id: 'pf-founder', name: 'founder', checked: p.founder }), ' Show the founder rate (display only, nothing is charged)') : null,
        members,
        plan !== 'free' ? field('pf-exp', 'End date (optional)', { type: 'date', name: 'expires_at', min: tomorrow() }, 'Empty = no end. Ends at the end of that day, your time.') : null,
        field('pf-reason', 'Reason', { name: 'reason', required: true, maxlength: 300, autocomplete: 'off', placeholder: 'e.g. Contributor, sponsor, refund made good' }),
        paid && plan !== 'free' ? h('label', { class: 'cbrow', for: 'pf-ackpaid' }, h('input', { type: 'checkbox', id: 'pf-ackpaid', name: 'ack' }), ' They already pay through Polar; add this grant anyway') : null,
        h('div', { class: 'row' }, h('button', { class: 'btn primary', type: 'submit', disabled: plan === 'free' && paid }, plan === 'free' ? 'Remove grant' : `Set ${plan === 'team' ? 'Team' : 'Pro'}`)),
        h('span', { class: 'err form', role: 'alert' }));
    };
    form.setAttribute('aria-labelledby', 'pf-t');
    draw();
    form.addEventListener('submit', async (ev) => {
      ev.preventDefault();
      showErr(form, null);
      const reason = form.reason.value.trim();
      if (reason.length < 3) return showErr(form, { field: 'reason', message: 'Write why (a few words).' });
      const body = { plan, reason };
      if (plan !== 'free') {
        const exp = form.expires_at.value;
        if (exp) { const t = endOfDay(exp); if (!t || t <= Date.now()) return showErr(form, { field: 'expires_at', message: 'The end date must be in the future.' }); body.expires_at = new Date(t).toISOString(); }
      }
      if (plan === 'pro') body.founder = form.founder.checked;
      if (plan === 'team') {
        const ms = [...form.querySelectorAll('[name=members]')].map((i) => i.value.trim()).filter(Boolean);
        for (const m of ms) { const c = checkEmail(m); if (!c.ok && c.code !== 'typo') return showErr(form, { message: `${m}: ${c.message}` }); }
        body.members = ms;
        const keep = [...form.querySelectorAll('[data-keep="1"]')].map((i) => checkEmail(i.value).email);
        if (keep.length) body.keep_typos = keep;
        const typos = ms.map(checkEmail).filter((c) => c.code === 'typo' && !keep.includes(c.email));
        if (typos.length) return showErr(form, { message: `${typos[0].email}: ${typos[0].message} Use the suggestion or keep it as typed.` });
      }
      if (form.ack) body.acknowledge_paid = form.ack.checked;
      const what = plan === 'free' ? `Remove the grant of ${u.email}` : `Set ${u.email} to ${plan === 'team' ? `Team (${1 + (body.members || []).length} people)` : 'Pro'}`;
      const c = await typedConfirm({ title: what + '?', lines: [`Now: ${p.label}. ${plan === 'free' ? 'They lose the granted plan on their next request; you can restore it from the grant list.' : 'Any current grant is replaced (it stays in History).'}`], expect: u.email, okLabel: plan === 'free' ? 'Remove grant' : 'Change plan', danger: plan === 'free' });
      if (!c.ok) return;
      body.confirm = c.value;
      const btn = form.querySelector('button[type=submit]'); setBusy(btn, true, 'Saving…');
      try { await api(`/users/${u.id}/plan`, { method: 'POST', body }); toast(plan === 'free' ? 'Grant removed.' : 'Plan changed.'); renderUser(u.id); }
      catch (e) { setBusy(btn, false); showErr(form, e); }
    });
    return h('section', { class: 'card' }, form);
  }
  const tomorrow = () => { const d = new Date(Date.now() + 86_400_000); return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`; };
  /** "2026-12-31" → the end of that day in the admin's time zone (ms), or 0. */
  const endOfDay = (s) => { const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(s || ''); if (!m) return 0; const d = new Date(+m[1], +m[2] - 1, +m[3], 23, 59, 59); return isNaN(d) ? 0 : d.getTime(); };

  /** An email input with the live format + typo check ("Did you mean …?" with Use / Keep). */
  function emailField(id, label, name, attrs = {}) {
    const f = field(id, label, { type: 'email', name, autocomplete: 'off', spellcheck: 'false', inputmode: 'email', placeholder: 'name@example.com', ...attrs });
    const inp = f.querySelector('input');
    const tip = h('div', { class: 'typo', id: `${id}-typo`, 'aria-live': 'polite' });
    f.append(tip);
    const run = () => {
      tip.replaceChildren(); inp.removeAttribute('aria-invalid');
      const v = inp.value.trim();
      if (!v) return;
      const c = checkEmail(v);
      if (c.code === 'typo' && inp.dataset.keep !== '1') {
        tip.append(h('span', {}, c.message), ' ',
          h('button', { class: 'btn sm', type: 'button', onclick: () => { inp.value = c.suggestion; inp.dataset.keep = ''; run(); inp.focus(); } }, `Use ${c.suggestion}`), ' ',
          h('button', { class: 'btn sm ghost', type: 'button', onclick: () => { inp.dataset.keep = '1'; run(); } }, 'Keep as typed'));
        tip.className = 'typo warn';
      } else if (!c.ok && c.code !== 'typo') { tip.append(c.message); tip.className = 'typo bad'; inp.setAttribute('aria-invalid', 'true'); }
      else if (c.notes.length) { tip.append(c.notes.join(' ')); tip.className = 'typo note2'; }
    };
    inp.addEventListener('input', () => { inp.dataset.keep = ''; clearTimeout(inp._t); inp._t = setTimeout(run, 350); });
    inp.addEventListener('blur', run);
    return f;
  }
  // ── Grants ─────────────────────────────────────────────────────────────────────────────────
  const G = { mode: 'one', founder: false, status: 'active', q: '', preview: null, limit: 100 };
  const STATUS_CHIP = { valid: ['Valid', 'ok'], invalid: ['Invalid', 'bad'], duplicate: ['Duplicate', 'dup'], typo: ['Check spelling', 'warn'], paid: ['Already pays', 'warn'] };
  async function renderGrants() {
    document.title = 'Grants · Oxyn admin';
    G.preview = null;
    const form = h('form', { class: 'section gform', novalidate: true, 'aria-labelledby': 'gf-t' });
    const drawForm = () => {
      const mode = seg('How many', [['one', 'One email'], ['team', 'Team'], ['bulk', 'Bulk paste']], G.mode, (v) => { G.mode = v; G.preview = null; drawForm(); drawPreview(); });
      const who = G.mode === 'one' ? emailField('g-email', 'Email', 'email', { required: true })
        : G.mode === 'team' ? h('div', { class: 'stack sm' }, emailField('g-owner', 'Team owner email', 'owner', { required: true }),
          h('fieldset', { class: 'members' }, h('legend', {}, 'Teammates (optional, up to 3; a Team is 4 people with the owner)'), [1, 2, 3].map((i) => emailField(`g-m${i}`, `Teammate ${i}`, 'members'))))
        : h('div', { class: 'field' }, h('label', { for: 'g-bulk' }, 'Emails'),
          h('textarea', { class: 'input ta', id: 'g-bulk', name: 'bulk', rows: 7, spellcheck: 'false', placeholder: 'One per line, or separated by commas\nname@example.com\nName <other@example.com>', 'aria-describedby': 'g-bulk-hint g-bulk-err' }),
          h('span', { class: 'hint', id: 'g-bulk-hint' }, 'Up to 200. Each line is checked before anything is granted; more than 20 asks for an extra confirmation.'),
          h('span', { class: 'err', id: 'g-bulk-err' }));
      fill(form,
        h('h2', { id: 'gf-t' }, 'Give a plan without payment'),
        h('p', { class: 'muted' }, 'Works before or after someone signs up: the grant turns on when they sign in with Google or GitHub on that exact email (a password-only account must verify first).'),
        h('div', { class: 'row' }, h('span', { class: 'lbl' }, 'Who'), mode),
        who,
        G.mode !== 'team' ? h('label', { class: 'cbrow', for: 'g-founder' }, h('input', { type: 'checkbox', id: 'g-founder', name: 'founder', checked: G.founder, onchange: (e) => { G.founder = e.target.checked; } }), ' Pro with the founder rate shown (display only, nothing is charged)') : null,
        h('div', { class: 'grid2' },
          field('g-reason', 'Reason', { name: 'reason', required: true, maxlength: 300, autocomplete: 'off', placeholder: 'e.g. Contributor, sponsor, refund made good' }, 'Required. Kept with the grant and in the audit log.'),
          field('g-exp', 'End date (optional)', { type: 'date', name: 'expires_at', min: tomorrow() }, 'Empty = no end. Ends at the end of that day, your time.')),
        h('div', { class: 'row' }, h('button', { class: 'btn primary', type: 'submit' }, 'Check and preview'), h('span', { class: 'muted small' }, 'Nothing is granted until you confirm.')),
        h('span', { class: 'err form', role: 'alert' }));
    };
    const previewBox = h('div', { id: 'g-preview' });
    const drawPreview = () => {
      const pv = G.preview;
      if (!pv) { previewBox.replaceChildren(); return; }
      const s = pv.summary;
      const keep = pv.keep;
      const bad = pv.lines.filter((l) => l.status === 'invalid' || l.status === 'duplicate' || (l.status === 'typo' && !keep.has(l.email)) || (l.status === 'paid' && !pv.ackPaid));
      const ready = pv.lines.filter((l) => !bad.includes(l));
      // Bulk: lines that do not pass are left out (listed above); one email or a Team must all pass.
      const blocking = G.mode === 'bulk' ? [] : bad;
      const rows = pv.lines.map((l) => {
        const [lbl, cls] = STATUS_CHIP[l.status];
        const kept = l.status === 'typo' && keep.has(l.email);
        return h('tr', { class: 'st-' + l.status },
          h('td', { class: 'n' }, String(l.line)),
          h('td', { class: 'em2' }, l.email || h('span', { class: 'muted' }, l.input || '(empty)'), l.role === 'owner' ? h('span', { class: 'chip outline' }, 'Owner') : l.role === 'member' ? h('span', { class: 'chip outline' }, 'Teammate') : null),
          h('td', {}, h('span', { class: 'chip ' + (kept ? 'ok' : cls) }, kept ? 'Kept as typed' : lbl)),
          h('td', { class: 'msg2' },
            l.message ? h('div', {}, l.message) : null,
            ...[...l.notes, ...l.warnings].map((w) => h('div', { class: 'muted' }, w)),
            l.status === 'typo' && !kept ? h('div', { class: 'row' },
              h('button', { class: 'btn sm', type: 'button', onclick: () => useSuggestion(l) }, `Use ${l.suggestion}`),
              h('button', { class: 'btn sm ghost', type: 'button', onclick: () => { keep.add(l.email); drawPreview(); } }, 'Keep as typed')) : null));
      });
      const plan = G.mode === 'team' ? 'Team' : G.founder ? 'Pro (founder rate shown)' : 'Pro';
      const n = ready.length;
      const go = h('button', { class: 'btn primary', type: 'button', disabled: !n || blocking.length > 0 }, n ? `Grant ${plan} to ${n} ${n === 1 ? 'email' : 'emails'}${G.mode === 'bulk' && bad.length ? ` (skip ${bad.length})` : ''}` : 'Nothing to grant');
      go.addEventListener('click', () => commit(pv, ready, go));
      previewBox.replaceChildren(h('section', { class: 'card section preview', 'aria-labelledby': 'pv-t' },
        h('div', { class: 'head' }, h('h2', { id: 'pv-t' }, 'Preview'),
          h('span', { class: 'muted' }, [`${s.total} checked`, s.valid ? `${s.valid} valid` : '', s.invalid ? `${s.invalid} invalid` : '', s.duplicate ? `${s.duplicate} duplicate` : '', s.typo ? `${s.typo} to check` : '', s.paid ? `${s.paid} already pay` : ''].filter(Boolean).join(' · '))),
        h('div', { class: 'twrap' }, h('table', { class: 'audit report' },
          h('caption', { class: 'sr' }, 'Each email and its check'),
          h('thead', {}, h('tr', {}, ['#', 'Email', 'Check', 'Details'].map((t) => h('th', { scope: 'col' }, t)))), h('tbody', {}, rows))),
        s.paid ? h('label', { class: 'cbrow', for: 'g-ackpaid' }, h('input', { type: 'checkbox', id: 'g-ackpaid', checked: pv.ackPaid, onchange: (e) => { pv.ackPaid = e.target.checked; drawPreview(); } }), ' Grant anyway to the ones who already pay (they keep paying in Polar)') : null,
        blocking.length ? h('p', { class: 'err' }, 'Fix the marked email and check again. Nothing has been granted.') : null,
        G.mode === 'bulk' && bad.length ? h('p', { class: 'hint' }, `${bad.length} ${bad.length === 1 ? 'line is' : 'lines are'} left out: ${bad.map((l) => l.line).join(', ')}. Fix them in the list and check again to include them, or keep a spelling as typed.`) : null,
        h('div', { class: 'row' }, go, h('button', { class: 'btn ghost', type: 'button', onclick: () => { G.preview = null; drawPreview(); } }, 'Cancel'))));
      previewBox.querySelector('h2').scrollIntoView({ block: 'nearest' });
    };
    function useSuggestion(l) {
      // Put the suggestion into the form and check again.
      if (G.mode === 'bulk') { const ta = form.bulk; const lines = splitBulk(ta.value); lines[l.line - 1] = l.suggestion; ta.value = lines.join('\n'); }
      else { const inputs = G.mode === 'one' ? [form.email] : [form.owner, ...form.querySelectorAll('[name=members]')].filter((i) => i.value.trim()); const i = inputs[l.line - 1]; if (i) { i.value = l.suggestion; i.dataset.keep = ''; i.dispatchEvent(new Event('blur')); } }
      form.requestSubmit();
    }
    const gather = () => {
      const body = { plan: G.mode === 'team' ? 'team' : 'pro' };
      if (G.mode === 'one') body.emails = [form.email.value];
      else if (G.mode === 'bulk') body.emails = splitBulk(form.bulk.value);
      else { body.owner = form.owner.value; body.members = [...form.querySelectorAll('[name=members]')].map((i) => i.value).filter((v) => v.trim()); }
      return body;
    };
    form.addEventListener('submit', async (ev) => {
      ev.preventDefault();
      showErr(form, null);
      const body = gather();
      // Instant checks first (the server checks everything again).
      if (G.mode === 'one' && !form.email.value.trim()) return showErr(form, { field: 'email', message: 'Enter an email address.' });
      if (G.mode === 'team' && !form.owner.value.trim()) return showErr(form, { field: 'owner', message: 'Enter the Team owner’s email.' });
      if (G.mode === 'bulk' && !body.emails.length) return showErr(form, { field: 'bulk', message: 'Paste at least one email.' });
      if (G.mode === 'bulk' && body.emails.length > 200) return showErr(form, { field: 'bulk', message: `That is ${body.emails.length} emails; up to 200 at a time.` });
      const reason = form.reason.value.trim();
      if (reason.length < 3) return showErr(form, { field: 'reason', message: 'Write why this grant is given (a few words).' });
      const exp = form.expires_at.value;
      let expires = null;
      if (exp) { const t = endOfDay(exp); if (!t || t <= Date.now()) return showErr(form, { field: 'expires_at', message: 'The end date must be in the future.' }); expires = new Date(t).toISOString(); }
      const btn = form.querySelector('button[type=submit]'); setBusy(btn, true, 'Checking…');
      try {
        const r = await api('/grants/check', { method: 'POST', body });
        const keep = new Set([...form.querySelectorAll('[data-keep="1"]')].map((i) => checkEmail(i.value).email));
        G.preview = { ...r, body, reason, expires, founder: G.mode !== 'team' && G.founder, keep, ackPaid: false };
        drawPreview();
      } catch (e) { showErr(form, e); }
      finally { setBusy(btn, false); }
    });
    async function commit(pv, ready, btn) {
      const n = ready.length;
      const single = G.mode !== 'bulk' || n === 1;
      const expect = single ? ready[0].email : String(n);
      const plan = G.mode === 'team' ? `Team (${n} ${n === 1 ? 'person' : 'people'})` : pv.founder ? 'Pro with the founder rate shown' : 'Pro';
      const lines = [
        `${plan} for ${single ? ready[0].email : `${n} emails`}${pv.expires ? `, until ${fmtDate(pv.expires)}` : ', with no end date'}.`,
        `Reason: ${pv.reason}`,
      ];
      const skipped = pv.lines.length - n;
      if (G.mode === 'bulk' && skipped) lines.push(`${skipped} ${skipped === 1 ? 'line is' : 'lines are'} left out.`);
      if (G.mode === 'bulk' && n > 1) lines.push(h('ul', { class: 'clist' }, ready.slice(0, 8).map((l) => h('li', {}, l.email)), n > 8 ? h('li', { class: 'muted' }, `and ${n - 8} more`) : null));
      const c = await typedConfirm({ title: `Grant ${G.mode === 'team' ? 'Team' : 'Pro'}?`, lines, expect, okLabel: `Grant to ${n}`, checkbox: n > 20 ? `I checked all ${n} emails in the list` : null });
      if (!c.ok) return;
      const body = { ...pv.body, reason: pv.reason, expires_at: pv.expires, founder: pv.founder, confirm: c.value, keep_typos: [...pv.keep], acknowledge_paid: pv.ackPaid, bulk_ack: n > 20 };
      if (G.mode === 'bulk') body.emails = ready.map((l) => l.input);
      setBusy(btn, true, 'Granting…');
      try {
        const r = await api('/grants', { method: 'POST', body });
        toast(`Granted to ${r.count} ${r.count === 1 ? 'email' : 'emails'}.`);
        G.preview = null; renderGrants();
      } catch (e) {
        setBusy(btn, false);
        toast(e.message);
        if (e.code === 'duplicate' || e.code === 'typo_suspected' || e.code === 'paid_subscription') form.requestSubmit();
      }
    }
    drawForm();
    const search = h('input', { class: 'input search', type: 'search', placeholder: 'Search email or reason', 'aria-label': 'Search grants', value: G.q });
    let st = 0;
    search.addEventListener('input', () => { clearTimeout(st); st = setTimeout(() => { G.q = search.value; loadGrants(); }, 250); });
    main().replaceChildren(h('div', { class: 'page' },
      h('div', { class: 'head' }, h('h1', {}, 'Grants'), h('span', { class: 'muted' }, 'Pro or Team without payment')),
      h('div', { class: 'stack' },
        h('section', { class: 'card' }, form),
        previewBox,
        h('section', { class: 'card', 'aria-labelledby': 'gl-t' },
          h('div', { class: 'section fhead' }, h('h2', { id: 'gl-t' }, 'Granted'),
            h('div', { class: 'tools' }, h('div', { id: 'g-status' }), search)),
          h('div', { id: 'g-list', 'aria-live': 'polite' }, h('p', { class: 'empty' }, 'Loading…'))))));
    loadGrants();
  }
  async function loadGrants() {
    const box = $('#g-list'); if (!box) return;
    let r;
    try { r = await api('/grants?' + new URLSearchParams({ status: G.status, q: G.q.trim() })); }
    catch (e) { box.replaceChildren(errorBox(e.message, loadGrants)); return; }
    const c = r.counts;
    $('#g-status').replaceChildren(seg('Status', [['active', `Active ${c.active + c.pending}`], ['pending', `Waiting ${c.pending}`], ['revoked', `History ${c.revoked}`], ['expired', `Expired ${c.expired}`], ['all', 'All']], G.status, (v) => { G.status = v; G.limit = 100; loadGrants(); }));
    if (!r.items.length) {
      box.replaceChildren(h('p', { class: 'empty' }, G.q ? 'No grant matches this search.' : G.status === 'revoked' ? 'Nothing revoked yet. Revoked grants stay here and can be restored.' : G.status === 'expired' ? 'No expired grants.' : G.status === 'pending' ? 'No grant is waiting for a sign-in.' : 'No grants yet. Give one above.'));
      return;
    }
    const row = (g, member) => {
      const st = { active: ['Active', 'ok'], pending: ['Waiting for sign-in', 'warn'], revoked: ['Revoked', 'outline'], expired: ['Expired', 'outline'] }[g.status];
      return h('tr', { class: member ? 'member' : '' },
        h('td', { class: 'em2' }, member ? h('span', { class: 'tree', 'aria-hidden': 'true' }, '└ ') : null,
          g.user_id ? h('a', { href: '#users/' + g.user_id, onclick: (e) => { e.preventDefault(); state.cameFrom = 'grants'; go('users/' + g.user_id); } }, g.email) : g.email,
          g.paid ? h('span', { class: 'chip', title: 'Also has a paid Polar subscription' }, 'Pays') : null),
        h('td', {}, g.label + (g.founder && g.key !== 'founder' ? ' · founder' : '')),
        h('td', {}, h('span', { class: 'chip ' + st[1], title: g.status === 'pending' ? (g.account === 'none' ? 'No account with this email yet' : 'Account not verified with Google or GitHub yet') : g.revoked_at ? `Revoked by ${g.revoked_by} on ${fmtDate(g.revoked_at)}` : '' }, st[0])),
        h('td', { class: 't' }, g.expires_at ? fmtDay(g.expires_at) : 'No end'),
        h('td', { class: 'rsn' }, g.reason || '—'),
        h('td', { class: 't' }, h('span', { title: fmtDate(g.created_at) }, `${g.granted_by.split('@')[0]} · ${fmtDay(g.created_at)}`)),
        h('td', { class: 'act' }, g.revoked_at
          ? h('button', { class: 'btn sm', type: 'button', 'aria-label': `Restore ${g.email}`, onclick: async (e) => {
              const b = e.currentTarget; setBusy(b, true, 'Restoring…');
              try { const x = await api(`/grants/${g.id}/restore`, { method: 'POST', body: {} }); toast(x.expired ? 'Restored, but its end date has passed: it stays expired.' : x.skipped.length ? `Restored. Left out (has another grant now): ${x.skipped.join(', ')}.` : 'Restored.'); loadGrants(); }
              catch (er) { setBusy(b, false); toast(er.message); }
            } }, 'Restore')
          : h('button', { class: 'btn sm danger', type: 'button', 'aria-label': `Revoke ${g.email}`, onclick: () => revokeFlow(g, loadGrants) }, 'Revoke')));
    };
    const rows = [];
    for (const g of r.items) { rows.push(row(g, false)); for (const m of g.members || []) rows.push(row(m, true)); }
    fill(box, h('div', { class: 'twrap' }, h('table', { class: 'audit grants' },
      h('caption', { class: 'sr' }, 'Grants'),
      h('thead', {}, h('tr', {}, ['Email', 'Plan', 'Status', 'Ends', 'Reason', 'Granted', ''].map((t) => h('th', { scope: 'col' }, t)))),
      h('tbody', {}, rows.slice(0, G.limit)))),
      rows.length > G.limit ? h('div', { class: 'more' }, h('button', { class: 'btn sm', type: 'button', onclick: () => { G.limit += 100; loadGrants(); } }, `Show more (${rows.length - G.limit} left)`)) : null,
      r.items.length >= 1000 ? h('p', { class: 'hint pad' }, 'Showing the newest 1,000. Search to find older ones.') : null);
  }
  async function revokeFlow(g, after) {
    const team = g.plan === 'team' && !g.parent_id;
    const ok = await confirmDlg(`Revoke ${g.email}?`,
      `${team ? 'The Team grant and its teammates lose' : 'They lose'} the plan on their next request. You can undo for 30 seconds, and restore it later from History. A paid Polar subscription is not touched.`, 'Revoke');
    if (!ok) return;
    try {
      const r = await api(`/grants/${g.id}/revoke`, { method: 'POST', body: {} });
      after();
      undoToast(`Revoked ${g.email}${r.ids.length > 1 ? ` and ${r.ids.length - 1} teammate${r.ids.length > 2 ? 's' : ''}` : ''}.`, async () => {
        await api(`/grants/${g.id}/restore`, { method: 'POST', body: {} });
        toast('Restored.'); after();
      });
    } catch (e) { toast(e.message); }
  }

  // ── Activity ───────────────────────────────────────────────────────────────────────────────
  let actDays = 30;
  /** Bars for a daily series, plain SVG. Values in HTML around it so the text never stretches. */
  function barChart(series, key, label) {
    const max = Math.max(1, ...series.map((d) => d[key]));
    const W = series.length * 10, H = 100;
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('viewBox', `0 0 ${W} ${H}`); svg.setAttribute('preserveAspectRatio', 'none'); svg.setAttribute('class', 'bars');
    svg.setAttribute('role', 'img'); svg.setAttribute('aria-label', `${label}, last ${series.length} days. Highest ${max}.`);
    for (let i = 1; i <= 3; i++) { const l = document.createElementNS(svg.namespaceURI, 'line'); l.setAttribute('x1', 0); l.setAttribute('x2', W); l.setAttribute('y1', (H / 4) * i); l.setAttribute('y2', (H / 4) * i); l.setAttribute('class', 'grid'); svg.append(l); }
    series.forEach((d, i) => {
      const v = d[key]; const bh = v ? Math.max(2, (v / max) * (H - 4)) : 0;
      const r = document.createElementNS(svg.namespaceURI, 'rect');
      r.setAttribute('x', i * 10 + 1.5); r.setAttribute('width', 7); r.setAttribute('y', H - bh); r.setAttribute('height', bh); r.setAttribute('rx', 1.5);
      const t = document.createElementNS(svg.namespaceURI, 'title'); t.textContent = `${d.day}: ${v}`; r.append(t);
      svg.append(r);
    });
    const dayLbl = (s) => new Date(s + 'T12:00:00').toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
    const total = series.reduce((a, d) => a + d[key], 0);
    return h('div', { class: 'chart' },
      h('div', { class: 'chart-h' }, h('h3', {}, label), h('span', { class: 'muted' }, `${num(total)} in ${series.length} days · peak ${num(max === 1 && !total ? 0 : max)}`)),
      h('div', { class: 'chart-b' }, h('span', { class: 'ymax', 'aria-hidden': 'true' }, num(total ? max : 0)), svg),
      h('div', { class: 'chart-x', 'aria-hidden': 'true' }, h('span', {}, dayLbl(series[0].day)), h('span', {}, dayLbl(series[Math.floor(series.length / 2)].day)), h('span', {}, 'Today')),
      // Screen readers get the numbers as a table.
      h('table', { class: 'sr' }, h('caption', {}, label), h('tbody', {}, series.map((d) => h('tr', {}, h('th', { scope: 'row' }, d.day), h('td', {}, String(d[key])))))));
  }
  function hbars(title, items, empty, opts = {}) {
    const max = Math.max(1, ...items.map((x) => x.n));
    return h('section', { class: 'card section hb', 'aria-label': title },
      h('h3', {}, title),
      items.length ? h('ul', { class: 'hbars' }, items.map((x) => h('li', {},
        h('span', { class: 'hl' }, x.href ? h('a', { href: x.href, target: '_blank', rel: 'noopener' }, x.label) : x.label, x.sub ? h('small', {}, x.sub) : null),
        h('span', { class: 'hv' }, num(x.n)),
        h('span', { class: 'track', 'aria-hidden': 'true' }, h('span', { class: 'fillbar' + (opts.cls ? ' ' + opts.cls : ''), style: null, 'data-w': String(Math.round((x.n / max) * 100)) }))))) : h('p', { class: 'empty sm' }, empty));
  }
  async function renderActivity() {
    document.title = 'Activity · Oxyn admin';
    const body = h('div', { class: 'stack', id: 'act' }, h('div', { class: 'card' }, h('p', { class: 'empty' }, 'Loading…')));
    main().replaceChildren(h('div', { class: 'page' },
      h('div', { class: 'head' }, h('h1', {}, 'Activity'),
        h('div', { class: 'row' }, h('span', { class: 'muted' }, `Days in ${tzName()}`), seg('Period', [['7', '7 days'], ['30', '30 days'], ['90', '90 days']], String(actDays), (v) => { actDays = Number(v); renderActivity(); }))),
      body));
    let r;
    try { r = await api('/activity?' + new URLSearchParams({ days: String(actDays), tz: String(tzMin()) })); }
    catch (e) { body.replaceChildren(h('div', { class: 'card' }, errorBox(e.message, renderActivity))); return; }
    const pv = r.pricing, co = r.checkouts;
    const proCopy = r.pro_copy.length ? h('div', { class: 'twrap' }, h('table', { class: 'audit pc' },
      h('caption', { class: 'sr' }, 'Pro copy attempts by item'),
      h('thead', {}, h('tr', {}, ['Item', 'Attempts', 'Copy', 'Ctrl/⌘ C', 'CLI', 'Views', 'Per view'].map((t) => h('th', { scope: 'col' }, t)))),
      h('tbody', {}, r.pro_copy.map((x) => h('tr', {}, h('td', { class: 'mono' }, x.item), h('td', {}, h('b', {}, num(x.n))), h('td', {}, num(x.copy)), h('td', {}, num(x.keys + x.menu)), h('td', {}, num(x.cli)), h('td', {}, num(x.views)), h('td', {}, pct(x.n, x.views)))))))
      : h('p', { class: 'empty sm' }, 'No Pro copy attempts in this period.');
    const funnel = [['Pricing views', pv.views, `${num(pv.visitors)} visitors`], ['Checkouts started', co.started, pct(co.started, pv.views) + ' of views'], ['Paid', co.paid, pct(co.paid, co.started) + ' of started']];
    const fmax = Math.max(1, ...funnel.map((f) => f[1]));
    const paidPlans = co.paid_by_plan.map((x) => `${x.n} ${x.plan === 'team' ? 'Team' : x.founder ? 'Founder' : 'Pro'}`).join(' · ');
    const noEvents = !r.events_kept;
    fill(body,
      noEvents ? h('div', { class: 'note' }, h('div', {}, h('p', { class: 'nline' }, 'No site events recorded yet. Page views, Pro copy attempts, pricing views and checkout clicks are counted from this version on (no cookies, nothing personal, kept 90 days).'))) : null,
      h('div', { class: 'stats' },
        statCard('Daily active', num(r.active.dau), 'accounts, last 24 h'),
        statCard('Weekly active', num(r.active.wau), 'accounts, last 7 days'),
        statCard('Monthly active', num(r.active.mau), 'accounts, last 30 days'),
        statCard('Visitors today', num(r.today.visitors), `${num(r.today.views)} page views`)),
      h('div', { class: 'stats' },
        statCard('MRR', usd(r.mrr_cents), `${usd(r.arr_cents)} a year`),
        statCard('Active subscriptions', num(r.subs.active), `${r.subs.pro} Pro · ${r.subs.founder} Founder · ${r.subs.team} Team`),
        statCard('Churn, 30 days', num(r.churn.canceled_30d), `${pct(r.churn.canceled_30d, r.subs.active + r.churn.canceled_30d)} · ${r.subs.canceling} set to cancel`),
        statCard('Grants in effect', num(r.subs.grants), `${num(r.users)} accounts in all`)),
      h('section', { class: 'card section' }, barChart(r.series, 'signups', 'Signups per day')),
      h('section', { class: 'card section' }, barChart(r.series, 'views', 'Page views per day')),
      h('div', { class: 'grid2' },
        hbars('Sign-ins by provider', r.signins.map((x) => ({ label: x.provider === 'password' ? 'Email and password' : prov(x.provider), n: x.n })), 'No sign-ins in this period.'),
        h('section', { class: 'card section hb', 'aria-label': 'Checkout funnel' }, h('h3', {}, 'Pricing to paid'),
          h('ul', { class: 'hbars' }, funnel.map(([l, n, sub], i) => h('li', {}, h('span', { class: 'hl' }, l, h('small', {}, sub)), h('span', { class: 'hv' }, num(n)),
            h('span', { class: 'track', 'aria-hidden': 'true' }, h('span', { class: 'fillbar' + (i === 2 ? ' good' : ''), 'data-w': String(Math.round((n / fmax) * 100)) }))))),
          paidPlans ? h('p', { class: 'hint' }, `Paid: ${paidPlans}`) : null,
          pv.from.length ? h('p', { class: 'hint' }, 'Came to pricing from: ', pv.from.map((x) => `${x.item} (${x.n})`).join(', ')) : null)),
      h('div', { class: 'grid2' },
        hbars('Top viewed components and screens', r.top.map((x) => ({ label: x.item, href: x.path, sub: `${cap(x.kind)} · ${num(x.visitors)} visitors`, n: x.views })), 'No views of component or screen pages yet.'),
        hbars('Active subscriptions by plan', [{ label: 'Pro', n: r.subs.pro }, { label: 'Founder', n: r.subs.founder }, { label: 'Team', n: r.subs.team }, { label: 'Grants', n: r.subs.grants, sub: 'no payment' }], 'No subscriptions yet.')),
      h('section', { class: 'card section', 'aria-labelledby': 'pc-t' }, h('h3', { id: 'pc-t' }, 'Pro copy attempts'),
        h('p', { class: 'muted small' }, 'Someone without a plan tried to copy Pro code (Copy button, Ctrl/⌘+C or the CLI command) and was sent to pricing. The strongest sign of what people want to buy.'),
        proCopy),
      h('p', { class: 'hint pad0' }, `Updated ${fmtDate(r.generated_at)}. Events are kept ${r.retention_days} days; accounts count as active when they sign in or browse while signed in.`));
    // Widths after insertion (no inline style attributes from the API; CSP has no unsafe-inline).
    body.querySelectorAll('.fillbar[data-w]').forEach((b) => { b.style.width = b.dataset.w + '%'; });
  }

  // ── Boot ───────────────────────────────────────────────────────────────────────────
  document.addEventListener('DOMContentLoaded', () => {
    syncThemeBtn();
    $('#theme').addEventListener('click', () => {
      const t = root.dataset.theme === 'dark' ? 'light' : 'dark';
      root.dataset.theme = t; localStorage.setItem('starlight-theme', t); syncThemeBtn();
    });
    $('#logout').addEventListener('click', async () => {
      try { await fetch('/api/auth/logout', { method: 'POST', credentials: 'same-origin', headers: { 'content-type': 'application/json' }, body: '{}' }); } catch { /* ignore */ }
      location.assign('/login/');
    });
    const tabs = [...document.querySelectorAll('#tabs [role=tab]')];
    tabs.forEach((b) => b.addEventListener('click', () => go(b.dataset.tab)));
    $('#tabs').addEventListener('keydown', (e) => {
      const i = tabs.indexOf(document.activeElement); if (i < 0) return;
      const n = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0;
      if (e.key === 'Home' || e.key === 'End') { e.preventDefault(); const t = tabs[e.key === 'Home' ? 0 : tabs.length - 1]; t.focus(); t.click(); return; }
      if (!n) return;
      e.preventDefault(); const t = tabs[(i + n + tabs.length) % tabs.length]; t.focus(); t.click();
    });
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && state.tab === 'inbox' && state.sel && !$('#confirm').open) go('inbox');
      if (e.key === 'Escape' && state.tab === 'users' && /^#users\//.test(location.hash) && !document.querySelector('dialog[open]')) go('users');
    });
    window.addEventListener('hashchange', () => { if (me && me.signed_in) route(); });
    start();
  });
})();
