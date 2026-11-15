**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Chat composer (`chat-composer`)

Family: Composer ([composer.md](composer.md)). Closes the parity row(s): affectUI · Chat Composer. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A prompt card with a model picker (Auto, Claude, ChatGPT), a + button, voice input whose demo transcript appears word by word in grey while recording (stop drops it into the field), a send arrow, and under the card a "Work in a folder" picker (picking the chosen folder again leaves it: `None`), a Request approval button with a loading state (`load_ms`, 1300) that settles to Approval requested, and a Connect apps menu with switches whose first three marks show in the row (connected marks filled).

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| idle | Card, row | type / dictate |
| recording | Mic lit, grey words | stop |
| requesting | Spinner | load_ms |
| requested | Check | — |

## 3. Timing and motion
- Dictation 170 ms per word; menus 160 ms.
- Reduced motion drops the menu rise.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Full |
| folder | In a folder |
| nofolders | No folder picker |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter | Send |
| Shift + Enter | New line |

## 6. Accessibility
- Menus use menuitemradio / menuitemcheckbox with aria-checked; dictation is aria-live.

## 7. Responsive
- The row under the card wraps.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| placeholder | String | Start by typing... |  |
| models | Vec<String> | Auto, Claude, ChatGPT |  |
| load_ms | u64 | 1300 |  |
| dictation | String | … | demo transcript |
| folders / folder | Vec<String> / Option<String> | — |  |
| apps / connected | Vec<(String, String)> / Vec<String> | Gmail, Slack, ClickUp / gmail |  |
| now | ClockMs | — |  |
| on_send / on_folder_change / on_apps_change | EventHandler | — |  |

## 9. Gaps against the source
- App logos are monochrome letter marks (brand marks are left to the host).

## 10. GPUI
ChatComposerEl composes the GPUI composer parts with the folder and apps rows.
