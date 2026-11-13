**Status:** v1 — implemented (plan #23).

# Spec: badge, chip, value pill, entity chip + monogram

> Implemented in `oxyn-core::primitives::display` (`Tone`, `Fill`, `Badge`, `badge_count`,
> `live_halo`, `Chip`, `ChipKind`, `Delta`, `Monogram`, `initials`, `seed_index`, `monogram_roles`,
> `EntityChip`), `oxyn-dioxus::{Badge, Chip, ValuePill, Monogram, EntityChip}`, and
> `oxyn-gpui::{BadgeEl, ChipEl, ValuePillEl, MonogramEl, EntityChipEl}`. Sources: affectUI Badge,
> Arc Badge and Chip group (behaviour only); Beautiful UI `value-pill` and `entity-chip` (MIT,
> © 2026 Shane Levine — ported with attribution: tone names, monogram-in-a-pill layout).

## 1. Purpose
Small labels that carry meaning at a glance: a run's status, an unread count, a model's version, a
filter, a number inside a sentence ("restock in **7 days**"), a company or person the agent is
talking about. These are the only chrome pieces allowed to carry colour, and only when the colour
*is* the meaning (status, agent identity, monogram).

## 2. State machine
- **Badge** has no state of its own. `live` adds a halo loop; the count changes with the data.
- **Chip:** `Static`; `Toggle: Off ⇄ On` (click / Enter / Space, ignored when disabled);
  `Removable: Present ─(× / Backspace / Delete)─▶ Removed` (the host drops it).
- **Value pill:** stateless; with a `delta (prev, now)` it derives `Up / Down / Flat` and its tone.
- **Entity chip:** `Rest ─hover / focus─▶ CardOpen ─leave / blur / Esc─▶ Rest` when it has a card.

## 3. States table
| Component | State | Shows | Keyboard | ARIA |
|---|---|---|---|---|
| Badge | tone × fill | soft tint (default), outline, solid or mono tag | — | text; tone spoken via the label |
| Badge | dot / icon | leading 6 px dot or 11 px icon | — | — |
| Badge | live | dot with a halo every 1.8 s | — | `role=status`, "(in progress)" |
| Badge | count | round, tabular figures, `99+` over `max` | — | "More than 99 unread" |
| Chip | static | label (+ dot, + count) | — | text |
| Chip | toggle off / on | on = inverted fill and a check that grows in | Enter / Space | `aria-pressed` |
| Chip | removable | × after the label | Backspace / Delete when focused | × named "Remove {label}" |
| Chip | disabled | 45 % | skipped | `disabled` |
| Value pill | tone | tinted mono text | — | text |
| Value pill | delta | ▲ / ▼ / – arrow; green when good, red when bad | — | "+12.4%, up" |
| Entity chip | rest | monogram (or image) + name (+ status dot) | Tab when it has a card | text |
| Entity chip | card open | card under the chip with key-value rows | Esc closes | `aria-describedby` → card (`role=tooltip`) |

## 4. Micro-interactions
- **Live dot:** a soft halo grows out of the dot (scale 1 → 2.6) and fades every 1.8 s — nothing
  else moves (affectUI).
- **Label change** (Arc): the new label rises in from a short blur while the pill width springs.
  Passive reflows (font swap) resize instantly.
- **Toggle chip:** the check grows in from zero width (`spring.snap`), the label slides over, the
  fill inverts in 120 ms; press 0.97.
- **Entity card:** fades and lifts 5 px in 180 ms; leaves at once.
- **Count roll** (Oxyn plus) uses the number ticker's digit roll when a count changes.

## 5. Timing table
| Transition | Duration | Easing | Notes |
|---|---|---|---|
| Live halo | 1800 ms loop | `ease.out` | affectUI |
| Label swap + width | 180 ms fade/blur, width `spring.snap` 214 ms | `ease.out` | Arc |
| Toggle chip fill | 120 ms | `ease.out` | |
| Toggle check grow | 214 ms | `spring.snap` | |
| Chip press | 80 ms, 0.97 | `ease.out` | |
| Entity card | 180 ms in, instant out | `ease.out` | |
| Count roll | 280 ms (`dur.digit.roll`) | `ease.out` | Oxyn plus |
| **Reduced motion** | halo off (static dot), swaps fade in 80 ms, no grow | linear | |

## 6. Variants
Source: affectUI badge tones `neutral`, `info`, `success`, `warning`, `danger`, `purple`; sizes
`sm`, `md`; `dot`, `live`, `icon`, `count`, `max`. Arc adds `lg`, `outline`, `pill`. Beautiful UI
value pill tones `neutral`, `green`, `orange`, `red`, `accent`; entity chip `name`, `color`,
`monogram`.

**Oxyn plus:**
1. Badge tones 6 → 10: `working`, `waiting`, `queued`, `stalled` (agent states; `Tone::of(&AgentState)`).
2. Badge fills: `soft`, `outline`, `solid`, `mono` (version / hash tags).
3. Chip kinds on one component: static, toggle, removable; count badge and status dot.
4. Value pill: delta arrows with good-up / good-down tone (cost and latency read red when up).
5. Entity chip: hover card, image instead of the monogram, presence dot.
6. Monograms: 12 seeded colour pairs from the tokens (stable per name), round or square.

## 7. Edge cases
- Counts above `max` show `99+`; zero counts still render (badges are explicit), unlike the
  reaction bar's hide-zero rule.
- Long chip labels truncate with an ellipsis at the host's max width; the full text is the name.
- Names with one word → one initial; punctuation and emoji are skipped (`initials("Jean-Luc Picard")`
  = "JP"; empty = "?").
- The same name always gets the same monogram colour (`seed_index`, case- and space-insensitive).
- Inline pills sit on the text baseline (`vertical-align: middle`) and never change line height.
- High contrast theme: soft fills gain a 1 px border through the tokens.

## 8. Accessibility
Colour is never the only signal: badges carry words, live badges add "(in progress)", deltas speak
"up / down". Toggle chips use `aria-pressed`; removable chips are a focusable group whose × button
is named "Remove {label}". The entity card is linked with `aria-describedby` and closes on Esc. Tone
foregrounds meet 4.5:1 on their soft backgrounds in both themes.

## 9. Props sketch
```rust
Badge { badge: Badge { label, tone, fill, dot, live, count, max, pill }, size: "sm|md|lg", icon }
Chip { label, kind: "static|toggle|removable", selected, count, dot: Option<tone>, disabled, on_toggle, on_remove }
ValuePill { value, tone: "neutral|green|orange|red|accent|<tone>", delta: Option<(f64, f64)>, good_up }
Monogram { monogram: Monogram { letters, colour }, size, square }
EntityChip { entity: EntityChip { name, monogram, image, status, card }, card_open }
```

## 10. Related components and references
`status-indicator.md` (agent-state colours), `avatar.md` (same monogram colours), `number-ticker.md`
(count roll), `composer-slash-mentions.md` (mention chips). Licence: Beautiful UI value pill and
entity chip ported under MIT with attribution in the module docs.
