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
| `oxyn login [--token <token>] [--no-browser]` | Signs this computer in for Pro items: approve a code in the browser, or save a token from your account page. |
| `oxyn whoami [--json]` | The signed-in account, its plan, and this month's Pro items. |
| `oxyn logout` | Forgets the saved token on this computer. |

`list`, `search` and `view` take `--json` for scripts and agents. Coding agents can also use the [MCP server](/docs/mcp/), which offers the same registry as tools.

## Your changes are safe

`oxyn.lock` records a hash of every file `oxyn add` wrote. A later `add` that would replace a file you edited stops and tells you which one; `oxyn diff <slug>` shows the difference, and `--overwrite` replaces it on purpose.

## Free and Pro

74 components are free (MIT) and install from the public registry, no account needed. Everything else (Pro components, blocks, screens and agent apps) is listed by `oxyn list` and installs with an [Oxyn Pro or Team plan](/pricing/) once the CLI is signed in:

```sh
oxyn login          # opens the browser: approve the code your terminal shows
oxyn add chat-thread
oxyn whoami         # plan and this month's Pro items
```

`oxyn login` saves a token in your user config folder (`~/.config/oxyn/credentials.toml` on Linux, `~/Library/Application Support/oxyn/` on macOS, `%APPDATA%\oxyn\` on Windows), readable only by you. For CI or a machine without a browser, create a token under **CLI and MCP** on your [account page](/account/#cli) and set it as `OXYN_TOKEN` (it wins over a saved one) or pass it to `oxyn login --token`. Revoke a token there at any time.

### Monthly Pro items

A plan opens a set number of distinct Pro items each billing month:

| Plan | Pro components | Screens, blocks and agent apps |
|---|---|---|
| Pro, founder rate | 20 | 8 |
| Team (shared by everyone on it) | 75 | 22 |

An item counts once, the first time its code is opened in a billing month, whether that is `oxyn add`, the MCP server or Open code / Copy on the site. Installing, updating or diffing it again that month is free, and Pro items that come in as dependencies of an item you add do not count. The month starts on your plan's renewal day. `oxyn whoami` and your account page show the count and when it resets; at the limit, `oxyn add` says so with the reset date.

### Exit codes

`oxyn add` exits with `2` when an item needs a Pro plan or a sign-in, `3` when this month's Pro items are used, and `1` for other errors.

## Registry

The CLI reads a static registry: `https://ui.oxyn.workers.dev/r/index.json` lists every item, and `https://ui.oxyn.workers.dev/r/<slug>.json` holds the files of each free item. Pro items come from `https://ui.oxyn.workers.dev/api/r/<slug>.json` with your token (`OXYN_API` overrides the API base). Point it elsewhere with `--registry <url-or-directory>` or the `OXYN_REGISTRY` environment variable, for example a registry built from a checkout with `npm run gen` in `site/` (`site/public/r`).
