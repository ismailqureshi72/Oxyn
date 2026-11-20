// Landing hero "Live edit" (Hero.astro): the headline lockup is designed live. No dependencies.
// Autoplay is a 15.2 s loop; any visitor input stops it and hands over control, and it resumes after
// 7 s idle. It pauses off screen. Reduced motion (OS setting or the site toggle) shows a static edit.
(() => {
  const root = document.querySelector('[data-dz]'); if (!root) return;
  const $ = (s, el = root) => el.querySelector(s), $$ = (s, el = root) => [...el.querySelectorAll(s)];
  const stage = $('[data-dz-stage]'), lock = $('[data-dz-lock]'), h1 = $('.dz-h1'), cta = $('.dz-cta');
  const hov = $('.dz-hov'), sel = $('.dz-sel'), gaps = $$('.dz-gap'), gapB = $$('.dz-bdg:not(.dz-red)'), gls = $$('.dz-gl'), redB = $$('.dz-red');
  const curA = $('.dz-cur.a'), curB = $('.dz-cur.b'), handleE = $('.dz-h[data-h="e"]'), live = $('[data-dz-live]');
  const NAMES = { hero: 'Hero', badge: 'Badge', h1: 'Headline', sub: 'Subhead', cta: 'Actions', b1: 'Browse library', b2: 'Get started', b3: 'Install' };
  const el = (k) => (k === 'hero' ? lock : $(`.dz-lock [data-k="${k}"]`));
  const reduced = () => document.documentElement.hasAttribute('data-oxyn-reduced') || matchMedia('(prefers-reduced-motion: reduce)').matches;
  const reduce = reduced();
  const small = () => innerWidth <= 760;
  const r0 = { w: 760, wt: 620, tr: -0.048, r: '999px' };
  let cur = null, word = null;

  const rel = (n) => { const s = stage.getBoundingClientRect(), r = n.getBoundingClientRect(); return { x: r.left - s.left, y: r.top - s.top, w: r.width, h: r.height }; };
  const place = (n, r) => Object.assign(n.style, { left: r.x + 'px', top: r.y + 'px', width: r.w + 'px', height: r.h + 'px' });
  const show = (n, on) => n.classList.toggle('on', !!on);
  const at = (n, x, y) => {
    n.style.transform = `translate(${x}px,${y}px)`;
    // a cursor near the stage's right edge carries its name tag on the left so it stays on screen
    const tag = n.classList.contains('dz-cur') && n.querySelector('span');
    if (tag) n.classList.toggle('flip', x + 13 + tag.offsetWidth > stage.clientWidth - 4);
  };

  function geometry() {
    gaps.concat(gapB, gls, redB).forEach((n) => show(n, false));
    if (cur === 'cta') {
      const kids = [...cta.children].map(rel);
      for (let i = 0; i < 2 && i + 1 < kids.length; i++) {
        const a = kids[i], b = kids[i + 1];
        if (Math.abs(a.y - b.y) > 4) continue; // wrapped row on mobile
        const g = { x: a.x + a.w, y: a.y, w: b.x - a.x - a.w, h: a.h };
        place(gaps[i], g); show(gaps[i], true);
        at(gapB[i], g.x + g.w / 2 - 8, g.y - 21); show(gapB[i], true);
      }
    }
    if (cur === 'sub' && !small()) {
      const t = rel(h1), s = rel($('.dz-sub')), c = rel(cta), x = s.x + s.w + 14;
      const seg = [[t.y + t.h, s.y], [s.y + s.h, c.y]];
      seg.forEach(([y0, y1], i) => {
        place(gls[i], { x, y: y0, w: 0, h: y1 - y0 }); show(gls[i], true);
        redB[i].textContent = Math.round(y1 - y0); at(redB[i], x + 5, (y0 + y1) / 2 - 7); show(redB[i], true);
      });
    }
  }
  function frame() {
    if (!cur) return;
    const n = word || el(cur), r = rel(n);
    place(sel, r);
    $('[data-sel-dim]').textContent = `${Math.round(r.w)} × ${Math.round(r.h)}${cur === 'cta' ? ' · Hug' : ''}`;
    $('[data-ins-w]').textContent = Math.round(r.w); $('[data-ins-h]').textContent = Math.round(r.h);
    if (cur === 'sub') { const cs = getComputedStyle(n); $('[data-ins-size]').textContent = parseFloat(cs.fontSize); $('[data-ins-tr]').textContent = '0%'; }
    else if (cur === 'h1') { $('[data-ins-size]').textContent = parseFloat(getComputedStyle(h1).fontSize); setTr(parseFloat(h1.style.getPropertyValue('--dz-tr')) || r0.tr); }
    sel.classList.toggle('up', !!word && word.offsetTop < h1.offsetTop + 20);
    handleE.setAttribute('aria-valuenow', Math.round(h1.getBoundingClientRect().width)); handleE.setAttribute('aria-valuemax', Math.round(lock.getBoundingClientRect().width));
    geometry();
  }
  function select(k, w) {
    cur = k; word = w || null;
    sel.classList.toggle('rs', k === 'h1' && !word); sel.classList.toggle('word', !!word);
    handleE.tabIndex = k === 'h1' && !word ? 0 : -1;
    $('[data-sel-name]').textContent = NAMES[k] || '';
    $$('.dz-lay [data-k]').forEach((b) => b.setAttribute('aria-pressed', b.dataset.k === k && !word));
    const text = k === 'h1' || k === 'sub';
    $('[data-ins-text]').hidden = !text; $('[data-ins-auto]').hidden = k !== 'cta';
    $('[data-ins-title]').textContent = word ? word.textContent : NAMES[k]; $('[data-ins-kind]').textContent = k === 'cta' ? 'Auto layout' : text ? 'Text' : 'Frame';
    if (!sel.classList.contains('on')) { sel.classList.add('drag'); frame(); sel.offsetWidth; requestAnimationFrame(() => sel.classList.remove('drag')); }
    show(sel, true); frame();
  }
  function clear() { cur = word = null; show(sel, false); geometry(); $$('.dz-lay [data-k]').forEach((b) => b.setAttribute('aria-pressed', 'false')); }

  // Design state the inspector edits.
  const setW = (w) => { h1.style.setProperty('--dz-w', Math.round(w) + 'px'); };
  const setWt = (wt) => { h1.style.setProperty('--dz-wt', wt); $$('.dz-seg [data-wt]').forEach((b) => b.setAttribute('aria-pressed', b.dataset.wt == wt)); };
  // (the weight segment edits the headline; selecting it from the subhead re-selects the headline)
  const setTr = (tr) => { h1.style.setProperty('--dz-tr', tr + 'em'); $('[data-ins-tr]').textContent = (tr * 100).toFixed(1).replace('-', '−') + '%'; };
  const setR = (r) => { cta.style.setProperty('--dz-r', r); $$('.dz-tok [data-r]').forEach((b) => b.setAttribute('aria-pressed', b.dataset.r === r)); };
  const reset = () => { setW(r0.w); setWt(r0.wt); setTr(r0.tr); setR(r0.r); };
  const flash = (n) => { n.classList.add('flash'); setTimeout(() => n.classList.remove('flash'), 600); };
  const tween = (from, to, ms, fn, done) => { const t0 = performance.now(); const step = (t) => { const p = Math.min(1, (t - t0) / ms), e = 1 - Math.pow(1 - p, 3); fn(from + (to - from) * e); p < 1 ? (raf = requestAnimationFrame(step)) : done && done(); }; let raf = requestAnimationFrame(step); return () => cancelAnimationFrame(raf); };
  // keep frame glued to the content while fonts/transitions settle
  new ResizeObserver(frame).observe(h1); addEventListener('resize', frame);
  h1.addEventListener('transitionend', frame);

  // ── Autoplay: a ~15 s loop of two collaborators editing the hero ──
  let timers = [], stopT = null, playing = false, touched = 0;
  const later = (ms, f) => timers.push(setTimeout(f, ms));
  const move = (c, n, fx = 0.5, fy = 0.5, dx = 0, dy = 0) => { const r = rel(n); at(c, r.x + r.w * fx + dx, r.y + r.h * fy + dy); };
  const press = (c) => { c.classList.add('press'); setTimeout(() => c.classList.remove('press'), 160); };
  const insVisible = () => getComputedStyle($('.dz-ins')).display !== 'none';
  function loop() {
    timers.forEach(clearTimeout); timers = []; clearTimeout(stopT);
    playing = true; reset(); clear();
    const s = rel(stage), words = $$('.dz-w');
    at(curA, s.w * 0.78, s.h * 0.9); at(curB, s.w * 0.2, s.h * 0.95); show(curA, true); show(curB, !small());
    later(250, () => move(curA, words[2], 0.3, 0.32));
    later(1250, () => { press(curA); select('h1'); });
    later(2200, () => { if (insVisible()) move(curB, $('.dz-seg [data-wt="500"]'), 0.5, 0.6); });
    later(2950, () => { press(curB); setWt(500); setTr(-0.03); flash($('[data-ins-trf]')); });
    later(4100, () => { if (insVisible()) move(curB, $('.dz-seg [data-wt="620"]'), 0.5, 0.6); });
    later(4800, () => { press(curB); setWt(620); setTr(-0.048); flash($('[data-ins-trf]')); });
    later(4300, () => move(curA, handleE, 0.5, 0.5, -2, -2));
    later(5300, () => {
      press(curA); sel.classList.add('drag'); const w0 = h1.getBoundingClientRect().width, w1 = Math.max(300, w0 - (small() ? 90 : 250));
      const follow = () => { frame(); move(curA, handleE, 0.5, 0.5, -2, -2); };
      curA.classList.add('drag');
      tween(w0, w1, 1200, (w) => { setW(w); follow(); }, () => later(700, () => tween(w1, w0, 900, (w) => { setW(w); follow(); }, () => { sel.classList.remove('drag'); curA.classList.remove('drag'); })));
    });
    later(8300, () => move(curB, cta, 0.5, 0.5, 0, 34));
    later(9000, () => { press(curB); select('cta'); });
    later(9800, () => { if (insVisible()) move(curB, $('.dz-tok [data-r="10px"]'), 0.5, 0.6); });
    later(10500, () => { press(curB); setR('10px'); setTimeout(frame, 460); });
    later(11800, () => { setR('999px'); setTimeout(frame, 460); if (insVisible()) move(curB, $('.dz-tok [data-r="999px"]'), 0.5, 0.6); });
    later(11200, () => move(curA, $('.dz-sub'), 0.7, 0.3));
    later(12300, () => { press(curA); select('sub'); });
    later(14000, () => { clear(); at(curA, s.w * 0.8, s.h * 0.92); at(curB, s.w * 0.18, s.h * 0.98); });
    later(15200, loop);
  }
  function stop() { timers.forEach(clearTimeout); timers = []; playing = false; show(curA, false); show(curB, false); sel.classList.remove('drag'); curA.classList.remove('drag'); }
  function idle() { clearTimeout(stopT); if (reduce) return; stopT = setTimeout(loop, 7000); }
  const takeOver = () => { touched = Date.now(); if (playing) { stop(); reset(); } idle(); };
  function staticState() { reset(); select('h1'); const r = rel(handleE); at(curA, r.x + 4, r.y + 6); show(curA, true); }

  // ── Interaction ──
  const targetOf = (t) => { const w = t.closest('.dz-w'); if (w) return ['h1', w]; const k = t.closest('.dz-lock [data-k]'); return k ? [k.dataset.k, null] : [null, null]; };
  lock.addEventListener('pointermove', (e) => { const [k, w] = targetOf(e.target); if (!k) return show(hov, false); place(hov, rel(w || el(k))); show(hov, true); });
  lock.addEventListener('pointerleave', () => show(hov, false));
  lock.addEventListener('click', (e) => {
    if (e.target.closest('a,button')) { takeOver(); return; } // links navigate, copy copies
    const [k, w] = targetOf(e.target); takeOver(); k ? select(k, w) : clear();
  });
  $$('.dz-lay [data-k]').forEach((b) => b.addEventListener('click', () => { takeOver(); select(b.dataset.k); live.textContent = `${NAMES[b.dataset.k]} selected, ${$('[data-sel-dim]').textContent.replace('×', 'by')}`; }));
  $$('.dz-seg [data-wt]').forEach((b) => b.addEventListener('click', () => { takeOver(); setWt(+b.dataset.wt); setTr(b.dataset.wt === '400' ? -0.02 : b.dataset.wt === '500' ? -0.03 : -0.048); if (cur !== 'h1') select('h1'); }));
  $$('.dz-tok [data-r]').forEach((b) => b.addEventListener('click', () => { takeOver(); setR(b.dataset.r); select('cta'); setTimeout(frame, 460); }));
  // drag the side handles: the headline box resizes symmetrically and the text reflows
  $$('.dz-h[data-h="e"],.dz-h[data-h="w"]').forEach((h) => h.addEventListener('pointerdown', (e) => {
    e.preventDefault(); takeOver(); h.setPointerCapture(e.pointerId); sel.classList.add('drag');
    const x0 = e.clientX, w0 = h1.getBoundingClientRect().width, sgn = h.dataset.h === 'e' ? 1 : -1, max = lock.getBoundingClientRect().width;
    const mv = (ev) => { setW(Math.max(280, Math.min(max, w0 + 2 * sgn * (ev.clientX - x0)))); frame(); };
    const up = () => { h.removeEventListener('pointermove', mv); h.removeEventListener('pointerup', up); sel.classList.remove('drag'); };
    h.addEventListener('pointermove', mv); h.addEventListener('pointerup', up);
  }));
  handleE.addEventListener('keydown', (e) => {
    const d = { ArrowRight: 1, ArrowUp: 1, ArrowLeft: -1, ArrowDown: -1 }[e.key]; if (!d) return;
    e.preventDefault(); takeOver(); sel.classList.add('drag');
    setW(Math.max(280, Math.min(lock.getBoundingClientRect().width, h1.getBoundingClientRect().width + d * (e.shiftKey ? 64 : 16)))); frame();
    setTimeout(() => sel.classList.remove('drag'), 50);
  });
  root.addEventListener('keydown', (e) => { if (e.key === 'Escape' && cur) { takeOver(); clear(); } });
  root.addEventListener('focusin', (e) => { if (e.target.closest('.dz-pan,.dz-h')) takeOver(); });
  // site.js copies every [data-copy]; the hero only announces it.
  $$('[data-copy]').forEach((b) => b.addEventListener('click', () => { live.textContent = 'Install command copied'; b.setAttribute('aria-label', 'Copied'); setTimeout(() => b.setAttribute('aria-label', 'Copy install command'), 1600); }));

  const go = () => { root.classList.add('ready'); reduce ? staticState() : loop(); };
  (document.fonts?.ready || Promise.resolve()).then(() => requestAnimationFrame(go));
  // pause the loop while the hero is off screen
  new IntersectionObserver(([en]) => { if (reduce) return; if (!en.isIntersecting) { stop(); clearTimeout(stopT); } else if (!playing && root.classList.contains('ready')) { if (Date.now() - touched > 7000) loop(); else idle(); } }).observe(root);
})();
