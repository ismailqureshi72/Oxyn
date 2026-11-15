**Status:** built v0.1 (2026-11-15, plan #30).

# Spec: AI inline edit (`ai-inline-edit`)

Family: Diff & review. Closes Arc UI **AI inline edit** (Pro block). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
Edit a document with AI where you read it. Click a word to select its sentence (Shift+click extends); an Edit bubble appears; ⌘K opens a prompt under the selection with quick actions (Shorten, Make clearer, Friendlier tone, More formal, Fix grammar) and a free instruction. The rewrite (`scripted_rewrite` in the preview; a host passes its model output) streams in three words every 34 ms as a word diff (`rewrite_chunks`): text the stream has not reached waits dimmed, deletions strike and insertions develop in. In review each change has its own Accept / Reject; Accept all applies. ⌘Z / ⇧⌘Z undo and redo AI edits. At the end of the last paragraph, ghost text (`scripted_completion`) offers a continuation that Tab accepts.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| idle | Document; words hover-highlight | click a word |
| selected | Sentence boxed, Edit bubble | ⌘K / Edit |
| prompt | Input + quick-action listbox | Enter / Esc |
| streaming | Diff growing at the stream edge, Stop | end / Esc |
| review | Changes with mini toolbars, Accept all / Reject all | decide |
| applied | Receipt "Applied 3 of 4 changes" + Undo | click / ⌘Z |
| ghost | Dimmed continuation + hint | Tab / Esc |

## 3. Timing and motion
- One floating layer morphs prompt → stream bar → review bar → receipt (220 ms, smooth).
- Inserted words develop over 150 ms (opacity + 1.5 px blur clearing); accepted words settle with a 600 ms tint.
- Change toolbars fade up in 120 ms on hover or focus.
- Reduced motion: no fades, blur or morph.

## 4. Variants
| Variant | What differs |
|---|---|
| rewrite | Default speed |
| slow-stream | Half speed |
| no-ghost | Completions off |

## 5. Keyboard
| Key | Does |
|---|---|
| ⌘K / Ctrl+K | Open / close the prompt |
| ↑ / ↓, Enter | Pick a quick action; typed text sends first |
| Esc | Close prompt / stop / reject all / dismiss ghost |
| Tab | Accept ghost text; move between changes in review |
| Enter / Backspace | Accept / reject the focused change |
| ⌘↵ | Accept all |
| ⌘Z / ⇧⌘Z | Undo / redo |

## 6. Accessibility
- Labelled region; the prompt is a dialog with a combobox and a listbox (aria-activedescendant).
- Each change is a focusable group named "Change “x” to “y”".
- Streaming, review and applied are announced in a status region; ghost text is aria-hidden.
- ⌘K is claimed only while focus is inside.

## 7. Responsive
- Prompt keeps to 320 px or the page width; review bar wraps.
- Below 560 px the title steps down and the key hint hides.

## 8. Edge cases
- A rewrite with no changes applies nothing ("No changes applied").
- Stopping keeps reached changes and leaves the rest of the original untouched.
- Selection is clamped to one paragraph.

## 9. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| title | String | "Launch plan" |  |
| paragraphs | Vec<String> | — |  |
| now | ClockMs | — | stream clock |
| actions | Vec<QuickAction> | 5 defaults |  |
| speed | f32 | 1.0 | Oxyn plus |
| ghost | bool | true |  |
| author | String | "You" |  |
| review_each | bool | true |  |
| on_change | EventHandler<Vec<String>> | — |  |

## 10. Gaps against the source
- Paragraphs are not free-typing contenteditables (no Enter to split / Backspace to join); text is edited through AI changes and ghost text only.
- No async model hook (`onRequestEdit` / `onSuggest`); the host swaps the scripted rewrite for its own.
- Touch selection uses taps on words instead of native selection.

## 11. GPUI
`InlineEditEl::new(title, before, after).decisions(map)` renders the chunks and the review bar.
