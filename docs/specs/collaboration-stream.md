**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 6).

# Spec: Collaboration Stream (`collaboration-stream`)

Family: Agents. Closes the parity row: affectUI · Collaboration Stream (Pro). Behaviour-only rebuild; nothing is copied
(the run, agent names, messages and files in the demo are our own). Related Oxyn components: subagent-tree (who started
whom) and messages.
Components: `CollabStream` (Dioxus) · `CollabStreamEl` (GPUI), model in `oxyn_core::parity::collabfid`.

## 1. Purpose
What agents say to each other: a quiet chat where every message shows who it is from and who it is for, handoffs and
questions for you are tagged, files are chips that open a preview, a picked message shows its details, and a composer
names agents with @ or answers a question.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 760 px (max 100 %), container-sized, no frame of its own |
| Bar | toggle tabs Everyone · You · each agent with its 3×3 pixel mark (own colour and pattern per agent); picked tab on a grey pill; a blue Live dot at the end when live |
| Message | a 12 px pixel mark, then a bubble hugging its words up to 560 px: white, hairline, 10 px radius; name 13/600, Handoff / Needs you tags, Reply, time 12 faint; a grey "to ▪ Name" chip, or a quote of what it answers (unless that is right above); text 13 with brand marks before known names; grey file chips with drawn icons |
| Spawn | a quiet faint line "Planner started Analyst" |
| Details | picked: the rest fades to 35 %, a card of facts drops in under it |
| Composer | white, 14 px radius: field, then @ button, "to everyone" / "to Builder" / "reply to Builder", round send (dark when there is text) |
| @ list | a listbox above the composer: mark, name, model on the right |
| Preview | a dialog over the component (scrim inside it): icon, label, "Analyst · 14:04 · 4 calls", × and the content in mono |

## 3. Behaviour
- `live`: messages arrive `step` ms apart; the 900 ms before each, it shows "Writing…" (shimmer) and the sender's mark
  twinkles. A question to you stays tagged Needs you until a message from you replies to it.
- Tabs fade everyone not from or to the picked agent; clicking a bubble (or Enter / Space) picks it and drops its
  details in; Escape lets go. Chips open the preview; Escape or × closes it and focus returns to the chip.
- Composer: @ (typed or the button) opens the agents, typing filters, arrows move, Enter / Tab name one; Reply sets a
  reply; Enter sends `on_send(text, to, reply_to)` (Shift+Enter a new line). With demo data the component keeps what you
  sent; with your own `messages` append it yourself.
- `max_height` makes the stream scroll (0 lets it grow – the gallery uses 0, no inner scroll).
- prefers-reduced-motion: everything appears at once.

## 4. Variants
live (default) · settled · details · agent-tab · attachment-preview · mention · narrow (380 px) · classic (tabs in a grey
strip, flat grey bubbles without border, white "to" chips, a flat composer).

## 5. Keyboard and accessibility
Tabs: one stop, ← → Home End. Stream: a log (polite while live), one roving stop, ↑ ↓ Home End, Enter / Space picks;
only the focused message's Reply and chips are in the tab order. The @ list is a listbox tied to the field
(aria-expanded); the preview is a modal dialog. Tags are words; marks and logos are aria-hidden.

## 6. Responsive
760 px up to its container; under 520 px the gutters shrink; the tabs scroll sideways with an edge fade.

## 7. Known gaps
- No "N new ↓" pill (the gallery stream does not scroll); a click on a quote does not jump to the quoted message.
- The live demo plays once and stops (affectUI loops).
- GPUI: a static snapshot – no composer, typing shimmer, details card or preview; pixel marks are small squares.
