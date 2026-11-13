**Status:** v1 — implemented (plan #25).

# Spec: password strength

> Implemented in `oxyn-core::primitives::password` (`Rule`, `RuleKind`, `Policy`, `Level`,
> `evaluate`, `entropy_bits`, `is_common`, `segment_delay`), `oxyn-dioxus::PasswordStrength`,
> `oxyn-gpui::PasswordStrengthEl`. Source: Arc UI Password strength (behaviour only).

## 1. Purpose
Choosing a new password for an account or a workspace secret. The user wants to see, as they
type, what's missing and when it's good enough.

## 2. State machine
```
Empty ─type─▶ TooShort (< short_length code points; meter holds at step 1)
TooShort ─type─▶ Weak | Fair | Good | Strong   (share of rules met: <40 %, ≥40 %, ≥60 %, 100 %)
any ─common password─▶ capped at Weak
Level ─up─▶ word rolls up, segments fill left → right
Level ─down─▶ word rolls down, segments empty right → left
any ─error from host─▶ field shakes once per new error
```

## 3. States table
| Level | Segments | Colour (status only) | Word |
|---|---|---|---|
| too short | 1 | danger | Too short |
| weak | 1 | danger | Weak |
| fair | 2 | warning | Fair |
| good | 3 | info | Good |
| strong | 4 | success | Strong |

Rules: met rules draw a tick in a success circle and the text brightens; length rules show
"· 3 more". Keyboard: Tab to the reveal button, Enter / Space toggles.

## 4. Micro-interactions
Segments fill in a 60 ms staggered wave toward the new level (reversed when it drops). The
strength word morphs: rises when it improves, drops when it gets worse. Ticks draw on as each
rule is met; remaining counts roll.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Segment colour | 220 ms, stagger 60 ms | `ease.out` | 1 ms, no stagger |
| Word roll | 200 ms, 0.6 em | `ease.out` | swap |
| Rule tick | 220 ms draw | `ease.out` | drawn |
| Remaining count | 160 ms roll | `ease.out` | swap |
| Error shake | 360 ms | damped | none |

## 6. Variants
Free: default rules (8+, upper, lower, number, symbol), custom `short_length`, revealed, error,
custom meter label.

**Oxyn plus:** policy presets — Basic, NIST SP 800-63B (length + not common + no edge spaces, no
composition rules), Enterprise (12+, all classes, not common); a common-password check; an
entropy estimate in bits; a meter-only variant.

## 7. Edge cases
Length counts code points, so emoji count once. Pasted passwords score at once. Leading or trailing
spaces fail the NIST edge-space rule. The common list is a small built-in set; real breach checks
belong to the host (k-anonymity API). Narrow columns: the four segments share the row.

## 8. Accessibility
The meter is `role=meter` with `aria-valuemin=0`, `aria-valuemax=4`, `aria-valuenow` and
`aria-valuetext` set to the word. The rule list is linked to the input with `aria-describedby`;
each rule carries hidden "Met:" / "Not met:". A polite status line: "Good, 4 of 5 rules met".
Colour is never alone: the word and ticks carry the same meaning.

## 9. Props sketch
```rust
PasswordStrength { label, value, policy: "basic" | "nist" | "enterprise", short_length, error,
                   revealed, variant: "checklist" | "meter", entropy, meter_label, disabled,
                   on_change: EventHandler<(String, String)> }
```

## 10. Related components and references
`password-field.md`, `text-field.md`. Arc `/components/password-strength` — behaviour only. NIST
SP 800-63B §5.1.1 for the policy preset.
