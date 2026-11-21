---
title: MCP server
description: Let Claude Code, Cursor, Codex or Zed browse Oxyn components and add them to your project.
---

`oxyn-mcp` is a [Model Context Protocol](https://modelcontextprotocol.io) server for the Oxyn registry. Connect it to your coding agent and ask for what you need ("add an approval card to the settings page"): the agent finds the component, reads its props, and copies the code into `src/oxyn/` exactly as the [oxyn CLI](/docs/cli/) would.

## Install

The server is not on crates.io yet. Install it from the repository:

```sh
cargo install --git https://github.com/ismailqureshi72/Oxyn oxyn-mcp
```

That puts `oxyn-mcp` in `~/.cargo/bin`. It speaks MCP over stdio; your agent starts it, you never run it by hand.

## Connect your agent

Each agent starts `oxyn-mcp` itself once it knows the command.

### Claude Code

```sh
claude mcp add oxyn -- oxyn-mcp
```

Add `--scope project` to share it with your team through `.mcp.json`.

### Cursor

`.cursor/mcp.json` in your project (or `~/.cursor/mcp.json` for every project):

```json
{
  "mcpServers": {
    "oxyn": { "command": "oxyn-mcp" }
  }
}
```

### Codex

```sh
codex mcp add oxyn -- oxyn-mcp
```

or in `~/.codex/config.toml`:

```toml
[mcp_servers.oxyn]
command = "oxyn-mcp"
```

### Zed

In your settings file (`zed: open settings file`):

```json
{
  "context_servers": {
    "oxyn": { "command": "oxyn-mcp", "args": [] }
  }
}
```

Other MCP clients work the same way: the command is `oxyn-mcp`, with no arguments.

## Tools

- `list_components`: every item, free and Pro, filtered by `category`, `tier` or `framework`.
- `search_components`: searches names, categories and descriptions.
- `get_component`: one item. What it is, its props (Dioxus) or builder methods (GPUI), the items and crates it needs, the install command, its docs page, and for free items the source files.
- `add_component`: copies items and what they need into a project, adds crates to `Cargo.toml` and declares `mod oxyn;`. Sets the project up on first use.
- `get_install_instructions`: setup steps for Dioxus or GPUI: dependencies, styles and theme.

`add_component` takes `project_path` (the folder with your `Cargo.toml`); without it, it uses the folder the agent started the server in. Files you changed since they were added are kept unless the agent passes `overwrite`, the same protection `oxyn add` gives you.

## Free and Pro

74 components are free (MIT) and install directly. Pro items appear in the lists with their previews, but `add_component` answers with a link to [Oxyn Pro](/pricing/) instead of the code. Pro installs through the server arrive with sign-in in a later release.

## Registry

The server reads `https://ui.oxyn.workers.dev/r`, the same registry as the CLI. Point it elsewhere with `--registry <url-or-directory>` in the command, or the `OXYN_REGISTRY` environment variable, for example a registry built from a checkout (`site/public/r`).
