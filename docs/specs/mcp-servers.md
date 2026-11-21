**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: MCP servers (`mcp-servers`)

Section: Blocks · Agents (page `/blocks/agents/mcp-servers/`, Pro). Reference: uiarc "Blocks · Agents" ("MCP servers"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `McpServers` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Mcp` (GPUI), rules in `oxyn_core::blocks::mcp`.

## 1. Purpose
Manage the Model Context Protocol servers that give an agent its tools, and see what the enabled tools cost in context.

## 2. States
Connected · Needs sign in · Error (Retry) · Connecting · Open (tools, permission, endpoint, Remove) · Removed (Undo) · Sheet: Browse (search) → Authorize (simulated) → Discovering tools → Added · Over budget (amber).

## 3. Micro-interactions
Toggling a tool resizes its server's segment and rolls the totals; discovered tools appear one by one; a new server lands on top, open.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Connect after Retry | 1500 ms | — |
| Authorize | 1200 ms | — |
| Tool discovered | 450 ms each | — |
| Undo window | 6000 ms | — |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `list` | Five servers: connected, needs sign in and error. |
| `tools` | GitHub open with its tools (Oxyn extra). |
| `add` | The Add server sheet open (Oxyn extra). |

## 6. Responsive and edge cases
Up to 720 px; below 520 px the tool list drops its indent and the sheet rises higher; below 400 px the tool count leaves the row and read / write become icons. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Server rows are buttons with aria-expanded named with server, status, transport, tools on and tokens; the context bar is a meter whose value text says how far over budget; switches are named with tool, access and tokens; the sheet is a modal dialog (Escape closes, Cmd/Ctrl+Z undoes a remove). Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Mcp, ..)`; presses by id (mcp-open-<n>, mcp-tool-<n>-<t>, mcp-perm-<n>-<v>, mcp-retry-<n>, mcp-signin-<n>, mcp-remove-<n>, mcp-undo, mcp-add, mcp-pick-<d>, mcp-auth, mcp-finish, mcp-close), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
connector-setup, provider-keys, permission-rules.
