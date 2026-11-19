**Status:** built v0.2 (2026-11-20, S15: full affectUI Agent Trail spec; v0.1 2026-11-14, plan #28).

# Spec: Agent trail (`agent-trail`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): affectUI · Agent Trail (https://www.affectui.com/components/agent-trail), behaviour-only rebuild, no source copied.

## 1. Purpose
The steps an agent works through as one quiet trail, for a chat answer, a side panel or a task card: what is done, what it is doing now and what comes next, and a stop that needs your approval coming into view before it is reached.

## 2. Parts and states
- **Header**: the agent's official coloured mark (or its initial) and name, the task (ellipsised, full text on hover) and the count `3/5`, read as "3 of 5 steps done". No title, no header.
- **Rail**: one hairline through the step marks; a short **fading light** runs down it from the top into the running sub-step (or the running step), 1.8 s, looping.
- **Step row** (fixed 32 px; 40 px on touch): mark, the tool's official coloured logo at 14 px (faded while the step is still to come), title, and on the right (never more than 40%) the running step's shimmering status and live timer or a settled step's time.

| State | Mark | Right | Title |
|---|---|---|---|
| pending | hollow ring, faded | — | faded |
| pending + gate | **dashed orange ring**, `Approval` tag (dashed) | — | faded |
| running | spinning ring | status shimmer + live timer (`<1s`, `12s`, `1m 4s`) | |
| waiting | orange disc with pause, **Needs you** | — | |
| done | tinted check | time | |
| failed | red ×, `Failed` | time | red |
| declined | grey ban, `Declined` | — | **struck through**, muted |

- **Sub-steps** (24 px rows, a branch from the rail): mark, title, a short fact on the right that keeps its end (`src/…CartWrapper.tsx:1284`, `182/214`). The running step opens by default; any step with sub-steps opens with a click. More than six fold to five around the current one with **+N earlier** and **+N more**; either shows them all, then **Show fewer**.
- **Needs you**: with `on_approve` or `on_decline`, the tag is a button that opens a panel under the row with the step's `ask` ("Open PR #482 against main – 3 files, 214 tests passing") and **Approve** (focused) / **Decline**. Escape closes it.
- **Empty**: the header stays and "No steps yet" takes the list's place; a step without a title reads "Untitled step".

## 3. Timing and motion
| Motion | Timing | Reduced motion |
|---|---|---|
| Light down the rail | 1.8 s cubic-bezier(.45,0,.3,1), loop, fades in and out | still, at the target |
| Running ring | 0.9 s linear spin | 3 s |
| Status shimmer | 1.6 s linear | static muted text |
| Live timer | ticks with the host clock | same |

## 4. Variants
| Variant | What differs |
|---|---|
| plain | Default: header rule, rows, rail |
| card | A bordered, raised card |
| inline | No header rule; pairs with `dense` for chat answers |
| dense | 28 px step rows |
| rtl | The rail moves to the right edge and the branch bends the other way; times and details keep their own direction |

Gallery variants: running, needs-you, declined, failed, folded (12 sub-steps), card (all done), inline dense, rtl, empty. Controls: Stage (running, waiting, failed, declined, done, empty), Sub-steps (short, long), Variant, Direction, Show sub-steps, Dense.

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Each step with sub-steps, +N earlier / +N more / Show fewer, Needs you |
| Enter / Space | Opens or closes a step's sub-steps; unfolds every sub-step |
| Enter on Needs you | Opens Approve and Decline; focus lands on Approve |
| Escape | Closes the answer panel |

## 6. Accessibility
- An ordered list named after the task; the running step and its running sub-step carry `aria-current="step"`.
- The header count reads "3 of 5 steps done"; each row reads "Read the bug report – Done, 8s"; a step with sub-steps toggles them and reports `aria-expanded`.
- Times, badges and details read in their own direction (`<1s` stays `<1s` in RTL).
- A polite live region says "Approved: …" or "Declined".
- The focus outline appears only when moving by keyboard (`:focus-visible`), in light and dark.

## 7. Responsive
Rows are fixed, so a changing status, timer or detail never moves another row; long titles end in … with the full text on hover; long sub-step lists fold to five around where the agent is.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| trail | Trail | — | title, agent, steps: `TrailStep { title, state, status, started, took_ms, gate, tool, sub, details, ask }` |
| now | ClockMs | — | live timers |
| substeps | bool | true | show sub-steps |
| dense | bool | false | 28 px rows |
| variant | String | `plain` | `plain` · `card` · `inline` |
| rtl | bool | false | right to left |
| on_approve | Option<EventHandler<usize>> | — | a waiting step approved; call `Trail::approve(i, now)` |
| on_decline | Option<EventHandler<usize>> | — | a waiting step declined; call `Trail::decline(i)` |

## 9. Edge cases
- Durations format as `<1s`, 12s, 1m 4s, 1h 2m, 3d 4h.
- No steps; an untitled step; a sub-step list of 12 around the 9th.

## 10. Implementation
- Core (`oxyn_core::parity::plan`): `Trail::{count, count_label, current, step_label, approve, decline}`, `TrailStep::{detail, current_sub, badge}`, `fold`, `keep_end`, `trail_time`, with tests.
- Dioxus: `AgentTrail` (parity/run.rs, `oxt-` styles in run.css).
- GPUI: `AgentTrailEl` — header with the agent's mark, dashed gate ring, coloured tool logos, struck-through declined steps, the folded sub-steps and an Approve / Decline panel under a waiting step that reports `trail-approve` / `trail-decline` through `on_act`. GPUI has no RTL and no moving light yet.

## 11. References
- affectUI Agent Trail (https://www.affectui.com/components/agent-trail) — behaviour-only rebuild, no source copied.
