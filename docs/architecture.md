# Architecture (planned)

> Design document. Implemented so far: `oxyn-tokens` (plan #13) and the `oxyn-core` state machine, clock and verb packs (plan #14), and the first component, `StatusIndicator`, in GPUI and Dioxus over one shared model (`oxyn-core::indicator`, plan #15), and the loader engine with the 14 free glyph loaders (`oxyn-core::loader`, plan #16).

## Crates

```
crates/
  oxyn-core              no UI deps: AgentState machine, event stream model, verb packs,
                         motion presets (durations, easings, springs, frame sets), timing clock
  oxyn-tokens            tokens/*.json -> Rust theme values + tokens/dist/oxyn.css (generator: tools/oxyn-tokens-gen)
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

Implemented in `oxyn-core::state` (plan #14). One enum drives every component. Adapters map external
protocols onto `AgentEvent`s; `AgentMachine` folds them into the state.

```
Idle
Composing                       user typing (draft present)
Submitted                       sent, waiting for the first event ("submitted" pulse, not a spinner, < 500 ms)
Thinking { header }             reasoning / planning; header = model-written line if the protocol has one,
                                otherwise the verb comes from the verb pack at render time
Streaming                       tokens arriving
ToolRunning { tool, concurrent } most recently started call + how many run at once ("Running 3 tools");
                                tool carries id, name, ActivityKind (picks the pinned verb) and target
AwaitingApproval { request }    run timer paused; wins over running tools
Queued { count }                user sent while the agent works (shown until the next work event;
                                AgentMachine::queued() keeps the count)
Steering                        user interrupt being applied mid-run
Done { elapsed, waited }        elapsed excludes time waiting for the user -> "Worked for 12s"
Error { kind }                  Tool | Model | Timeout | Offline | RateLimited | Other
Cancelled { after }             -> "Interrupted after 42s"
Stalled { since, level, was }   derived: no event since `since`; level Soft | Hard; `was` = the busy state
                                it stalled from, restored when the next event arrives
```

Rules:
- Components receive `&AgentState` + their own props; no hidden timers except the shared motion clock.
- Time is passed in (`ClockMs`, monotonic ms from the host). `oxyn-core` never reads a clock, so it is
  deterministic in tests and safe on `wasm32-unknown-unknown`.
- `Stalled` is derived by `AgentMachine::tick` from event timestamps, never set by the app. Default
  thresholds (`StallPolicy`, proposed): soft 5 s / hard 45 s; while a tool runs 120 s / 300 s.
  Idle, finished and awaiting-approval states never stall.
- The machine is lenient: a work event with no `Submit` starts a run; a second terminal event is ignored.
- One `MotionClock` per window (`oxyn-core::clock`) gives loop phase from absolute time (paused loops
  resume at the current phase, never catch up), the frame-rate policy of motion.md §9 and leases.
- Reduced-motion is a flag on the clock; every motion preset defines its static fallback.
- Verb packs (`oxyn-core::verbs`, TOML in `crates/oxyn-core/verb_packs/`): free packs `neutral`,
  `coder`, `terse`, `crm`; the activity table; `{d}` duration phrases. `multilingual` waits for
  native-speaker review; Pro packs ship with the Pro registry.

### Planned: `RunLog` (with plan #20 tool-calls)
`tools.md` §2 and `run-replay.md` §9 each sketch a `RunLog`. They become one type in `oxyn-core::run`:
the append-only `RunEvent` list of run-replay.md (seq, time, agent, kind) is the storage; the
`ToolCall` records of tools.md are a fold over it. `AgentMachine` will then consume the same events,
so replay to time *t* = folding events up to *t*.

## Adapters (later)
Vercel AI SDK stream parts, AG-UI events, ACP (Zed agent protocol), MCP tool-call states,
OpenAI / Anthropic streaming events. Each adapter is a pure function `Event -> StateTransition`.

## Surfaces
`Chat`, `Ide`, `Desktop`, `Mobile`, `Voice`, `Terminal` (TUI look). Variants declare which surfaces they support.
