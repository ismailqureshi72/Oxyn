**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Tool-kind map (`tool-kind-map`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): affectUI · Tool Call Card (icons); Arc UI · Tool call (marks).

## 1. Purpose
One map from raw tool names of any agent (Claude Code `MultiEdit`, Codex `apply_patch` / `shell`, OpenCode `todowrite`, Cline `execute_command`, MCP `mcp__server__tool`) to a `ToolKind`, an icon and present / past verbs (Reading / Read), with exact-name and prefix overrides a host adds (`with`, `with_prefix`). MCP names split into server and tool. Every tool component reads it.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| — | Pure lookup |  |

## 3. Timing and motion
- Reduced motion: no entrance rise or row stagger, no shimmer (static muted text), chevrons and the card morph jump to their end state.

## 4. Variants
| Variant | What differs |
|---|---|
| claude-code | Claude Code names |
| codex | Codex |
| mcp | MCP |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Read-only |

## 6. Accessibility
- A plain table.

## 7. Responsive
- Scrolls sideways when narrow.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| names | Vec<String> | — | for ToolKindLegend |

## 9. Edge cases
- Names are matched case-insensitively; unknown names fall back to `ToolKind::from_name` and then Tool.

## 10. Implementation
- Core: `ToolKindMap`, `ToolInfo` (`oxyn_core::parity::tools`), with tests.
- Dioxus: `ToolKindLegend` (parity/tools.rs, tools.css).
- GPUI: Uses the same core map; no separate element.

## 11. References
- Oxyn — consolidates the tool icon and verb rules referenced by the affectUI and Arc UI rows.
