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

  // Before/after compare: drag (the range input) or let it drift. When idle it eases back and forth
  // on its own (a sine around the middle, picked up from wherever the user left it), only while it
  // is on screen. Moving it writes three transforms (clip box, its content, the divider) inline, so
  // the live demo inside is never restyled by an inherited --pos per frame (--pos stays in the CSS
  // as the no-script state). The drawn wave and the masked edge flow together in one element
  // (.cmp-ride); its counter-flow (.cmp-anti, which keeps the content still) is pinned to the same
  // start time whenever the slider comes into view.
  const calm = () => d.hasAttribute('data-oxyn-reduced') || matchMedia('(prefers-reduced-motion: reduce)').matches;
  document.querySelectorAll('.x-cmp').forEach((c) => {
    const r = c.querySelector('.cmp-range');
    if (!r) return;
    const MID = 50, AMP = 20, PERIOD = 7000, IDLE = 2200;
    let pos = MID, phase = 0, last = 0, raf = 0, seen = false, held = false, touched = -1e9;
    const clip = c.querySelector('.cmp-clip'), after = c.querySelector('.cmp-after'), line = c.querySelector('.cmp-line');
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
    const tick = (t) => {
      raf = 0;
      if (!seen || calm() || document.hidden) return;
      if (!held && t - touched > IDLE) {
        if (last) phase += (t - last) / PERIOD;
        put(MID + AMP * Math.sin(phase * 2 * Math.PI));
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
    // Resume from the current spot: choose the phase whose sine gives the current position.
    const resumeFrom = () => { phase = Math.asin(Math.max(-1, Math.min(1, (pos - MID) / AMP))) / (2 * Math.PI); };
    const user = () => { touched = performance.now(); resumeFrom(); };
    r.addEventListener('input', () => { put(Number(r.value)); user(); });
    r.addEventListener('pointerdown', () => { held = true; user(); });
    addEventListener('pointerup', () => { if (held) { held = false; user(); } });
    r.addEventListener('focus', () => { held = true; });
    r.addEventListener('blur', () => { held = false; user(); });
    if ('IntersectionObserver' in window) {
      new IntersectionObserver((es) => es.forEach((e) => { seen = e.isIntersecting; if (seen) { sync(); run(); } })).observe(c);
    }
    document.addEventListener('visibilitychange', run);
    put(Number(r.value) || MID);
  });
})();

// Hero scene: its loop runs only while it is on screen.
document.querySelectorAll('[data-fix]').forEach((f) => {
  f.toggleAttribute('data-paused', true);
  if ('IntersectionObserver' in window) new IntersectionObserver((es) => es.forEach((e) => f.toggleAttribute('data-paused', !e.isIntersecting))).observe(f);
  else f.removeAttribute('data-paused');
});

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
