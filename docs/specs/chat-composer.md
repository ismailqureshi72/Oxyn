**Status:** built v0.2 (2026-11-15, plan #32 affectUI fidelity pass; v0.1 was plan #28).

# Spec: Chat composer (`chat-composer`)

Family: Composer ([composer.md](composer.md)). Closes the parity row: affectUI · Chat Composer.
Behaviour-only rebuild from the live page and a screen recording; nothing is copied. Oxyn's own
"What should we work on?" composer stays as [ai-composer.md](ai-composer.md) and as the
`oxyn-ai-composer` variant on this page.

## 1. Purpose
The main input of an assistant or agent that acts on connected tools. A large message field
("Start by typing…"); a row with **+**, the approval chip, the **Auto** model menu, the mic and a
send button; and under the card a grey tray with "Work in a folder" (left) and "Connect apps"
with the first three app marks (right). Turn the tray off once a chat has started: the apps move
into the plus menu.

## 2. Parts
| Part | Behaviour |
|---|---|
| Field | 16.5 px text, grows to 168 px. Clicking anywhere on the card focuses it. Enter sends, Shift + Enter breaks a line. |
| + | A 34 px filled square; opens "Add to the chat", a drop-up spanning the whole composer (6 px above it): a list capped at 144 px that scrolls inside (snap, thin pill thumb, the last visible row fading). Group **Add**: Files and folders (paperclip; adds a file chip), Attach Claude (coloured Claude mark), Work in a folder ("Choose a folder for this chat"; opens the folder picker) · Goal, Plan mode (switch, `menuitemcheckbox`), Record a skill, Draw (teal scribble). Group **Plugins**: Documents (blue page), PDF (red page). Rows 39 px, label 16 px with its hint inline in grey. `on_attach` gets files, claude, folder, goal, plan, skill, draw, documents or pdf. Oxyn extra: attachment chips above the field (`attachments`). |
| Approval chip | Listbox "How should actions be approved?" + Learn more: Request approval (hand), Approve for me (check circle), Full access (shield, semantic orange). Picking Full access turns the chip into the orange-tinted **Unrestricted access**. |
| Model menu | "Auto ⌄", right-aligned drop-up: Auto (grey sparkle), Claude Fable 5.1, Opus 5.5, Sonnet 5.5, Haiku 5.5 (coloured Anthropic burst), GPT-5, GPT-5 mini (grey OpenAI knot). Check on the picked row; the name blur-swaps. |
| Mic | Recording mode: the model button hides; a level meter of 16 blue capsules (3.5 px wide, 2.2 px apart, `#8AB8F0` dark / `#3D74D0` light) rests as dots and rises up to 16 px with the voice (`meter_levels`, a frame per 110 ms); the transcript arrives word by word in grey; ✕ discards, ✓ (filled white) keeps the words. |
| Send | 32 px rounded square: disabled grey, enabled near-white (dark arrow) in dark, black in light. Sending clears the field. No running state here (see Run controls). |
| Tray | Blue filled folder + "Work in a folder" (left-aligned drop-down; picking the picked folder again leaves it). "Connect apps" + coloured Gmail / Slack / ClickUp marks (right-aligned check menu). |

## 3. States
| State | Looks | Leaves on |
|---|---|---|
| idle | Placeholder, send disabled | type, attach, dictate |
| typing | White text, send enabled | send (clears) |
| menu open | One menu; a transparent veil closes it on outside click | pick, Esc, outside click, Tab |
| recording | Trace + grey words, ✕ / ✓ | ✕ / Esc (discard), ✓ (keep) |
| unrestricted | Orange chip | another approval mode |
| in chat (`tray=false`) | No tray; apps live in the plus menu | — |

## 4. Timing and motion
- Menus morph out of the corner next to their button (scale + fade, 170 ms, `cubic-bezier(.2,.8,.2,1)`), opening toward the side with more room; the plus menu spans the composer.
- Approval chip morphs width between modes (320 ms); the hand waves on hover.
- Recording: meter frames every 110 ms (bars ease 110 ms); words every 190 ms; ✕ / ✓ pop in (220 ms).
- Attachment chips pop in (240 ms); check marks draw in (300 ms).
- Reduced motion: no scale, wave or pop; colour changes only.

## 5. Variants
| Variant | What differs |
|---|---|
| default | Clean (affectUI default): one continuous 680 px container, 18 px radius, the tray a darker band under a hairline |
| classic | Classic: the writing card (16 px radius) inset 12 px on a darker band (22 px radius); the tray sits on the band, no hairline |
| unrestricted | Starts in Full access |
| recording | Starts recording |
| attachments | Starts with PDF, image and sheet chips |
| plus-menu / models / approvals-menu | Starts with that menu open |
| in-chat | `tray=false` |
| oxyn-ai-composer | Oxyn's own AI composer, kept as an extra |

## 6. Keyboard
| Key | Does |
|---|---|
| Enter | Send |
| Shift + Enter | New line |
| Enter / Space / ↓ on a menu button | Opens its menu |
| ↑ / ↓, Home / End | Move through the open menu |
| Enter / Space in a menu | Pick (apps: connect / disconnect) |
| Esc | Close the menu; while recording, discard |

## 7. Accessibility
- Plus = `menu` (Plan mode is `menuitemcheckbox`), approval / model / folder = `listbox`, apps = `menuitemcheckbox`; every button carries `aria-expanded`.
- A polite live region announces modes, models, folders, apps, "Recording. Press Escape to discard.", "Recording added to your message." and "Message sent."
- Long folder names ellipsize with the full name in a tooltip. A closed menu leaves the layout.

## 8. Colour
Chrome is monochrome (exact affectUI greys per theme, `--oxf-*`). Colour only where it means something: brand marks in their official colours, file-type icons and the light-blue folder (`#00A1FF`, `--ox-ic-folder`) in their semantic tones, the orange Full access.

## 9. Responsive
- `container-type: inline-size` on a 540 px root (`max-width: 100%`); the approval menu is `min(352px, 100cqi - 40px)`; labels ellipsize before the row wraps.

## 10. API
Dioxus `ChatComposer` / GPUI `ChatComposerEl`; models in `oxyn_core::kit::fidelity` (`Approval`, `FileKind`, `attach`, `meter_levels`). Props: placeholder, text, models, dictation, approval, folders, folder, apps, connected, tray, style, attachments, recording, open, clock, on_send, on_model, on_attach, on_approval, on_folder, on_apps.


## Sizes (chunk 2 of the fidelity pass, measured against affectUI at 1920 px)
Container 680 px; field 17 px / 25 px; toolbar labels 16 px; buttons 34 px (9 px radius, squircle where supported); tray 40 px with 15 px labels; menus 14 px radius; model menu 316 px, six 39 px rows visible, then it scrolls with a thin pill thumb.
