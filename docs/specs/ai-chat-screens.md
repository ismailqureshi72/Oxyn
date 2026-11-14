# Spec: AI Chat screens (Start · Overview · Agents · Double sidebar · Progress · Run · Agent thread)

**Status:** v1 — implemented (plan #27, chunk 4) in `oxyn-core::shell::Screen`, `oxyn-dioxus::ChatShell`
(styles in `SHELL_CSS`, bundled into `PRIMITIVES_CSS`) and `oxyn-gpui::ChatShellEl`. The screens are
compositions of existing parts; the shell only owns the frame.

Behaviour follows affectUI's AI Chat screens (Pro), rebuilt from their public pages only; no source
was read or copied.

## 1. Purpose
Seven ready-made screens for an agent app, so a team starts from a whole page rather than parts:

| Screen | What the user is doing | Parts |
|---|---|---|
| Start | Opening the app with nothing running | `StarterPrompts` (greeting, starters, composer) |
| Overview | Glancing at every agent and recent thread | `StatusIndicator`, thread rows, a composer |
| Agents | Talking to an agent while its sub-agents work | `MessageBubble`, `StreamingMarkdown`, `SubagentTree` side panel |
| Double sidebar | Switching apps and threads | App rail + thread list, `MessageBubble`, `StreamingMarkdown` |
| Progress | Following a long task | `StreamingMarkdown`, `TodoList` side panel |
| Run | Watching one run and approving its next step | `PlanCard`, `ToolStack`, `ApprovalCard` |
| Agent thread | Reading one agent's thread with tool calls | `MessageBubble`, `ToolCallRow`, `StreamingMarkdown` |

## 2. Anatomy
```
┌ app rail (Double sidebar only) ┬ thread list ───────┬ main column ─────────────┬ side panel ┐
│ logo · Chats · Agents · …      │ New chat           │ header (title · share)   │ Agents /   │
│                                │ threads (title,    │ body (the screen's parts)│ Progress   │
│                                │ meta) · active     │ composer                 │            │
└────────────────────────────────┴────────────────────┴──────────────────────────┴────────────┘
```

## 3. State machine
The frame keeps one piece of state, the active thread. Everything else belongs to the parts.
```
thread i ── click / Enter ──▶ thread j  (aria-current moves; host loads j)
```

## 4. States table
| Region | State | Shows | ARIA |
|---|---|---|---|
| Thread list | active | raised row | `aria-current="page"` |
| Main | streaming | the answer grows with a caret | polite (part) |
| Side | present | Agents / Progress | `complementary`, labelled “Details” |
| Run | awaiting approval | approval card | the card's own keys |

## 5. Micro-interactions
Thread rows tint on hover and raise when active. The streamed answer, plan ticks, tool rows and the
approval card keep their own motion (see their specs).

## 6. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Row hover / active | `--ox-fast` | linear | instant |
| Answer stream (demo) | 60 chars/s | — | full text |

## 7. Variants
Free: Start, Overview, Agents, Double sidebar, Progress, Run, Agent thread (one per screen).

**Oxyn plus:** the side panel takes any element; the rail and the thread list are independent; “Show
finished” jumps every stream to its end.

## 8. Edge cases
- No threads: the thread list is hidden and the main column fills the frame.
- No title: the header is hidden (Start).
- Narrow frames: the side panel and list keep their widths; the main column has `min-width:0` and its
  parts wrap or scroll.

## 9. Accessibility
Navigation landmarks (“Apps”, “Threads”), a `main`, and a complementary “Details”. Thread rows are
buttons with `aria-current`. Parts keep their own live regions and keys.

## 10. Props sketch
```rust
ChatShell { rail: bool, threads: Vec<(String, String)>, active: usize, title: String,
            side: Option<Element>, height: f32, children: Element }
ChatShellEl::new().rail(true).threads(th, 0).title("Run 248").main(el).side(el)
```
Related: `chat-thread`, `starter-prompts`, `plan-todo`, `approval-card`, `tools`, `subagent-tree`,
`workspace-chrome`, `side-panels`.
