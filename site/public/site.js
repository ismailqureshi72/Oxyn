// Site behaviour: theme toggle, ⌘K search (Pagefind), copy buttons, menus, announcement, TOC.
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

  // ⌘K search.
  const dlg = document.getElementById('search');
  if (!dlg) return;
  const input = dlg.querySelector('input');
  const res = dlg.querySelector('.search-res');
  let pf = null;
  let type = '';
  let sel = 0;
  let lastFocus = null;
  const loadPf = async () => {
    if (pf) return pf;
    try {
      pf = await import('/pagefind/pagefind.js');
      await pf.init();
    } catch {
      pf = false;
    }
    return pf;
  };
  const open = () => {
    lastFocus = document.activeElement;
    dlg.hidden = false;
    dlg.classList.add('open');
    input.focus();
    input.select();
    loadPf();
    if (!input.value) render([]);
  };
  const close = () => {
    dlg.classList.remove('open');
    dlg.hidden = true;
    lastFocus?.focus?.();
  };
  const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
  const render = (items, q) => {
    sel = 0;
    if (!items.length) {
      res.innerHTML = `<div class="search-empty">${
        q ? `No results for “${esc(q)}”.` : 'Type to search components, agent states, props and docs.'
      }${pf === false ? '<br>The search index is built with the site (npm run build).' : ''}</div>`;
      return;
    }
    const groups = {};
    for (const it of items) (groups[it.meta?.type || 'Page'] ??= []).push(it);
    const order = ['Component', 'Category', 'Docs', 'Page'];
    const label = { Component: 'Components', Category: 'Categories', Docs: 'Docs & guides', Page: 'Pages' };
    let i = 0;
    res.innerHTML = order
      .filter((g) => groups[g])
      .map(
        (g) =>
          `<div class="grp">${label[g]}</div>` +
          groups[g]
            .map(
              (it) =>
                `<a href="${it.url}" role="option" id="sr-${i}" aria-selected="${i++ === 0}"><b>${esc(
                  it.meta?.title || it.url,
                )}<span>${esc(it.meta?.category || '')}</span></b><small>${it.excerpt || ''}</small></a>`,
            )
            .join(''),
      )
      .join('');
  };
  let timer;
  const search = async () => {
    const q = input.value.trim();
    const p = await loadPf();
    if (!q || !p) return render([], q);
    const r = await p.search(q, type ? { filters: { type } } : {});
    const data = await Promise.all(r.results.slice(0, 12).map((x) => x.data()));
    render(data, q);
  };
  input.addEventListener('input', () => {
    clearTimeout(timer);
    timer = setTimeout(search, 120);
  });
  dlg.querySelectorAll('.search-filters button').forEach((b) =>
    b.addEventListener('click', () => {
      dlg.querySelectorAll('.search-filters button').forEach((x) => x.setAttribute('aria-pressed', String(x === b)));
      type = b.dataset.type;
      search();
      input.focus();
    }),
  );
  const move = (dir) => {
    const opts = [...res.querySelectorAll('a')];
    if (!opts.length) return;
    sel = (sel + dir + opts.length) % opts.length;
    opts.forEach((o, i) => o.setAttribute('aria-selected', String(i === sel)));
    opts[sel].scrollIntoView({ block: 'nearest' });
  };
  dlg.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') { e.preventDefault(); close(); }
    else if (e.key === 'ArrowDown') { e.preventDefault(); move(1); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); move(-1); }
    else if (e.key === 'Enter') {
      const a = res.querySelectorAll('a')[sel];
      if (a) { e.preventDefault(); location.href = a.href; }
    }
  });
  dlg.addEventListener('click', (e) => { if (e.target === dlg) close(); });
  document.addEventListener('click', (e) => { if (e.target.closest('[data-open-search]')) open(); });
  document.addEventListener('keydown', (e) => {
    const a = document.activeElement;
    // Custom editors (role=textbox, e.g. the rich-text editor) take '/' as text too.
    const typing = /INPUT|TEXTAREA|SELECT/.test(a?.tagName) || a?.isContentEditable || a?.getAttribute('role') === 'textbox';
    if ((e.key === 'k' && (e.metaKey || e.ctrlKey)) || (e.key === '/' && !typing)) {
      e.preventDefault();
      dlg.classList.contains('open') ? close() : open();
    }
  });
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
    if (!m || stage.closest('.site-card, .lp-screen')) return;
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

  // Before/after compare.
  document.querySelectorAll('.x-cmp').forEach((c) => {
    const r = c.querySelector('.cmp-range');
    const set = () => c.style.setProperty('--pos', `${r.value}%`);
    r?.addEventListener('input', set);
  });
})();
