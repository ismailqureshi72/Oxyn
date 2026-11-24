---
title: MCP server
description: Let Claude Code, Codex, Cursor, Windsurf, VS Code, Zed, Antigravity, Gemini CLI, Kimi Code, MiniMax Code, Cline or OpenCode find Oxyn components and add them to your project.
---

`oxyn-mcp` is a [Model Context Protocol](https://modelcontextprotocol.io) server for the Oxyn registry. Connect it to your coding agent and ask for what you need ("add an approval card to the settings page"): the agent finds the component, reads its props, and copies the code into `src/oxyn/` exactly as the [oxyn CLI](/docs/cli/) would.

## Install

Install it from crates.io:

```sh
cargo install oxyn-mcp
```

That puts `oxyn-mcp` in `~/.cargo/bin`. It speaks MCP over stdio; your agent starts it, you never run it by hand.

## Connect your agent

Each agent starts `oxyn-mcp` itself once it knows the command: `oxyn-mcp`, with no arguments. Pick yours.

<nav class="x-ag-grid" aria-label="Pick your agent">
<a href="#claude-code"><img class="x-ag-logo" src="/brand/marks/claude-code.svg" alt="" width="20" height="20" /><span>Claude Code</span></a>
<a href="#openai-codex"><img class="x-ag-logo" src="/brand/marks/codex.svg" alt="" width="20" height="20" /><span>OpenAI Codex</span></a>
<a href="#cursor"><img class="x-ag-logo" src="/brand/marks/cursor.svg" alt="" width="20" height="20" /><span>Cursor</span></a>
<a href="#windsurf"><img class="x-ag-logo" src="/brand/marks/windsurf.svg" alt="" width="20" height="20" /><span>Windsurf</span></a>
<a href="#vs-code-github-copilot"><img class="x-ag-logo" src="/brand/marks/github-copilot.svg" alt="" width="20" height="20" /><span>VS Code (GitHub Copilot)</span></a>
<a href="#zed"><img class="x-ag-logo" src="/brand/marks/zed.svg" alt="" width="20" height="20" /><span>Zed</span></a>
<a href="#google-antigravity"><img class="x-ag-logo" src="/brand/marks/antigravity.svg" alt="" width="20" height="20" /><span>Google Antigravity</span></a>
<a href="#gemini-cli"><img class="x-ag-logo" src="/brand/marks/gemini-cli.svg" alt="" width="20" height="20" /><span>Gemini CLI</span></a>
<a href="#kimi-code"><img class="x-ag-logo" src="/brand/marks/kimi.svg" alt="" width="20" height="20" /><span>Kimi Code</span></a>
<a href="#minimax-code"><img class="x-ag-logo" src="/brand/marks/minimax.svg" alt="" width="20" height="20" /><span>MiniMax Code</span></a>
<a href="#cline"><img class="x-ag-logo" src="/brand/marks/cline.svg" alt="" width="20" height="20" /><span>Cline</span></a>
<a href="#opencode"><img class="x-ag-logo" src="/brand/marks/opencode.svg" alt="" width="20" height="20" /><span>OpenCode</span></a>
</nav>

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/claude-code.svg" alt="" width="20" height="20" />Claude Code</span>

```sh
claude mcp add --transport stdio oxyn -- oxyn-mcp
```

Add `--scope project` to share it with your team through `.mcp.json`, or `--scope user` for every project.

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/codex.svg" alt="" width="20" height="20" />OpenAI Codex</span>

```sh
codex mcp add oxyn -- oxyn-mcp
```

Or add it to `~/.codex/config.toml` (or `.codex/config.toml` in a trusted project). The Codex CLI, the IDE extension and the ChatGPT desktop app share this file.

```toml
[mcp_servers.oxyn]
command = "oxyn-mcp"
```

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/cursor.svg" alt="" width="20" height="20" />Cursor</span>

`.cursor/mcp.json` in your project, or `~/.cursor/mcp.json` for every project:

```json
{
  "mcpServers": {
    "oxyn": { "type": "stdio", "command": "oxyn-mcp" }
  }
}
```

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/windsurf.svg" alt="" width="20" height="20" />Windsurf</span>

Windsurf is now Devin Desktop. Add the server to `~/.config/devin/mcp_config.json` (on Windows `%APPDATA%\devin\mcp_config.json`), or to `.devin/mcp_config.json` in your project. Older Windsurf builds read `~/.codeium/windsurf/mcp_config.json`; the editor's Open MCP config file action opens the right one.

```json
{
  "mcpServers": {
    "oxyn": { "command": "oxyn-mcp", "args": [] }
  }
}
```

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/github-copilot.svg" alt="" width="20" height="20" />VS Code (GitHub Copilot)</span>

Run **MCP: Add Server**, choose **Command (stdio)** and enter `oxyn-mcp`, or write the file yourself: `.mcp.json` at the workspace root to share it, or `~/.copilot/mcp-config.json` for all your workspaces.

```json
{
  "mcpServers": {
    "oxyn": { "type": "stdio", "command": "oxyn-mcp", "args": [], "tools": ["*"] }
  }
}
```

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/zed.svg" alt="" width="20" height="20" />Zed</span>

Settings → AI → MCP Servers → **Add Local Server**, or in your settings file (`zed: open settings file`):

```json
{
  "context_servers": {
    "oxyn": { "command": "oxyn-mcp", "args": [], "env": {} }
  }
}
```

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/antigravity.svg" alt="" width="20" height="20" />Google Antigravity</span>

In the app, open **Customizations** → **Installed** → **MCP Servers** and copy the config file path; in the IDE, **MCP Servers** → **Manage MCP Servers** → **View raw config**. The file is `~/.gemini/config/mcp_config.json`, or `.agents/mcp_config.json` in a workspace. Antigravity CLI reads the same files (`/mcp` shows the status).

```json
{
  "mcpServers": {
    "oxyn": { "command": "oxyn-mcp" }
  }
}
```

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/gemini-cli.svg" alt="" width="20" height="20" />Gemini CLI</span>

```sh
gemini mcp add oxyn oxyn-mcp
```

That writes `.gemini/settings.json` in the project; add `--scope user` for `~/.gemini/settings.json`. The same entry by hand:

```json
{
  "mcpServers": {
    "oxyn": { "command": "oxyn-mcp" }
  }
}
```

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/kimi.svg" alt="" width="20" height="20" />Kimi Code</span>

Run `/mcp-config` in Kimi Code CLI, or add the server to `~/.kimi-code/mcp.json` (every project) or `.kimi-code/mcp.json` (this repository). `/mcp` shows the connection.

```json
{
  "mcpServers": {
    "oxyn": { "command": "oxyn-mcp" }
  }
}
```

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/minimax.svg" alt="" width="20" height="20" />MiniMax Code</span>

Open the plugin manager, switch to **MCP Servers** and add it in JSON mode, or edit `~/.minimax/mcp.json`. Check it with `mcode mcp list --human`.

```json
{
  "mcpServers": {
    "oxyn": { "type": "stdio", "command": "oxyn-mcp", "enabled": true }
  }
}
```

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/cline.svg" alt="" width="20" height="20" />Cline</span>

```sh
cline mcp add oxyn --yes -- oxyn-mcp
```

In the editor extensions: **MCP Servers** → **Configure** → **Configure MCP Servers**, then add the entry under `mcpServers` (the CLI keeps it in `~/.cline/data/settings/cline_mcp_settings.json`):

```json
{
  "mcpServers": {
    "oxyn": { "command": "oxyn-mcp", "args": [], "disabled": false }
  }
}
```

### <span class="x-ag-h"><img class="x-ag-logo" src="/brand/marks/opencode.svg" alt="" width="20" height="20" />OpenCode</span>

```sh
opencode mcp add oxyn -- oxyn-mcp
```

Add `--global` for every project. By hand, in `opencode.jsonc`:

```json
{
  "mcp": {
    "servers": {
      "oxyn": { "type": "local", "command": ["oxyn-mcp"] }
    }
  }
}
```

### Other clients

Any MCP client that can start a local (stdio) server works: the command is `oxyn-mcp`, with no arguments.

## Tools

- `list_components`: every item, free and Pro, filtered by `category`, `tier` or `framework`.
- `search_components`: searches names, categories and descriptions.
- `get_component`: one item. What it is, its props (Dioxus) or builder methods (GPUI), the items and crates it needs, the install command, its docs page, and the source files (for a Pro item only when the agent passes `include_code: true`, since that counts toward the month's Pro items).
- `add_component`: copies items and what they need into a project, adds crates to `Cargo.toml` and declares `mod oxyn;`. Sets the project up on first use.
- `get_install_instructions`: setup steps for Dioxus or GPUI: dependencies, styles and theme.

`add_component` takes `project_path` (the folder with your `Cargo.toml`); without it, it uses the folder the agent started the server in. Files you changed since they were added are kept unless the agent passes `overwrite`, the same protection `oxyn add` gives you.

## Free and Pro

74 components are free (MIT) and install directly. Pro items install with an [Oxyn Pro or Team plan](/pricing/): the server uses the same sign-in as the CLI, so run `oxyn login` once on that computer, or give the server a token from your [account page](/account/#cli) as `OXYN_TOKEN` in its environment, for example:

```sh
claude mcp add oxyn --env OXYN_TOKEN=oxyn_… -- oxyn-mcp
```

Each distinct Pro item counts once per billing month toward your plan's Pro items (Pro: 20 components and 8 screens or blocks; Team: 75 and 22, shared), the same count as the CLI and the site; adding it again that month is free. When an item needs a plan, a sign-in, or this month's items are used, `get_component` and `add_component` say so, with the count and the reset date.

## Registry

The server reads `https://oxynui.com/r`, the same registry as the CLI. Point it elsewhere with `--registry <url-or-directory>` in the command, or the `OXYN_REGISTRY` environment variable, for example a registry built from a checkout (`site/public/r`).

## Prompt for your agent

Paste this into Claude Code, Codex, Cursor or any other coding agent to set Oxyn up in your project. The copy button is in the corner of the block.

```text title="Prompt"
Set up Oxyn (Rust UI components for agent apps,
https://oxynui.com) in this project.

1. If the `oxyn` command is missing, install it:
   cargo install oxyn-cli
   If the oxyn MCP server (oxyn-mcp) is connected, you can use its tools
   (search_components, get_component, add_component) instead of the CLI.
2. In the root of this Dioxus or GPUI project, run `oxyn init` once.
3. Find components with `oxyn search <words>` or `oxyn list` (add --json to
   read the output) and check one with `oxyn view <slug>`.
4. Add what you need with `oxyn add <slug>`. Use the `use crate::oxyn::...`
   lines and the stylesheet it prints, keep the src/oxyn/ module layout, and
   theme through data-oxyn-theme and the Oxyn token CSS variables instead of
   hard-coded colours.
5. Treat the files in src/oxyn/ as generated: build on them from my code and
   do not edit them unless I ask (`oxyn diff <slug>` shows local changes).
6. Pro items need an Oxyn Pro plan. If `oxyn add` says an item is Pro, stop
   and ask me to run `oxyn login`; do not work around it.

When you are done, tell me what you installed and where.
```
