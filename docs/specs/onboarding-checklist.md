**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Onboarding checklist (`onboarding-checklist`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Arc UI · Onboarding checklist.

## 1. Purpose
Guide a new workspace through setup: a progress ring and “n of m done”, one open step at a time with what it is for and its action button, checks that draw in, the next open step opening after a check, Hide / Show for the body, and a done title once all are complete.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| in progress | Ring, open step |  |
| hidden | Header only | show → in progress |
| done | Done title |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| progress | 2 of 5 |
| fresh | 0 of 5 |
| done | All set |
| hidden | Folded |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Checkbox, title, action, Hide |
| Enter / Space | Toggle / open / run |

## 6. Accessibility
- Checkboxes are role=checkbox buttons; Hide has aria-expanded.

## 7. Responsive
- Max 480 px.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| list | Checklist | — | items, open, hidden |
| title / done_title | String | Set up your workspace / You’re all set |  |
| on_toggle / on_open | EventHandler<String> | — |  |
| on_hide | EventHandler<bool> | — |  |

## 9. Edge cases
- Unchecking re-opens the first open step.

## 10. Implementation
- Core: `Checklist`, `CheckItem` (`oxyn_core::parity`), with tests.
- Dioxus: `OnboardingChecklist` (parity/run.rs, run.css).
- GPUI: `ChecklistEl`.

## 11. References
- Arc UI Onboarding checklist (https://uiarc.dev/components/onboarding-checklist) — behaviour-only rebuild, no source copied.
