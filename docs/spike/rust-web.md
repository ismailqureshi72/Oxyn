# Spike report: Rust on desktop and web (PR #11, V2.6)

Date: 2026-11-12. Machine: 2 vCPU aarch64 Linux sandbox, 7 GB RAM, no GPU (Mesa llvmpipe / lavapipe).
Toolchain: rustc 1.99.1 stable, wasm-bindgen 0.2.130, binaryen wasm-opt version_133, Chromium 156 headless.

The spike code was throwaway and was removed in plan #12 (`chore/workspace-ci`). It is preserved in history at commit [`52886d2`](https://github.com/ismailqureshi72/Oxyn/tree/52886d2/spike).
What carries forward is the decisions below and the `AgentState` shape in `spike/core`.

## Summary

| Question | Result | Status |
|---|---|---|
| Can `oxyn-core` stay UI-free and derive `Stalled` from timestamps? | Yes. 13 states, reducer + `state_at(now)`, 5 tests pass | ✅ confirmed |
| Dioxus web component size | **89 KB brotli** / 107 KB gzip / 245 KB raw WASM + 4.7 KB brotli JS glue | ✅ far under the 500 KB warn budget |
| Dioxus component renders and steps through states in a browser | Yes, Chromium headless, all script states | ✅ confirmed |
| Headless poster from the **web** renderer | Works: Chromium `--headless --virtual-time-budget=<ms> --screenshot` | ✅ confirmed |
| GPUI native build on Linux | Builds (with one patch, below). Release binary 10.4 MB | ✅ with caveat |
| Headless poster from **GPUI** on a GPU-less Linux runner | Window opens and the Vulkan renderer initialises on llvmpipe, but every capture is black | ❌ not working here |
| GPUI-on-web WASM size | Built from Zed `gpui_web` (rev `6ceb6fe`): **1.87 MB brotli** / 2.49 MB gzip / 6.24 MB raw for one component, font included | ✅ measured |
| GPUI-on-web renders in a real browser | Yes. Chrome on a laptop with integrated graphics: all states, pulse, crisp text. WebGPU first, WebGL2 fallback | ✅ confirmed (manual) |
| GPUI video decode | Not tested | ⚠️ open |

## 1. `oxyn-core` (spike/core)

- `AgentState` with the 13 values from `architecture.md`. `Stalled` carries `since_ms`, a `level` (Soft / Hard) and the state it interrupted (`was`), so a component can keep showing "Running cargo test" in a warning tone instead of losing context.
- `Run::apply(event, now_ms)` reduces adapter events. `Run::state_at(now_ms)` derives `Stalled`. No clock inside the crate: time is passed in, which makes the logic deterministic and testable and lets posters freeze any instant.
- Thresholds as proposed: 5 s soft, 45 s hard, 120 s hard while a tool runs.
- Only busy states stall. `AwaitingApproval`, `Done`, `Error`, `Cancelled`, `Idle` never do (waiting on the user is not a stall).
- Tests: script ends `Done { elapsed 9.6 s }`; soft → hard → recovers on the next event; tools use the longer threshold; approval never stalls; all 13 states have a label.

**Proposed for #14:** keep `Stalled { since, level, was }` (adds `level` and `was` to the documented `Stalled { since }`). Needs sign-off because `architecture.md` lists only `since`.

## 2. Dioxus web (spike/dioxus-status)

Dioxus 0.7.10, `web` feature only. One `StatusIndicator` (dot + label, `role="status"`, `aria-live="polite"`, 1 Hz pulse on busy states, pulse removed under `prefers-reduced-motion`).

Build: `cargo build --release --target wasm32-unknown-unknown` → `wasm-bindgen --target web` → `wasm-opt -Oz`.

| File | Raw | gzip -9 | brotli -q 11 |
|---|---|---|---|
| WASM after wasm-bindgen | 267,727 B | 109,818 B | 91,831 B |
| WASM after `wasm-opt -Oz` | 245,294 B | 106,708 B | **89,456 B** |
| JS glue | 25,624 B | 5,457 B | 4,714 B |

Clean build of the Dioxus crate: about 3 minutes on 2 cores.

Poster strip from Chromium headless at 1.5 s, 4 s, 6.5 s, 9 s and 12 s of the demo script: [`dioxus-poster-strip.png`](dioxus-poster-strip.png).

Findings:

1. **Most of the 89 KB is the Dioxus runtime, not the component.** Each extra component should add little. Needs measuring with 10+ components in #25 before we trust a per-component number.
2. **Do not use `wasm-opt --all-features`.** It emitted compact imports that Chromium 156 rejects (`Invalid import kind 127`). Enable features explicitly: `--enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-multivalue`.
3. **Do not set `strip = true` for the wasm profile.** wasm-bindgen warns it cannot read `target_features`. Strip with `wasm-opt` instead.
4. **Use typed style attributes for anything that changes.** A single `style: "...background:{colour}..."` string lost the `background` declaration on re-render while other declarations stayed. Switching to `background_color: "{colour}"` fixed it. Rule for `oxyn-dioxus`: dynamic values go in typed attributes or CSS custom properties, never interpolated into one long style string.
5. Our size budget (warn 500 KB / fail 800 KB) is generous. Suggest tightening to warn 250 KB / fail 400 KB brotli per demo bundle once #25 measures a real page.

## 3. GPUI native (spike/gpui-status)

`gpui` 0.2.2 from crates.io. Same `StatusIndicator`, drawn with `div()` styling; a 16 ms timer drives the pulse; `OXYN_POSTER_MS=<t>` freezes the script for posters.

- Clean release build (`opt-level 3`, fat LTO, 1 codegen unit): **about 25 minutes on 2 cores**. Binary 10,361,736 B. CI will need caching (Swatinem/rust-cache) and should not use fat LTO for check jobs.
- **Build break:** `gpui 0.2.2 → gpui_http_client → zed-async-tar → xattr 0.2.3` uses `libc::ENOATTR`, which libc 0.2.191 no longer exports on Linux. Spike fix: `[patch.crates-io]` with a local `xattr` that maps to `ENODATA` (`spike/patches/xattr`). Real fix for #12: track GPUI from git or wait for a crates.io release that drops `xattr 0.2`. File upstream before #12.
- **Headless poster failed on this machine.** Under Xvfb + lavapipe the GPUI window opens, blade selects llvmpipe, builds pipelines and picks MAILBOX present on a 32-bit ARGB visual, but every capture (`import`, `xwd`, root and window) is fully black. `vkcube` presents correctly on the same display, so Vulkan itself works. Under headless Weston (Wayland) GPUI panics because the compositor exposes no seat (`client.rs:503`, `seat.unwrap()`).

**Decision proposed:** change level 0 of the demo runtime from "poster rendered headlessly from GPUI" to **"poster rendered headlessly from the web renderer (Chromium), with GPUI captures made on a real GPU runner for the reference look"**. The website only shows web output anyway, and this path works today on any CI runner. GPUI posters move to an optional job on a macOS runner (Metal) or a machine with a GPU, to be tried in #12 or #15.

## 4. GPUI on the web (spike/gpui-web-status)

Built from Zed's repo at rev `6ceb6fe` (2026-11-12), not crates.io: `gpui`, `gpui_platform` with `default-features = false` and `gpui_platform::single_threaded_web()`. Stable rustc 1.99.1, no nightly, no `SharedArrayBuffer`, so no COOP/COEP headers are needed and any static host works. Same `StatusIndicator` and the same `oxyn-core` script as the native build. The crate is its own workspace with path dependencies into a Zed checkout next to the Oxyn repo (`git clone https://github.com/zed-industries/zed ../zed`).

Clean release build (`opt-level = "z"`, fat LTO): 811 s on 2 cores.

| File | Raw | gzip -9 | brotli -q 11 |
|---|---|---|---|
| WASM after wasm-bindgen | 9,160,134 B | 2,873,314 B | 2,037,035 B |
| WASM after `wasm-opt -Oz` | **6,244,217 B** | **2,494,523 B** | **1,865,206 B** |
| JS glue | 156,005 B | 23,195 B | 19,407 B |

The embedded font (Inter Medium, 411,500 B raw) is part of that. The rest is GPUI's floor: layout, text shaping and the wgpu renderer. A second component adds far less than the first.

**Rendering.** The backend probes WebGPU and falls back to WebGL2 (`WebBackendPreference::Auto`). In headless Chromium on this GPU-less sandbox it initialises on SwiftShader WebGL2, but canvas readback is blank and screenshots hang, so it cannot be verified here. On a real laptop (Chrome, integrated graphics) it renders and animates correctly through every state, confirmed by a screen recording on 2026-11-12. Integrated GPUs are enough; the blank result is a limit of this sandbox, not of GPUI.

**Comparison** (same component unless noted, compressed transfer):

| Build | Transfer |
|---|---|
| Dioxus `StatusIndicator` | 89 KB brotli |
| GPUI-web `StatusIndicator` | 1.87 MB brotli |
| affectUI whole site, first load (React, Vite, Vercel; entry JS + CSS) | ~263 KB gzip |
| Cydonia full GPUI app (external, removed 2026-10-10) | 3.9 MB brotli |

affectUI's app-shell demo (`/screens/app-shell-flat`) is plain DOM rendered by React: the browser draws the text, scrolls and handles input, which is why it feels instant on any machine. GPUI-web draws everything into one canvas and ships its own renderer and fonts, so it costs about 1.9 MB before the first pixel, and canvas text is not selectable, searchable or indexed.

**Proposed:** to match affectUI's lightness, the site, gallery and the interactive app-shell demo use the DOM renderer (Dioxus). GPUI-web stays as an opt-in "native renderer" view on detail pages, loaded only on click, cached across pages, and preceded by a poster. Both fit Cloudflare Pages' free tier (25 MiB per file). Vercel Hobby is non-commercial only, so it would not fit once Pro is sold.

## 5. Decisions for the maintainers

1. Poster source: web renderer (Chromium headless) instead of GPUI. **Recommended.**
2. `Stalled { since, level, was }` in `oxyn-core`. **Recommended.**
3. GPUI dependency: crates.io 0.2.2 + `xattr` patch, or a pinned Zed git revision (needed anyway for `gpui_web`). **Recommended: pinned git revision from #12.**
4. Tighter Dioxus budget (warn 250 KB / fail 400 KB brotli). Optional, after #25 measures a real page.
5. Site renderer split: Dioxus (DOM) for the site, gallery and app-shell demo; GPUI-web as an on-click native view on detail pages. **Recommended.**
6. GPUI video decode still untested; check before #25. Not blocking M0–M2.

## Reproduce

```bash
cd spike
cargo test -p oxyn-core-spike
cargo build --release -p oxyn-dioxus-status-spike --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir out \
  target/wasm32-unknown-unknown/release/oxyn-dioxus-status-spike.wasm
wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
  --enable-mutable-globals --enable-reference-types --enable-multivalue \
  out/oxyn-dioxus-status-spike_bg.wasm -o out/opt.wasm
# serve out/ with index.html, then:
chromium --headless=new --window-size=480,160 --virtual-time-budget=6500 \
  --screenshot=poster.png http://localhost:8765/index.html
cargo build --profile release-native -p oxyn-gpui-status-spike   # Linux needs xkbcommon, xcb, wayland, fontconfig, vulkan dev packages
```

GPUI on the web (needs Zed cloned next to the Oxyn repo):

```bash
git clone https://github.com/zed-industries/zed ../zed && git -C ../zed checkout 6ceb6fe
cd spike/gpui-web-status
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir out \
  target/wasm32-unknown-unknown/release/oxyn-gpui-web-status.wasm
wasm-opt -Oz <same --enable flags as above> out/oxyn-gpui-web-status_bg.wasm -o out/opt.wasm
cp index.html out/ && cd out && python -m http.server 8000   # open http://localhost:8000
```

`spike/dioxus-status/index.html` is the page used for the posters.
