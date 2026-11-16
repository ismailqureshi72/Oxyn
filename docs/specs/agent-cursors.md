**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 3).

# Spec: Agent Cursors (`agent-cursors`)

Family: Agents. Closes the parity row: affectUI · Agent Cursors (Pro). Behaviour-only rebuild; nothing is copied (the
demo document, agent activities and comments are our own text).
Components: `AgentCursors` (Dioxus) · `AgentCursorsEl` (GPUI), model in `oxyn_core::parity::cursorsfid`.

## 1. Purpose
Agents working in a document like teammates in a multiplayer editor: live pointers with a flag (name, cost, what it is
doing), selected passages, suggestions typed letter by letter, an image that loads and gets resized, code fixes and
comment pins beside the page. Every change stays a suggestion until you accept it.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 700 px (max 100 %), container-sized, `overflow: clip`; no background of its own |
| Tabs | above the page, 28 px pills 7 px radius, 13/500: open tab white with a hairline and a soft shadow, others grey fill |
| Page | white card, hairline, 12 px radius |
| Header | 44 px, hairline under it: file glyph + title 13/500 grey; right: faces, "3 suggestions" (or "4 here") 12.5 grey, total cost 13/600, "Accept all" ink pill 28 px (grey and disabled with no suggestions) |
| Faces | 24 px round toggles with the agent's coloured brand mark; in the document: a hairline ring and an arc in the agent's ink running around it; out of it: faded; followed: filled with the tint, the others faded further |
| Body | 30/44 px padding; a fixed height (470 px by default) with the last 110 px fading out – a window onto a long document |
| Text | title 20/600, headings 14.5/600, body 14.5 px / 1.52 in a softer grey; code 12.5 mono on a grey block, 8 px radius; list markers faint |
| Pointer | the soft rounded arrow (18×20) in the agent's colour – the shared ink when it has none – with a thin page-coloured edge; the caret a 1.5 px bar while it types |
| Flag | ink card 9 px radius at the pointer's bottom right: name 12/600 + cost chip, the activity 11.5 grey under it; a pink-to-blue glow along its bottom edge. Light in dark mode |
| Selection | a soft grey-lavender band behind the passage, 3 px radius |
| Suggestion | removed text struck through and faint; inserted text underlined on a light wash, both in the agent's colour when it has one |
| Suggestion window | dialog under the suggestion, 12 px radius, shadow: mark + name 13/600 + "suggests a rewrite" grey; the note; Reject (outlined) and Accept (ink) |
| Image | floated left (or right) at a share of the page width, 8 px radius: grey with a shimmer while it loads, then a soft render; held, a 1.5 px ink outline, four handles and a "290 × 114" tag |
| Comment pin | 27 px ink pin (round, one square corner) with the agent's mark beside its block; hover, focus or `open` grows it into a 240 px ink window: mark, name, "now", the text, a Reply field and a check (Resolve) |
| Table | inside a hairline card, 10 px radius; 40 px rows, header in grey 12.5; status cells as pills (Doing blue, To do grey, Idea amber, Done green, Blocked red); a last "+ Add row" row |

## 3. Behaviour
- Demo: scripted passes per tab (Doc ≈ 24 s, Table ≈ 21 s) that start over. Doc: the Writer reads the title, selects a
  word and types a better one; the Researcher reads the intro, selects a figure, writes a comment and then suggests the
  fix; the Designer waits for the hero image, then holds its corner and resizes it; the Coder reads the config, fixes a
  date, asks about the format in a comment and checks the rest. Table: a status, an owner and a date change, a task row
  is typed in and the Reviewer leaves a note. Costs tick up while agents are in the document.
- Offsets are character indexes into a block's current text; a cursor typing into a suggestion counts in its insert.
- Pointers are anchored inline (a zero-width span at their character), so they stay on their letters while text wraps.
  A new pointer pops in where it lands; the activity slides out beside the name.
- Clicking or focusing a suggestion opens its window; Accept writes it into the text (`accept_suggestion` moves the other
  suggestions in that block along), Reject drops it, Esc closes. "Accept all" accepts every waiting suggestion.
- A face toggles following; the other pointers, selections and pins fade back. The check in a comment's window resolves
  it; Enter in its Reply field sends `on_reply`.
- Switching tabs restarts the clock and fades the new view in; added table rows slide in.
- prefers-reduced-motion: everything moves at once.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Live Doc pass |
| writing | Frozen while the Researcher writes its comment (window open), the Writer's rewrite just typed |
| suggestion-window | Settled, the Writer's suggestion window open |
| following | Following the Researcher: others faded |
| table | The Table tab |
| classic | The page as raised paper (16 px radius, shadow) on a grey header band; the body an inset white sheet |

## 5. Keyboard and accessibility
Tab moves through the tabs, the faces, then from suggestion to suggestion (a focused suggestion opens its window); Enter
or Space on a face follows or stops following; Esc closes a suggestion window. The text is real text (h2, h3, p, ul, pre,
table); pointers, carets, flags and selections are `aria-hidden`. Suggestions are labelled "Writer suggests removing:
outline" / "…adding: schedule"; their windows are dialogs named "Suggestion from Writer". Faces are toggle buttons
(aria-pressed) named "Follow Coder – Reading the config" or "Follow Coder (not in the document)". A polite live region
announces what agents do ("Researcher is writing a comment") and new suggestions with their note.

## 6. Responsive
Positions come from the text itself, so any width, font size or zoom works. Flags sit to the bottom right of their
pointer and are cut off at the right edge rather than moved. Below 520 px the page gets tighter margins and the
suggestion count gives way to the faces. Dark mode via `data-oxyn-theme`.

## 7. Known gaps
- Pointers jump between places rather than gliding (gliding needs measuring the page; the anchors keep them on their
  letters instead). Within a block, typing moves the pointer letter by letter.
- Comment windows always open towards the bottom right.
- GPUI: a static snapshot – carets in the text and flags at the end of their block's line, not on the letter.
