# oxyn-mcp

[Oxyn](https://oxynui.com)'s MCP server: lets AI coding agents (Claude Code, Cursor, Codex, Zed)
browse the Oxyn registry and add agent UI components to a Dioxus or GPUI project, over stdio.

```sh
cargo install oxyn-mcp
claude mcp add oxyn -- oxyn-mcp
```

Tools: `list_components`, `search_components`, `get_component` (props, dependencies, source),
`add_component` (the same install as `oxyn add`) and `get_install_instructions`. Pro items install
after `oxyn login` (from `oxyn-cli`) on a Pro or Team plan. Setup for each agent:
[oxynui.com/docs/mcp](https://oxynui.com/docs/mcp/).

## License

MIT. Source, issues and the full component catalogue: [github.com/ismailqureshi72/Oxyn](https://github.com/ismailqureshi72/Oxyn) · [oxynui.com](https://oxynui.com).
