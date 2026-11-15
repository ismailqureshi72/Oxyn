**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Thread list (`thread-list`)

Family: App shells & screens ([workspace-chrome.md](workspace-chrome.md)). Closes the parity row(s): X #3 @aryagm01 local T3 Code; X #53 Threadbear (thread sidebar). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
Agent threads grouped by where they stand (`group_threads`). The top group puts what needs you first: Approval, then Input, then unseen Done, then everything else newest first, each idle row showing its age (`21m`). Working threads sit in a counted group with a dashed spinner and their run time (`Working 2m`); Settled threads fold into a counted group that starts closed. Each row has a square project badge with initials (`initials`), the project, the chip or age, the title and the agent mark. A search narrows every group by title or project, and a status line counts what needs you (`2 need you`). The list is a single tab stop with an active descendant.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Groups, chips, ages | search / keys / click |
| filtered | Only matching rows; empty text when none | clear the search |
| group closed | Header with count and a turned chevron | Enter / → / click |
| selected | Sunken row | another row |

## 3. Timing and motion
- Rows rise in over 220 ms; chips swap with a 200 ms rise when a thread changes state.
- The group chevron turns in 200 ms.
- Working spinner: 2.4 s a turn.
- Reduced motion: no entrance, no spinner turn.

## 4. Variants
| Variant | What differs |
|---|---|
| needs-you | Approval and Input at the top |
| calm | Nothing waiting: ages only |
| open | Settled open, no search |

## 5. Keyboard
| Key | Does |
|---|---|
| ↑ / ↓ / Home / End | Move the active row (headers included) |
| Enter | Open a thread; open / close a group on its header |
| ← / → | Close / open the group on its header |

## 6. Accessibility
- role=listbox with aria-activedescendant; rows are role=option with aria-selected; headers carry aria-expanded.
- Chips are words, not colour alone (Approval amber, Input blue and Done green are status colour, allowed by the colour rule).

## 7. Responsive
- Fills its column up to 300 px; titles and projects ellipsize.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| threads | Vec<Thread> | — | `Thread::new(id, project, title, ThreadState)`, `.agent(icon)` |
| selected | Option<String> | None |  |
| search | bool | true |  |
| settled_open | bool | false |  |
| on_select | EventHandler<String> | — |  |
| on_new | EventHandler<()> | — |  |

## 9. Gaps against the source
- No unread dot or pinned threads; no drag to reorder.
- The project home beside the list in the demo is plain markup, not a component.

## 10. GPUI
ThreadListEl renders the same groups and chips; the host keeps the cursor and reports `toggle-settled`.
