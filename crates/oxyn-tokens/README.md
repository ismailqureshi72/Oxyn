# oxyn-tokens

Design tokens for [Oxyn](https://oxynui.com), the agent UI component library for Rust, as plain Rust
values: themes (dark, light and high-contrast dark), colour roles, semantic scales (space, radius, type) and motion
(durations and easings).

The values are generated from the JSON tokens in the Oxyn repository, the same source as Oxyn's CSS
variables, so a Dioxus component, a GPUI element and the website all read the same palette. You
normally get this crate through `oxyn-core`, which components added with the `oxyn` CLI depend on.

```toml
[dependencies]
oxyn-tokens = "0.1"
```

## License

MIT. Source, issues and the full component catalogue: [github.com/ismailqureshi72/Oxyn](https://github.com/ismailqureshi72/Oxyn) · [oxynui.com](https://oxynui.com).
