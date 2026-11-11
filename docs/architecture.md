# Architecture (planned)

> Design document. Nothing here is implemented yet.

## Crates

```
crates/
  oxyn-core              no UI deps: AgentState machine, event stream model, verb packs,
                         motion presets (durations, easings, springs, frame sets), timing clock
  oxyn-tokens            tokens/*.json -> Rust theme structs, CSS variables, GPUI theme JSON
  oxyn-registry-schema   meta.toml schema for registry items (free + pro)
  oxyn-codegen           renders a component + props + tokens to Rust source (CLI and Studio share it)
  oxyn-gpui              GPUI components (primary; built on gpui-kit's unstyled base)
  oxyn-dioxus            Dioxus components (web demos, Studio, web apps)
  oxyn-cli               `oxyn add`, `oxyn init`, `oxyn mcp`
registry/
  free/<component>/{meta.toml, gpui.rs, dioxus.rs, story.rs}
  pro/                   private submodule
apps/
  gallery-dioxus-web     live demos for the docs
  gallery-gpui-web       opt-in desktop-accurate demos (WebGPU)
  studio                 Dioxus web app
tokens/                  source of truth for design tokens
site/                    Astro + Starlight docs shell
workers/                 registry-pro, mcp (Cloudflare Workers)
xtask/                   build-registry | render-posters | build-wasm | release
```

## The agent state model

One enum drives every component. Adapters map external protocols onto it.

```
Idle
Composing            user typing (draft present)
Submitted            message sent, waiting for first token (show "submitted" pulse, not a spinner, < 500 ms)
Thinking { verb }    reasoning / planning; verb comes from the verb pack
Streaming            tokens arriving
ToolRunning { tool, verb }
AwaitingApproval { request }
Queued { count }     user messages queued while the agent works
Steering             user interrupt being applied mid-run
Done { elapsed }     -> "Worked for 12s"
Error { kind }
Cancelled
Stalled { since }    no event for N seconds (colour shifts toward warning)
```

Rules:
- Components receive `&AgentState` + their own props; no hidden timers except the shared motion clock.
- A shared clock (slowed when the window is unfocused) drives all animation so loaders stay in sync.
- `Stalled` is derived by `oxyn-core` from event timestamps, not set by the app.
- Reduced-motion is a global token; every motion preset defines its static fallback.

## Adapters (later)
Vercel AI SDK stream parts, AG-UI events, ACP (Zed agent protocol), MCP tool-call states,
OpenAI / Anthropic streaming events. Each adapter is a pure function `Event -> StateTransition`.

## Surfaces
`Chat`, `Ide`, `Desktop`, `Mobile`, `Voice`, `Terminal` (TUI look). Variants declare which surfaces they support.
