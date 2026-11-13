// <oxyn-demo>: mounts a live Dioxus (Rust → WASM) demo when it scrolls into view.
//
// The element already holds the server-rendered markup of the same component (real Dioxus SSR),
// so the page is complete without JavaScript. When the element is visible, the one shared WASM
// module is fetched (once, then cached) and `mount(root, slug, variant, controls)` replaces the
// static markup with the live component. Off-screen demos keep running only while visible.
const cfg = window.OXYN_WASM || {};
let modPromise = null;

function loadModule() {
  if (!cfg.js) return Promise.reject(new Error('no wasm bundle in this build'));
  modPromise ??= import(cfg.js).then(async (m) => {
    await m.default({ module_or_path: cfg.wasm });
    return m;
  });
  return modPromise;
}

const io = 'IntersectionObserver' in window
  ? new IntersectionObserver((entries) => {
      for (const e of entries) {
        if (e.isIntersecting) {
          io.unobserve(e.target);
          e.target.mountLive();
        }
      }
    }, { rootMargin: '240px 0px' })
  : null;

class OxynDemo extends HTMLElement {
  connectedCallback() {
    if (this.dataset.state) return;
    this.dataset.state = 'static';
    if (!cfg.js) { this.dataset.state = 'static-only'; return; }
    if (io) io.observe(this); else this.mountLive();
  }
  async mountLive() {
    if (this.dataset.state === 'live' || this.dataset.state === 'loading') return;
    this.dataset.state = 'loading';
    const root = this.querySelector('.oxd-root');
    try {
      const m = await loadModule();
      m.mount(root, this.dataset.slug, Number(this.dataset.variant ?? -1), this.dataset.controls === 'true');
      this.dataset.state = 'live';
      this.dispatchEvent(new CustomEvent('oxyn-demo-live', { bubbles: true }));
    } catch (err) {
      console.warn('oxyn-demo: staying on the server-rendered preview', err);
      this.dataset.state = 'static-only';
    }
  }
}
customElements.define('oxyn-demo', OxynDemo);
