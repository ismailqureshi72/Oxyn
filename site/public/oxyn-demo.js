// <oxyn-demo>: mounts a live Dioxus (Rust → WASM) demo when it scrolls into view.
//
// The element already holds the server-rendered markup of the same component (real Dioxus SSR),
// so the page is complete without JavaScript. Nothing about the WASM bundle is fetched until a demo
// is actually on screen (IntersectionObserver); then the one shared module is fetched (once, then
// cached: the bundle is content-hashed and served immutable) and `mount(root, slug, variant,
// controls)` brings the component to life.
// - Mounts are queued and run in batches on animation frames, only for demos still on screen
//   after a short dwell, so a fast scroll past a grid of cards does not bring every card it passed
//   to life (fewer live demos running = less CPU and memory on long pages).
// - A preview with nothing visible to show (no text, no graphics) gets a neutral skeleton until the
//   live component is in (data-skel; demo.css).
const cfg = window.OXYN_WASM || {};
let modPromise = null;

function loadModule() {
  if (!cfg.js) return Promise.reject(new Error('no wasm bundle in this build'));
  modPromise ??= import(cfg.js).then(async (m) => {
    const t0 = performance.now();
    await m.default({ module_or_path: cfg.wasm });
    window.OXYN_PERF = { wasmReady: Math.round(performance.now()), init: Math.round(performance.now() - t0) };
    return m;
  });
  return modPromise;
}

const visible = new Set();
const rects = new WeakMap();
const since = new WeakMap();
const queue = new Set();
let pumping = false;
const DWELL = 160;
const pump = () => {
  if (pumping) return;
  pumping = true;
  const next = () => requestAnimationFrame(() => {
    // Everything still on screen mounts together in this frame (one style + layout pass), nearest
    // the middle of the viewport first; whatever already scrolled away is dropped.
    const vh = innerHeight;
    const now = performance.now();
    for (const el of queue) if (!visible.has(el)) queue.delete(el);
    const batch = [...queue].filter((el) => now - (since.get(el) || 0) >= DWELL);
    batch.forEach((el) => queue.delete(el));
    if (!batch.length) { if (queue.size) setTimeout(next, DWELL / 2); else pumping = false; return; }
    const mid = (el) => { const r = rects.get(el); return r ? Math.abs(r.top + r.height / 2 - vh / 2) : 0; };
    batch.sort((x, y) => mid(x) - mid(y));
    Promise.allSettled(batch.map((el) => el.mountLive())).then(() => (queue.size ? setTimeout(next, DWELL) : (pumping = false)));
  });
  // A demo has to stay on screen for a moment (DWELL) before it starts: a fling past it does not.
  setTimeout(next, DWELL);
};

const io = 'IntersectionObserver' in window
  ? new IntersectionObserver((entries) => {
      for (const e of entries) {
        rects.set(e.target, e.boundingClientRect);
        if (e.isIntersecting) { if (!visible.has(e.target)) since.set(e.target, performance.now()); visible.add(e.target); queue.add(e.target); }
        else { visible.delete(e.target); }
      }
      if (queue.size) pump();
    }, { rootMargin: '120px 0px' })
  : null;

const hasContent = (el) => {
  const root = el.querySelector('.oxd-mount') || el;
  if (root.querySelector('svg,img,canvas,video,input,textarea,button')) return true;
  return (root.textContent || '').trim().length > 0;
};

class OxynDemo extends HTMLElement {
  connectedCallback() {
    if (this.dataset.state) return;
    this.dataset.state = 'static';
    const root = this.querySelector('.oxd-root');
    if (root && !hasContent(root)) this.dataset.skel = '';
    if (!cfg.js) { this.dataset.state = 'static-only'; delete this.dataset.skel; return; }
    // data-defer: mounted on demand (mountLive), e.g. a laptop tab the first time it is picked.
    if (this.dataset.defer === 'true') return;
    if (io) io.observe(this); else this.mountLive();
  }
  async mountLive() {
    delete this.dataset.defer;
    if (this.dataset.state === 'live' || this.dataset.state === 'loading') return;
    this.dataset.state = 'loading';
    io?.unobserve(this);
    visible.delete(this);
    const root = this.querySelector('.oxd-root');
    try {
      const m = await loadModule();
      m.mount(root, this.dataset.slug, Number(this.dataset.variant ?? -1), this.dataset.controls === 'true');
      delete this.dataset.skel;
      this.dataset.state = 'live';
      if (!window.OXYN_PERF?.firstLive && window.OXYN_PERF) window.OXYN_PERF.firstLive = Math.round(performance.now());
      this.dispatchEvent(new CustomEvent('oxyn-demo-live', { bubbles: true }));
    } catch (err) {
      console.warn('oxyn-demo: staying on the server-rendered preview', err);
      delete this.dataset.skel;
      this.dataset.state = 'static-only';
    }
  }
}
customElements.define('oxyn-demo', OxynDemo);
