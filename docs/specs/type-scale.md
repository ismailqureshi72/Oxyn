**Status:** built v0.1 (2026-11-17, variants 3: Theme Selector family).

# Spec: Type scale (`type-scale`)

Family: Theme Selector. Reference: none (an Oxyn original).
Components: `TypeScale` (Dioxus) · `TypeScaleEl` (GPUI), models in `oxyn_core::kit::themefid`.

## 1. Purpose
Set how text reads – family, size and line spacing – on a real paragraph, with a hint about line length.

## 2. Parts
| Part | Look |
|---|---|
| Family | Sans / Serif / Mono segmented, each label set in its own face |
| Size | A− and A+ (30 px squares) around a six-step slider (13–20 px, `TYPE_STEPS`) and the px readout |
| Spacing | Tight / Normal / Relaxed (1.3 / 1.5 / 1.75) |
| Sample | heading and paragraph in an inset panel at the chosen face, size and leading |
| Readout | "16 px · 1.5 lines · about 63 characters a line, comfortable" (`chars_per_line`, `measure_word`) |

## 3. Behaviour
Every control restyles the sample at once through `--tsz` and `--tlh` with short transitions. A− and A+ step the slider and disable at the ends. `on_change((family, px, line_height))`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default (Sans 16) · serif relaxed (18) · mono tight (14).

## 5. Keyboard and accessibility
Family and spacing are radio groups; A− / A+ are named Smaller text / Larger text; the slider's value text is its size in px; the readout is a polite live region.

## 6. Responsive
Controls stack under 480 px.

## 7. Known gaps
Line length is estimated from an average glyph width at 520 px, not measured. GPUI: static snapshot (host routes `fam-<i>`, `smaller`, `larger`, `sp-<i>`).
