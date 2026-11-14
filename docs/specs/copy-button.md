**Status:** v1 — implemented (plan #27).

# Spec: copy button

> Implemented in `oxyn-core::primitives::layout (`CopyState`, `COPY_HOLD`)`, `oxyn-dioxus::CopyButton`, `oxyn-gpui::CopyButtonEl`.
> Source: Arc UI Copy button (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Copy a command, a code block, an answer or an ID with instant confirmation.

## 2. State machine
```
Idle ─press─▶ Copied(t) ─1.5 s─▶ Idle
Idle ─press (write fails)─▶ Failed(t) ─1.5 s─▶ Idle
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| idle | copy icon, label | Enter / Space | named by label |
| copied | drawn check, “Copied” | — | status “Copy: Copied” |
| failed | x, “Failed” | — | status “Copy: Could not copy” |

## 4. Micro-interactions
The old icon and word fade out in ~100 ms while the new ones fade in over ~160 ms; the check draws its stroke. The label cell reserves its widest word so the button never changes width.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Out | 100 ms | linear | instant |
| In | 160 ms (60 ms delay) | linear | instant |
| Check draw | 260 ms | `ease.out` | none |
| Hold | 1500 ms | — | same |

## 6. Variants
Free: `outline`, `plain`, icon-only, sizes.

**Oxyn plus:** `solid`, a failure simulation, use inside the code block header.

## 7. Edge cases
The host writes the clipboard (`on_copy`; the site handles `data-copy`). Disabled stays idle.

## 8. Accessibility
Native button named by `label`; polite `status` announces the result.

## 9. Props sketch
```rust
CopyButton { value: String, label: String, icon_only: bool, variant: "outline"|"plain"|"solid", size: "sm"|"md",
            disabled: bool, fail: bool, on_copy: EventHandler<String> }
```

## 10. Related components and references
`code-block.md`, `message-bubble.md` (message actions). Arc UI Copy button — behaviour only.
