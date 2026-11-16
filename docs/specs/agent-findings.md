**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B).

# Spec: Agent Findings (`agent-findings`)

Family: Agents. Closes the parity row: affectUI · Agent Findings. Behaviour-only rebuild; nothing is copied.
Components: `AgentFindings` (Dioxus) · `AgentFindingsEl` (GPUI), models in `oxyn_core::parity::agentsfid`.

## 1. Purpose
Follow a swarm of agents that check, audit or verify something in parallel, without drowning in detail: one row per agent and a short feed of what they found.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Card | 480 px, 12 px radius, hairline |
| Bar | 48 px under a hairline: blue sparkle, title 15/600, "25 of 64 checks" grey; right: "1 critical" and "2 warnings" with a dot – grey at 0, red / amber above |
| Agent row | 44 px, 10 px radius: 24 px tile with the coloured model mark, name 14/600 over the current step 13 grey, a 96 × 4 px progress bar (blue fill on a grey track), "8/24", two counters with a 2×2 severity mark (red, amber, faded at 0) |
| Feed | "LATEST FINDINGS" 11 px uppercase tracked (+ an "Auditor ×" chip when filtered); rows 36 px: severity mark, title 13.5/600, agent · location (mono), age ("now", "5s", "2m"); at most `rows` (4), newest on top |

## 3. Behaviour
- Each agent ticks through its checks at its own pace and its step follows; findings land at their time and slide in at the top.
- Pressing an agent shows only its findings (others fade to 38%, chip appears); again shows all.
- Pressing a finding selects it and calls on_open. The demo loops 5 s after the last agent finishes.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Live run |
| finished | All agents done |
| filtered | One agent's findings |
| classic | Softer card, the feed on a grey band, title-case label, monochrome bars |

## 5. Keyboard and accessibility
Tab moves through agents and findings; agents are aria-pressed toggles; the feed is a polite live region; each severity has a text label.

## 6. Responsive
Below 480 px the check counts and locations drop (rows keep one line); below 340 px the counters and agent names in the feed drop.
