**Status:** draft v0.1 (2026-11-12).

# Spec: MCP server & plugin list (`mcp-list`)

Part of family J ([workspace-chrome.md](workspace-chrome.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **4 free + 12 Pro = 16 variants**. Owns the
**management surface** for MCP servers and plugins: one row per server with a status, transport and scope,
tool count, per-tool enable / disable, auth and retry, plus the compact **health chip** the status bar
and composer use. The quick "connectors" section inside the `+` menu is
[composer-tools-menu.md](composer-tools-menu.md) and reuses this row's status vocabulary; rules about
whether a tool call asks first are [permission-rules.md](permission-rules.md).

## 1. Purpose
Agents get their tools from MCP servers and plugins. The user wants to know **which servers are
connected, which need them (sign-in, approval), which are broken and why, how many tools each adds and
what those tools cost in context** — and to switch off a server or a single tool without editing a config
file. "MCP server list with status" is table-stakes (research note 03 §1H): Paradev `/mcp` "✓ connected"
with config path; OpenCode sidebar connection status + error indicator; Codex `/mcp` inventory with
connection states and masked secrets; Zed's warning icon / "No tools" for models that can't use a
server's tools; Conductor's MCP status indicator in the message box, tool count per server in the popover,
copyable OAuth callback URL and re-auth toasts with the server logo; Windsurf per-session MCP toggles;
Antigravity's "Add MCP Server" dialog with search (notes 03, 08).

### 1.1 The nine states — brief vs research
The brief proposed: disconnected, connecting, connected, auth needed, error, disabled, updating,
rate-limited, tools-changed. Research shows (note 05 §6): Claude Code `claude mcp list` prints
`✔ Connected`, `! Connected · tools fetch failed`, `! Needs authentication`, `✘ Failed to connect`,
`✘ Connection error`, `⏸ Pending approval`, `✘ Rejected`, `⊘ Disabled for this project`, plus `cached`; the
Claude Agent SDK uses pending / connected / failed / needs-auth / disabled, with auto-reconnect up to 5
attempts; Cursor shows a green / yellow / red dot, a **Needs login** badge with Connect, tool count,
per-tool toggles, logs and a soft cap of ~40 enabled tools ("Too many MCP tools enabled"); Arc UI's MCP
servers block shows Connected / Needs sign in / Error with Retry (note 12). Plugins add `update
available` and `config needed` (note 05). **No source in our notes shows "rate-limited" or
"tools-changed" as an MCP list state.** The MCP protocol itself lets a server announce that its tool list
changed (a list-changed notification; verify the exact name against the current MCP spec before
implementing), so "tools changed" is real but momentary.

**Decision (proposed):** nine **primary** connection states, each exclusive, plus **overlay flags** that
can sit on top. Rate-limited and tools-changed become flags because they happen *while connected*;
pending-approval and blocked-by-policy are promoted to primary because research shows them and they need
distinct actions.

| # | Primary state | Glyph + word | From research |
|---|---|---|---|
| 1 | `Disconnected` | ○ "Not connected" | idle / not started (SDK pending before start) |
| 2 | `Connecting{attempt}` | ◌ "Connecting…" / "Reconnecting 2/5" | SDK pending, auto-reconnect 5 (note 05), Cursor yellow |
| 3 | `Connected` | ● "Connected" | all |
| 4 | `NeedsAuth` | 🔑 "Sign in" | Claude "Needs authentication", Cursor "Needs login", Arc "Needs sign in" |
| 5 | `PendingApproval` | ⏸ "Approve server" | Claude "Pending approval" (project servers) |
| 6 | `Error{kind}` | ✕ "Failed to connect" / "Connection lost" | Claude failed / connection error, Arc Error, OpenCode error indicator |
| 7 | `Disabled{scope}` | ⊘ "Off" / "Off for this project" | Claude "Disabled for this project", SDK disabled |
| 8 | `Blocked{by}` | ⛔ "Blocked by policy" / "Rejected" | Claude "Rejected", Cursor team allowlisting (note 05) |
| 9 | `Updating` | ⟳ "Updating…" | plugins "update available" → updating (note 05); our proposal for the in-progress step |

| Overlay flag | Glyph | On which primary | Source |
|---|---|---|---|
| `ToolsFetchFailed` | ! amber | Connected | Claude "Connected · tools fetch failed" |
| `ToolsChanged{+a −r}` | ↻ "+2 −1 tools" for 8 s, then a dot until seen | Connected | MCP list-changed notification; note 08 shows ↻ for an MCP resource update in a glyph table |
| `RateLimited{retry_in}` | ⏱ "Rate limited · 30s" | Connected | **our proposal** (not observed) — shown only when the adapter reports it |
| `Cached` | ◷ "Cached" | Connected / Error | Claude `cached` |
| `UpdateAvailable` | ⬆ | Connected / Disabled | plugins (note 05) |
| `ConfigNeeded` | ⚙ "Needs configuration" | Disconnected | Claude plugins (note 05) |
| `UnsupportedByModel` | ⚠ "Not available with this model" | Connected | Zed "No tools" + warning (note 03) |
| `OverBudget` | amber context bar | list level | Arc budget overflow (note 12), Cursor ~40-tool cap |

## 2. Anatomy
```
┌ MCP servers ─────────────────────────────── 18 of 23 tools · 6.4k tok ▕██████▋  ▏ 8k ── [+ Add] ┐
│ ● GitHub        Remote · user      5 of 6 tools · 2.2k        [●━] ›                               │
│   ├ search_code        👁 Read    420   [━●]                                                     │
│   ├ create_issue       ✎ Write    420   [━●]                                                     │
│   └ merge_pull_request ✎ Write    360   [●━] off                                                 │
│   Before a tool runs: (Ask every time | Allow reads)       Logs · Copy config · Remove            │
│ 🔑 Notion       Remote · project   Sign in                                  [Sign in]            │
│ ◌ Postgres      Local (stdio)      Reconnecting 2/5 …                       [Cancel]             │
│ ✕ Sentry        Remote             Failed to connect · 401                  [Retry] [Logs]       │
│ ⏸ repo-tools    Local · project    Added by .mcp.json — approve?            [Approve] [Reject]   │
│ ● Linear  ↻ +2 tools               5 of 7 tools · 1.1k                      [●━] ›               │
│ ⊘ Figma         Off for this project                                        [━●]                 │
│ ⛔ shell-exec    Blocked by organisation policy                               (i)                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
 health chip (status bar / composer):  ⚡ 5/6   ·  ⚡ ! 1 needs sign-in   ·  ⚡ ✕ 1
```
- **Row:** server mark (from family L plugin/MCP icons; letter avatar fallback) · name · transport
  (`Local (stdio)`, `Remote` — the app supplies the label) · scope (`user`, `project`, `managed`; note 05)
  · status word · tools "5 of 6 tools" · context cost (only when the app supplies token counts) · master
  switch · expand ›.
- **Tool row:** name in mono, access label Read 👁 / Write ✎ / Destructive ⚠ when the server or app
  declares it, per-tool cost, switch. Write / destructive tools default **off** only if the app says so.
- **Secrets:** env values and headers in "Copy config" and logs are masked (`ghp_••••3f2a`), as Codex's
  inventory masks sensitive values (note 08).
- **Budget bar:** segmented per server; past the budget the bar rescales and the overflow turns amber
  (Arc idea, note 12); a soft warning at **40 enabled tools** (Cursor's soft cap, note 05) — the number is
  an app setting, default 40.

## 3. State machine
Per server:
```
                    enable / app start                         handshake ok + tools listed
 Disconnected ───────────────────────────▶ Connecting{1} ───────────────────────────────────▶ Connected
   ▲   ▲  │ project-scoped & unapproved          │  │ 401 / auth required                   │  │  │ list-changed
   │   │  ▼                                       │  ▼                                       │  │  ▼
   │   │ PendingApproval ─approve─▶ Connecting    │ NeedsAuth ─sign in ok─▶ Connecting       │  │ +ToolsChanged (8 s, then dot)
   │   │        └─reject─▶ Blocked{user}          │       └─cancel─▶ Disconnected          │  │ tools fetch fails ▶ +ToolsFetchFailed
   │   │                                          │ fail (attempt < 5) ─backoff 1,2,4,8 s─▶ Connecting{n+1}   429 / adapter says ▶ +RateLimited
   │   │                                          └ fail (attempt = 5) ─▶ Error{kind} ─retry─▶ Connecting{1}
   │   └──────────── disable (user / project) ◀── any ──▶ Disabled{scope} ─enable─▶ Connecting
   │                         policy denies (any time) ──▶ Blocked{policy}  (only an admin change leaves it)
   │                    Connected ─connection drops─▶ Connecting{1} ("Reconnecting") ─…
   │                    Connected | Disabled ─update─▶ Updating ─ok─▶ previous state ; ─fail─▶ Error{UpdateFailed}
   └── remove (Undo 6 s) ── any
 Per tool:  On ⇄ Off   (switch; Off tools are never offered to the model)   + Pending(write in flight) ─fail─▶ revert + toast
 List:      Idle ─Add─▶ AddSheet{Directory → Configure → Auth → Discovering → Done}  ·  Filtering{query}  ·  OverBudget
```
Backoff 1 / 2 / 4 / 8 s between the five attempts is our proposal; the attempt count of 5 is the Claude
SDK's documented auto-reconnect limit (note 05).

## 4. States table
| State | Shows | Primary | Secondary | Keyboard (row focused) |
|---|---|---|---|---|
| `Disconnected` | ○ "Not connected" | Connect | Remove, Edit config | Enter connect |
| `Connecting{n}` | ◌ loader, "Connecting…" (> 300 ms) / "Reconnecting 2/5" | — | Cancel | Esc cancel |
| `Connected` | ● + "5 of 6 tools" + cost | expand tools | master switch, Logs, Copy config, Remove | Enter expand, Space toggle server |
| `NeedsAuth` | 🔑 "Sign in" in amber | **Sign in** (opens browser / OAuth; callback URL copyable, Conductor idea) | Remove | Enter |
| `PendingApproval` | ⏸ "Added by .mcp.json — approve?" + command / URL preview | **Approve** | Reject, View config | Enter approve (focus starts on View config for local commands) |
| `Error{kind}` | ✕ word + short reason (`401`, `ENOENT npx`, `timed out`) | **Retry** | Logs, Edit config | Enter retry, L logs |
| `Disabled{scope}` | ⊘ dimmed row "Off" / "Off for this project" | switch on | scope menu | Space |
| `Blocked{by}` | ⛔ "Blocked by organisation policy" / "Rejected by you"; no switch | (i) why | Undo reject (user only) | Enter opens why |
| `Updating` | ⟳ "Updating to 1.4.0…" | — | — | — |
| +`ToolsFetchFailed` | ! "Tools couldn't be listed" | Retry tools | Logs | R |
| +`ToolsChanged` | ↻ "+2 −1 tools" chip; new tools land **off** if `new_tools_default = Off` | review | dismiss | Enter review |
| +`RateLimited` | ⏱ "Rate limited · retry in 30s" (countdown) | — | Logs | — |
| +`UpdateAvailable` | ⬆ "Update" | Update | release notes | U |
| +`ConfigNeeded` | ⚙ "Needs configuration" | Configure | — | Enter |
| +`UnsupportedByModel` | ⚠ "Not available with Opus" | switch model | — | — |
| List `OverBudget` | amber bar overflow "1.2k over" | Review tools (sorts by cost) | — | — |
| Tool `On / Off` | switch | toggle | — | Space |
| Health chip | `⚡ 5/6` neutral; `⚡ ! 1` amber (auth/approval); `⚡ ✕ 1` danger | open list filtered to problems | — | Enter |

## 5. Micro-interactions
- **Status change.** Glyph morphs (`spring.snap` 160 ms); status word and action pill crossfade ~100 ms
  out / 160 ms in (Arc's status crossfade numbers, note 12 — study only, re-implemented); no per-letter
  motion.
- **Connecting.** Loader appears only after 300 ms; reconnect attempts roll their counter "2/5".
- **Connected arrival.** One 400 ms success tint on the glyph, then neutral; the tool count rolls up as
  tools are listed (tabular digits).
- **Tool discovery** (Add flow and `ToolsChanged`): "Discovering tools" in [text-shimmer.md](text-shimmer.md)
  settles to "Found 5 tools"; tool rows arrive one by one with a 4 px rise + fade, 30 ms stagger cap 8.
- **Tool toggle.** Switch thumb slides 120 ms; the server's "5 of 6" and the cost roll; the server's
  segment in the budget bar resizes with `spring.snap`; hovering a server highlights its segment and dims
  others.
- **Expand.** Tool list height-opens with `spring.snap` + 120 ms fade; chevron rotates 90°.
- **Add server.** Sheet rises from the list's bottom edge over a scrim, no overshoot; steps crossfade with
  a 16 px shift in the direction of travel; the new row slides into the top of the list with a brief accent
  tint.
- **Remove.** Row collapses (160 ms); Undo toast restores it in place.
- **Error.** Glyph nudge 2 × 3 px once; never repeated on each failed retry (only the counter rolls).
- **Health chip.** Count rolls; changing tone ramps 400 ms; one pulse when a server newly needs the user.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Status glyph morph | 160 ms | 0 | `spring.snap` | — | 60 Hz |
| Status word / action crossfade | 100 ms out / 160 ms in | 0 | `ease.in` / `ease.out` | — | 60 Hz |
| Connecting loader show | 120 ms | 300 ms | `ease.out` | — | loader rate |
| Reconnect backoff | 1, 2, 4, 8 s | — | — | — | — |
| Connected tint | 400 ms | 0 | `ease.out` | — | 30 Hz |
| Count / cost roll | 280 ms | max 2 Hz | `ease.out` | 30 ms per digit | 30 Hz |
| Tool discovery rows | 160 ms, 4 px rise | after shimmer | `ease.out` | 30 ms, cap 8 | 60 Hz |
| Switch thumb | 120 ms | 0 | `ease.out` | — | 60 Hz |
| Budget segment resize | ~200 ms | 0 | `spring.snap` | — | 60 Hz |
| Expand / collapse tools | ~200 ms / 140 ms | 0 | `spring.snap` / `ease.in` | — | 60 Hz |
| Add sheet in / step change | 220 ms / 160 ms (16 px) | 0 | `ease.firm` / `ease.out` | — | 60 Hz |
| New row slide + tint | 200 ms + 600 ms tint | 0 | `spring.snap` / `ease.out` | — | 60 Hz |
| ToolsChanged chip hold | 8 s, then dot | — | — | — | — |
| Remove collapse / Undo window | 160 ms / 6 s | 0 | `ease.in` | — | 60 Hz |
| Error nudge | 300 ms (2 × 3 px) | after morph | linear | — | 60 Hz |
| **Reduced motion** | glyphs and words swap with an 80 ms crossfade; no rises, slides, tints, nudges or shimmer ("Discovering tools…" static text); heights instant; counts update without roll | — | linear | none | — |

## 7. Variants
### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `mcp-list/rows` | Baseline settings list (anatomy above): rows, expand to tools, per-tool switches | desktop, ide |
| `mcp-list/popover` | Compact popover from the health chip: one line per server with status + tool count, problems first (Conductor idea, note 08) | desktop, chat |
| `mcp-list/chip` | Health chip only `⚡ 5/6` with tone and count of problems; opens the list | desktop, ide, chat |
| `mcp-list/tui` | `/mcp`-style inventory: `✔ github  5/6 tools  stdio  user` with numbered actions (Codex / Claude CLI look, notes 05, 08) | terminal |

### Pro (12)
| Name | One-line idea |
|---|---|
| `mcp-list/budget-bar` | Segmented context-budget bar per server with hover highlight and overflow in amber |
| `mcp-list/tool-matrix` | Grid of servers × tools with read/write/destructive columns and bulk toggles |
| `mcp-list/directory` | Add-server sheet with searchable directory, OAuth step and live tool discovery |
| `mcp-list/per-session` | Session-header strip of server toggles that apply only to this thread (Windsurf idea, note 08) |
| `mcp-list/health-timeline` | 24 h strip per server: connected / error / reconnect segments |
| `mcp-list/latency-dots` | Per-server latency dot and last-call time from adapter metrics |
| `mcp-list/plug-glyphs` | Animated plug glyph per state: connected, needs-auth (plug + key), failed (plug + ×), pending (plug + clock) (note 05 icon set) |
| `mcp-list/profiles` | Named tool profiles ("Read-only", "Full") switching many toggles at once (Zed agent profiles idea, note 03) |
| `mcp-list/diff-on-change` | ToolsChanged opens a mini diff of the tool list (added / removed / schema changed) |
| `mcp-list/marketplace-cards` | Plugin cards with update badge, scope and "Needs configuration" setup flow (note 03/05) |
| `mcp-list/reauth-toasts` | Re-auth toasts carrying the server mark with a one-click Sign in (Conductor idea, note 08) |
| `mcp-list/orbit` | Servers as small marks orbiting the agent mark; distance = latency, tone = state |

## 8. Edge cases
- **Many servers / tools** (30 servers, 500 tools): list virtualised; tools within a server search-filter
  after 8 items; problems sort first; the 40-tool soft warning links to "Review tools by cost".
- **Duplicate names across scopes** (user and project both define `github`): show both rows with scope
  tags; the effective one has a "used" badge and the shadowed one says "Overridden by project".
- **Long names / URLs**: name 28 ch end-truncate; endpoint middle-truncate; full value in tooltip.
- **Auth in a browser that never returns**: `NeedsAuth` with "Waiting for sign-in…" for 120 s, then back
  to "Sign in" with "Didn't hear back · Try again"; copyable callback URL for manual completion.
- **Server flaps** (connect / drop every few seconds): after 3 drops in 60 s, stop auto-reconnect and show
  `Error{Unstable}` with Retry, so the row stops flickering.
- **Tools changed mid-run**: the change applies to the **next** model call; the running turn keeps its
  tool list; the chip says "Applies next turn".
- **Disabled tool requested by the agent** (stale plan): the tool call shows "Tool is off · Turn on"
  linking to this row (family F tool row).
- **Policy-blocked server in a project file**: `Blocked{policy}` with the policy name; no way to enable
  from here.
- **Local server command missing** (`npx` not found): `Error{SpawnFailed}` with the exact missing binary.
- **Secrets**: never shown unmasked; Copy config copies masked values unless the app allows reveal with
  OS auth.
- **RTL**: rows mirror; tool names and URLs stay LTR in bidi isolates; switches mirror.
- **Tiny sizes**: chip at 16 px = ⚡ glyph with a tone dot; popover rows 28 px.
- **Dense IDE**: 24 px rows, cost column hidden by default.
- **Touch**: 44 px rows; swipe to reveal Remove; switches 44 px target.
- **Slow networks**: Connecting shows elapsed after 5 s ("Connecting… 12s"); remote status older than
  30 s is marked stale.
- **Very fast connects** (< 300 ms): no loader; direct morph to Connected.

## 9. Accessibility
- List: `role="list"` of server rows; each row is a group with name "GitHub, connected, 5 of 6 tools
  enabled, user scope"; the expand button has `aria-expanded`; tool switches `role="switch"` named "Enable
  merge_pull_request, write tool".
- Master switch: "Enable GitHub server"; disabled-with-reason switches expose the reason via
  `aria-describedby`.
- Live region: polite for connected / disconnected / tools changed ("Linear added 2 tools"); assertive
  for **NeedsAuth during a run** and **PendingApproval** only. Reconnect attempts are not announced.
- Health chip: button "MCP servers, 5 of 6 connected, 1 needs sign-in".
- Contrast: every state = glyph + word (never colour alone, as in Arc's status design, note 12); text
  ≥ 4.5:1; budget bar segments ≥ 3:1 against the track.
- Keyboard: §4 per state; `/` focuses the list filter; Esc closes the popover and returns focus to the
  chip.
- Reduced motion: §6 last row.

## 10. Props sketch
```rust
pub struct McpList<'a> {
    pub servers: &'a [McpServer],
    pub variant: McpListVariant,            // Rows | Popover | Chip | Tui | Pro(..)
    pub budget_tokens: Option<u32>,         // e.g. 8_000; None hides the bar
    pub soft_tool_cap: Option<u32>,         // 40 (Cursor's soft cap as default)
    pub new_tools_default: ToolDefault,     // On | Off
    pub reconnect: ReconnectPolicy,         // attempts 5, backoff [1,2,4,8] s, flap_limit 3 / 60 s
    pub agent: &'a AgentState,              // "applies next turn" while a run is live
    pub model_supports_tools: bool,         // UnsupportedByModel flag
    pub on_toggle_server: Callback<(ServerId, bool, Scope)>,
    pub on_toggle_tool: Callback<(ServerId, ToolName, bool)>,
    pub on_connect: Callback<ServerId>,
    pub on_sign_in: Callback<ServerId>,
    pub on_approve: Callback<(ServerId, bool)>,
    pub on_update: Option<Callback<ServerId>>,
    pub on_remove: Callback<ServerId>,
    pub on_open_logs: Callback<ServerId>,
    pub labels: McpLabels,
    pub motion: MotionLevel,
}

pub struct McpServer {
    pub id: ServerId,
    pub name: SharedString,
    pub mark: Option<IconRef>,              // family L plugin / MCP icons
    pub kind: ServerKind,                   // Mcp | Plugin
    pub transport_label: SharedString,      // app-supplied ("Local (stdio)", "Remote")
    pub scope: Scope,                       // User | Project | Managed
    pub state: McpState,                    // Disconnected | Connecting{attempt} | Connected | NeedsAuth | PendingApproval
                                            // | Error{kind, detail} | Disabled{scope} | Blocked{by} | Updating{to}
    pub flags: McpFlags,                    // tools_fetch_failed, tools_changed{added, removed}, rate_limited{retry_in},
                                            // cached, update_available, config_needed, unsupported_by_model
    pub tools: Vec<McpTool>,                // { name, access: Read|Write|Destructive|Unknown, tokens: Option<u32>, enabled }
    pub endpoint_masked: Option<SharedString>,
}
```

## 11. Related components and references
**Related:** [composer-tools-menu.md](composer-tools-menu.md) (connectors section, same status words) ·
[permission-rules.md](permission-rules.md) ("Ask every time / Allow reads" writes rules there) ·
[approval-card.md](approval-card.md) (server approval can reuse its footer) ·
[composer-context-meter.md](composer-context-meter.md) (MCP tools category) ·
[side-panels.md](side-panels.md) (status bar health chip, MCP pane) · [tool-call.md](tool-call.md)
("Tool is off") · [text-shimmer.md](text-shimmer.md) · family L plugin / MCP icons.

| Reference | What we take | Licence note |
|---|---|---|
| Claude Code `claude mcp list` / SDK statuses (note 05) | state vocabulary, 5 reconnect attempts, scopes, cached | Proprietary — reference only |
| Cursor Tools & MCP (note 05) | dot tones, Needs login, per-tool toggles, ~40 tool soft cap, logs | Proprietary — reference only |
| Arc UI MCP servers block (note 12) | budget bar, read/write tools, 100/160 ms status crossfade, discovery shimmer, sheet | Paid / unclear — study only |
| Codex CLI `/mcp` inventory (note 08) | connection states, masked secrets | MIT / Apache-2.0 — portable with attribution (re-check) |
| Conductor (note 08) | MCP status in message box, tool count in popover, copyable OAuth callback, re-auth toasts with logo | Proprietary — reference only |
| Windsurf, Antigravity, Zed, Paradev, OpenCode (notes 03, 08) | per-session toggles; Add dialog with search; "No tools" warning; config path; sidebar status | Windsurf / Antigravity proprietary; Zed study only; Paradev not checked; OpenCode MIT |
| Research note 05 MCPServerChip / PluginRow proposal | 9 statuses × 2 forms, plugin update / config states, plug glyphs | Our own proposal |
| MCP protocol list-changed notification | ToolsChanged flag | Open protocol; behaviour only — verify name/version before implementing |
