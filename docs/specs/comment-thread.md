**Status:** built v0.1 (2026-11-15, plan #30).

# Spec: Comment thread (`comment-thread`)

Family: Diff & review. Closes Arc UI **Comment thread** (free). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
Review threads on a doc, design, ticket or pull request. Nested replies (`Thread::reply` attaches anything deeper than `max_depth` to the deepest allowed ancestor), reactions that toggle per person, edit and delete for your own comments (`delete` keeps a placeholder when there are replies), `@` mentions (`mention_query`, `mention_matches`, `mention_spans`), agent comments marked as such, and Resolve, which folds the thread into a chip.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| open | Comments, composer | resolve |
| resolved | Chip "Resolved · 4 comments · Show" | reopen |
| replying / editing | Inline mention box under the comment | send / Esc |
| confirm delete | Inline confirm row | Delete / Esc |
| picker | Emoji row | pick / Esc |
| empty | "No comments yet" | first comment |

## 3. Timing and motion
- New comments rise in over 240 ms; reaction counts roll; picker and confirm rows slide in from the side (200 ms).
- Reduced motion: no travel or transitions.

## 4. Variants
| Variant | What differs |
|---|---|
| thread | Replies, reactions, mentions, an agent reply |
| resolved | Folded |
| empty | No comments |

## 5. Keyboard
| Key | Does |
|---|---|
| ⌘↵ / Ctrl+Enter | Send |
| @ | Mention suggestions |
| ↑ / ↓ | Move through suggestions |
| Enter / Tab | Insert mention |
| Esc | Close suggestions → cancel edit or reply; close picker / confirm |

## 6. Accessibility
- Composer is a combobox with a listbox of people and aria-activedescendant.
- Each comment is an article labelled author · time; deleted reads "Deleted comment".
- Reaction chips are toggle buttons with aria-pressed and "👍 2, including you"; reply toggles expose aria-expanded.

## 7. Responsive
- Below 420 px the composer hides its avatar and replies indent 12 px; actions stay visible on touch.

## 8. Edge cases
- Whitespace-only text is never sent.
- Deleting a leaf removes it; deleting a parent keeps the placeholder.
- Mentions only highlight known people.

## 9. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| thread | Thread | — |  |
| me | Person | — | who can edit / delete |
| people | Vec<Person> | thread + you | mention list |
| title | Option<String> | None |  |
| reactions | Vec<String> | 6 emoji |  |
| placeholder | String | "Add a comment…" |  |
| max_depth | usize | 2 |  |
| default_resolved | bool | false |  |
| on_change | EventHandler<(Thread, ThreadEvent)> | — | reply / edit / delete / react |
| on_resolved | EventHandler<bool> | — |  |

## 10. Gaps against the source
- The mention query reads the text up to its end, not the caret position.
- No localisation `labels` map yet; words are English.

## 11. GPUI
`CommentThreadEl::new(thread).title(..)` renders the tree, reactions and the resolved chip.

## affectUI Comment Thread (`AgentCommentThread`, plan #32)
**Status:** built v0.1 (2026-11-15, plan #32). Closes the parity row affectUI · Comment Thread; the
Arc UI review thread above stays as `CommentThread`.

- **Card:** "Comment · 3 messages" header with an optional ✕. People show a portrait, name, relative
  time ("12min ago", "just now") and text; a leading `/Edit` is greyed as a command (`split_command`).
- **Agent:** a gradient orb avatar and a status line "✓ Edit completed ⌄" that folds its details open
  (chevron turns, 200 ms height).
- **Reply:** "Reply, or ask @Atlas…" with a round send button. Your reply lands as "just now"; when the
  agent answers, a shimmering "Thinking…" comes first (shared clock), then its message.
- **Variants:** default, details (status folded open), people-only.
- **Keyboard:** Enter posts, Shift + Enter breaks a line; the status line is a button with `aria-expanded`.
- **Avatars:** round 24 px portrait photos of fictional people (generated, no real person; six WebP files, 72 px, about 1 KB each, `avatar_webp` / `PhotoAvatar`), as affectUI shows photos; `avatars: "drawn"` keeps the illustrated `Portrait` (Oxyn extra). The agent keeps its gradient orb.
- **API:** Dioxus `AgentCommentThread` (`ThreadNote`, `AgentOrb`, `PhotoAvatar`, `Portrait`) / GPUI `AgentCommentThreadEl` (`ThreadMsg`).
