---
title: Compare
description: How Oxyn differs from other agent-UI kits.
---

Most agent-UI kits are React. Oxyn exists for teams building agent products in **Rust**.

| | Oxyn | React agent-UI kits |
|---|---|---|
| Language | Rust | TypeScript / React |
| Web | Dioxus | React DOM |
| Desktop | GPUI (native, GPU-rendered) | Electron or a webview |
| State model | One `AgentState` machine in `oxyn-core`, shared by every component | Usually per component |
| Time | Passed in (`now`), deterministic in tests and posters | Usually component timers |
| Licence | MIT core | Varies |

## When to pick something else

- Your app is React or Next.js: a React kit fits better than calling into WebAssembly.
- You need hundreds of finished screens today: Oxyn is early, and its screens and blocks are still [planned](/screens/).

## Design reference

The layout of this site and the item-by-item parity plan follow [affectUI](https://www.affectui.com), studied from its public pages. No affectUI source or assets are used; every Oxyn component is written from Oxyn's own specs.
