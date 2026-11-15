**Status:** built v0.1 (2026-11-15, plan #30).

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
