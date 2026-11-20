// Site behaviour: theme toggle, ⌘K search palette, copy buttons, menus, announcement, TOC.
(function () {
  const d = document.documentElement;
  const isMac = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
  document.querySelectorAll('[data-mod-k]').forEach((k) => (k.textContent = isMac ? '⌘K' : 'Ctrl K'));

  // Theme: one switch for the site and the Starlight docs (same storage key as Starlight).
  // Radio pairs: [data-theme-scope="page"] sets the site; "preview" sets only the preview below it.
  const syncRadios = (root, t) => root.querySelectorAll('[data-set-theme]').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.setTheme === t)));
  const setTheme = (t) => {
    d.dataset.theme = t;
    d.setAttribute('data-oxyn-theme', t);
    localStorage.setItem('starlight-theme', t);
    document.querySelectorAll('[data-theme-scope="page"]').forEach((g) => syncRadios(g, t));
    document.querySelectorAll('[data-theme-scope="preview"]').forEach((g) => {
      const pv = g.closest('.pvwrap')?.querySelector('.pv');
      if (pv && !pv.dataset.themeSet) syncRadios(g, t);
    });
  };
  setTheme(d.dataset.theme === 'dark' ? 'dark' : 'light');
  document.addEventListener('click', (e) => {
    const b = e.target.closest('[data-set-theme]');
    if (b) {
      const g = b.closest('[data-theme-scope]');
      const t = b.dataset.setTheme;
      if (g?.dataset.themeScope === 'preview') {
        const pv = g.closest('.pvwrap')?.querySelector('.pv');
        if (pv) { pv.dataset.theme = t; pv.setAttribute('data-oxyn-theme', t); pv.dataset.themeSet = '1'; }
        syncRadios(g, t);
      } else setTheme(t);
      return;
    }
    if (e.target.closest('[data-theme-toggle]')) setTheme(d.dataset.theme === 'light' ? 'dark' : 'light');
  });
  // Arrow keys move within a radio pair.
  document.addEventListener('keydown', (e) => {
    const b = e.target.closest?.('[data-set-theme]');
    if (!b || !/Arrow(Left|Right|Up|Down)/.test(e.key)) return;
    e.preventDefault();
    const sib = [...b.parentElement.querySelectorAll('[data-set-theme]')].find((x) => x !== b);
    sib.focus();
    sib.click();
  });

  // Announcement bar.
  document.addEventListener('click', (e) => {
    if (e.target.closest('[data-close-announce]')) {
      d.dataset.announce = 'hidden';
      localStorage.setItem('oxyn-announce', 'closed');
    }
  });

  // Reduce motion (site switch; only ever reduces).
  document.querySelectorAll('[data-reduce-motion]').forEach((c) => {
    c.checked = d.hasAttribute('data-oxyn-reduced');
    c.addEventListener('change', () => {
      if (c.checked) d.setAttribute('data-oxyn-reduced', ''); else d.removeAttribute('data-oxyn-reduced');
      localStorage.setItem('oxyn-reduce-motion', c.checked ? '1' : '0');
    });
  });

  // Mobile menu.
  document.addEventListener('click', (e) => {
    const b = e.target.closest('[data-menu-toggle]');
    if (!b) return;
    const m = document.getElementById('mobilemenu');
    const open = !m.classList.contains('open');
    m.classList.toggle('open', open);
    b.setAttribute('aria-expanded', String(open));
  });

  // Copy buttons: [data-copy="text"] or [data-copy-from="next"] (the following <pre>).
  document.addEventListener('click', async (e) => {
    const b = e.target.closest('[data-copy],[data-copy-from]');
    if (!b) return;
    let text = b.dataset.copy;
    if (!text && b.dataset.copyFrom === 'pv') {
      text = b.closest('.pvwrap')?.querySelector('.oxd-code pre')?.innerText ?? '';
    } else if (!text) {
      const host = b.closest('.codeblk, .oxd-code, .copyable') || b.parentElement.parentElement;
      text = host.querySelector('pre')?.innerText ?? host.querySelector('code')?.innerText ?? '';
    }
    try {
      await navigator.clipboard.writeText(text);
      // Component copy buttons show their own feedback; only the site's text buttons swap label.
      if (b.matches('.cc-copy, .pill-copy')) {
        b.classList.add('done');
        const lbl = b.getAttribute('aria-label');
        b.setAttribute('aria-label', 'Copied');
        setTimeout(() => { b.classList.remove('done'); b.setAttribute('aria-label', lbl); }, 1400);
        return;
      }
      if (!b.matches('.oxd-copy, .copy-btn, .pill-black')) return;
      const old = b.textContent;
      b.textContent = 'Copied';
      setTimeout(() => (b.textContent = old), 1400);
    } catch {}
  });

  // Table of contents highlight.
  const toc = document.querySelectorAll('.toc a');
  if (toc.length && 'IntersectionObserver' in window) {
    const map = new Map([...toc].map((a) => [a.getAttribute('href').slice(1), a]));
    const io = new IntersectionObserver((entries) => {
      for (const en of entries) {
        if (en.isIntersecting) {
          toc.forEach((a) => a.classList.remove('on'));
          map.get(en.target.id)?.classList.add('on');
        }
      }
    }, { rootMargin: '-80px 0px -70% 0px' });
    map.forEach((_, id) => { const el = document.getElementById(id); if (el) io.observe(el); });
  }

  // ⌘K search: a palette over /search-index.json (every component, screen, agent app, docs page),
  // written at build time. Matching is client-side: exact > prefix > word start > substring >
  // typo-tolerant, so "appr", "lanes" and "chekpoint" all find their part.
  const dlg = document.getElementById('search');
  if (!dlg) return;
  const input = dlg.querySelector('input');
  const res = dlg.querySelector('.search-res');
  const count = dlg.querySelector('[data-search-count]');
  const GROUPS = { c: 'Components', s: 'Screens', a: 'Agent apps', d: 'Docs', p: 'Pages' };
  const BOOST = { c: 40, s: 25, a: 20, d: 0, p: -15 };
  const CAP = { c: 8, s: 5, a: 5, d: 5, p: 4 };
  const SUGGEST = ['Approval Card', 'Checkpoint lanes', 'Chat Composer', 'Agent Trail', 'Agent workspace', 'Installation'];
  const RKEY = 'oxyn-recent-search';
  let items = null;
  let loading = null;
  let flat = [];
  let sel = 0;
  let lastFocus = null;
  const norm = (s) => s.toLowerCase().replace(/[^a-z0-9]+/g, ' ').trim();
  const load = () =>
    (loading ??= fetch('/search-index.json')
      .then((r) => (r.ok ? r.json() : Promise.reject(r.status)))
      .then((j) => {
        items = j.items.map((it) => {
          const t = norm(it.t);
          return { ...it, n: t, tw: t.split(' '), kw: norm(it.k + ' ' + it.w).split(' '), dn: norm(it.d) };
        });
        if (count) count.textContent = `${items.length} items`;
        return items;
      })
      .catch(() => { loading = null; items = null; return null; }));
  // Damerau-Levenshtein, stopped early past `max`.
  const dist = (a, b, max) => {
    if (Math.abs(a.length - b.length) > max) return max + 1;
    let p2 = null, p = [...Array(b.length + 1).keys()];
    for (let i = 1; i <= a.length; i++) {
      const c = [i];
      let lo = i;
      for (let j = 1; j <= b.length; j++) {
        let v = Math.min(p[j] + 1, c[j - 1] + 1, p[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
        if (p2 && i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1]) v = Math.min(v, p2[j - 2] + 1);
        c[j] = v;
        if (v < lo) lo = v;
      }
      if (lo > max) return max + 1;
      p2 = p; p = c;
    }
    return p[b.length];
  };
  const allow = (t) => (t.length >= 8 ? 2 : t.length >= 4 ? 1 : 0);
  // How well one query token matches a list of words: 4 exact, 3 prefix, 2 inside a word, 1 typo
  // (a typo match needs a word at least as long as the token, so "appr" never lands on "app").
  const tokenHit = (t, ws) => {
    let best = 0, at = -1, fz = 9;
    ws.forEach((w, i) => {
      let s = 0;
      if (w === t) s = 4;
      else if (w.startsWith(t)) s = 3;
      else if (t.length >= 3 && w.includes(t)) s = 2;
      else if (allow(t) && w.length >= t.length) {
        const d = Math.min(dist(t, w, allow(t)), t.length < w.length ? dist(t, w.slice(0, t.length), allow(t)) : 9);
        if (d <= allow(t)) { s = 1; if (d < fz) fz = d; }
      }
      if (s > best) { best = s; at = i; }
    });
    return [best, at, fz];
  };
  const score = (it, q, qt) => {
    let s = 0;
    const hits = [];
    if (it.n === q) s = 1000;
    else if (it.n.startsWith(q)) s = 850;
    else if (q.length >= 2 && it.tw.some((w, i) => i && (it.tw.slice(i).join(' ')).startsWith(q))) s = 700;
    else if (q.length >= 3 && it.n.includes(q)) s = 560;
    if (s) {
      for (const t of qt) hits.push(t);
    } else {
      // Every token must land in the name or the keywords (typos allowed); names weigh more.
      let sum = 0;
      for (const t of qt) {
        const [a, at] = tokenHit(t, it.tw);
        const [b] = a >= 3 ? [0] : tokenHit(t, it.kw);
        if (!a && !b) {
          if (t.length >= 4 && it.dn.includes(t)) { sum += 40; continue; }
          return null;
        }
        if (a >= b) { sum += [0, 230, 330, 420, 480][a]; hits.push(a === 1 ? it.tw[at] : t); }
        else sum += [0, 120, 170, 260, 300][b];
      }
      s = sum / qt.length;
    }
    return { it, s: s + BOOST[it.g] - Math.min(it.t.length, 40) * 0.6, hits };
  };
  const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
  // Bold the matched characters of the name: each query token (or the word it fuzzily matched).
  const mark = (title, hits) => {
    const lo = title.toLowerCase();
    const on = new Array(title.length).fill(false);
    for (const h of hits) {
      if (!h) continue;
      let i = -1;
      const re = new RegExp(`(^|[^a-z0-9])${h.replace(/[^a-z0-9]/g, '')}`, 'g');
      const m = re.exec(lo);
      if (m) i = m.index + m[1].length;
      else i = lo.indexOf(h);
      if (i >= 0) for (let k = i; k < i + h.length && k < on.length; k++) on[k] = true;
    }
    let out = '', open = false;
    for (let k = 0; k < title.length; k++) {
      if (on[k] && !open) { out += '<mark>'; open = true; }
      if (!on[k] && open) { out += '</mark>'; open = false; }
      out += esc(title[k]);
    }
    return out + (open ? '</mark>' : '');
  };
  const ICON = {
    c: '<path d="M4 5h7v7H4zM13 5h7v4h-7zM13 11h7v8h-7zM4 14h7v5H4z"/>',
    s: '<path d="M3 5h18v13H3zM3 9h18M7 7h.01"/>',
    a: '<path d="M4 5h16v12H4zM9 21h6M12 17v4M8 10l2 2-2 2M13 14h3"/>',
    d: '<path d="M6 3h9l4 4v14H6zM14 3v5h5M9 13h7M9 17h5"/>',
    p: '<path d="M4 4h16v16H4zM4 9h16"/>',
    r: '<path d="M12 7v5l3 2M21 12a9 9 0 1 1-3-6.7M21 4v5h-5"/>',
  };
  const icon = (g) => `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${ICON[g]}</svg>`;
  const recent = () => { try { return JSON.parse(localStorage.getItem(RKEY) || '[]'); } catch { return []; } };
  const remember = (it, q) => {
    const r = recent().filter((x) => x.u !== it.u);
    r.unshift({ t: it.t, u: it.u, g: it.g, k: it.k, q });
    try { localStorage.setItem(RKEY, JSON.stringify(r.slice(0, 5))); } catch {}
  };
  const row = (it, i, title, ic = it.g) =>
    `<a href="${esc(it.u)}" role="option" id="sr-${i}" data-i="${i}" aria-selected="false" tabindex="-1"><span class="sr-ic">${icon(ic)}</span><span class="sr-tx"><b>${title}</b>${it.d ? `<small>${esc(it.d)}</small>` : ''}</span><span class="sr-k">${esc(it.k || GROUPS[it.g] || '')}</span></a>`;
  const section = (label, id, html) => `<div role="group" aria-labelledby="${id}"><div class="grp" id="${id}" role="presentation">${label}</div>${html}</div>`;
  const select = (i) => {
    const opts = res.querySelectorAll('[role=option]');
    if (!opts.length) { input.removeAttribute('aria-activedescendant'); return; }
    sel = (i + opts.length) % opts.length;
    opts.forEach((o, k) => o.setAttribute('aria-selected', String(k === sel)));
    input.setAttribute('aria-activedescendant', opts[sel].id);
    opts[sel].scrollIntoView({ block: 'nearest' });
  };
  const suggestChips = () =>
    `<div class="sr-chips">${SUGGEST.map((s) => `<button type="button" data-q="${esc(s)}">${esc(s)}</button>`).join('')}</div>`;
  const render = () => {
    const raw = input.value.trim();
    const q = norm(raw);
    flat = [];
    let html = '';
    if (!q) {
      const r = recent();
      if (r.length) {
        html += section('Recent', 'sg-r', r.map((it, i) => (flat.push({ it, q: it.q }), row(it, i, esc(it.t), 'r'))).join(''));
      }
      if (items) {
        const pick = SUGGEST.map((s) => items.find((it) => it.t === s)).filter(Boolean).filter((it) => !r.some((x) => x.u === it.u));
        html += section('Suggestions', 'sg-s', pick.map((it) => { const i = flat.length; flat.push({ it, q: '' }); return row(it, i, esc(it.t)); }).join(''));
      } else html += `<div class="search-empty">Loading the index…</div>`;
    } else if (!items) {
      html = `<div class="search-empty">${loading ? 'Loading the index…' : 'Search could not load. Check your connection and try again.'}</div>`;
    } else {
      const qt = q.split(' ');
      const found = [];
      for (const it of items) { const m = score(it, q, qt); if (m) found.push(m); }
      found.sort((a, b) => b.s - a.s);
      const by = {};
      for (const m of found) { const l = (by[m.it.g] ??= []); if (l.length < CAP[m.it.g]) l.push(m); }
      const order = Object.keys(by).sort((a, b) => by[b][0].s - by[a][0].s);
      if (!order.length) {
        html = `<div class="search-empty"><p>No results for “${esc(raw)}”.</p><p class="sr-try">Try one of these</p>${suggestChips()}</div>`;
      } else {
        html = order
          .map((g) => section(GROUPS[g], `sg-${g}`, by[g].map((m) => { const i = flat.length; flat.push({ it: m.it, q: raw }); return row(m.it, i, mark(m.it.t, m.hits)); }).join('')))
          .join('');
      }
    }
    res.innerHTML = html;
    dlg.querySelector('.sr-live').textContent = q ? (flat.length ? `${flat.length} results` : 'No results') : '';
    input.setAttribute('aria-expanded', String(flat.length > 0));
    select(0);
  };
  const go = (i, newTab) => {
    const f = flat[i];
    if (!f) return;
    remember(f.it, f.q);
    if (newTab) window.open(f.it.u, '_blank');
    else { close(); location.href = f.it.u; }
  };
  const open = () => {
    lastFocus = document.activeElement;
    dlg.hidden = false;
    dlg.classList.add('open');
    document.documentElement.classList.add('search-on');
    input.focus();
    input.select();
    render();
    load().then(render);
  };
  const close = () => {
    dlg.classList.remove('open');
    dlg.hidden = true;
    document.documentElement.classList.remove('search-on');
    input.setAttribute('aria-expanded', 'false');
    lastFocus?.focus?.();
  };
  input.addEventListener('input', render);
  res.addEventListener('click', (e) => {
    const a = e.target.closest('[role=option]');
    if (a && !e.metaKey && !e.ctrlKey && !e.shiftKey) { e.preventDefault(); go(+a.dataset.i); }
    else if (a) remember(flat[+a.dataset.i].it, input.value.trim());
    const c = e.target.closest('[data-q]');
    if (c) { input.value = c.dataset.q; input.focus(); render(); }
  });
  res.addEventListener('mousemove', (e) => {
    const a = e.target.closest('[role=option]');
    if (a && +a.dataset.i !== sel) select(+a.dataset.i);
  });
  dlg.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') { e.preventDefault(); close(); }
    else if (e.key === 'ArrowDown') { e.preventDefault(); select(sel + 1); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); select(sel - 1); }
    else if (e.key === 'Home' && e.ctrlKey) { e.preventDefault(); select(0); }
    else if (e.key === 'End' && e.ctrlKey) { e.preventDefault(); select(-1); }
    else if (e.key === 'Enter' && flat.length) { e.preventDefault(); go(sel, e.metaKey || e.ctrlKey); }
    else if (e.key === 'Tab') { e.preventDefault(); input.focus(); }
  });
  dlg.addEventListener('click', (e) => { if (e.target === dlg || e.target.closest('[data-search-close]')) close(); });
  document.addEventListener('click', (e) => { if (e.target.closest('[data-open-search]')) open(); });
  // Warm the index once the page is idle, so the first keystroke already has it.
  if ('requestIdleCallback' in window) requestIdleCallback(() => load(), { timeout: 4000 });
  else setTimeout(load, 1500);
  // Capture phase on window: a live demo that also binds ⌘K (a command menu) cannot swallow it.
  window.addEventListener('keydown', (e) => {
    const a = document.activeElement;
    // Custom editors (role=textbox, e.g. the rich-text editor) take '/' as text too.
    const typing = /INPUT|TEXTAREA|SELECT/.test(a?.tagName) || a?.isContentEditable || a?.getAttribute('role') === 'textbox';
    if ((e.key.toLowerCase() === 'k' && (e.metaKey || e.ctrlKey)) || (e.key === '/' && !typing)) {
      if (e.key === '/' && dlg.classList.contains('open')) return;
      e.preventDefault();
      e.stopPropagation();
      dlg.classList.contains('open') ? close() : open();
    }
  }, true);
})();
document.addEventListener('oxyn-demo-live', (e) => {
  if (e.target.dataset.controls !== 'true') return;
  const l = document.querySelector('[data-live-label]');
  if (l) { l.textContent = 'Live · Dioxus → WASM'; l.classList.add('on'); }
});

(() => {
  // Fit wide demos into their stage: zoom down (never up, never below 42%) so a preview never
  // needs a scrollbar just because the column is narrower than the component.
  const fitStage = (stage) => {
    const m = stage.querySelector(':scope > .oxd-mount');
    if (!m || stage.closest('.site-card, .lp-screen, .x-lap-scr')) return;
    m.style.zoom = '';
    const cs = getComputedStyle(stage);
    const avail = stage.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight);
    const need = Math.max(m.scrollWidth, ...[...m.children].map((c) => c.scrollWidth));
    if (avail > 0 && need > avail + 1) m.style.zoom = String(Math.max(0.42, Math.floor((avail / need) * 1000) / 1000));
  };
  if ('ResizeObserver' in window) {
    const ro = new ResizeObserver((es) => es.forEach((e) => fitStage(e.target.closest('.oxd-stage') || e.target)));
    const watch = (root) => root.querySelectorAll('.oxd-stage').forEach((s) => {
      if (s.dataset.fit) return;
      s.dataset.fit = '1';
      ro.observe(s);
      fitStage(s);
    });
    watch(document);
    new MutationObserver((ms) => ms.forEach((m) => {
      const d = m.target.closest && m.target.closest('oxyn-demo');
      if (d) requestAnimationFrame(() => { d.querySelectorAll('.oxd-stage').forEach((s) => { delete s.dataset.fit; }); watch(d); });
    })).observe(document.body, { subtree: true, attributes: true, attributeFilter: ['data-state'] });
  }
})();

(() => {
  // Keep demo menus and popovers on screen: anything floating that opens past the viewport
  // edge (a phone, a demo near the edge) is shifted back inside, never resized. Works through
  // the stage zoom (shift is divided by the element's effective zoom).
  const FLOAT = '.oxyn-popover,.oxyn-pop,.oxyn-menu,.oxk-menu,.oxk-mmenu,.oxyn-drp .panel,.hc-card,.oxp-aa-menu,.oxf-mssurf,.oxyn-ctx-pop,.oxyn-ef-menu,.oxyn-xsearch .panel,.um-pnl,.ws-menu,[role=menu],[role=listbox]:not(.oxyn-layer *):not([class*="list"]),[role=tooltip]';
  const PAD = 8;
  let queued = false, mo = null;
  const place = (el) => {
    if (el.dataset.kiv) { el.style.translate = el.dataset.kivOrig || ''; delete el.dataset.kiv; delete el.dataset.kivOrig; }
    const cs = getComputedStyle(el);
    if (cs.position !== 'absolute' && cs.position !== 'fixed') return;
    const r = el.getBoundingClientRect();
    if (!r.width || !r.height) return;
    const vw = document.documentElement.clientWidth;
    let dx = 0;
    if (r.width > vw - 2 * PAD || r.left < PAD) dx = PAD - r.left;
    else if (r.right > vw - PAD) dx = vw - PAD - r.right;
    if (Math.abs(dx) < 1) return;
    const z = el.currentCSSZoom || 1;
    const t = cs.translate && cs.translate !== 'none' ? cs.translate.split(' ') : null;
    el.dataset.kivOrig = el.style.translate || '';
    el.dataset.kiv = '1';
    el.style.translate = t ? `calc(${t[0]} + ${dx / z}px) ${t[1] || '0px'}` : `${dx / z}px 0px`;
  };
  const run = () => {
    queued = false;
    document.querySelectorAll('oxyn-demo').forEach((d) => d.querySelectorAll(FLOAT).forEach(place));
    if (mo) mo.takeRecords();
  };
  const queue = () => { if (!queued) { queued = true; requestAnimationFrame(run); } };
  mo = new MutationObserver((ms) => { if (ms.some((m) => m.target.closest && m.target.closest('oxyn-demo'))) { queue(); setTimeout(queue, 260); } });
  mo.observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ['class', 'style', 'open', 'data-state', 'aria-expanded'] });
  addEventListener('resize', queue);
})();

// ── Plan #29: library shell, landing tabs, preview toolbar, compare ──────────────────────────
(function () {
  const d = document.documentElement;
  // Library dropdown in the top bar: click or keyboard opens, outside click and Escape close.
  const dds = document.querySelectorAll('.tb-dd');
  // Keep the menu inside the viewport: shift it left when it would run past the right edge.
  const fit = (dd) => {
    const m = dd.querySelector('.tb-menu');
    if (!m) return;
    m.style.left = '';
    const r = m.getBoundingClientRect();
    const over = r.right - (document.documentElement.clientWidth - 12);
    if (over > 0) m.style.left = `${parseFloat(getComputedStyle(m).left) - over}px`;
    if (r.left - Math.max(over, 0) < 12) m.style.left = `${parseFloat(getComputedStyle(m).left) + (12 - (r.left - Math.max(over, 0)))}px`;
  };
  const closeDd = (dd) => { dd.classList.remove('open'); dd.querySelector('[data-dd]')?.setAttribute('aria-expanded', 'false'); };
  dds.forEach((dd) => {
    const btn = dd.querySelector('[data-dd]');
    btn?.addEventListener('click', () => {
      // A menu the pointer opened on hover stays open on the click that follows.
      if (dd.dataset.hoverOpen) { delete dd.dataset.hoverOpen; return; }
      const open = !dd.classList.contains('open');
      dd.classList.toggle('open', open);
      btn.setAttribute('aria-expanded', String(open));
      if (open) fit(dd);
    });
    dd.addEventListener('mouseenter', () => { if (matchMedia('(hover:hover)').matches && !dd.classList.contains('open')) { dd.classList.add('open'); dd.dataset.hoverOpen = '1'; btn.setAttribute('aria-expanded', 'true'); fit(dd); } });
    dd.addEventListener('mouseleave', () => { if (matchMedia('(hover:hover)').matches) { delete dd.dataset.hoverOpen; closeDd(dd); } });
    dd.addEventListener('focusout', (e) => { if (!dd.contains(e.relatedTarget)) closeDd(dd); });
  });
  document.addEventListener('click', (e) => dds.forEach((dd) => { if (!dd.contains(e.target)) closeDd(dd); }));
  document.addEventListener('keydown', (e) => {
    if (e.key !== 'Escape') return;
    dds.forEach((dd) => { if (dd.classList.contains('open')) { closeDd(dd); dd.querySelector('[data-dd]')?.focus(); } });
    if (d.classList.contains('drawer-open')) setDrawer(false);
  });

  // Sidebar: hide/show on desktop (remembered), a drawer on phones.
  document.addEventListener('click', (e) => {
    if (e.target.closest('[data-sidebar-toggle]') && !matchMedia('(max-width:900px)').matches) {
      const hidden = d.dataset.sidebar !== 'hidden';
      if (hidden) d.dataset.sidebar = 'hidden'; else delete d.dataset.sidebar;
      localStorage.setItem('oxyn-sidebar', hidden ? 'hidden' : 'shown');
      (hidden ? document.querySelector('.lbar-open') : document.querySelector('.lsb-hide'))?.focus();
    }
  });
  const setDrawer = (open) => {
    d.classList.toggle('drawer-open', open);
    document.querySelectorAll('[data-drawer-open]').forEach((b) => b.setAttribute('aria-expanded', String(open)));
    if (open) document.querySelector('.lsb-search')?.focus();
    else document.querySelector('[data-drawer-open]')?.focus();
  };
  document.addEventListener('click', (e) => {
    if (e.target.closest('[data-drawer-open]')) setDrawer(true);
    else if (e.target.closest('[data-drawer-close]')) setDrawer(false);
    else if (d.classList.contains('drawer-open') && e.target.closest('.lsb a')) d.classList.remove('drawer-open');
  });
  // Keep the current sidebar item in view.
  // Page-to-page crossfade (cross-document view transitions): skipped under reduced motion.
  addEventListener('pagereveal', (e) => {
    if (e.viewTransition && (d.hasAttribute('data-oxyn-reduced') || matchMedia('(prefers-reduced-motion: reduce)').matches)) e.viewTransition.skipTransition();
  });

  // Tabs (landing): roving tabindex, arrow keys, panels shown one at a time.
  document.querySelectorAll('[data-tabs]').forEach((box) => {
    const tabs = [...box.querySelectorAll(':scope > [role="tablist"] [role="tab"]')];
    const pick = (t, focus) => {
      tabs.forEach((x) => {
        const on = x === t;
        x.setAttribute('aria-selected', String(on));
        x.tabIndex = on ? 0 : -1;
        const p = document.getElementById(x.getAttribute('aria-controls'));
        if (p) p.hidden = !on;
      });
      if (focus) t.focus();
    };
    tabs.forEach((t, i) => {
      t.addEventListener('click', () => pick(t));
      t.addEventListener('keydown', (e) => {
        const n = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0;
        if (n) { e.preventDefault(); pick(tabs[(i + n + tabs.length) % tabs.length], true); }
        if (e.key === 'Home') { e.preventDefault(); pick(tabs[0], true); }
        if (e.key === 'End') { e.preventDefault(); pick(tabs[tabs.length - 1], true); }
      });
    });
  });

  // Preview toolbar: Preview/Code, controls fold-out, reset.
  document.querySelectorAll('.pvwrap').forEach((w) => {
    const pv = w.querySelector('.pv');
    const tabs = [...w.querySelectorAll('[data-pv]')];
    tabs.forEach((t, i) => {
      t.addEventListener('click', () => {
        tabs.forEach((x) => { x.setAttribute('aria-selected', String(x === t)); x.tabIndex = x === t ? 0 : -1; });
        pv.dataset.view = t.dataset.pv;
      });
      t.tabIndex = i === 0 ? 0 : -1;
      t.addEventListener('keydown', (e) => {
        if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') { e.preventDefault(); const o = tabs[1 - i]; o.focus(); o.click(); }
      });
    });
    w.querySelector('[data-pv-controls]')?.addEventListener('click', (e) => {
      const b = e.currentTarget;
      const show = pv.dataset.controls !== 'shown';
      pv.dataset.controls = show ? 'shown' : 'hidden';
      b.setAttribute('aria-expanded', String(show));
      b.setAttribute('aria-label', show ? 'Hide controls' : 'Show controls');
    });
    // Component style: Clean (one surface) or Classic (inset card); components read it from CSS.
    const cs = [...w.querySelectorAll('[data-cstyle]')].filter((b) => b.tagName === 'BUTTON');
    cs.forEach((b) => b.addEventListener('click', () => {
      cs.forEach((x) => x.setAttribute('aria-checked', String(x === b)));
      pv.dataset.cstyle = b.dataset.cstyle;
    }));
    w.querySelector('[data-pv-reset]')?.addEventListener('click', () => {
      const r = pv.querySelector('.oxd-reset');
      if (r) r.click();
      else { const demo = pv.querySelector('oxyn-demo'); if (demo) { demo.dataset.state = ''; demo.mountLive?.(); } }
    });
  });

  // Before/after compare: drag (the range input) or let it travel. When idle it sweeps from one
  // edge to the other and back (eased at each end), only while it is on screen. After a drag or a
  // key press it waits a moment, then carries on from wherever the user left it, in the direction
  // it was last moving, easing back up to speed (never jumping to a fixed spot). Moving it writes
  // three transforms (clip box, its content, the divider) inline, so the live demo inside is never
  // restyled by an inherited --pos per frame (--pos stays in the CSS as the no-script state). The
  // drawn wave and the masked edge flow together in one element (.cmp-ride); its counter-flow
  // (.cmp-anti, which keeps the content still) is pinned to the same start time whenever the
  // slider comes into view.
  const calm = () => d.hasAttribute('data-oxyn-reduced') || matchMedia('(prefers-reduced-motion: reduce)').matches;
  const ease = (s) => (1 - Math.cos(Math.PI * s)) / 2;
  const unease = (u) => Math.acos(1 - 2 * Math.max(0, Math.min(1, u))) / Math.PI;
  document.querySelectorAll('.x-cmp').forEach((c) => {
    const r = c.querySelector('.cmp-range');
    if (!r) return;
    const LEG = 5200, IDLE = 1600, RAMP = 700, EDGE = 22;
    let pos = 50, dir = 1, s = 0.5, last = 0, raf = 0, seen = false, held = false, touched = -1e9;
    const clip = c.querySelector('.cmp-clip'), after = c.querySelector('.cmp-after'), line = c.querySelector('.cmp-line');
    // The travel range: edge to edge, keeping the divider's handle just inside the box.
    const ends = () => { const w = c.clientWidth || 1; const m = Math.min(12, (EDGE / w) * 100); return [m, 100 - m]; };
    const put = (v) => {
      pos = v;
      const p = `${v.toFixed(2)}%`;
      if (clip && after && line) {
        clip.style.transform = `translateX(calc(${p} - 24px))`;
        after.style.transform = `translateX(calc(24px - ${p}))`;
        line.style.transform = `translateX(${p})`;
      } else c.style.setProperty('--pos', p);
      const iv = String(Math.round(v));
      if (r.value !== iv) r.value = iv;
    };
    // Progress along the current leg that gives the current position (so motion resumes there).
    const resumeFrom = () => { const [lo, hi] = ends(); const u = (pos - lo) / (hi - lo); s = unease(dir > 0 ? u : 1 - u); };
    const tick = (t) => {
      raf = 0;
      if (!seen || calm() || document.hidden) return;
      const quiet = t - touched;
      if (!held && quiet > IDLE) {
        if (last) {
          const k = Math.min(1, (quiet - IDLE) / RAMP);
          s += ((t - last) / LEG) * (k * k * (3 - 2 * k));
          if (s >= 1) { s = Math.min(1, s - 1); dir = -dir; }
        }
        const [lo, hi] = ends();
        put(dir > 0 ? lo + (hi - lo) * ease(s) : hi - (hi - lo) * ease(s));
      }
      last = t;
      raf = requestAnimationFrame(tick);
    };
    const run = () => { if (!raf && seen && !calm()) { last = 0; raf = requestAnimationFrame(tick); } };
    const sync = () => {
      if (!c.getAnimations) return;
      const t0 = document.timeline?.currentTime;
      if (t0 == null) return;
      c.getAnimations({ subtree: true }).forEach((a) => {
        if (a.animationName === 'cmp-ride' || a.animationName === 'cmp-anti') a.startTime = t0 - (t0 % 3200);
      });
    };
    const user = () => { touched = performance.now(); resumeFrom(); };
    r.addEventListener('input', () => {
      const v = Number(r.value);
      if (v !== pos) dir = v > pos ? 1 : -1;
      put(v); user();
    });
    r.addEventListener('pointerdown', () => { held = true; user(); });
    addEventListener('pointerup', () => { if (held) { held = false; user(); } });
    addEventListener('pointercancel', () => { if (held) { held = false; user(); } });
    r.addEventListener('keydown', user);
    if ('IntersectionObserver' in window) {
      new IntersectionObserver((es) => es.forEach((e) => { seen = e.isIntersecting; if (seen) { sync(); run(); } })).observe(c);
    }
    document.addEventListener('visibilitychange', run);
    put(Number(r.value) || 50);
    resumeFrom();
  });
})();

// ── Landing pass 2: laptop screens, condensing top bar ────────────────────────────────────────
(function () {
  const d = document.documentElement;
  const reduced = () => d.hasAttribute('data-oxyn-reduced') || matchMedia('(prefers-reduced-motion: reduce)').matches;
  // Laptop: tabs pick the screen on the lid (crossfade); the caption is the tab panel.
  document.querySelectorAll('[data-lap]').forEach((box) => {
    const tabs = [...box.querySelectorAll('.x-lap-tab')];
    const shots = [...box.querySelectorAll('.x-lap-shot')];
    const pick = (t, focus) => {
      tabs.forEach((x) => {
        const on = x === t;
        x.setAttribute('aria-selected', String(on));
        x.tabIndex = on ? 0 : -1;
        const p = document.getElementById(x.getAttribute('aria-controls'));
        if (p) p.hidden = !on;
      });
      shots.forEach((s) => {
        const on = s.dataset.shot === t.dataset.shot;
        s.toggleAttribute('data-on', on);
        s.inert = !on;
        // Lazy mount: a screen goes live the first time its tab is picked.
        const demo = on && s.querySelector('oxyn-demo[data-defer]');
        if (demo && demo.mountLive) demo.mountLive();
      });
      if (focus) t.focus();
    };
    tabs.forEach((t, i) => {
      t.addEventListener('click', () => pick(t));
      t.addEventListener('keydown', (e) => {
        const n = e.key === 'ArrowRight' || e.key === 'ArrowDown' ? 1 : e.key === 'ArrowLeft' || e.key === 'ArrowUp' ? -1 : 0;
        if (n) { e.preventDefault(); pick(tabs[(i + n + tabs.length) % tabs.length], true); }
        if (e.key === 'Home') { e.preventDefault(); pick(tabs[0], true); }
        if (e.key === 'End') { e.preventDefault(); pick(tabs[tabs.length - 1], true); }
      });
    });
    // The lid follows the scroll position across most of the section. On wide screens the scene is
    // pinned under the tab bar (CSS sticky) while the pin's extra height scrolls by: the lid starts
    // opening once the closed laptop is in view, keeps opening while it rises and while it is
    // pinned, and is fully open with a little of the pinned stretch left.
    // Smoothness: the scroll position only sets a target; one requestAnimationFrame loop eases the
    // shown progress toward it (frame-rate independent lerp) and stops when it arrives. Each frame
    // reads scrollY only (the section's page offsets are measured on load / resize, never in the
    // loop) and writes transform + opacity straight onto the three moving parts (dev, lid, shade),
    // so no inherited custom property restyles the live screen inside the lid every frame.
    // Phones (<= 640px) show the screen alone (no laptop body) and reduced motion shows it open.
    const scene = box.querySelector('.x-lap-scene');
    const pin = box.querySelector('.x-lap-pin') || scene;
    const dev = box.querySelector('.x-lap-dev');
    const lid = box.querySelector('.x-lap-lid');
    const shade = box.querySelector('.x-lap-shade');
    if (!scene || !dev || !lid) return;
    const phone = matchMedia('(max-width:640px)');
    let raf = 0, live = false, cur = -1, target = 1, last = 0;
    let a = 0, z = 0, pinTop = 0;
    const LIFT = 22;
    const still = () => reduced() || phone.matches;
    const paint = (p) => {
      const q = 1 - p;
      // Closed, the laptop is lifted into the middle of the scene (the open lid's space above it
      // would otherwise sit empty); it settles back down as the lid opens.
      dev.style.transform = `translate3d(0,${(q * q * -LIFT).toFixed(3)}cqw,0) scale(${(0.94 + p * 0.06).toFixed(4)})`;
      lid.style.transform = `perspective(340cqw) rotateX(${(q * -106).toFixed(3)}deg)`;
      if (shade) shade.style.opacity = (q * 0.8).toFixed(3);
      scene.toggleAttribute('data-closed', p < 0.02);
    };
    const clear = () => {
      dev.style.transform = lid.style.transform = '';
      if (shade) shade.style.opacity = '';
      scene.removeAttribute('data-closed');
      cur = -1;
    };
    // Page offsets of the pin and where opening starts (a) / ends (z), as pin-top viewport px.
    const measure = () => {
      if (still()) return;
      const vh = innerHeight;
      const sh = scene.offsetHeight;
      const stick = Math.max(0, pin.offsetHeight - sh);
      const cs = getComputedStyle(scene);
      // Not pinned (tablets): fully open once the scene is centred in the viewport.
      const top = cs.position === 'sticky' ? parseFloat(cs.top) || 0 : Math.max(16, (vh - sh) / 2);
      pinTop = pin.getBoundingClientRect().top + scrollY;
      z = top - stick * 0.82;
      // Where the fully closed laptop sits in the scene: opening starts only once all of it is in
      // view, with its middle at about 60% of the viewport (never later than its bottom edge
      // reaching 24 px above the viewport's bottom).
      paint(0);
      const sr = scene.getBoundingClientRect(), dr = dev.getBoundingClientRect(), lr = lid.getBoundingClientRect();
      const cb = dr.bottom - sr.top, cm = (lr.top + dr.bottom) / 2 - sr.top;
      if (cur >= 0) paint(cur); else clear();
      a = Math.max(z + 240, Math.min(vh - 24 - cb, vh * 0.6 - cm));
    };
    const aim = () => {
      const t = Math.min(1, Math.max(0, (a - (pinTop - scrollY)) / (a - z)));
      target = t * t * (3 - 2 * t);
    };
    const frame = (now) => {
      raf = 0;
      if (still()) { clear(); return; }
      aim();
      const dt = Math.min(64, last ? now - last : 16.7);
      last = now;
      if (cur < 0) cur = target;
      else cur += (target - cur) * (1 - Math.pow(1 - 0.16, dt / 16.7));
      if (Math.abs(target - cur) < 0.0008) cur = target;
      paint(cur);
      if (cur !== target) raf = requestAnimationFrame(frame);
      else last = 0;
    };
    const kick = () => { if (!raf) raf = requestAnimationFrame(frame); };
    const listen = (on) => {
      if (on === live) return;
      live = on;
      (on ? addEventListener : removeEventListener)('scroll', kick, { passive: true });
      kick();
    };
    const remeasure = () => { measure(); kick(); };
    addEventListener('resize', remeasure, { passive: true });
    phone.addEventListener?.('change', remeasure);
    // Content above the section can change height (fonts, live demos): keep the offsets current.
    if ('ResizeObserver' in window) new ResizeObserver(remeasure).observe(document.body);
    if ('IntersectionObserver' in window) new IntersectionObserver((es) => es.forEach((e) => listen(e.isIntersecting)), { rootMargin: '160px 0px' }).observe(pin);
    else listen(true);
    measure();
    aim();
    if (!still()) { cur = target; paint(cur); }
  });
  // Top bar on the white pages (headroom): slides away on scroll down, back on scroll up. It never
  // shows over the laptop section's sticky screen-tab bar: while that bar is stuck at the top (or
  // about to reach the top bar), the top bar stays away and the tab bar takes over. The band of
  // scroll positions where that holds is measured on load / resize (page offsets), so each frame
  // reads scrollY only and writes two attributes when they change.
  const tb = document.querySelector('.page-white .x-tb');
  if (tb) {
    let q = 0, lastY = scrollY, hidden = false, band = null;
    const menuOpen = () => document.getElementById('mobilemenu')?.classList.contains('open') || tb.querySelector('.tb-dd.open') || tb.contains(document.activeElement);
    const apply = (h) => { if (h !== hidden) { hidden = h; tb.toggleAttribute('data-hide', h); } };
    const lap = document.querySelector('.x-lap');
    const bar = lap?.querySelector('.x-lap-bar');
    const measureBand = () => {
      band = null;
      if (!bar || getComputedStyle(bar).position !== 'sticky') return;
      const y = scrollY;
      const lr = lap.getBoundingClientRect();
      const navH = tb.offsetHeight;
      // The bar is the section's first child, so its unstuck page offset is the section's.
      // From when the bar's top comes within the top bar's height (+ room for the slide-out) until
      // the section's end has scrolled the stuck bar out of the top of the viewport.
      band = [lr.top + y - navH - 96, lr.bottom + y + 8];
    };
    const set = () => {
      q = 0;
      const y = Math.max(0, scrollY);
      const dy = y - lastY;
      let h = hidden;
      if (y < 80) h = false;
      else if (dy > 6) h = true;
      else if (dy < -6) h = false;
      if (Math.abs(dy) > 6 || y < 80) lastY = y;
      if (band && y >= band[0] && y <= band[1] && y >= 80) h = true;
      if (menuOpen()) h = false;
      apply(h);
      tb.toggleAttribute('data-solid', y > 8);
    };
    const kick = () => { if (!q) q = requestAnimationFrame(set); };
    addEventListener('scroll', kick, { passive: true });
    const re = () => { measureBand(); kick(); };
    addEventListener('resize', re, { passive: true });
    if (lap && 'ResizeObserver' in window) new ResizeObserver(re).observe(document.body);
    tb.addEventListener('focusin', () => apply(false));
    measureBand();
    set();
  }
  // Brand intro (set in <head> on the first page of a visit): drop it once it has played.
  if (d.hasAttribute('data-intro')) setTimeout(() => d.removeAttribute('data-intro'), 1300);
  // Close the phone menu when one of its links is followed or Escape is pressed.
  const mm = document.getElementById('mobilemenu');
  const closeMm = () => {
    if (!mm?.classList.contains('open')) return;
    mm.classList.remove('open');
    const b = document.querySelector('[data-menu-toggle]');
    b?.setAttribute('aria-expanded', 'false');
    b?.focus();
  };
  mm?.addEventListener('click', (e) => { if (e.target.closest('a,[data-open-search]')) closeMm(); });
  document.addEventListener('keydown', (e) => { if (e.key === 'Escape') closeMm(); });
})();

// Full-screen view: "Back" returns to the page that opened it (same tab), else to the screen page.
document.querySelector('.fs-back')?.addEventListener('click', (e) => {
  try {
    if (document.referrer && new URL(document.referrer).origin === location.origin && history.length > 1) {
      e.preventDefault();
      history.back();
    }
  } catch {}
});

// Back buttons (a[data-x-back], components/v2/BackButton.astro): go back when the previous history entry
// is a page of this site, else follow the link (the logical parent route). With the Navigation API
// (Chromium) the same-site entries are known exactly and in-page #hash entries are skipped; elsewhere
// a same-origin referrer stands in for it.
(function () {
  const strip = (u) => String(u).split('#')[0];
  let refSame = false;
  try { refSame = !!document.referrer && new URL(document.referrer).origin === location.origin && strip(document.referrer) !== strip(location.href); } catch {}
  function goBack() {
    const nav = window.navigation;
    if (nav && typeof nav.entries === 'function' && nav.currentEntry) {
      const list = nav.entries();
      const here = strip(location.href);
      for (let i = nav.currentEntry.index - 1; i >= 0; i--) {
        const u = list[i] && list[i].url;
        if (!u) break;
        if (new URL(u).origin !== location.origin) break;
        if (strip(u) !== here) { nav.traverseTo(list[i].key); return true; }
      }
      return false;
    }
    if (refSame && history.length > 1) { history.back(); return true; }
    return false;
  }
  document.addEventListener('click', (e) => {
    const a = e.target.closest && e.target.closest('a[data-x-back]');
    if (!a || e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    try { if (goBack()) e.preventDefault(); } catch { /* follow the link */ }
  });
})();

// Privacy-friendly events for the admin Activity tab (POST /api/events, site/worker/events.ts):
// page views, Pro copy attempts, pricing views and checkout clicks. No cookies are set, nothing
// is sent with Do Not Track or Global Privacy Control on, and events go in small batches.
(function () {
  const off = navigator.doNotTrack === '1' || window.doNotTrack === '1' || navigator.globalPrivacyControl === true;
  const q = [];
  let timer = 0;
  function flush(beacon) {
    clearTimeout(timer);
    while (q.length) {
      const body = JSON.stringify({ events: q.splice(0, 20) });
      let sent = false;
      if (beacon && navigator.sendBeacon) {
        try { sent = navigator.sendBeacon('/api/events', new Blob([body], { type: 'text/plain;charset=UTF-8' })); } catch { sent = false; }
      }
      if (!sent) {
        try { fetch('/api/events', { method: 'POST', body, headers: { 'content-type': 'application/json' }, credentials: 'same-origin', keepalive: true }).catch(() => {}); } catch { /* offline */ }
      }
    }
  }
  /** oxynTrack(type, item?, detail?, now?): now = send at once (the page is about to change). */
  window.oxynTrack = function (type, item, detail, now) {
    if (off) return;
    const e = { type, path: location.pathname };
    if (item) e.item = String(item);
    if (detail) e.detail = String(detail);
    q.push(e);
    if (now || q.length >= 10) flush(true);
    else { clearTimeout(timer); timer = setTimeout(() => flush(false), 1000); }
  };
  addEventListener('pagehide', () => flush(true));
  document.addEventListener('visibilitychange', () => { if (document.visibilityState === 'hidden') flush(true); });
  if (!/^\/(admin|api)(\/|$)/.test(location.pathname)) window.oxynTrack('view');
})();
