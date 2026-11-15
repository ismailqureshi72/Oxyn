**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Run controls (`run-controls`)

Family: Composer ([composer-send-control.md](composer-send-control.md)). Closes the parity row(s): affectUI · Run Controls (with send menus). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
Every control a running agent needs, in one composer. The agent's 5×5 pixel mark comes from its id (`pixel_mark`). Running shows the streaming answer with a caret, a Working shimmer and Stop; stopped and error show Resume (from the character it stopped at), Retry and Dismiss; done shows "Done in 0:14" and Retry. While running, Enter steers: the note is drawn into the answer at the character it landed on (`splice`); ⌥ Enter queues: messages wait above the composer, × removes one, and once the run is done the head gets Run next. The send button's chevron opens a send menu with the same three choices (Send now, Steer this run, Queue for later). Chat only answers; Act may change things and carries the permission menu (Ask first, Auto-edit, Full access in red). The context ring turns amber from 80 %.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| idle | Quiet start | send |
| running | Caret, Stop | stop / done |
| stopped / error | Resume, Retry, Dismiss | — |
| done | Done in m:ss, Retry | — |

## 3. Timing and motion
- Answer streams on the clock; steer chips fade in 200 ms.
- Reduced motion stops the shimmer and caret blink.

## 4. Variants
| Variant | What differs |
|---|---|
| running | Streaming with a queue |
| stopped | Stopped |
| full | Full access, high context |
| chat | Chat mode |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter | Send / steer |
| ⌥ Enter | Queue |
| Shift + Enter | New line |

## 6. Accessibility
- The run area is aria-live; Mode is a radiogroup; menus use menuitemradio.

## 7. Responsive
- Up to 560 px.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| agent / agent_id | String | Agent / agent |  |
| prompt / answer | String | — |  |
| status | idle · running · stopped · error · done | idle |  |
| queue | Vec<String> | — |  |
| mode | chat · act | act |  |
| permission | ask · edits · full | ask |  |
| context | f32 | 0.42 |  |
| placeholder | String | Ask the agent… |  |
| now | ClockMs | — |  |
| on_action | EventHandler<(String, String)> | — | send, steer, queue, unqueue, stop, resume, retry, dismiss, mode, permission |

## 9. Gaps against the source
- The answer is simulated on the clock; a host passes its own stream by re-rendering `answer`.

## 10. GPUI
RunControlsEl composes QueueStack, AgentSteering, ModeSwitch and PermissionSelector in GPUI.
