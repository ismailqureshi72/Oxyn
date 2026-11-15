**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Document grid (`doc-grid`)

Family: App shells & screens. Closes the parity row(s): X #28 @s1dashu quiet.md (home: document grid). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
Local documents as a sidebar list (A to Z) and a card grid. Tabs switch My documents and Favorites; the search narrows by title or text; the sort flips Last visited and Title (`doc_view`); the grid / list toggle re-lays the same keyed cards. Stars toggle in place with a press squash; an empty Favorites tab says how to fill it.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| grid | Cards in three columns | layout |
| list | One line a document | layout |
| favorites | Starred only | tab |
| empty | Hint text | search / star |

## 3. Timing and motion
- Cards rise 240 ms; hover lifts 1 px; star squash on press.
- Reduced motion: no rise or lift.

## 4. Variants
| Variant | What differs |
|---|---|
| grid | Cards with the sidebar |
| list | Rows, no sidebar |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Tabs, sort, layout, cards |
| Enter | Open a card |

## 6. Accessibility
- Tabs are role=tab; layout buttons and stars are aria-pressed with labels that say what they do.

## 7. Responsive
- Container query: under 560 px of its own width the sidebar hides and two columns remain.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| docs | Vec<Doc> | — |  |
| sidebar | bool | true |  |
| list | bool | false | start in list view |
| on_open | EventHandler<String> | — |  |

## 9. Gaps against the source
- No folders, covers or page previews; the document editor itself is the rich-text-editor component.

## 10. GPUI
DocGridEl renders the same view (favourites and sort) as GPUI cards.
