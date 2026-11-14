**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Confirm popover (`confirm-popover`)

Family: G Approvals ([approvals.md](approvals.md)). Closes the parity row(s): Arc UI · Confirm popover.

## 1. Purpose
A destructive button that grows into a small confirmation card: the consequence in one or two sentences, an optional type-to-confirm phrase (case and outer spaces ignored) that fills Confirm as the typed prefix matches, progress on the Confirm button (reported or an indeterminate sweep), then a check in the button's shape, or the failure with the rejection message and Try again.

Anatomy:
```
[ ⌫ Delete project ]
╭──────────────────────────────╮
│ Delete this project?          │
│ This removes acme-web, its…  │
│ Type acme-web to confirm      │
│ [ acme-w           ]          │
│            Cancel [Delete▓▓░] │
╰──────────────────────────────╯
```

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| idle | Trigger | press → confirming |
| confirming | Card; Delete fills with the phrase match | submit (unlocked) → working; submit (locked) → shake; Esc/Cancel → idle |
| working | Bar on Delete: reported % or a 1.2 s sweep | ok → success; err → error |
| success | Trigger shows ✓ + success label for 1.4 s | → idle |
| error | Error heading + message; Delete becomes Try again | retry → working; dismiss → idle |

## 3. Timing and motion
- Card grows from the kept edge (scale .94 → 1, 200 ms).
- Wrong submit: 320 ms damped shake (6 px).
- Reduced motion: no grow, no shake (the field is marked invalid instead).

## 4. Variants
| Variant | What differs |
|---|---|
| phrase | Type to confirm (default) |
| plain | Consequence and two buttons |
| neutral | Reversible: foreground fill |
| error | Failure with Try again |
| sweep | Indeterminate progress |
| align end | Keeps the right edge |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter | Confirm |
| Escape | Dismiss |
| Tab | Phrase, Cancel, Delete |

## 6. Accessibility
- Trigger: aria-expanded + aria-haspopup=dialog.
- Card: role=alertdialog named by its heading, described by the consequence.
- Delete is aria-disabled until unlocked; the phrase field gets aria-invalid when it stops matching.

## 7. Responsive
- 300 px card, max 100 %; align start / center / end.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| model | ConfirmPopover | — | tone, confirm_text, align, typed, error |
| label / title / description | String | — |  |
| confirm/cancel/pending/success/retry labels | String | Delete, Cancel, Deleting, Deleted, Try again |  |
| error_title | String | That didn't work |  |
| on_show / on_dismiss / on_submit | EventHandler<()> | — |  |
| on_type | EventHandler<String> | — |  |

## 9. Edge cases
- Demos start open but draw no page-covering layer; outside press is the host's job.
- Success label should not be longer than the trigger label (width holds).

## 10. Implementation
- Core: `ConfirmPopover`, `PopoverPhase`, `PopoverAlign` (match_fill, submit, progress, finish, shake, sweep) with tests.
- Dioxus: `ConfirmPopover`.
- GPUI: `ConfirmPopoverEl` (`show`, `dismiss`, `submit`); the phrase is shown, editing is the host's input (see `ime-input`).

## 11. References
- Arc UI Confirm popover (https://uiarc.dev/components/confirm-popover) — behaviour-only rebuild.
