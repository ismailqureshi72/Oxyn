**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Rich text editor (`rich-text-editor`)

Family: Composer & inputs ([composer-input-shell.md](composer-input-shell.md)). Closes the parity row(s): Arc UI · Rich text editor. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A lightweight block editor drawn by Oxyn (no contentEditable) on `parity::compose::Doc`. Markdown shortcuts at a line start (`# `, `## `, `### `, `- `/`* `, `1. `, `[] `, `> `, ```` ``` ````, `---`). Marks: bold, italic, strikethrough, inline code (within one block) and links, toggled on the selection or for the next typed text. Selecting text (Shift+arrows, Home/End, or dragging) shows a floating toolbar over the line; its face swaps to a link field (⌘K or the link button). `/` after a space or at a line start opens the block menu, which filters as you type; ↑ ↓ move, Enter or Tab chooses, Escape closes. Enter splits a block; an empty list item turns into text; Shift+Enter is a line break; Backspace at a block start turns a styled block back into text, outdents a list item or joins the block above. Tab / Shift+Tab indent list items (two spaces in code). Its own undo and redo (typing coalesces per word). Output: clean HTML (canonical tags, only href) and Markdown.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| empty | Placeholder; block hint on the caret line | type |
| editing | Caret, marks | — |
| selection | Toolbar | collapse |
| link face | URL field | Enter / Esc |
| slash | Block menu | choose / Esc |

## 3. Timing and motion
- Toolbar glides 200 ms with the selection; faces crossfade 160 ms; slash highlight glides 140 ms.
- Reduced motion jumps the toolbar and fades the menus.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Launch notes |
| html | HTML output with undo bar |
| empty | Placeholder |

## 5. Keyboard
| Key | Does |
|---|---|
| ⌘B / ⌘I / ⌘⇧X / ⌘E | Marks |
| ⌘K | Link |
| ⌘Z / ⌘⇧Z / ⌘Y | Undo / redo (⌘U blocked) |
| / | Block menu |
| Tab / Shift+Tab | Indent / outdent |
| Enter / Shift+Enter | New block / break |

## 6. Accessibility
- role=textbox, aria-multiline; toolbar buttons have aria-pressed; the slash menu is a listbox.

## 7. Responsive
- Fills its container up to 560 px.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| markdown | String | — | initial content |
| placeholder / block_hint | String | Start writing / Type / for blocks |  |
| read_only / history_bar | bool | false |  |
| label | String | Editor |  |
| on_change | EventHandler<(String, String, String)> | — | html, markdown, text |

## 9. Gaps against the source
- Selections live inside one block; IME composition and clipboard paste need the browser's editing surface and are not offered in the Dioxus view (GPUI has IME through `ImeInput`).

## 10. GPUI
RichTextEl renders blocks and marks from `Doc`; GPUI editing reuses the IME buffer for single blocks.
