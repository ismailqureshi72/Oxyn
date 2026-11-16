**Status:** built v0.2 (2026-11-16: review mode added for plan #32, affectUI fidelity pass family B part 9, against the ref-3 capture notes; v0.1 2026-11-15, plan #30).

# Spec: Inline diff (`inline-diff`)

Family: Diff & review ([diff-viewer.md](diff-viewer.md)). Oxyn plus for plan #30 (no single source row; covers the `diff-viewer/redline` idea). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
Prose changed by an agent (a reply draft, a policy, release notes). The user wants to read the new text and see exactly which words moved, without a code gutter. `rewrite_chunks` tokenises both texts into words, spaces and punctuation, runs an LCS, and merges two changes separated by a single space so an edit reads as one phrase.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| redline | One paragraph: struck deletions, underlined insertions | form change |
| lines | A Before line over an After line | form change |
| after-only | New text with changed words highlighted | form change |

## 3. Timing and motion
- No motion of its own: it renders a settled diff. Hosts that stream wrap it in a text reveal.
- Reduced motion: nothing to reduce.

## 4. Variants
| Variant | What differs |
|---|---|
| redline | Default |
| lines | Before over After |
| after-only | Only the new words |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | No stops; read as text |

## 6. Accessibility
- Real `del` and `ins` elements, announced by screen readers; colour is paired with strike-through and underline.
- The +N / −N word counts are text.

## 7. Responsive
- Wraps like a paragraph; max 640 px.

## 8. Edge cases
- Identical texts render one plain run.
- A total rewrite shows one deletion and one insertion.
- Punctuation-only edits are their own tokens, so a comma change marks just the comma.

## 9. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| before | String | — |  |
| after | String | — |  |
| form | String | "inline" | inline · lines · after-only |
| counts | bool | true | word counts |

## 10. Gaps against the source
- No character-level mode inside a word yet (`inline = Char` in diff-viewer.md §2.2).

## 11. GPUI
`InlineDiffEl::new(before, after)` draws the same chunks with GPUI `line_through` and `underline`.

## Review mode (v0.2, plan #32)
Closes the parity row affectUI · Inline Diff (Pro) by upgrading this component (`review: true`), not a second one.
Behaviour-only rebuild; texts and copy are our own.

| Part | Look (Clean, light / dark) |
|---|---|
| Card | up to 420 px: raised surface, 1 px hairline, 12 px radius, soft shadow, 14×16 px padding |
| Label | spark glyph + muted "Suggested edit · Tighten it" (prop) |
| Prose | 13.5 px / 1.75; pending removals struck in the removal tint, insertions in the addition tint; the highlighted change gets a soft wash |
| Review bar | small dark (light in dark mode) bar anchored above the highlighted change: ✓ Accept `Y`, ✕ Reject `N`; it flips sideways or below to stay inside the card |
| Footer | "n suggestions" chip, round prev / next (disabled at the ends), Undo once something is decided, Reject all (outline), Accept all (solid) |
| Done | green ✓ "All changes reviewed" and Undo |

Behaviour: `rewrite_apply` builds the text from the decisions (accepted takes the new words, rejected or pending keeps
the old); after a decision the highlight moves to the next pending change (`rewrite_next`); Accept all / Reject all
decide every pending change; Undo restores the decisions before the last action. `on_change` fires with the text after
each decision, `on_done` when nothing is left. Y / N act on the highlighted change while focus is inside; each change
is a button named "Change i of n: replace … Y keeps it, N drops it"; a polite status line says what was
decided and how many are left. Classic moves the footer above the text and drops the tints (colour plus strike /
underline only). Reduced motion: no fade-up or resolve wash.
Extras beyond the reference: Undo is available mid-review too, not only at the end; `decided` seeds earlier decisions.
Known gaps: the review bar does not glide between changes (it re-anchors); GPUI keeps the read-only forms only.
