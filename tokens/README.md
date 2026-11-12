# tokens/

Source of truth for Oxyn's design tokens. Format: [W3C Design Tokens (DTCG) 2025.10](https://www.designtokens.org/)
(`$type` / `$value`, aliases as `{dotted.path}`). Human spec and contrast tables: [../docs/tokens.md](../docs/tokens.md).

## Files
| File | Holds |
|---|---|
| `oxyn.tokens.json` | colour, type, space, radius, border, size, density, z-index, opacity, blur, focus, elevation, themes |
| `oxyn.motion.json` | durations, easings, springs, loops, stagger, delays, `motion.reduced` (spec: `docs/motion.md`) |

### Generated (never edit by hand)
| Output | For |
|---|---|
| `dist/oxyn.css` | Web: one `--oxyn-*` custom property per semantic, motion and theme token. Themes switch with `data-oxyn-theme="dark\|light\|hc-dark"`; dark is the default on `:root`. |
| `crates/oxyn-tokens/src/generated.rs` | Rust: `Theme::{DARK, LIGHT, HC_DARK}`, `ColorRole` / `ElevationRole` enums, `primitive::*`, `semantic::*`, `motion::*` consts. `oxyn-gpui` maps these to GPUI types. |

After changing a JSON file run `cargo run -p oxyn-tokens-gen`. It validates (structure, aliases, tiers, theme parity,
WCAG contrast) and rewrites both outputs. CI runs `cargo run -p oxyn-tokens-gen -- --check` and fails on any error or
stale output.

`oxyn.tokens.json` does not repeat motion values; `semantic.motion` only points at the motion file.

## Tiers
| Tier | Root | May reference | Example |
|---|---|---|---|
| 1 Primitive | `primitive.*` | nothing (except brand pins inside `primitive`) | `primitive.color.rust.500` = `#FF5A1F` |
| 2 Semantic | `semantic.*` | primitives | `semantic.font.size.status` → `{primitive.font.size.13}` |
| 3 Theme | `theme.<dark\|light\|hc-dark>.*` | primitives, semantic, the same theme | `theme.dark.color.loader.waiting` → `{theme.dark.color.status.waiting.fg}` |

- Components read **semantic and theme tokens only**, never primitives.
- Every theme has exactly the same keys. `dark` is the default.
- Consumers drop the `theme.<name>.` prefix: `theme.dark.color.fg.default` is `color.fg.default` in code.
- Colours with alpha, OKLCH monogram pairs and the floored shimmer base are literal values; their source
  is kept in `$extensions."dev.oxyn".derivedFrom` / `source`.

## Naming
`<category>.<role>[.<variant>][.<state>]` — lowercase, words joined with `-`, no dots inside a name.

| Part | Examples |
|---|---|
| category | `color`, `font`, `space`, `radius`, `border`, `size`, `z`, `opacity`, `blur`, `focus`, `elevation`, `density` |
| role | `fg`, `surface`, `status`, `risk`, `diff`, `loader`, `git` |
| variant | `muted`, `raised`, `success`, `add`, `high` |
| state | `hover`, `pressed`, `disabled`, `-warn` suffix when a state can't be a child |

A name is either a token or a group, never both (DTCG rule). So `color.diff.add.stripe`, not `color.diff.stripe`.

## Adding a token
1. Need a new raw value? Add it to `primitive` first. Otherwise reuse one.
2. Add the role to `semantic` (theme-independent) or to **all three** themes (colour, elevation).
3. Prefer an alias over a literal. Literal only for alpha tints and computed mixes, with `derivedFrom`.
4. Run the token check (lands with #13; rules in `docs/tokens.md` §Validation): JSON parses, every alias resolves, no cycles, themes have equal key sets, and every
   text pair passes contrast (text ≥ 4.5:1, UI ≥ 3:1, high contrast 7:1 / 4.5:1).
5. Add the row to the matching table in `docs/tokens.md` and name the spec that reads it.

## Generated outputs (#13 `feat/tokens`, crate `oxyn-tokens`)
| Target | Shape |
|---|---|
| Rust | `oxyn_tokens::{primitive, semantic}` consts + `Theme` struct per theme (`Theme::dark()`, `light()`, `hc_dark()`), density as `Density::{Comfortable, Compact}` |
| CSS | custom properties `--oxyn-<path-with-dashes>` on `:root` (dark), `[data-oxyn-theme="light"]`, `[data-oxyn-theme="hc-dark"]`, `[data-oxyn-density="compact"]`; e.g. `--oxyn-color-fg-muted`, `--oxyn-space-md` |
| GPUI | theme JSON / `gpui::Theme` colours from the `theme.*` sets; font families as `font.ui` / `font.mono` with fallbacks |
| Studio | the exposed subset listed in `docs/tokens.md` §Theming |

Generated files are build output. The JSON here is the only hand-edited source.
