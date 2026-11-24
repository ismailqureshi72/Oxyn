# Changelog

All notable changes to Oxyn are listed here. Versions follow [Semantic Versioning](https://semver.org/);
before 1.0, a minor version (0.x) may change APIs.

## [Unreleased]

### oxyn-cli 0.1.1

- **`oxyn init` adds `oxyn-core = "0.1"` from crates.io** instead of a git dependency pinned to the
  registry's commit. `--oxyn-path <dir>` still uses a local checkout, and the new `--oxyn-git` keeps
  the git source. Projects set up with 0.1.0 keep their git source (`[oxyn]` in `oxyn.toml`). The MCP
  server's `add_component` sets up a new project the same way.
- `cargo test` works from the published `oxyn-core` package: the icon-tone CSS check reads the
  `oxyn-dioxus` stylesheet at run time and is skipped outside the repository.

## [0.1.0] - 2026-11-24

The first public release: the component library, the site at [oxynui.com](https://oxynui.com), and the
crates on crates.io.

### Components

- **454 registry items**: 74 free components (MIT) plus 264 Pro components, 76 screens, 20 blocks and
  19 agent-app templates, grouped into chat, inputs, agents, surfaces, feedback, data and the other
  site categories.
- **Two renderers from one core.** `oxyn-dioxus` (web and Dioxus apps) and `oxyn-gpui` (desktop) read
  the same `oxyn-core` models, so a component behaves the same in both.
- **One agent state model.** `AgentState` (`idle → composing → submitted → thinking → streaming →
  tool_running → awaiting_approval → done | error | cancelled | stalled`) drives every border, chip,
  orb, caret and label; `Stalled` is derived from event timestamps.
- **Shared motion and tokens.** One motion clock and easing set with reduced-motion fallbacks; design
  tokens (dark, light, high-contrast dark) generated once for Rust and CSS.

### Tools

- **`oxyn` CLI** (`oxyn-cli`): `init`, `list`, `search`, `view`, `add`, `diff`, plus `login`, `logout`
  and `whoami` for Pro. Copies component source into your project (`src/oxyn/`) with its
  dependencies. Reads the registry at `https://oxynui.com/r` by default.
- **MCP server** (`oxyn-mcp`): five tools over stdio (`list_components`, `search_components`,
  `get_component`, `add_component`, `get_install_instructions`) for Claude Code, Cursor, Codex and
  Zed.
- **Free and Pro registry.** Free items are public JSON under `/r`; Pro items install after
  `oxyn login` on a Pro or Team plan, counted once per item per month.

### Site and billing

- **oxynui.com**: docs, a live gallery of Dioxus demos (Rust compiled to WASM), component and screen
  pages with code, and Pro previews open to everyone.
- **Pricing**: Free, Pro ($19.99/month or $199.99/year, with a founder rate for the first 100
  subscribers) and Team ($44.99/month or $449/year, up to 4 people). Polar is the merchant of record.
- **In-site billing**: plan changes, cancel and resume, invoices and the payment method from the
  account page; sign-in with GitHub or email; CLI tokens from the account page.

### Crates published

`oxyn-tokens`, `oxyn-core`, `oxyn-registry-schema`, `oxyn-cli`, `oxyn-mcp` (all 0.1.0). The
renderer crates (`oxyn-dioxus`, `oxyn-gpui`) are not published: you get components through the CLI,
which copies their source.

### Known gaps

- GPUI text fields are mostly not editable yet (they render and take focus, but most do not accept
  typing).
- Accounts have no email verification or password reset yet.
- Lighthouse has not been run on the site.
- The Memory Graph Pro component is deferred to a later release.
- Studio is marked "Coming soon".

[0.1.0]: https://github.com/ismailqureshi72/Oxyn/releases/tag/v0.1.0
