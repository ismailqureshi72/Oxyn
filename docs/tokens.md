# Design tokens

> Draft v0.1. Source: [`tokens/oxyn.tokens.json`](../tokens/oxyn.tokens.json) (DTCG 2025.10). Motion:
> [`tokens/oxyn.motion.json`](../tokens/oxyn.motion.json). File layout and naming: [`tokens/README.md`](../tokens/README.md).
> Generated outputs (plan #13): Rust in `oxyn-tokens` (`crates/oxyn-tokens/src/generated.rs`) and CSS variables in
> `tokens/dist/oxyn.css`, both written by `cargo run -p oxyn-tokens-gen`. CI runs it with `--check`.

## 0. Default palette: monochrome (decided 2026-11-12)
The shipped themes are black and grey (`dark`, `hc-dark`) and white and grey (`light`), like affectUI. No hue is used by
default: the accent is off-white on dark and near-black on light, and status, risk, agent, loader, diff, merge, git and
monogram roles are neutral greys. State is carried by glyph, label, weight and fill, never by colour alone. Terminal
`ansi.*` keeps its colours because it renders program output. The earlier coloured palette is kept as
`tokens/presets/ember.tokens.json`, which Studio (plan #26) applies only when the user picks colours. The brand mark is
separate from component themes. Contrast: 715 rows, 0 failures.

## 1. Principles
| # | Rule |
|---|---|
| 1 | **Ink, paper, rust.** Background `#0B0B0C`, text `#F2EFEA`, one accent `#FF5A1F`. Everything else is quiet. |
| 2 | **Accent means "the agent is doing something" or "this is the action".** Not decoration. |
| 3 | **Never colour alone.** State, risk, diff side and review severity always pair colour with a glyph, shape or word. |
| 4 | **Contrast is measured, not eyeballed.** Every text/surface pair in §8 is computed; text ≥ 4.5:1, UI ≥ 3:1, high contrast ≥ 7:1 / 4.5:1. |
| 5 | **Components read roles, not palettes.** `color.status.warning.fg`, never `amber.400`. |
| 6 | **Same keys in every theme.** A theme can change a value, never remove a role. |
| 7 | **Colour, radius, font and density are tokens, not variants** ([variants.md](variants.md)). |

## 2. Tiers
| Tier | Root | What | Count |
|---|---|---|---|
| 1 Primitive | `primitive.*` | raw palette (warm-grey neutral 0–1000, rust, red, amber, green, teal, blue, violet, pink, 4 low-chroma `-soft` scales), font families, size steps, 4 px space steps, radii, border widths | 205 |
| 2 Semantic | `semantic.*` | theme-independent roles: font, typography, space, radius, border width, size, density, z, opacity, blur, focus, voice | 94 |
| 3 Theme | `theme.dark` · `theme.light` · `theme.hc-dark` | colour roles (202), elevation (4), focus ring (1) | 207 × 3 |
| | | **Total** | **920** (585 aliases) |

Code drops the `theme.<name>.` prefix, so specs write `color.fg.muted`.

## 3. Naming
`<category>.<role>[.<variant>][.<state>]`. Lowercase, `-` inside words. A path is a token or a group, never both.

| Category | Roles |
|---|---|
| `color` | `bg`, `surface`, `fg`, `border`, `accent`, `focus`, `selection`, `scrim`, `highlight`, `link`, `status`, `state`, `risk`, `diff`, `merge`, `git`, `review`, `impact`, `icon`, `loader`, `text-status`, `agent`, `mcp`, `voice`, `monogram`, `ansi`, plus one-offs (`term`, `plan`, `feedback`, `code`, `bubble`, `checkpoint`) |
| `font` / `typography` | `ui`, `mono`, `size.*`, `weight.*`, `line-height.*`, `letter-spacing.*`; composites `typography.*` |
| `space`, `radius`, `border.width`, `size` | t-shirt roles over a px primitive scale |
| `density` | `comfortable` (default), `compact` |
| `z`, `opacity`, `blur`, `focus`, `elevation` | layers, alphas, blur radii, ring, shadows |

### Spec names → token names
Specs were written before this file. Where a spec name could not be kept, this is the mapping.

| Name in spec | Token | Why |
|---|---|---|
| `text.fg`, `fg` (text-reveal, text-shimmer, checkpoints-git) | `color.fg.default` | one fg family |
| `fg.muted`, `fg-muted` (icons), `text.secondary` (reasoning-block) | `color.fg.muted` | three spellings of one role |
| `text.status.fg / .base / .crest` (status-text, text-shimmer) | `color.text-status.fg / .base / .crest` | `color.` prefix; `text` is not a category |
| `tone.warning / .danger / .success`, bare `warn`, `danger`, `success` | `color.status.<name>.fg` (`.border`, `.bg`) | status has three parts |
| `risk.low … risk.critical` (approvals) | `color.risk.<tier>.{fg,border,bg}`, `semantic.border.width.risk-<tier>`, `color.risk.critical-hatch` | text, border and fill differ |
| `focus.ring` (composer-model-picker) | `theme.<t>.focus.ring` (border composite) + `semantic.focus.{width,offset}` + `color.focus` | ring colour is themed, size is not |
| `surface.sunken` | `color.surface.sunken` | prefix |
| `color.diff.stripe` (checkpoints-git) | `color.diff.add.stripe`, `color.diff.del.stripe` | stripe needs a hue per side |
| `color.diff.hunk.header` = "fg.muted on surface.sunken" | `color.diff.hunk.header` (text) + `color.diff.hunk.bg` | one token = one colour |
| `color.git.behind` "→ amber at `behind_warn`" | `color.git.behind` + `color.git.behind-warn` | two values |
| `color.loader.spectral[0..n]` (3–6 stops) | `color.loader.spectral.0 … .5` | fixed at 6 |
| `color.monogram[0..12]` (plugin-icons, "12 hues") | `color.monogram.0 … .11`, each `{bg, fg}` | 12 slots, tile + letter pair |
| `color.agent.{red..cyan}` (subagent-tree, "8 identity tokens") | `color.agent.{red, orange, yellow, green, blue, violet, pink, cyan}` | order proposed |
| `color.ansi.0…15` (terminal-output) | `color.ansi.0 … .15` | kept |
| `motion.reduced`, `dur.*`, `ease.*`, `spring.*` | `oxyn.motion.json` | not in this file |

## 4. Primitives

### 4.1 Neutral (warm grey)
`50` and `950` are brand pins. Generated in OKLCH at hue 75, chroma ≤ 0.0075.

| Step | Hex | | Step | Hex |
|---|---|---|---|---|
| 0 | `#FFFFFF` | | 600 | `#6A6763` |
| 25 | `#FDF9F5` | | 700 | `#53514E` |
| 50 | `#F2EFEA` | | 800 | `#3D3B39` |
| 100 | `#E7E4DF` | | 850 | `#2E2C2A` |
| 150 | `#DCD8D4` | | 900 | `#22201E` |
| 200 | `#CFCBC7` | | 925 | `#1A1816` |
| 300 | `#B7B4AF` | | 950 | `#0B0B0C` |
| 400 | `#9B9893` | | 975 | `#070604` |
| 500 | `#817E7A` | | 1000 | `#000000` |

### 4.2 Hue scales
OKLCH ladders, same lightness per step across hues (L 0.97 → 0.27), gamut-mapped by chroma. `rust.500` is pinned
to the brand `#FF5A1F`. `-soft` scales are 45 % chroma, for "low chroma" states in the specs.

| Step | rust | red | amber | green | teal | blue | violet | pink |
|---|---|---|---|---|---|---|---|---|
| 50 | `#FFF2EE` | `#FFF1F2` | `#FDF4E7` | `#EDF9EE` | `#EBF8F7` | `#EFF6FF` | `#F6F3FF` | `#FFF0FA` |
| 100 | `#FFE2DA` | `#FFE1E3` | `#F9E7CB` | `#D8F2DC` | `#D5F0EE` | `#DCEBFF` | `#EBE5FF` | `#FEDFF4` |
| 200 | `#FFC9B9` | `#FFC7CC` | `#F6D19A` | `#B2E7BC` | `#ACE4E0` | `#BEDAFF` | `#DAD0FF` | `#FDC2EB` |
| 300 | `#FFA78D` | `#FFA3AE` | `#EEB658` | `#83D896` | `#77D4CE` | `#96C4FF` | `#C6B3FF` | `#F89EDF` |
| 400 | `#FF845F` | `#FF7E92` | `#E19E00` | `#5AC777` | `#45C2BC` | `#70AFFF` | `#B498FF` | `#EC81D0` |
| 500 | `#FF5A1F` | `#F65274` | `#C48800` | `#37B05D` | `#06ABA5` | `#4996F5` | `#9E7CF1` | `#D566B9` |
| 600 | `#D04000` | `#D0365A` | `#9F6E00` | `#149144` | `#008B86` | `#3079D0` | `#8260CD` | `#B24C99` |
| 700 | `#B13500` | `#B4274B` | `#875D00` | `#007B36` | `#007671` | `#2366B4` | `#6E4FB1` | `#9A3D83` |
| 800 | `#8C2800` | `#8F1B39` | `#6A4800` | `#006129` | `#005D59` | `#184F90` | `#563D8D` | `#7A2E68` |
| 900 | `#691C00` | `#6B1429` | `#4F3500` | `#01481D` | `#004441` | `#113A6B` | `#3F2C69` | `#5A214D` |
| 950 | `#481000` | `#4B0519` | `#352200` | `#003011` | `#002E2B` | `#05264C` | `#2A1B4B` | `#3F1235` |

`-soft` scales: `rust-soft`, `amber-soft`, `green-soft`, `violet-soft` (same steps).

## 5. Semantic colour (per theme)
Source column shows the dark-theme alias; light and HC alias other steps of the same family. `@N%` = alpha.

#### `color.bg.*`
Used by: all.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.bg.base` | `#0B0B0C` | `#FDF9F5` | `#000000` | neutral.950 |  |

#### `color.surface.*`
Used by: all.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.surface.sunken` | `#070604` | `#F2EFEA` | `#000000` | neutral.975 |  |
| `color.surface.base` | `#0B0B0C` | `#FDF9F5` | `#000000` | bg.base |  |
| `color.surface.raised` | `#22201E` | `#FFFFFF` | `#0B0B0C` | neutral.900 |  |
| `color.surface.overlay` | `#2E2C2A` | `#FFFFFF` | `#1A1816` | neutral.850 |  |
| `color.surface.hover` | `#F2EFEA @6%` | `#0B0B0C @5%` | `#F2EFEA @10%` | fg.default @6% |  |
| `color.surface.pressed` | `#F2EFEA @10%` | `#0B0B0C @9%` | `#F2EFEA @16%` | fg.default @10% |  |
| `color.surface.selected` | `#FF5A1F @16%` | `#D04000 @12%` | `#FF845F @24%` | accent.solid @16% |  |

#### `color.fg.*`
Used by: all.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.fg.default` | `#F2EFEA` | `#0B0B0C` | `#F2EFEA` | neutral.50 |  |
| `color.fg.muted` | `#B7B4AF` | `#53514E` | `#DCD8D4` | neutral.300 |  |
| `color.fg.subtle` | `#9B9893` | `#6A6763` | `#CFCBC7` | neutral.400 |  |
| `color.fg.disabled` | `#6A6763` | `#B7B4AF` | `#817E7A` | neutral.600 |  |
| `color.fg.on-accent` | `#0B0B0C` | `#FFFFFF` | `#000000` | neutral.950 |  |
| `color.fg.inverse` | `#0B0B0C` | `#F2EFEA` | `#000000` | neutral.950 |  |

#### `color.border.*`
Used by: all.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.border.subtle` | `#2E2C2A` | `#E7E4DF` | `#53514E` | neutral.850 | workspace-chrome |
| `color.border.default` | `#3D3B39` | `#DCD8D4` | `#817E7A` | neutral.800 |  |
| `color.border.strong` | `#817E7A` | `#817E7A` | `#B7B4AF` | neutral.500 |  |

#### `color.accent.*`
Used by: all.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.accent.solid` | `#FF5A1F` | `#D04000` | `#FF845F` | rust.500 |  |
| `color.accent.solid-hover` | `#FF845F` | `#B13500` | `#FFA78D` | rust.400 |  |
| `color.accent.solid-pressed` | `#FFA78D` | `#8C2800` | `#FFC9B9` | rust.300 |  |
| `color.accent.fg` | `#FF845F` | `#B13500` | `#FFA78D` | rust.400 |  |
| `color.accent.border` | `#FF5A1F` | `#D04000` | `#FFA78D` | rust.500 |  |
| `color.accent.bg` | `#FF5A1F @14%` | `#FF5A1F @10%` | `#FF5A1F @20%` | rust.500 @14% |  |

#### `color.status.*`
Used by: all.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.status.success.fg` | `#5AC777` | `#007B36` | `#83D896` | green.400 |  |
| `color.status.success.border` | `#37B05D` | `#149144` | `#83D896` | green.500 |  |
| `color.status.success.bg` | `#37B05D @14%` | `#37B05D @10%` | `#37B05D @20%` | green.500 @14% |  |
| `color.status.warning.fg` | `#E19E00` | `#875D00` | `#EEB658` | amber.400 |  |
| `color.status.warning.border` | `#C48800` | `#9F6E00` | `#EEB658` | amber.500 |  |
| `color.status.warning.bg` | `#C48800 @14%` | `#C48800 @10%` | `#C48800 @20%` | amber.500 @14% |  |
| `color.status.danger.fg` | `#FF7E92` | `#B4274B` | `#FFA3AE` | red.400 |  |
| `color.status.danger.border` | `#F65274` | `#D0365A` | `#FFA3AE` | red.500 |  |
| `color.status.danger.bg` | `#F65274 @14%` | `#F65274 @10%` | `#F65274 @20%` | red.500 @14% |  |
| `color.status.info.fg` | `#70AFFF` | `#2366B4` | `#96C4FF` | blue.400 |  |
| `color.status.info.border` | `#4996F5` | `#3079D0` | `#96C4FF` | blue.500 |  |
| `color.status.info.bg` | `#4996F5 @14%` | `#4996F5 @10%` | `#4996F5 @20%` | blue.500 @14% |  |
| `color.status.waiting.fg` | `#E19E00` | `#875D00` | `#EEB658` | amber.400 |  |
| `color.status.waiting.border` | `#C48800` | `#9F6E00` | `#EEB658` | amber.500 |  |
| `color.status.waiting.bg` | `#C48800 @14%` | `#C48800 @10%` | `#C48800 @20%` | amber.500 @14% |  |

#### `color.state.*`
Used by: architecture (AgentState), loaders, status-text, tools.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.state.idle` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted |  |
| `color.state.composing` | `#F2EFEA` | `#0B0B0C` | `#F2EFEA` | fg.default |  |
| `color.state.submitted` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg |  |
| `color.state.thinking` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg |  |
| `color.state.streaming` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg |  |
| `color.state.tool-running` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg |  |
| `color.state.awaiting-approval` | `#E19E00` | `#875D00` | `#EEB658` | status.waiting.fg |  |
| `color.state.queued` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted |  |
| `color.state.steering` | `#70AFFF` | `#2366B4` | `#96C4FF` | status.info.fg |  |
| `color.state.done` | `#5AC777` | `#007B36` | `#83D896` | status.success.fg |  |
| `color.state.error` | `#FF7E92` | `#B4274B` | `#FFA3AE` | status.danger.fg |  |
| `color.state.cancelled` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted |  |
| `color.state.stalled` | `#E19E00` | `#875D00` | `#EEB658` | status.warning.fg |  |

#### `color.risk.*`
Used by: approvals, approval-card, risk-badge.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.risk.low.fg` | `#70AFFF` | `#2366B4` | `#96C4FF` | status.info.fg |  |
| `color.risk.low.border` | `#4996F5` | `#3079D0` | `#96C4FF` | status.info.border |  |
| `color.risk.low.bg` | `#4996F5 @14%` | `#4996F5 @10%` | `#4996F5 @20%` | status.info.bg |  |
| `color.risk.medium.fg` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg |  |
| `color.risk.medium.border` | `#FF5A1F` | `#D04000` | `#FFA78D` | accent.border |  |
| `color.risk.medium.bg` | `#FF5A1F @14%` | `#FF5A1F @10%` | `#FF5A1F @20%` | accent.bg |  |
| `color.risk.high.fg` | `#E19E00` | `#875D00` | `#EEB658` | status.warning.fg |  |
| `color.risk.high.border` | `#C48800` | `#9F6E00` | `#EEB658` | status.warning.border |  |
| `color.risk.high.bg` | `#C48800 @14%` | `#C48800 @10%` | `#C48800 @20%` | status.warning.bg |  |
| `color.risk.critical.fg` | `#FF7E92` | `#B4274B` | `#FFA3AE` | status.danger.fg |  |
| `color.risk.critical.border` | `#F65274` | `#D0365A` | `#FFA3AE` | status.danger.border |  |
| `color.risk.critical.bg` | `#F65274 @14%` | `#F65274 @10%` | `#F65274 @20%` | status.danger.bg |  |
| `color.risk.critical-hatch` | `#F65274 @8%` | `#D0365A @8%` | `#FFA3AE @8%` | red.500 @8% |  |

#### `color.diff.*`
Used by: checkpoints-git, diff-viewer, file-edit-chip.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.diff.add.fg` | `#5AC777` | `#007B36` | `#83D896` | status.success.fg | checkpoints-git, file-edit-chip |
| `color.diff.add.bg` | `#37B05D @14%` | `#37B05D @10%` | `#37B05D @22%` | green.500 @14% | checkpoints-git, diff-viewer |
| `color.diff.add.word` | `#37B05D @34%` | `#37B05D @26%` | `#37B05D @36%` | green.500 @34% | checkpoints-git |
| `color.diff.add.stripe` | `#37B05D` | `#149144` | `#83D896` | green.500 |  |
| `color.diff.gutter.add` | `#37B05D @22%` | `#37B05D @18%` | `#37B05D @30%` | green.500 @22% | checkpoints-git |
| `color.diff.gutter.del` | `#F65274 @22%` | `#F65274 @18%` | `#F65274 @30%` | red.500 @22% |  |
| `color.diff.del.fg` | `#FF7E92` | `#B4274B` | `#FFA3AE` | status.danger.fg | checkpoints-git, file-edit-chip |
| `color.diff.del.bg` | `#F65274 @14%` | `#F65274 @10%` | `#F65274 @22%` | red.500 @14% | checkpoints-git, diff-viewer |
| `color.diff.del.word` | `#F65274 @34%` | `#F65274 @26%` | `#F65274 @36%` | red.500 @34% | checkpoints-git |
| `color.diff.del.stripe` | `#F65274` | `#D0365A` | `#FFA3AE` | red.500 |  |
| `color.diff.hunk.header` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted | checkpoints-git |
| `color.diff.hunk.bg` | `#070604` | `#F2EFEA` | `#000000` | surface.sunken |  |
| `color.diff.kept` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted | checkpoints-git |
| `color.diff.undone` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted |  |

#### `color.merge.*`
Used by: merge-conflict.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.merge.current` | `#70AFFF` | `#2366B4` | `#96C4FF` | blue.400 | merge-conflict |
| `color.merge.incoming` | `#45C2BC` | `#007671` | `#77D4CE` | teal.400 | merge-conflict |
| `color.merge.base` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted | merge-conflict |
| `color.merge.conflict` | `#E19E00` | `#875D00` | `#EEB658` | loader.waiting | merge-conflict |
| `color.merge.suggested` | `#AFA4D6` | `#695E89` | `#C3BAE4` | violet-soft.400 | merge-conflict |
| `color.merge.resolved` | `#8EB996` | `#4B7153` | `#A8CCAE` | green-soft.400 | merge-conflict |
| `color.merge.marker` | `#F65274 @16%` | `#F65274 @16%` | `#F65274 @16%` | red.500 @16% | merge-conflict |

#### `color.git.*`
Used by: git-chips.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.git.branch` | `#F2EFEA` | `#0B0B0C` | `#F2EFEA` | fg.default | git-chips |
| `color.git.ahead` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted | git-chips |
| `color.git.behind` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted | git-chips |
| `color.git.behind-warn` | `#E19E00` | `#875D00` | `#EEB658` | status.warning.fg |  |
| `color.git.dirty` | `#C5A77A` | `#7B6139` | `#D6BD97` | amber-soft.400 | git-chips |
| `color.git.detached` | `#C5A77A` | `#7B6139` | `#D6BD97` | amber-soft.400 | git-chips |
| `color.git.worktree` | `#DF9983` | `#8F5442` | `#ECB19F` | rust-soft.400 | git-chips |

#### `color.review.*`
Used by: review-card.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.review.critical` | `#FF7E92` | `#B4274B` | `#FFA3AE` | status.danger.fg | review-card |
| `color.review.warning` | `#E19E00` | `#875D00` | `#EEB658` | status.warning.fg | review-card |
| `color.review.suggestion` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg | review-card |
| `color.review.nit` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted | review-card |
| `color.review.info` | `#F2EFEA` | `#0B0B0C` | `#F2EFEA` | fg.default | review-card |

#### `color.impact.*`
Used by: rollback-impact, checkpoints-git.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.impact.external` | `#E19E00` | `#875D00` | `#EEB658` | status.warning.fg | checkpoints-git, rollback-impact |
| `color.impact.irreversible` | `#FF7E92` | `#B4274B` | `#FFA3AE` | status.danger.fg | checkpoints-git |

#### `color.icon.*`
Used by: icons, icon-states.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.icon.fg` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted | icons |
| `color.icon.fg-strong` | `#F2EFEA` | `#0B0B0C` | `#F2EFEA` | fg.default | icons |
| `color.icon.on-accent` | `#0B0B0C` | `#FFFFFF` | `#000000` | fg.on-accent | icons |

#### `color.loader.*`
Used by: loaders, loader-*, animated-marks, thinking-orb, tools, terminal-tabs.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.loader.fg` | `#F2EFEA` | `#0B0B0C` | `#F2EFEA` | fg.default | loaders |
| `color.loader.track` | `#F2EFEA @20%` | `#0B0B0C @20%` | `#F2EFEA @20%` | neutral.50 @20% | loaders |
| `color.loader.accent` | `#FF5A1F` | `#D04000` | `#FF845F` | accent.solid | loaders |
| `color.loader.waiting` | `#E19E00` | `#875D00` | `#EEB658` | status.waiting.fg | animated-marks, approvals, loader-glyph, loader-vector, loaders, merge-conflict, terminal-tabs, tools |
| `color.loader.warn` | `#E19E00` | `#875D00` | `#EEB658` | status.warning.fg | animated-marks, loaders |
| `color.loader.danger` | `#FF7E92` | `#B4274B` | `#FFA3AE` | status.danger.fg | animated-marks, loaders |
| `color.loader.success` | `#5AC777` | `#007B36` | `#83D896` | status.success.fg | loaders |
| `color.loader.spectral.0` | `#FF5A1F` | `#D04000` | `#FF845F` | accent.solid |  |
| `color.loader.spectral.1` | `#E19E00` | `#9F6E00` | `#EEB658` | amber.400 |  |
| `color.loader.spectral.2` | `#EC81D0` | `#B24C99` | `#F89EDF` | pink.400 |  |
| `color.loader.spectral.3` | `#B498FF` | `#8260CD` | `#C6B3FF` | violet.400 |  |
| `color.loader.spectral.4` | `#70AFFF` | `#3079D0` | `#96C4FF` | blue.400 |  |
| `color.loader.spectral.5` | `#45C2BC` | `#008B86` | `#77D4CE` | teal.400 |  |

#### `color.text-status.*`
Used by: status-text, text-shimmer, verb-cycler.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.text-status.fg` | `#F2EFEA` | `#0B0B0C` | `#F2EFEA` | fg.default |  |
| `color.text-status.crest` | `#F2EFEA` | `#0B0B0C` | `#F2EFEA` | fg.default |  |
| `color.text-status.base` | `#969491` | `#605E5E` | `#F2EFEA` | mix(fg → bg), floored 4.5:1 |  |

#### `color.agent.*`
Used by: subagent-tree, fanout-grid, plan-todo.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.agent.red` | `#FF7E92` | `#D0365A` | `#FFA3AE` | red.400 |  |
| `color.agent.orange` | `#FF845F` | `#D04000` | `#FFA78D` | rust.400 |  |
| `color.agent.yellow` | `#E19E00` | `#9F6E00` | `#EEB658` | amber.400 |  |
| `color.agent.green` | `#5AC777` | `#149144` | `#83D896` | green.400 |  |
| `color.agent.blue` | `#70AFFF` | `#3079D0` | `#96C4FF` | blue.400 |  |
| `color.agent.violet` | `#B498FF` | `#8260CD` | `#C6B3FF` | violet.400 |  |
| `color.agent.pink` | `#EC81D0` | `#B24C99` | `#F89EDF` | pink.400 |  |
| `color.agent.cyan` | `#45C2BC` | `#008B86` | `#77D4CE` | teal.400 |  |

#### `color.mcp.*`
Used by: mcp-plugin-list.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.mcp.disconnected` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted |  |
| `color.mcp.connecting` | `#70AFFF` | `#2366B4` | `#96C4FF` | status.info.fg |  |
| `color.mcp.connected` | `#5AC777` | `#007B36` | `#83D896` | status.success.fg |  |
| `color.mcp.needs-auth` | `#E19E00` | `#875D00` | `#EEB658` | status.waiting.fg |  |
| `color.mcp.pending-approval` | `#E19E00` | `#875D00` | `#EEB658` | status.waiting.fg |  |
| `color.mcp.error` | `#FF7E92` | `#B4274B` | `#FFA3AE` | status.danger.fg |  |
| `color.mcp.disabled` | `#9B9893` | `#6A6763` | `#CFCBC7` | fg.subtle |  |
| `color.mcp.blocked` | `#FF7E92` | `#B4274B` | `#FFA3AE` | status.danger.fg |  |
| `color.mcp.updating` | `#70AFFF` | `#2366B4` | `#96C4FF` | status.info.fg |  |
| `color.mcp.flag-warning` | `#E19E00` | `#875D00` | `#EEB658` | status.warning.fg |  |

#### `color.voice.*`
Used by: live-voice, voice-orb, voice-control-bar, live-captions.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.voice.idle` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted |  |
| `color.voice.connecting` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted |  |
| `color.voice.listening` | `#70AFFF` | `#2366B4` | `#96C4FF` | status.info.fg |  |
| `color.voice.user-speaking` | `#70AFFF` | `#2366B4` | `#96C4FF` | status.info.fg |  |
| `color.voice.thinking` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg |  |
| `color.voice.agent-speaking` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg |  |
| `color.voice.awaiting-approval` | `#E19E00` | `#875D00` | `#EEB658` | status.waiting.fg |  |
| `color.voice.interrupted` | `#E19E00` | `#875D00` | `#EEB658` | status.warning.fg |  |
| `color.voice.muted` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted |  |
| `color.voice.asleep` | `#9B9893` | `#6A6763` | `#CFCBC7` | fg.subtle |  |
| `color.voice.latency-good` | `#5AC777` | `#007B36` | `#83D896` | status.success.fg |  |
| `color.voice.latency-ok` | `#E19E00` | `#875D00` | `#EEB658` | status.warning.fg |  |
| `color.voice.latency-slow` | `#FF7E92` | `#B4274B` | `#FFA3AE` | status.danger.fg |  |
| `color.voice.latency-very-slow` | `#FF7E92` | `#B4274B` | `#FFA3AE` | status.danger.fg |  |

#### `color.monogram.*`
Used by: plugin-icons.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.monogram.0.bg` | `#602A27` | `#FFD6D2` | `#4B1D1B` | oklch(0.36 0.08 25) |  |
| `color.monogram.0.fg` | `#FFDBD7` | `#742E2B` | `#FFF2F0` | oklch(0.92 0.05 25) |  |
| `color.monogram.1.bg` | `#5C300B` | `#FDD9C2` | `#482304` | oklch(0.36 0.08 55) |  |
| `color.monogram.1.fg` | `#FFDDC7` | `#6F3600` | `#FFF2EA` | oklch(0.92 0.05 55) |  |
| `color.monogram.2.bg` | `#4F3900` | `#F1DFBC` | `#3C2B00` | oklch(0.36 0.08 85) |  |
| `color.monogram.2.fg` | `#F4E3BF` | `#5C4300` | `#FBF4E6` | oklch(0.92 0.05 85) |  |
| `color.monogram.3.bg` | `#3C4202` | `#E0E6C0` | `#2D3100` | oklch(0.36 0.08 115) |  |
| `color.monogram.3.fg` | `#E3E9C4` | `#474D00` | `#F4F7E8` | oklch(0.92 0.05 115) |  |
| `color.monogram.4.bg` | `#1E4821` | `#CEEBCD` | `#133716` | oklch(0.36 0.08 145) |  |
| `color.monogram.4.fg` | `#D1EED1` | `#1D5522` | `#EDF9ED` | oklch(0.92 0.05 145) |  |
| `color.monogram.5.bg` | `#00483B` | `#C0EDDF` | `#00372C` | oklch(0.36 0.08 175) |  |
| `color.monogram.5.fg` | `#C3F0E3` | `#005545` | `#E8FAF4` | oklch(0.92 0.05 175) |  |
| `color.monogram.6.bg` | `#00464C` | `#BBECF1` | `#00353A` | oklch(0.36 0.08 205) |  |
| `color.monogram.6.fg` | `#BFEFF4` | `#005259` | `#E7F9FC` | oklch(0.92 0.05 205) |  |
| `color.monogram.7.bg` | `#00435F` | `#C2E7FE` | `#003249` | oklch(0.36 0.08 235) |  |
| `color.monogram.7.fg` | `#C8EAFF` | `#004E6F` | `#EBF7FF` | oklch(0.92 0.05 235) |  |
| `color.monogram.8.bg` | `#283B67` | `#D3E2FF` | `#1C2C51` | oklch(0.36 0.08 265) |  |
| `color.monogram.8.fg` | `#D8E5FF` | `#2D457D` | `#F0F5FF` | oklch(0.92 0.05 265) |  |
| `color.monogram.9.bg` | `#413362` | `#E3DCFF` | `#31254D` | oklch(0.36 0.08 295) |  |
| `color.monogram.9.fg` | `#E6E0FF` | `#4C3B77` | `#F6F3FF` | oklch(0.92 0.05 295) |  |
| `color.monogram.10.bg` | `#522D54` | `#F3D7F4` | `#3F2041` | oklch(0.36 0.08 325) |  |
| `color.monogram.10.fg` | `#F6DAF7` | `#623265` | `#FCF1FD` | oklch(0.92 0.05 325) |  |
| `color.monogram.11.bg` | `#5D293F` | `#FED4E3` | `#481D30` | oklch(0.36 0.08 355) |  |
| `color.monogram.11.fg` | `#FFD9E6` | `#6F2D4A` | `#FFF1F6` | oklch(0.92 0.05 355) |  |

#### `color.ansi.*`
Used by: terminal-output, terminal-tabs.

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.ansi.0` | `#2E2C2A` | `#0B0B0C` | `#3D3B39` | neutral.850 | terminal-output |
| `color.ansi.1` | `#FF7E92` | `#B4274B` | `#FFA3AE` | red.400 |  |
| `color.ansi.2` | `#5AC777` | `#007B36` | `#83D896` | green.400 |  |
| `color.ansi.3` | `#E19E00` | `#875D00` | `#EEB658` | amber.400 |  |
| `color.ansi.4` | `#70AFFF` | `#2366B4` | `#96C4FF` | blue.400 |  |
| `color.ansi.5` | `#EC81D0` | `#9A3D83` | `#F89EDF` | pink.400 |  |
| `color.ansi.6` | `#45C2BC` | `#007671` | `#77D4CE` | teal.400 |  |
| `color.ansi.7` | `#CFCBC7` | `#B7B4AF` | `#E7E4DF` | neutral.200 |  |
| `color.ansi.8` | `#9B9893` | `#6A6763` | `#CFCBC7` | neutral.400 |  |
| `color.ansi.9` | `#FFA3AE` | `#8F1B39` | `#FFC7CC` | red.300 |  |
| `color.ansi.10` | `#83D896` | `#006129` | `#B2E7BC` | green.300 |  |
| `color.ansi.11` | `#EEB658` | `#6A4800` | `#F6D19A` | amber.300 |  |
| `color.ansi.12` | `#96C4FF` | `#184F90` | `#BEDAFF` | blue.300 |  |
| `color.ansi.13` | `#F89EDF` | `#7A2E68` | `#FDC2EB` | pink.300 |  |
| `color.ansi.14` | `#77D4CE` | `#005D59` | `#ACE4E0` | teal.300 |  |
| `color.ansi.15` | `#F2EFEA` | `#CFCBC7` | `#FFFFFF` | neutral.50 |  |

#### focus, selection & component one-offs

| Token | Dark | Light | HC dark | Source (dark) | Named in specs |
|---|---|---|---|---|---|
| `color.focus` | `#FF5A1F` | `#D04000` | `#FFA78D` | rust.500 | workspace-chrome |
| `color.selection` | `#FF5A1F @30%` | `#FF5A1F @20%` | `#FF5A1F @40%` | rust.500 @30% |  |
| `color.scrim` | `#000000 @60%` | `#0B0B0C @40%` | `#000000 @80%` | neutral.1000 @60% |  |
| `color.highlight` | `#FF5A1F` | `#D04000` | `#FF845F` | accent.solid | messages |
| `color.link` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg |  |
| `color.term.prompt` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg | terminal-output |
| `color.plan.changed` | `#FF5A1F` | `#D04000` | `#FF845F` | accent.solid | plan-todo |
| `color.feedback.on` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg | message-bubble |
| `color.code.inline.bg` | `#22201E` | `#E7E4DF` | `#1A1816` | neutral.900 | streaming-markdown |
| `color.bubble.user` | `#22201E` | `#E7E4DF` | `#1A1816` | neutral.900 | message-bubble |
| `color.checkpoint.marker` | `#B7B4AF` | `#53514E` | `#DCD8D4` | fg.muted | checkpoints-git |
| `color.checkpoint.current` | `#FF845F` | `#B13500` | `#FFA78D` | accent.fg | checkpoint-timeline |

### 5.1 Role notes
| Role | Mapping | Spec |
|---|---|---|
| Agent state | Idle/Queued/Cancelled → `fg.muted`; Composing → `fg.default`; Submitted/Thinking/Streaming/ToolRunning → `accent.fg`; AwaitingApproval → `status.waiting`; Steering → `status.info`; Done → `status.success`; Error → `status.danger`; Stalled → `status.warning` | architecture.md |
| Risk tiers | Low → info (blue), Medium → accent (rust), High → warning (amber), Critical → danger (red) | approvals.md §2.3 |
| MCP states | Disconnected → muted; Connecting/Updating → info; Connected → success; NeedsAuth/PendingApproval → waiting; Error/Blocked → danger; Disabled → subtle; overlay flags (`ToolsFetchFailed`, rate-limited) → `mcp.flag-warning` | mcp-plugin-list.md §1.1 |
| Voice | cool while taking sound in (Listening, UserSpeaking → info), warm while giving out (Thinking, AgentSpeaking → accent); Interrupted → warning; Muted/Idle → muted; Asleep → subtle; latency good/ok/slow/very-slow → success/warning/danger/danger | live-voice.md, voice-orb.md |
| Waiting vs warning | same amber today, separate roles so a theme can split "needs you" from "something is off" | loaders.md §5 |
| `highlight`, `plan.changed` | stored solid; components apply `semantic.opacity.highlight` (24 %) or their own tint | messages.md, plan-todo.md |
| `merge.current/incoming` tints | stored solid; hover tint at `semantic.opacity.tint` (18 %) | merge-conflict.md |
| Diff tints | pre-computed alpha: rows 14 % / 10 % / 22 %, words 34 % / 26 % / 36 %, gutters 22 % / 18 % / 30 % (dark / light / HC) | checkpoints-git.md §tokens |
| `icon.fg`, `loader.fg` | specs say `currentColor`; the token is the default when no colour is inherited | icons.md, loaders.md |
| `text-status.base` | mix(fg → bg, 40 % dark / 35 % light), then pulled toward fg until ≥ 4.5:1 on bg, raised and overlay. HC = fg (shimmer off) | text-shimmer.md |
| Monogram | 12 hue slots, 30° apart from 25°; tile/letter OKLCH pairs: dark (0.36 0.08 / 0.92 0.05), light (0.91 0.05 / 0.40 0.10), HC (0.30 0.07 / 0.97 0.02) | plugin-icons.md §5.3 |
| ANSI | 1–6 red, green, amber, blue, pink (magenta), teal (cyan); 9–14 brighter (light theme: darker) steps | terminal-output.md |

## 6. Type
| Token | Value | Notes |
|---|---|---|
| `font.ui` | Inter, system-ui, -apple-system, Segoe UI, Roboto, Noto Sans, sans-serif | |
| `font.mono` | JetBrains Mono, SF Mono, Cascadia Code, Menlo, Consolas, Noto Sans Mono, monospace | |
| `font.size.caption / ui / body / body-dense / title / heading / display` | 12 / 13 / 15 / 13 / 18 / 24 / 30 px | primitive steps 11–36 |
| `font.size.status` | 13 px desktop; mobile 15 px; terminal 1 cell (`$extensions` surface overrides) | status-text.md |
| `font.weight.regular / medium / semibold / bold` | 400 / 500 / 600 / 700 | ANSI bold = 600 |
| `font.line-height.glyph / tight / snug / normal / relaxed` | 1.0 / 1.2 / 1.35 / 1.5 / 1.6 | |
| `font.letter-spacing.tight / none / caps` | −0.01 rem / 0 / 0.02 rem | |

**Tabular figures.** Anything that counts or ticks (elapsed timers, `+12 −3`, `3/5`, token meters, latency) sets
tabular numerals: CSS `font-variant-numeric: tabular-nums`, GPUI `FontFeatures` `tnum`. `−` is U+2212.

| Composite `typography.*` | Family | Size / line-height | Weight |
|---|---|---|---|
| `body` | ui | 15 / 1.6 (24 px) | 400 |
| `body-dense` | ui | 13 / 1.5 (≈ 20 px) | 400 |
| `ui` | ui | 13 / 1.35 | 500 |
| `caption` | ui | 12 / 1.35 | 400 |
| `status` | ui | 13 / 1.35 | 400 |
| `code` | mono | 13 / 1.5 | 400 |
| `title` | ui | 18 / 1.35 | 600 |
| `heading` | ui | 24 / 1.2 | 600 |

## 7. Space, shape, layers

### 7.1 Space (4 px base)
| Token | px | | Primitive steps |
|---|---|---|---|
| `space.none / hair / xs / sm / md / lg / xl / 2xl / 3xl` | 0 / 2 / 4 / 8 / 12 / 16 / 24 / 32 / 48 | | `primitive.space.{0,2,4,6,8,12,16,20,24,32,40,48,64}` (key = px) |

### 7.2 Radius
| Token | px | Use |
|---|---|---|
| `radius.none` | 0 | terminal |
| `radius.xs` | 2 | diff word highlights |
| `radius.sm` | 4 | chips, badges |
| `radius.md` | 8 | buttons, inputs, rows |
| `radius.lg` | 12 | cards, popovers, composer |
| `radius.bubble` / `bubble-grouped` | 18 / 6 | message bubble |
| `radius.full` | 999 | pills, dots |

### 7.3 Border width, size, density
| Token | Value |
|---|---|
| `border.width.none / hairline / thick / stripe` | 0 / 1 / 2 / 3 px |
| `border.width.risk-low / -medium / -high / -critical` | 1 / 1 / 2 / 2 px |
| `size.icon-xs / -sm / -md / -lg` | 12 / 16 / 20 / 24 px |
| `size.dot` | 6 px |
| `size.touch-target` | 44 px |

| Density | control-height | row-height | padding-x | padding-y | gap | font-size |
|---|---|---|---|---|---|---|
| `comfortable` (default) | 36 | 32 | 12 | 8 | 8 | 15 (`body`) |
| `compact` (IDE, dense lists) | 28 | 28 | 8 | 4 | 4 | 13 (`body-dense`) |

### 7.4 Elevation (per theme)
| Token | Dark | Light | HC dark |
|---|---|---|---|
| `elevation.raised` | 0 1 2 0 black 50 % | 0 1 2 0 black 6 % | none (use `border.strong`) |
| `elevation.popover` | 0 4 12 0 black 55 % | 0 4 12 0 black 10 % | none |
| `elevation.modal` | 0 12 32 −4 black 60 % | 0 12 32 −4 black 14 % | none |
| `elevation.approval` | 0 16 40 −4 black 65 % | 0 16 40 −4 black 16 % | none |

Dark surfaces also step up in lightness (`surface.raised` → `overlay`), so shadow is never the only cue.

### 7.5 Z-index, opacity, blur, focus
| Token | Value | Note |
|---|---|---|
| `z.base / raised / sticky` | 0 / 10 / 100 | |
| `z.overlay` | 1000 | scrims, side sheets |
| `z.popover` | 1100 | menus, pickers, tooltips |
| `z.toast` | 1200 | |
| `z.approval` | 1300 | an approval is never covered by a toast |
| `z.max` | 9999 | dev overlays only |
| `opacity.disabled` | 0.4 | if a glyph drops below 3:1, use `color.fg.disabled` (icons.md) |
| `opacity.stalled` | 0.6 | soft stall |
| `opacity.track` | 0.2 | loader tracks |
| `opacity.tint` / `highlight` | 0.18 / 0.24 | hover tint / deep-link flash |
| `opacity.hatch` / `tab-tint` | 0.08 / 0.08 | critical header hatch / awaiting-input tab |
| `blur.sm / md / lg` | 8 / 16 / 24 px | `lg` = glass composer |
| `focus.width` / `focus.offset` | 2 px / 2 px | ring = `theme.<t>.focus.ring`, colour `color.focus`; keyboard focus only, no animation |
| `voice.asleep-brightness` | 0.4 | the ±10 % audio luminance cap is `motion.audio.luminance-max` in the motion file |
| `semantic.motion` | pointer | all motion lives in `tokens/oxyn.motion.json` |

## 8. Contrast
Computed with the WCAG 2.x relative-luminance formula. Alpha colours are composited onto the surface they sit on.
Surfaces checked: `bg.base`, `surface.sunken`, `surface.raised`, `surface.overlay`. Targets:

| Theme | Text | UI glyphs, borders, focus, swatches |
|---|---|---|
| dark, light | ≥ 4.5:1 | ≥ 3:1 |
| hc-dark | ≥ 7:1 | ≥ 4.5:1 |

Exempt (reported, not required): `fg.disabled` (WCAG disabled exemption; 2.5–3.6 dark, 1.8–2.1 light, 4.4–5.2 HC),
the ANSI slot that equals the background (`ansi.0` dark/HC; `ansi.7`, `ansi.15` light), decorative `loader.track`.

**Result: 715 pairs checked, 0 failures.**

### 8.1 Worst case on any surface
| Foreground | Kind | Dark (worst surface) | Light (worst surface) | HC dark (worst surface) |
|---|---|---|---|---|
| `fg.default` | text | 12.13 (surface.overlay) | 17.15 (surface.sunken) | 15.44 (surface.overlay) |
| `fg.muted` | text | 6.73 (surface.overlay) | 6.90 (surface.sunken) | 12.49 (surface.overlay) |
| `fg.subtle` | text | 4.84 (surface.overlay) | 4.90 (surface.sunken) | 10.98 (surface.overlay) |
| `accent.fg` | text | 5.77 (surface.overlay) | 5.42 (surface.sunken) | 9.41 (surface.overlay) |
| `status.success.fg` | text | 6.54 (surface.overlay) | 4.71 (surface.sunken) | 10.31 (surface.overlay) |
| `status.warning.fg` | text | 6.02 (surface.overlay) | 5.09 (surface.sunken) | 9.67 (surface.overlay) |
| `status.danger.fg` | text | 5.73 (surface.overlay) | 5.49 (surface.sunken) | 9.36 (surface.overlay) |
| `status.info.fg` | text | 6.14 (surface.overlay) | 5.04 (surface.sunken) | 9.81 (surface.overlay) |
| `status.waiting.fg` | text | 6.02 (surface.overlay) | 5.09 (surface.sunken) | 9.67 (surface.overlay) |
| `risk.low.fg` | text | 6.14 (surface.overlay) | 5.04 (surface.sunken) | 9.81 (surface.overlay) |
| `risk.medium.fg` | text | 5.77 (surface.overlay) | 5.42 (surface.sunken) | 9.41 (surface.overlay) |
| `risk.high.fg` | text | 6.02 (surface.overlay) | 5.09 (surface.sunken) | 9.67 (surface.overlay) |
| `risk.critical.fg` | text | 5.73 (surface.overlay) | 5.49 (surface.sunken) | 9.36 (surface.overlay) |
| `diff.add.fg` | text | 6.54 (surface.overlay) | 4.71 (surface.sunken) | 10.31 (surface.overlay) |
| `diff.del.fg` | text | 5.73 (surface.overlay) | 5.49 (surface.sunken) | 9.36 (surface.overlay) |
| `merge.current` | text | 6.14 (surface.overlay) | 5.04 (surface.sunken) | 9.81 (surface.overlay) |
| `merge.incoming` | text | 6.42 (surface.overlay) | 4.78 (surface.sunken) | 10.20 (surface.overlay) |
| `merge.suggested` | text | 6.03 (surface.overlay) | 5.14 (surface.sunken) | 9.67 (surface.overlay) |
| `merge.resolved` | text | 6.31 (surface.overlay) | 4.84 (surface.sunken) | 10.04 (surface.overlay) |
| `git.dirty` | text | 6.08 (surface.overlay) | 5.07 (surface.sunken) | 9.77 (surface.overlay) |
| `git.worktree` | text | 5.98 (surface.overlay) | 5.21 (surface.sunken) | 9.57 (surface.overlay) |
| `git.detached` | text | 6.08 (surface.overlay) | 5.07 (surface.sunken) | 9.77 (surface.overlay) |
| `text-status.base` | text | 4.60 (surface.overlay) | 5.62 (surface.sunken) | 15.44 (surface.overlay) |
| `term.prompt` | text | 5.77 (surface.overlay) | 5.42 (surface.sunken) | 9.41 (surface.overlay) |
| `checkpoint.current` | text | 5.77 (surface.overlay) | 5.42 (surface.sunken) | 9.41 (surface.overlay) |
| `border.strong` | ui | 3.44 (surface.overlay) | 3.52 (surface.sunken) | 8.57 (surface.overlay) |
| `accent.solid` | ui | 4.46 (surface.overlay) | 4.15 (surface.sunken) | 7.35 (surface.overlay) |
| `accent.border` | ui | 4.46 (surface.overlay) | 4.15 (surface.sunken) | 9.41 (surface.overlay) |
| `focus` | ui | 4.46 (surface.overlay) | 4.15 (surface.sunken) | 9.41 (surface.overlay) |
| `icon.fg` | ui | 6.73 (surface.overlay) | 6.90 (surface.sunken) | 12.49 (surface.overlay) |
| `status.success.border` | ui | 4.99 (surface.overlay) | 3.55 (surface.sunken) | 10.31 (surface.overlay) |
| `status.warning.border` | ui | 4.55 (surface.overlay) | 3.89 (surface.sunken) | 9.67 (surface.overlay) |
| `status.danger.border` | ui | 4.22 (surface.overlay) | 4.20 (surface.sunken) | 9.36 (surface.overlay) |
| `status.info.border` | ui | 4.61 (surface.overlay) | 3.84 (surface.sunken) | 9.81 (surface.overlay) |
| `risk.low.border` | ui | 4.61 (surface.overlay) | 3.84 (surface.sunken) | 9.81 (surface.overlay) |
| `risk.medium.border` | ui | 4.46 (surface.overlay) | 4.15 (surface.sunken) | 9.41 (surface.overlay) |
| `risk.high.border` | ui | 4.55 (surface.overlay) | 3.89 (surface.sunken) | 9.67 (surface.overlay) |
| `risk.critical.border` | ui | 4.22 (surface.overlay) | 4.20 (surface.sunken) | 9.36 (surface.overlay) |
| `diff.add.stripe` | ui | 4.99 (surface.overlay) | 3.55 (surface.sunken) | 10.31 (surface.overlay) |
| `diff.del.stripe` | ui | 4.22 (surface.overlay) | 4.20 (surface.sunken) | 9.36 (surface.overlay) |
| `agent.red` | ui | 5.73 (surface.overlay) | 4.20 (surface.sunken) | 9.36 (surface.overlay) |
| `agent.orange` | ui | 5.77 (surface.overlay) | 4.15 (surface.sunken) | 9.41 (surface.overlay) |
| `agent.yellow` | ui | 6.02 (surface.overlay) | 3.89 (surface.sunken) | 9.67 (surface.overlay) |
| `agent.green` | ui | 6.54 (surface.overlay) | 3.55 (surface.sunken) | 10.31 (surface.overlay) |
| `agent.blue` | ui | 6.14 (surface.overlay) | 3.84 (surface.sunken) | 9.81 (surface.overlay) |
| `agent.violet` | ui | 5.88 (surface.overlay) | 4.07 (surface.sunken) | 9.52 (surface.overlay) |
| `agent.pink` | ui | 5.71 (surface.overlay) | 4.17 (surface.sunken) | 9.23 (surface.overlay) |
| `agent.cyan` | ui | 6.42 (surface.overlay) | 3.63 (surface.sunken) | 10.20 (surface.overlay) |
| `loader.spectral.0` | ui | 4.46 (surface.overlay) | 4.15 (surface.sunken) | 7.35 (surface.overlay) |
| `loader.spectral.1` | ui | 6.02 (surface.overlay) | 3.89 (surface.sunken) | 9.67 (surface.overlay) |
| `loader.spectral.2` | ui | 5.71 (surface.overlay) | 4.17 (surface.sunken) | 9.23 (surface.overlay) |
| `loader.spectral.3` | ui | 5.88 (surface.overlay) | 4.07 (surface.sunken) | 9.52 (surface.overlay) |
| `loader.spectral.4` | ui | 6.14 (surface.overlay) | 3.84 (surface.sunken) | 9.81 (surface.overlay) |
| `loader.spectral.5` | ui | 6.42 (surface.overlay) | 3.63 (surface.sunken) | 10.20 (surface.overlay) |

### 8.2 Pairs on tints, fills and component backgrounds
| Foreground | On | Kind | Dark | Light | HC dark |
|---|---|---|---|---|---|
| `fg.on-accent` | `accent.solid` | text | 6.31 | 4.76 | 8.72 |
| `fg.on-accent` | `accent.solid-hover` | text | 8.17 | 6.22 | 11.16 |
| `fg.on-accent` | `accent.solid-pressed` | text | 10.46 | 8.68 | 14.31 |
| `status.success.fg` | `status.success.bg/bg.base` | text | 7.80 | 4.71 | 9.67 |
| `status.warning.fg` | `status.warning.bg/bg.base` | text | 7.28 | 5.07 | 9.23 |
| `status.danger.fg` | `status.danger.bg/bg.base` | text | 7.03 | 5.36 | 9.12 |
| `status.info.fg` | `status.info.bg/bg.base` | text | 7.39 | 5.01 | 9.31 |
| `status.waiting.fg` | `status.waiting.bg/bg.base` | text | 7.28 | 5.07 | 9.23 |
| `accent.fg` | `accent.bg/bg.base` | text | 7.07 | 5.30 | 9.08 |
| `fg.default` | `diff.add.bg/bg.base` | text | 14.47 | 17.15 | 13.91 |
| `fg.default` | `diff.add.word/row` | text | 7.95 | 13.74 | 7.38 |
| `fg.muted` | `diff.gutter.add/bg.base` | text | 6.97 | 6.39 | 9.57 |
| `diff.add.fg` | `diff.add.bg/bg.base` | text | 7.80 | 4.71 | 9.29 |
| `fg.default` | `diff.del.bg/bg.base` | text | 14.88 | 16.72 | 14.55 |
| `fg.default` | `diff.del.word/row` | text | 8.75 | 12.69 | 8.25 |
| `fg.muted` | `diff.gutter.del/bg.base` | text | 7.30 | 6.15 | 10.19 |
| `diff.del.fg` | `diff.del.bg/bg.base` | text | 7.03 | 5.36 | 8.82 |
| `diff.hunk.header` | `diff.hunk.bg` | text | 9.80 | 6.90 | 14.82 |
| `fg.default` | `merge.marker/bg.base` | text | 14.50 | 15.61 | 15.88 |
| `fg.default` | `merge.current@18%/bg.base` | text | 12.84 | 14.61 | 13.80 |
| `fg.default` | `merge.incoming@18%/bg.base` | text | 12.82 | 14.49 | 13.71 |
| `fg.default` | `surface.selected/bg.base` | text | 14.38 | 15.90 | 12.92 |
| `fg.default` | `selection/bg.base` | text | 11.26 | 14.95 | 10.05 |
| `fg.default` | `highlight@24%/bg.base` | text | 12.59 | 13.32 | 12.92 |
| `fg.default` | `code.inline.bg` | text | 14.16 | 15.51 | 15.44 |
| `fg.default` | `bubble.user` | text | 14.16 | 15.51 | 15.44 |
| `fg.muted` | `bubble.user` | text | 7.86 | 6.24 | 12.49 |
| `monogram.N.fg` (min of 12) | `monogram.N.bg` | text | 8.41 | 6.88 | 12.22 |
| `ansi.N` (min of checked slots) | `bg.base` | text | 6.84 | 5.16 | 10.95 |

### 8.3 Fixes made to reach 0 failures
First pass had 30 failures. Changes:

| Failure (first pass) | Ratio | Fix | Now |
|---|---|---|---|
| dark `accent.fg` (brand `#FF5A1F`) on `surface.overlay` | 4.46 | `accent.fg` → `rust.400` `#FF845F`; brand rust stays `accent.solid` | 5.77 |
| dark `fg.on-accent` on `accent.solid-pressed` (`rust.600`) | 4.37 | press goes lighter: hover `rust.400`, pressed `rust.300` | 10.46 |
| light `accent.solid` (brand) on paper / sunken | 2.98 / 2.72 | light `accent.solid` → `rust.600` `#D04000` (L 0.575), `fg.on-accent` → white | 4.15 min |
| light `fg.on-accent` (ink) on hover / pressed rust | 4.37 / 3.17 | white text on `rust.700` / `rust.800` | 6.22 / 8.68 |
| light `fg.subtle` (`neutral.500`) | 3.52–4.04 | `fg.subtle` → `neutral.600`, `fg.muted` → `neutral.700` | 4.90 min |
| light `fg.muted` on `diff.gutter.del` / `bubble.user` | 4.36 / 4.44 | (same change) | 6.13 / 6.24 |
| light `border.strong` (`neutral.400`) | 2.51–2.87 | → `neutral.500` | 3.52 min |
| light `loader.spectral.*` (500 steps) | 2.48–2.98 | 600 steps; `spectral.0` aliases `accent.solid` | 3.63 min |
| HC `fg.on-accent` on pressed | 6.73 | pressed `rust.200` | 14.31 |
| HC `fg.default` on `diff.add.word` | 6.16 | word tint 45 % → 36 % | 7.38 |

## 9. Colour-blind safety
Colour is always the **second** channel. These pairs are the risky ones (rust/red/amber sit close in hue; green/red
collapse under deuteranopia), so each carries a non-colour cue:

| Pair | Non-colour cue | Spec |
|---|---|---|
| diff add / del | `+` / `−` sign column always shown; stripe position; counts `+12 −3` | diff-viewer.md, file-edit-chip.md |
| risk Low / Medium / High / Critical | glyph shape ○ ◇ ▲ ⬣, label text, border 1 / 1 / 2 / 2 px, Critical hatch | approvals.md §2.3 |
| review Critical / Warning / Suggestion / Nit / Info | ● / ● + label / ◐ / ○ / ⓘ | review-card.md |
| merge current / incoming | pane position and labels; hues kept off the add/del greens and reds | merge-conflict.md |
| accent (rust) vs danger (red) | danger always has ✕ / ⬣ glyph or a word; red is pulled to hue 12° (accent 38°) | — |
| warning vs waiting (both amber) | different motion (hold-and-breathe) and glyph (⏸ / hourglass) | loaders.md §3.2 |
| MCP states | glyph + word for all nine states | mcp-plugin-list.md |
| agent identity (8 hues) | always paired with the agent's name; identity never encodes state | subagent-tree.md |
| voice states | text label for every state; motion direction (inward / outward) | live-voice.md |
| text shimmer | brightness only, no hue, so nothing is lost under any CVD | text-shimmer.md |

## 10. Theming and overrides
| Rule | |
|---|---|
| Override at the theme tier | Apps set `color.*` roles (or point them at other primitives). Never edit primitives to restyle. |
| Keep the key set | A custom theme is a full copy of `dark` or `light` with values changed; a missing key fails the build. |
| Accent only | The smallest theme is one value: `accent.solid`. The build derives `accent.fg`, `.border`, `.bg`, `focus`, `selection`, `loader.accent` and re-checks contrast; if `fg.on-accent` fails on the new accent it flips between ink and white. (Derivation rule: not in #13; lands with the Studio theme editor, #26.) |
| Contrast gate | Custom themes run the same check; failures are build warnings for user themes, errors for shipped themes. |
| Density | `data-oxyn-density="compact"` / `Density::Compact` swaps only the `density.*` set. |
| Reduced motion, HC | HC is a theme; reduced motion is `motion.reduced` in the motion file. They combine freely. |
| Per-component | A component may expose a colour prop, but its default is always a token, and its spec lists the tokens it reads. |

### 10.1 What the Studio exposes
| Group | Tokens | Control |
|---|---|---|
| Theme | dark / light / hc-dark | segmented |
| Accent | `color.accent.solid` (derived set follows) | swatch + hex / OKLCH |
| Neutrals | `color.bg.base`, `surface.raised`, `fg.default` (others derived by step) | swatches |
| Status | `color.status.{success,warning,danger,info,waiting}.fg` | swatches, contrast badge |
| Spectral | `color.loader.spectral.0–5` | gradient stops |
| Identity | `color.agent.*`, `color.monogram.*` hue slots | swatches |
| Shape | `radius.sm / md / lg / bubble` | sliders |
| Type | `font.ui`, `font.mono`, `font.size.body`, `font.size.ui` | font picker, slider |
| Density | comfortable / compact | segmented |

Not exposed: primitives, `z.*`, `opacity.*` (except via component props), diff alpha values, ANSI, focus width.
Every swatch shows its live contrast ratio against the surfaces it lands on and turns red below the target.

## 11. Validation
The reference check (`tools/oxyn-tokens-gen`, run in CI with `--check`) does, in order:
1. JSON parses; every token has a known DTCG `$type` (own or inherited).
2. No name contains `.`, `{`, `}`; a path is a token or a group, never both.
3. Every `{alias}` (including inside composites and `$extensions.derivedFrom`) resolves; no cycles; alias type matches.
4. Primitives reference only primitives; semantic never references a theme; a theme never references another theme.
5. All themes have identical key sets.
6. Contrast table §8 passes.

Current file: 920 tokens, 585 aliases, 0 structural errors, 715 contrast rows, 0 failures. Motion file: 163 tokens, 93 aliases.

## 12. Open questions
1. **Light accent.** Brand `#FF5A1F` is 2.98:1 on paper and neither ink nor white text passes on the next darker
   step, so light uses `#D04000` with white text. Keep, or accept the brand fill with an outline?
2. **Risk Medium colour.** approvals.md says "accent-amber"; High is "warning" (amber). We used rust accent for
   Medium so the two differ. Alternative: a distinct orange-amber step.
3. **Accent vs danger.** Rust (38°) and red (12°) are close at the text steps (ΔE_ok ≈ 0.07). Glyph rules cover it;
   should danger move further toward crimson?
4. **Fonts.** Bundle Inter and JetBrains Mono (both OFL) in `oxyn-tokens`, or rely on the system stack?
5. **Light high-contrast** theme (`hc-light`): needed for V1?
6. **Daltonized themes** (text-shimmer.md mentions colour-blind palettes): separate themes, or only the glyph rules?
7. **DTCG colour format.** We use the 2025.10 object form (`colorSpace`, `components`, `hex`, `alpha`). Settled in
   #13: our own generator reads it directly (no Style Dictionary); it uses `hex` + `alpha`.
8. **Low-chroma states** (`merge.suggested`, `merge.resolved`, `git.dirty/detached/worktree`) use new `-soft` scales
   at 45 % chroma. Tune in the Studio.
9. **Agent identity order.** `{red..cyan}` was read as red, orange, yellow, green, blue, violet, pink, cyan.
10. **Specs to update** to the names in §3 (text.fg, tone.*, risk.*, color.diff.stripe, text.status.*).
