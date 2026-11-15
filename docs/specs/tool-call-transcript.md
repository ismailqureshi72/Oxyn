**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Tool call (`tool-call-transcript`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Arc UI · Tool call.

## 1. Purpose
Tool call rows for agent transcripts. Each row shows the tool's mark (from the tool-kind map; MCP names keep their server), the call in plain words with a phrase per status (running uses `running_title` or the map's present verb and shimmers), quiet meta (results / rows derived from the output) and the duration, and its status. A call awaiting approval sits on a warning outline with a message (“Posts a 6-line summary…”) and Deny / Allow; an error shows the reason and Retry; denied reads Denied. Input (mono key/value list) and Output (numbered list with tags, table, tinted JSON, or text; lists and tables fold after five rows with Show n more) are a click away. `ToolCallList` wraps the rows: “Using tools…” (shimmering) while the turn is active, “Used 4 tools · 2.3s” after, the marks stacked, and it folds; the demo folds it 450 ms after the last call settles unless someone toggled (autoCollapse).

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| running | Spinner, shimmering phrase | → success / error |
| approval | Warning outline, message, Deny / Allow | allow → running; deny → denied |
| success | Check, meta, duration |  |
| error | Reason + Retry | retry → running |
| denied | Denied |  |

## 3. Timing and motion
- Rows stagger in 20 ms apart with a 5 px settle; the I/O well rises in.
- Reduced motion: no entrance rise or row stagger, no shimmer (static muted text), chevrons and the card morph jump to their end state.

## 4. Variants
| Variant | What differs |
|---|---|
| approval | Waiting for Allow |
| settled | Folds itself |
| error | Retry |
| table | Table output |
| running | Using tools… |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Group header, rows, Allow / Deny / Retry, show more |
| Enter / Space | Open the group or a row |

## 6. Accessibility
- Rows are buttons with aria-expanded; the input is a dl; messages are text.

## 7. Responsive
- Titles truncate; the I/O well indents under the mark.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| tool | String | — | raw name |
| title / running_title | String / Option<String> | — |  |
| status | CallStatus | — |  |
| message | Option<String> | None |  |
| input | Vec<(String, String)> | [] |  |
| output | Option<RowOutput> | None | List · Table · Json · Text |
| duration_ms | Option<u64> | None |  |
| open | bool | false |  |
| on_toggle / on_allow / on_deny / on_retry | EventHandler<()> | — |  |
| ToolCallList · tools / total_ms / active / open / on_open / children | — | — |  |

## 9. Edge cases
- Without a running title the map's present verb is used (“Searching…”).
- Custom output renderers: pass your own element as a row's sibling (renderOutput has no direct prop yet).

## 10. Implementation
- Core: `ToolKindMap`, `CallStatus`, `ms_label` (`oxyn_core::parity::tools`), with tests.
- Dioxus: `ToolCall`, `ToolCallList` (parity/tools.rs, tools.css).
- GPUI: `ToolCallEl` (`toggle`, `allow`, `deny`, `retry`).

## 11. References
- Arc UI Tool call (https://uiarc.dev/components/tool-call) — behaviour-only rebuild, no source copied.
