**Status:** built v0.1 (2026-11-17, variants 3: Theme Selector family).

# Spec: Accent and density (`accent-density`)

Family: Theme Selector. Reference: none (an Oxyn original).
Components: `AccentDensity` (Dioxus) · `AccentDensityEl` (GPUI), models in `oxyn_core::parity::themefid`.

## 1. Purpose
Tune the accent colour, row spacing and corner shape together on a sample that shows each change at once.

## 2. Parts
| Part | Look |
|---|---|
| Controls | 200 px column: Accent (Graphite and five colours as dots), Density (Compact / Cozy / Roomy) and Corners (Square / Soft / Round) segmented |
| Sample | settings list: three rows with a name, a second line (hidden when Compact) and a switch; the selected row is ringed in the accent |
| Buttons | Cancel and Save (accent) |

## 3. Behaviour
Each choice restyles the sample through CSS variables (`--ac`, `--rh`, `--rp`, `--rg`, `--rr` from `densities` and `corners`) with short transitions. Sample switches flip and rows select. `on_change((accent, density, corners))`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default · compact square graphite · roomy round rose · green.

## 5. Keyboard and accessibility
Three radio groups; accents are named, not only coloured. Sample switches are `role="switch"`. Every change is said in a polite note.

## 6. Responsive
Controls stack above the sample under 560 px.

## 7. Known gaps
Accent does not yet retint the rest of the page. GPUI: static snapshot (host routes `acc-<i>`, `den-<i>`, `cor-<i>`).
