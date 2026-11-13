// Site behaviour: theme toggle, ⌘K search (Pagefind), copy buttons, menus, announcement, TOC.
(function () {
  const d = document.documentElement;
  const isMac = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
  document.querySelectorAll('[data-mod-k]').forEach((k) => (k.textContent = isMac ? '⌘K' : 'Ctrl K'));

  // Theme: one switch for the gallery and the Starlight docs (same storage key as Starlight).
  const setTheme = (t) => {
    d.dataset.theme = t;
    d.setAttribute('data-oxyn-theme', t);
    localStorage.setItem('starlight-theme', t);
    document.querySelectorAll('[data-theme-toggle]').forEach((b) =>
      b.setAttribute('aria-label', t === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'),
    );
  };
  setTheme(d.dataset.theme === 'light' ? 'light' : 'dark');
  document.addEventListener('click', (e) => {
    const t = e.target.closest('[data-theme-toggle]');
    if (t) setTheme(d.dataset.theme === 'light' ? 'dark' : 'light');
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
    if (!text) {
      const host = b.closest('.oxd-code, .copyable') || b.parentElement.parentElement;
      text = host.querySelector('pre, code')?.innerText ?? '';
    }
    try {
      await navigator.clipboard.writeText(text);
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
    const typing = /INPUT|TEXTAREA|SELECT/.test(document.activeElement?.tagName) || document.activeElement?.isContentEditable;
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
