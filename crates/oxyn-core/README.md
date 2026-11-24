# oxyn-core

The UI-free core of [Oxyn](https://oxynui.com), the agent UI component library for Rust (Dioxus and
GPUI). Components you add with the `oxyn` CLI depend on it.

It holds what every renderer shares:

- **`AgentState`**, the one state machine behind every composer, chip, orb, caret and label:
  `idle → composing → submitted → thinking → streaming → tool_running → awaiting_approval → done | error | cancelled | stalled`.
- **The motion clock and easing**, so animations run at the same pace in Dioxus and GPUI, with
  reduced-motion fallbacks.
- **Verb packs** ("searching", "reading", "patching") for activity-aware loaders.
- The models behind the components: composer, approvals, checkpoints, runs, chat, text and the
  primitives they are built from.

No UI framework dependency: it compiles anywhere Rust does, WASM included.

```toml
[dependencies]
oxyn-core = "0.1"
```

Add components with the CLI rather than by hand: `cargo install oxyn-cli`, then `oxyn init` and
`oxyn add composer`. Docs: [oxynui.com/docs](https://oxynui.com/docs/).

## License

MIT. Source, issues and the full component catalogue: [github.com/ismailqureshi72/Oxyn](https://github.com/ismailqureshi72/Oxyn) · [oxynui.com](https://oxynui.com).
