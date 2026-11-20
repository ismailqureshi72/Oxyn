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
  const state = { tab: 'inbox', kind: '', status: 'new', q: '', sel: null, items: [], next: null };

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
  const TABS = ['inbox', 'admins', 'settings', 'audit'];
  function route() {
    const [tab, id] = location.hash.replace(/^#/, '').split('/');
    const prevTab = state.tab;
    state.tab = TABS.includes(tab) ? tab : 'inbox';
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
    ({ inbox: renderInbox, admins: renderAdmins, settings: renderSettings, audit: renderAudit })[state.tab]();
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
        const msg = self ? 'You are removing yourself. You will lose access to the admin panel right away.' : `${a.email} will no longer be able to use the admin panel.`;
        if (!(await confirmDlg(`Remove ${a.email}?`, msg, 'Remove'))) return;
        try { const x = await api('/admins/' + encodeURIComponent(a.email), { method: 'DELETE' }); toast('Removed.'); if (x.self) location.reload(); else renderAdmins(); }
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
    });
    window.addEventListener('hashchange', () => { if (me && me.signed_in) route(); });
    start();
  });
})();
