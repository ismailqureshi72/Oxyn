---
title: CLI
description: Add Oxyn components to a Dioxus or GPUI project with the oxyn command.
---

`oxyn` copies a component's code, and the components it needs, into your project, shadcn style. The code lands in `src/oxyn/`, so it is yours to read and change; nothing is hidden in a dependency except the framework-free `oxyn-core` and `oxyn-tokens` crates.

## Install

The CLI is not on crates.io yet. Install it from the repository:

```sh
cargo install --git https://github.com/ismailqureshi72/Oxyn oxyn-cli
```

## Start a project

Run it inside any Rust project that depends on `dioxus` or `gpui`:

```sh
oxyn init                 # writes oxyn.toml, creates src/oxyn/, declares `mod oxyn;`
oxyn add composer button  # copies both, and what they need
```

`init` detects the framework from `Cargo.toml`; pass `--framework dioxus` or `--framework gpui` when the project has both or neither. `--path` picks another folder inside `src/`.

Then use the components from `crate::oxyn`. `oxyn add` prints the exact lines for what it installed:

```rust
use crate::oxyn::{Composer, COMPOSER_CSS};

// once per page (Dioxus):
// document::Stylesheet { href: asset!("/assets/oxyn/oxyn-tokens.css") }
// style { {COMPOSER_CSS} }
```

Set `data-oxyn-theme="dark"`, `"light"` or `"hc-dark"` on any ancestor element to pick a theme.

## Commands

| Command | What it does |
|---|---|
| `oxyn init` | Sets up the project: `oxyn.toml`, `src/oxyn/`, `mod oxyn;` and the `oxyn-core` dependency. |
| `oxyn list [--category <c>] [--tier free\|pro]` | Every item in the registry, free and Pro. |
| `oxyn search <words>` | Searches names, categories and descriptions. |
| `oxyn view <slug>` | One item: what it is, its files, the items and crates it needs. |
| `oxyn add <slug>... [--framework dioxus\|gpui] [--path <dir>] [--overwrite]` | Installs items and their dependencies, adds the crates they need to `Cargo.toml`. |
| `oxyn diff <slug>` | Compares an installed item with the registry. |

`list`, `search` and `view` take `--json` for scripts and agents.

## Your changes are safe

`oxyn.lock` records a hash of every file `oxyn add` wrote. A later `add` that would replace a file you edited stops and tells you which one; `oxyn diff <slug>` shows the difference, and `--overwrite` replaces it on purpose.

## Free and Pro

74 components are free (MIT) and install from the public registry. Everything else (Pro components, blocks, screens and agent apps) is listed by `oxyn list` but needs [Oxyn Pro](/pricing/): `oxyn add` on a Pro item says so, links to the pricing page and exits with code 2. Pro installs from the CLI arrive with sign-in in a later release.

## Registry

The CLI reads a static registry: `https://ui.oxyn.workers.dev/r/index.json` lists every item, and `https://ui.oxyn.workers.dev/r/<slug>.json` holds the files of each free item. Point it elsewhere with `--registry <url-or-directory>` or the `OXYN_REGISTRY` environment variable, for example a registry built from a checkout with `npm run gen` in `site/` (`site/public/r`).
