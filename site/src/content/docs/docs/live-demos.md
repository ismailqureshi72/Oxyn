---
title: How the live demos work
description: What you are looking at on each component page.
---

Every preview on this site is the real Rust component, not a screenshot or a re-implementation.

1. **At build time** a small Rust program (`apps/gallery-dioxus-web`, binary `site-data`) server-renders each demo with Dioxus SSR. That HTML is in the page, so it reads without JavaScript and search engines see it.
2. **In the browser**, the `<oxyn-demo>` custom element waits until the demo scrolls into view, then loads **one shared WebAssembly bundle** (the same crate compiled to `wasm32-unknown-unknown`) and mounts the live Dioxus component in place of the static markup. The bundle is fetched once and cached for every demo on the site.
3. **The controls panel** on a component page is part of the same Dioxus app: changing a control changes the component's real props, and the code under it is regenerated from those values.

Components that animate with time (timers, countdowns, streams, rolling numbers) get a running clock with Play, Pause and Restart.

If the WebAssembly bundle can't load, the server-rendered preview stays in place.
