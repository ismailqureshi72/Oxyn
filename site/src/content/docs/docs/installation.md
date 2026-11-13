---
title: Installation
description: Add Oxyn to a Dioxus or GPUI project.
---

Oxyn is not published to crates.io yet. Depend on the repository directly.

## Dioxus (web)

```sh
cargo add oxyn-dioxus oxyn-core --git https://github.com/ismailqureshi72/Oxyn
```

Components style themselves with `--oxyn-*` CSS custom properties. Include the token stylesheet (`tokens/dist/oxyn.css` in the repository) once, plus the stylesheet constant of each family you use:

```rust
use dioxus::prelude::*;
use oxyn_dioxus::{Button, PRIMITIVES_CSS};

fn App() -> Element {
    rsx! {
        style { {PRIMITIVES_CSS} }
        Button { label: "Deploy", icon: "up", onclick: move |_| deploy() }
    }
}
```

| Constant | Components |
|---|---|
| `PRIMITIVES_CSS` | Buttons, inputs, data display, overlays |
| `COMPOSER_CSS` | Composer and its parts |
| `RUN_CSS` | Tool calls, stacks, plans, sub-agent trees |
| `HITL_CSS` | Approvals and checkpoints |
| `STATUS_TEXT_CSS` | Text shimmer, verb cycler, elapsed timer, text reveal |
| `GLYPH_LOADER_CSS` | Glyph loaders |
| `STATUS_INDICATOR_CSS` | Status indicator |

Set the theme with `data-oxyn-theme="dark"`, `"light"` or `"hc-dark"` on any element; dark is the default.

## GPUI (desktop)

```sh
cargo add oxyn-gpui oxyn-core --git https://github.com/ismailqureshi72/Oxyn
```

GPUI elements are `RenderOnce` types named after the Dioxus components with an `El` suffix (`ButtonEl`, `ApprovalCardEl`). Each component page lists its GPUI element, or says when there is none yet.

## Toolchain

The workspace pins Rust in `rust-toolchain.toml`. GPUI is pinned to one Zed revision; see the repository README.
