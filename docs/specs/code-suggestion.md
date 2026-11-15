**Status:** built v0.1 (2026-11-15, plan #30).

# Spec: Code suggestion (`code-suggestion`)

Family: Diff & review. Closes Arc UI **Code suggestion** (Pro). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
An AI proposes an edit to one file; the person accepts or rejects each hunk before anything is written. Hunks are `{id, start, remove, insert, summary}` (zero-based start; overlapping hunks after the first are ignored). `Suggestion::play(elapsed)` drives Thinking (0.8 s), Streaming (one inserted line every 38 ms) and Ready. Accepted hunks fold into the file (only new lines remain, renumbered); rejected hunks restore the original.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| thinking | Plain file, shimmering "Reading file.ts", Stop | first line |
| streaming | Removed lines tinted, new lines arriving, "Writing", Stop | last line |
| ready | Accept / Reject bar under each pending hunk; Reject all / Accept all ⌘↵ | all decided |
| resolved | Folded file, "All changes applied" or "Applied 1 of 3", Regenerate | regenerate |
| error | Message and Retry | retry |
| no changes | "No changes needed", Regenerate | — |

## 3. Timing and motion
- Thinking 800 ms with a 1.6 s shimmer loop; each new line fades in over 120 ms.
- Review bars rise in over 240 ms.
- Status label swaps with a 160 ms rise; the spinner keeps turning between Reading and Writing.
- Zero counts in the tab turn muted.
- Reduced motion: opens on the finished suggestion, no shimmer or spinner turn.

## 4. Variants
| Variant | What differs |
|---|---|
| stream | Plays the request |
| ready | Finished suggestion |
| error | Failure with Retry |
| capped | Code area capped (`max_code_height`) |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Code area → each Reject / Accept → footer |
| Enter / Space | Press |
| ⌘↵ / Ctrl+Enter | Accept all pending |
| ⌘⌫ / Ctrl+Backspace | Reject all pending |

## 6. Accessibility
- Labelled group "Code suggestion: <instruction>"; the code is a focusable region named after the file.
- Hidden "Removed:" / "Added:" on every pending line; Accept / Reject name their hunk; Accept all exposes aria-keyshortcuts.
- A polite status speaks the state, each decision ("Accepted: Import sleep. 2 changes to review") and the result.

## 7. Responsive
- Up to 720 px; long lines scroll sideways with numbers and bars pinned.
- Folder hides below 520 px; below 440 px the gutter narrows, summaries hide and the footer stacks.

## 8. Edge cases
- An insert-only hunk (remove 0) lands before `start`; a delete-only hunk (empty insert) shows just red lines.
- Stop keeps only the lines already streamed.
- Going back to thinking clears earlier decisions (host re-keys).

## 9. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| suggestion | Suggestion | — |  |
| now | ClockMs | — |  |
| started | Option<ClockMs> | None | None = finished |
| status | Option<SugStatus> | None | force thinking / error |
| instruction | String | "Add retry with backoff" |  |
| model | String | "Claude Sonnet" | empty hides |
| file_name | String | "fetch-invoice.ts" |  |
| file_path | Option<String> | None |  |
| error | String | … |  |
| shortcuts | bool | true |  |
| max_code_height | u32 | 0 |  |
| summaries | bool | true | Oxyn plus |
| on_decision | EventHandler<(String, bool, String)> | — | id, accepted, code |
| on_resolve | EventHandler<String> | — | final code |
| on_stop | Option<EventHandler<()>> | None | shows Stop |
| on_regenerate | Option<EventHandler<()>> | None | shows Regenerate / Retry |

## 10. Gaps against the source
- No self-reviewing autoplay loop (the source replays itself); the demo streams once and waits for you.
- The language mark is a text badge, not a brand logo.
- Highlighting uses the shared tokenizer, tinted monochrome.

## 11. GPUI
`CodeSuggestionEl::new(s).play(s.play(elapsed))` with `on_hunk(id, "accept"|"reject")` and `accept-all` / `reject-all` acts.
