**Status:** built v0.2 (2026-11-15, plan #32 affectUI fidelity pass, chunk 2; v0.1 plan #28 is kept as the Oxyn extra variant).

# Spec: Run controls (`run-controls`)

Family: Agents. Closes the parity row: affectUI · Run Controls. Behaviour-only rebuild; nothing is copied.
Components: `AgentRunPanel` (Dioxus) · `AgentRunPanelEl` (GPUI), models in `oxyn_core::kit::chatfid`.
The plan #28 `RunControls` (send menu with Send now / Steer / Queue, error state, amber context) stays
as the **Oxyn extra** variant `oxyn-run-controls` on the same page.

## 1. Purpose
Stop, steer and queue a running agent from one card: the run above, the queue band, the composer
below. Matches affectUI's Run Controls at its own size (520 px card).

## 2. Parts
| Part | Look (Clean) | Behaviour |
|---|---|---|
| Run | 250 px, scrolls inside (pinned to the newest line), padding 18/16 | A polite log, `aria-busy` while streaming |
| Turn | Optional "you" bubble (right, 14 px radius), header: blue 3×3 pixel mark 14 px, agent name 13/600, status 12.5 grey | Thinking… → Writing → Writing · steered → Stopped / Done |
| Answer | 13.5 px / 20 px, indented 40 px from the card, paragraphs 20 px apart | Words stream every 55 ms after 900 ms of thinking; a 7×14 block caret blinks at the end |
| Steer pill | 22 px, full radius, blue tint (`rgba(59,130,246,.18)` dark, `#DDEBFE` light), curved-arrow icon | Lands mid-sentence where it was sent; the answer continues after it |
| Footer | "Esc to stop" with a kbd chip while running; "Stopped at N words" + **Resume** (filled pill) · Retry · Dismiss; "Done in 0:05" + Retry | |
| Queue band | 40 px rows on a slightly lighter band: list icon, text, "Next" on the first, ✕ | Runs when the turn is done (Act mode) |
| Composer | Inset card (4 px), 13 px radius; textarea; Chat \| Act segmented; permissions button; context ring 14 px + %; round 30 px send / stop | Placeholder "Ask the agent…" idle, "Steer the agent, or queue what's next…" running |
| Permissions menu | 268 px, opens upward: Ask first · Auto-edit · Full access (red bolt) with descriptions and a check | Disabled (dimmed, reason in the title) in Chat mode |

## 3. States
| State | Looks | Leaves on |
|---|---|---|
| idle | Centred pixel mark and "Ask … anything – steer it or stop it while it works." | send |
| thinking | Header "Thinking…", caret only | first word |
| writing | Words, caret, "Esc to stop", stop button | stop / done |
| steered | "Writing · steered", blue pill | done |
| stopped | Frozen text with caret, Resume / Retry / Dismiss | Resume (continues from the word), Retry (restarts), Dismiss (hides the row) |
| done | "Done in m:ss", Retry; the queued message starts the next turn | send, Retry |

## 4. Timing and motion
- Thinking 900 ms; one word per 55 ms; context grows 0.12 % per word.
- Turns and queue rows fade up 3 px over 240 ms; the steer pill pops (scale .8 → 1, 260 ms); status words blur in.
- Caret blinks 1 s steps. Reduced motion: no blink, no pops.

## 5. Variants
| Variant | What differs |
|---|---|
| running | Streaming with a steer and a queued message (default) |
| stopped | Stopped at word 29 |
| done | Finished: Done in 0:05 |
| permissions | Idle with the permissions menu open |
| chat-mode | Chat selected: permissions disabled |
| classic | Classic: the run as a white panel on a grey tray, the queue on the tray, 22/16 px radii |
| oxyn-run-controls | Oxyn extra: the plan #28 run composer |

## 6. Keyboard and accessibility
- Enter sends when idle and steers while running; ⌥ Enter queues; Shift+Enter new line; Esc stops (or closes the menu).
- Mode is a `radiogroup`; the permissions button is `aria-disabled` in Chat; the menu is a `listbox`.
- A status region reads "Release writer: Writing · steered, 1 queued".

## 7. Responsive
Up to 520 px; under 440 px the composer's controls wrap and the answer drops its indent.
