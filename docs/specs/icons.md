**Status:** draft v0.1 (2026-11-12).

# Spec: agent icons (family L overview + catalogue)

Family L overview ([../component-taxonomy.md](../component-taxonomy.md), row L: "~60 agent icons in 11
groups, plugin/MCP icon variations, 20 animated mark styles × 10 states", target 200+). Same depth as
the worked example [composer-send-control.md](composer-send-control.md). This file owns the **static
icon system**: the 66-glyph custom agent set, the base map, the grid, stroke and naming rules, the
file formats and the licences. Its siblings own motion and logos:

| Spec | Owns | Free | Pro |
|---|---|---|---|
| **icons.md** (this) | `icon/*` drawing styles, 66 custom glyphs, base map | 4 styles | 12 styles |
| [icon-states.md](icon-states.md) | animated transitions between two glyphs (copy → check, play → stop, mic → waveform …) | 4 styles | 12 styles |
| [plugin-icons.md](plugin-icons.md) | plugin / MCP server icons: monogram fallback, user-supplied logos, status badges | 4 | 12 |
| [animated-marks.md](animated-marks.md) | 20 mark styles × 10 states, user logos, the Oxyn preset | 5 styles (50 combos) | 15 styles (150 combos) |

**Family L count (proposed):** 66 glyphs × 2 forms (132 static assets) + 16 icon styles + 16
transition styles × 20 transition pairs + 16 plugin-icon variants + 200 mark combos. The 200+ target
is met by the mark matrix alone; glyphs are content, not variants (see §6).

## 1. Purpose
Agent apps show the same dozen ideas on every screen: *is the agent thinking, working, waiting on me,
done or broken; who did this (agent, sub-agent, me); is it safe; what did it touch.* General icon sets
cover generic objects (file, folder, terminal, shield, key) but lack the composites agents need:
"agent waiting for you", "allow for this session", "restore code but not chat", "sandbox on, network
off", "MCP server needs auth", ten distinguishable trace-span types (research note 05 §7 lists these
gaps). Users meet these icons at **12–24 px** in tool rows, status strips, approval cards, sidebars and
trace trees, often in a dense IDE panel, so the job is **recognition at 16 px** and **meaning that
survives without colour**.

Rules:
1. **One geometry with Lucide.** Our grid, stroke and caps match Lucide's (24 px grid, 2 px stroke,
   round caps and joins; research note 05), so a custom glyph sits next to a mapped base glyph without
   visible drift.
2. **Composite = base + one modifier.** Every agent composite is a known base glyph with at most one
   modifier badge in a fixed corner, so users learn one modifier vocabulary (✓ ✕ clock key hand
   pause bolt …) once.
3. **State is never colour alone.** Any status reading (done, error, waiting, risk) changes shape or
   modifier; colour is a redundant fourth channel (the approvals rule; [approvals.md](approvals.md) §2.3).
4. **Hand-tuned optical sizes**, not one asset scaled: 16, 20 and 24 px masters.
5. **Static here, motion in [icon-states.md](icon-states.md).** An icon never animates on its own;
   loops (spinners, level bars) are loaders ([loaders.md](loaders.md)) or transitions.

## 2. State machine
An icon is mostly passive; its host (button, row, chip) owns hover and press. The icon reacts.
```
                   name resolves                     host hover / focus
  Unresolved ─────────────────────▶ Rest (outline) ──────────────────────▶ Hot (tint → fg-strong)
     │ unknown name                   │   ▲                                   │ press
     ▼                                │   │ release                           ▼
  Fallback (`circle-help`, dev warn)  │   └────────────────────────────── Pressed (scale 0.92)
                                      │ host sets active
                                      ▼
                                   Active (filled form) ──── host clears ───▶ Rest
                                      │
         badge set / changed          ▼
  Rest|Active ───────────────▶ Badged (modifier corner shown; badge enter 160 ms)
                                      │ state change requested with a transition pair
                                      ▼
                                   Transitioning ──▶ (owned by icon-states.md) ──▶ Rest of new glyph
  Overlays: Disabled (40 % opacity + host reason tooltip) · Reduced (no scale; 0–80 ms crossfades)
            · Terminal (TUI glyph from the table) · Mirrored (RTL-directional glyphs flipped)
```

## 3. States table
| State | Shows | Primary / secondary action (host) | Keyboard |
|---|---|---|---|
| `Unresolved` | nothing; box reserved at size | — | not focusable |
| `Rest` | outline form in `color.icon.fg` (= `currentColor`) | host's | host's (icon never focusable alone) |
| `Hot` | outline in `color.icon.fg-strong` | host's | focus ring drawn by host, 2 px, 2 px offset |
| `Pressed` | scale 0.92 | host's | Space / Enter on host |
| `Active` | filled form (selected tab, toggled on, current mode) | host toggles off | host's `aria-pressed` / `aria-selected` |
| `Badged` | modifier in the badge corner (§5.4) | — | badge meaning added to host label |
| `Transitioning` | morph between two glyphs ([icon-states.md](icon-states.md)) | — | input on host stays live |
| `Disabled` | 40 % opacity, never below 3:1 for the glyph (weight kept) | host shows reason tooltip | host focusable with `aria-disabled` |
| `Fallback` | `circle-help` + dev warning "unknown icon `x`" (release builds: same glyph, no warning) | — | — |
| `Terminal` | single-cell glyph from §6.3 column, or `[x]` bracket label | — | — |
| Reduced motion | same as above; Hot/Pressed have no scale, colour only | — | — |

## 4. Micro-interactions
- **Hover:** colour step `fg → fg-strong` over 120 ms; no lift, no scale (icons inside quiet action
  bars stay calm; affectUI's message actions only fade the bar in, research note 11).
- **Press:** scale 0.92 for the press duration, back with `spring.snap`; matches the send control's
  0.94/80 ms press (slightly deeper because icons are smaller).
- **Active toggle:** outline → filled crossfade 120 ms; the fill grows from the glyph's centre via a
  0.85 → 1 scale on the fill layer only, so the outline never moves.
- **Badge enter:** modifier scales 0.6 → 1 with `spring.snap` and fades in over 160 ms; the knock-out
  gap (§5.4) opens at the same time so the base glyph is cut cleanly, never overdrawn.
- **Badge change** (e.g. clock → check): handled as a transition pair in [icon-states.md](icon-states.md).
- **Sidebar collapse:** icons stay fixed while labels clip and fade (icon-anchored collapse [R016];
  snap to icon-only below ≈ 160 px with a ≈ 0.18 s label fade [R046]).
- **Number on icon** (count badges such as `agent-team` "3"): digits roll per family C odometer,
  never resize the badge more than one step (1 → 2 digits widens by 4 px at 16 px).

## 5. Timing, grid and drawing rules

### 5.1 Timing table
| Transition | Duration | Delay | Easing / spring | Frame rate |
|---|---|---|---|---|
| Hover colour | 120 ms (`dur.fast`) | 0 | `ease.out` | 60 Hz |
| Press down / release | 80 ms / spring | 0 | `ease.out` / `spring.snap` | 60 Hz |
| Active outline → filled | 120 ms crossfade + 160 ms fill scale | 0 | `ease.out` | 60 Hz |
| Badge enter / exit | 160 ms / 120 ms | 0 | `spring.snap` / `ease.in` | 60 Hz |
| Disabled fade | 120 ms | 0 | linear | 60 Hz |
| Fallback swap (name resolves late) | 120 ms crossfade | 0 | `ease.out` | 60 Hz |
| **Reduced motion** | all colour changes kept; scale removed; every swap is a 0–80 ms (`dur.instant`) opacity crossfade; badges appear without scale | 0 | linear | repaint on change only |

### 5.2 Grid and optical sizes (proposed values, Lucide-compatible at 24)
| Master | viewBox | Stroke | Live area (padding) | Min gap between strokes | Min dot | Used at |
|---|---|---|---|---|---|---|
| 16 | `0 0 16 16` | 1.5 px | 14 × 14 (1 px) | 1.5 px | 2 px | 12 px (scaled 0.75 → 1.125 px stroke), 16 px |
| 20 | `0 0 20 20` | 1.75 px | 17 × 17 (1.5 px) | 1.75 px | 2.5 px | 20 px |
| 24 | `0 0 24 24` | 2 px | 20 × 20 (2 px) | 2 px | 3 px | 24 px; 32 and 48 px scaled with stroke clamped to 2.5 / 3 px (loaders.md §2.4 minimums) |

- **Keyline shapes (24 master):** circle Ø 20, square 18 × 18 r 2, portrait rect 16 × 20, landscape
  rect 20 × 16. 16/20 masters scale these and snap to the half-pixel grid.
- **Pixel snapping:** horizontal and vertical stroke centres sit on .5 coordinates for odd stroke
  widths in device pixels (1.5 px at 1× is snapped per scale factor at build; see §7).
- **Caps / joins:** round / round. Corner radius on rectangles 2 at 24, 1.5 at 20, 1 at 16.
- **No hairlines:** nothing renders under 1 px (loaders.md §2.4 rule); the 12 px use is the 16 master
  at 0.75, which keeps 1.125 px.
- **Detail budget:** ≤ 3 interior features at 16 px (e.g. bot head = outline + 2 eyes; antenna drops
  at 16). Each glyph lists what drops at 16 in its source file's metadata.

### 5.3 Forms
| Form | Rule |
|---|---|
| `outline` | default; stroke only |
| `filled` | for `Active`; solid shape with interior details knocked out at stroke width (eyes become holes); never used to mean "error" or "important" |

### 5.4 Modifier badge (composites)
- **Corner:** bottom-trailing (bottom-right in LTR, bottom-left in RTL), a 10 × 10 box at 24 (8 × 8 at
  20, 7 × 7 at 16) aligned to the live-area corner.
- **Knock-out:** the base glyph is clipped by the badge box grown by 1.5 px (1.25 at 16), so the
  modifier never touches base strokes. The clip is in the asset, not drawn as a background fill, so
  icons work on any surface.
- **Modifier vocabulary (12):** check `✓`, cross `✕`, clock, key, hand, pause, bolt, plus, slash,
  dots `···`, warning `!`, arrow-in. At 16 px only check, cross, dots, `!` and slash are legible; the
  others fall back to a 4 px solid dot plus the host's text (proposed).
- **One modifier max.** A second status goes to the host (row text, chip), not a second badge.

### 5.5 Naming
- `kebab-case`, `<base>[-<modifier-or-qualifier>]`: `agent-waiting`, `plug-needs-auth`,
  `restore-code`. Names are stable once released ([../variants.md](../variants.md) naming rule).
- **Groups are prefixes where a family exists:** `agent-*`, `subagent-*`, `restore-*`, `allow-*`,
  `plug-*`, `trace-*`, `display-*`.
- **Aliases:** the name map accepts Lucide names and agent-protocol names, so an agent-emitted name
  resolves: every name in A2UI's basic-catalogue Icon enum (≈ 60 names such as `lock`, `lockOpen`,
  `warning`, `error`, `help`, `refresh`, `rewind`, `stop`; research note 05 §1) maps to an Oxyn or
  Lucide glyph; camelCase aliases are folded to kebab-case.

### 5.6 Source and runtime formats
| Layer | Format |
|---|---|
| Source of truth | `icons/agent/<name>/<16|20|24>.svg`, one file per master, stroke-based, `fill="none" stroke="currentColor"`, no transforms, no `<style>`, no ids except `badge-clip`; plus `meta.toml` (group, aliases, tui glyph, drops-at-16, keywords, licence) |
| Build (`xtask build-registry`) | validates grid, stroke width, caps, live area, no hairlines; outlines strokes into fill paths per master (so GPUI and web render identical geometry); emits `oxyn-icons` Rust data + per-size web SVG sprites |
| GPUI (primary) | `IconPath { size, d: &'static str, fill_rule }` drawn as a monochrome filled path tinted with the text colour; exact GPUI element (SVG asset vs path API) to confirm at implementation, as loaders.md §2.2 notes for T2 |
| Dioxus web | inline `<svg viewBox>` with the outlined `d` (default) or the stroke source when the app sets `IconRender::Stroke` (allows `vector-effect: non-scaling-stroke`, as Lucide offers; research note 05) |
| Terminal | TUI glyph from `meta.toml`, width-checked with `unicode-width` like glyph loaders ([loader-glyph.md](loader-glyph.md) §7) |

```rust
// Generated by xtask; one entry per (name, size, form).
pub struct IconPath { pub size: u8, pub form: IconForm, pub d: &'static str, pub fill_rule: FillRule }
pub struct IconMeta { pub name: &'static str, pub group: IconGroup, pub aliases: &'static [&'static str],
                      pub tui: &'static str, pub tui_width_safe: bool, pub licence: IconLicence }
```

## 6. Catalogue and variants

### 6.1 Base map (mapped, not drawn by us)
Generic objects come from **Lucide** (ISC) under Oxyn names, re-tuned by us at 16 and 20 px (ISC
permits modification; upstream notice kept). Tool kinds in [tools.md](tools.md) §2.4 use these:
`tool-read` (file text), `tool-list` (folder), `tool-search` (magnifier), `tool-edit` (pencil),
`tool-shell` (terminal prompt `›_`), `tool-web` (globe), `tool-plan` (list-checks), `tool-subagent`
(git-branch), `tool-media` (sparkles), `tool-generic` (cog), plus UI basics used by transitions:
`copy`, `check`, `x`, `play`, `stop`, `pause`, `mic`, `audio-lines`, `send` (arrow-up), `plus`,
`chevron-down`, `refresh`, `rotate-ccw`, `lock`, `lock-open`, `key`, `shield`, `shield-check`, `hand`,
`eye`, `eye-off`, `bell`, `thumbs-up`, `thumbs-down`, `volume`, `circle-help`, `triangle-alert`,
`octagon-alert`, `history`. Exact Lucide source names are resolved at port time (gap: not verified
glyph-by-glyph in this draft).

### 6.2 Custom agent set: 66 glyphs in 11 groups
Groups follow research note 05 §7 "agent-specific icons that are missing or weak". All drawings are
proposals; TUI glyphs are single-cell and pass the loader-glyph width rules unless marked `w?`
(East-Asian-ambiguous; falls back to the bracket label).

**G1 · Agent states (7)** — base: rounded bot head (square 16 × 14 r 4 at 24, two eye dots, antenna
that drops at 16)
| Name | Drawing | TUI |
|---|---|---|
| `agent` | bot head | `◉` w? |
| `agent-thinking` | bot + dots `···` | `◌` |
| `agent-working` | bot + 270° arc around the badge corner | `◐` w? |
| `agent-waiting` | bot + raised hand (needs you) | `[!]` (emoji are excluded from TUI tables) |
| `agent-done` | bot + check | `✓` |
| `agent-error` | bot + cross | `✕` |
| `agent-paused` | bot + pause bars | `‖` |

**G2 · Delegation (4)**
| Name | Drawing | TUI |
|---|---|---|
| `subagent-spawn` | large bot, line forking to a small bot (parent → child) | `⑂` w? → `[+a]` |
| `subagent-return` | arrow entering a bot from the trailing side | `↩` |
| `agent-team` | three bot heads, front one full, two behind clipped (swarm) | `⁂` |
| `agent-handoff` | bot → bot with a chevron between | `⇄` |

**G3 · Checkpoints & rewind (6)**
| Name | Drawing | TUI |
|---|---|---|
| `checkpoint` | flag on a horizontal timeline tick | `⚑` w? → `[cp]` |
| `restore-code` | file + counter-clockwise clock arrow | `↺` |
| `restore-chat` | speech bubble + counter-clockwise clock arrow | `↺` + label |
| `restore-both` | file and bubble overlapped + clock arrow | `↺` + label |
| `summarize-from-here` | three lines squeezing into one (arrows-in) | `≡` |
| `branch-from-checkpoint` | flag with a branch line leaving it | `⑂` → `[br]` |

**G4 · Approvals (7)** — risk tiers keep their own shapes ([approvals.md](approvals.md) §2.3: circle,
diamond, triangle `!`, octagon `!`); these icons are decisions and scopes, not risk
| Name | Drawing | TUI |
|---|---|---|
| `approval-needed` | raised open hand | `[?]` |
| `allow-once` | check + small "1" | `✓1` (2 cells) |
| `allow-session` | check + clock | `✓◷` w? → `✓s` |
| `allow-always` | check + infinity | `✓∞` |
| `deny-with-reason` | cross + speech bubble | `✕…` |
| `auto-approved` | shield + bolt (allowed by rule) | `[auto]` |
| `classifier-review` | shield + eye | `[rev]` |

**G5 · Sandbox & permissions (5)**
| Name | Drawing | TUI |
|---|---|---|
| `sandbox-on` | box (container) + shield modifier | `▣` w? → `[sbx]` |
| `network-off` | globe with slash inside a box | `[net-]` |
| `workspace-write` | folder + pencil, dashed boundary around | `[rw]` |
| `full-access` | open shield + `!` (danger styling from host) | `[!!]` |
| `protected-path` | folder + lock | `[lck]` |

**G6 · Diff review (6)**
| Name | Drawing | TUI |
|---|---|---|
| `accept-hunk` | bracketed block + check | `+✓` |
| `reject-hunk` | bracketed block + cross | `-✕` |
| `accept-all` | double check (two stacked ticks) | `✓✓` |
| `hunk-conflict` | two arrows colliding into a block + `!` | `<>` |
| `edit-by-agent` | pencil + bot eyes on the pencil end (agent authored) | `a~` |
| `edit-streaming` | pencil + two motion lines behind it | `~>` |

**G7 · Context & tokens (5)**
| Name | Drawing | TUI |
|---|---|---|
| `context-ring` | ring with fill sweep; levels `-0 -25 -50 -75 -90 -100` (one glyph, 6 level assets; 90 adds a `!` notch) | `○ ◔ ◑ ◕ ●` w? |
| `context-compacting` | stack of three bars with arrows pressing in | `⇥⇤` |
| `memory-file` | file with a small brain-curve / bookmark | `[mem]` |
| `skill` | puzzle piece with spark (skill / plugin chip) | `[sk]` |
| `token` | coin with a `T`-cut, hybrid token/coin | `¤` |

**G8 · MCP & tools (6)** — status overlays for server icons live in [plugin-icons.md](plugin-icons.md)
| Name | Drawing | TUI |
|---|---|---|
| `plug-connected` | two-prong plug inserted in a socket | `⏚`→`[on]` |
| `plug-pending` | plug + clock | `[..]` |
| `plug-needs-auth` | plug + key | `[key]` |
| `plug-failed` | plug + cross | `[x]` |
| `plug-tools-warning` | plug + `!` (connected, tools fetch failed) | `[!]` |
| `tool-call` | wrench inside `{ }` brackets | `{}` |

**G9 · Trace observation types (10)** — ten names from the Langfuse list (research note 05 §5, names
only); every glyph has a distinct silhouette so a 16 px trace tree stays readable in greyscale
| Name | Drawing | TUI |
|---|---|---|
| `trace-generation` | sparkle in a rounded square | `✦` |
| `trace-agent` | bot head (= `agent`, square frame) | `◉` |
| `trace-tool` | wrench | `⚒`→`[t]` |
| `trace-chain` | two linked rings | `∞`→`[ch]` |
| `trace-retriever` | magnifier over stacked pages | `[r]` |
| `trace-evaluator` | gauge / scale needle | `[ev]` |
| `trace-embedding` | 3 × 3 dot grid with one dot filled | `⠿` |
| `trace-guardrail` | shield outline, vertical rail inside | `[g]` |
| `trace-span` | horizontal bar with end ticks `├─┤` | `├┤` |
| `trace-event` | single dot with a ring (point in time) | `◎` w? → `•` |

**G10 · Orchestration (4)**
| Name | Drawing | TUI |
|---|---|---|
| `worktree` | branch line ending in a folder | `[wt]` |
| `local-cloud-handoff` | laptop ↔ cloud with arrows | `[⇅]` |
| `agent-scheduled` | bot + repeat-clock (looping / cron agent) | `[↻]` |
| `needs-you-inbox` | tray with hand modifier | `[in]` |

**G11 · Generative UI (6)**
| Name | Drawing | TUI |
|---|---|---|
| `genui-component` | square with a sparkle in its corner (generated component) | `[ui]` |
| `genui-pinned` | square + pin (interactable / pinned) | `[pin]` |
| `app-frame` | window frame + plug (embedded app, MCP Apps) | `[app]` |
| `display-inline` | frame sitting in text lines | `[in]` |
| `display-pip` | large frame + small frame bottom-trailing | `[pip]` |
| `display-fullscreen` | four corner brackets | `[fs]` |

Totals: 7 + 4 + 6 + 7 + 5 + 6 + 5 + 6 + 10 + 4 + 6 = **66 glyphs**, each with 3 masters × 2 forms.

### 6.3 Variant catalogue (drawing styles)
A style re-draws every glyph (custom set + base map) with a different structure; colour, radius and
density stay tokens ([../variants.md](../variants.md)). Stroke weight is a style, not a token, because
each weight needs re-tuned 16/20 masters.

**Free (4)**
| Name | Idea | Surfaces |
|---|---|---|
| `icon/outline` | Default: 2 px (24) / 1.75 (20) / 1.5 (16) round strokes, Lucide-compatible | all GUI |
| `icon/filled` | Solid form with knocked-out details; also the `Active` form of `outline` | all GUI |
| `icon/glyph` | Single-cell Unicode or bracket label per §6.2 for terminals and TUI looks | terminal, ide |
| `icon/dot` | 6 px status dot + text, for 12 px rows where a glyph won't read (state only, not objects) | ide, chat |

**Pro (12)**
| Name | One-line idea |
|---|---|
| `icon/thin` | 1.25 px masters for large, airy layouts (≥ 24 px only) |
| `icon/bold` | 2.5 px masters for small-screen and high-glare use |
| `icon/duotone` | Outline plus a 20 % alpha fill layer on the meaningful part (the hand, the lock) |
| `icon/sharp` | Square caps and mitre joins, 0 radius, for brutalist / terminal-adjacent apps |
| `icon/rounded-solid` | Soft filled shapes with 3 px radii, friendly consumer look |
| `icon/pixel` | 16 × 16 pixel-grid redraw, integer scaling only (pairs with `loader-mascot/pixel-bot`) |
| `icon/dot-matrix` | Every glyph rebuilt from a 7 × 7 dot grid (pairs with the dot-based Oxyn mark) |
| `icon/ascii` | Multi-cell ASCII art (3 × 2 cells) for TUI headers and empty states |
| `icon/isometric` | 30° isometric line drawings for onboarding and empty states (≥ 32 px) |
| `icon/glass` | Duotone with a frosted inner fill and a 1 px highlight, for liquid-glass shells |
| `icon/hand-drawn` | Slightly wobbly single stroke with a seeded jitter frozen at build (never animated) |
| `icon/badge-tile` | Glyph centred on a tinted rounded tile (app-launcher look), tile = tone token |

## 7. Edge cases
- **12 px rows:** only the 16 master at 0.75; composites drop to base + `icon/dot` status if the
  modifier can't be read (§5.4). The host keeps text so meaning survives.
- **HiDPI and fractional scale (1.25×, 1.5×):** the build emits snapped variants for 1×, 1.5× and 2×;
  at other scales, paths are drawn unsnapped (blur accepted) rather than fattened.
- **RTL:** directional glyphs (`subagent-return`, `agent-handoff`, `restore-*` clock arrows,
  `edit-streaming`, `display-pip`) flip horizontally; clocks, checks and text-like glyphs (`allow-once`
  "1") never flip. `meta.toml` carries `rtl_mirror: bool`. Badge corner moves to bottom-left.
- **Light / dark:** a single `currentColor`; `filled` icons on accent backgrounds take
  `color.icon.on-accent`. Badges on filled forms use the knock-out, not a white ring.
- **Dense IDE panels (28 px rows):** 16 px icons with 6 px gap to text; badges hidden below 16 px
  wide hit areas only if the row text carries the status.
- **Touch:** icons inside 44 px hit areas; icon size unchanged (no growing to fill).
- **Unknown or misspelled name from an agent:** `Fallback` (`circle-help`), never an empty box; a
  dev warning lists the nearest names (edit distance ≤ 2).
- **Theme fonts in terminals lacking a glyph:** TUI column falls back to the bracket label.
- **Very long labels beside icons:** the icon never shrinks; labels truncate.
- **Slow networks:** icons are compiled-in data; nothing loads at runtime (user logos are
  [plugin-icons.md](plugin-icons.md)'s job).
- **Fast state churn** (badge flips several times a second): badge changes are coalesced to one per
  160 ms (proposed); the last state wins.

## 8. Accessibility
- Decorative when next to a visible label: `aria-hidden="true"` (Dioxus), not exposed (GPUI).
- **Icon-only buttons** must have a label; the props make `label` required when `icon_only` is true
  (compile-time via a builder). Labels change with state ("Copy" → "Copied").
- **Badge meaning joins the label:** `agent-waiting` on a row reads "Agent, waiting for your approval";
  `plug-needs-auth` reads "Needs sign-in".
- **Contrast:** glyph ≥ 3:1 against its background in every state, including Disabled (opacity floor
  0.4 with weight kept; if 3:1 fails, Disabled uses the `fg-muted` token instead of alpha).
- **Not colour alone:** every status icon differs by modifier shape (§5.4); risk keeps the approvals
  shapes.
- **No live regions** from icons; hosts announce state changes.
- **Forced-colours / high-contrast:** `currentColor` maps to the system text colour; duotone fills drop.

## 9. Props sketch
```rust
pub struct Icon {
    pub name: IconName,              // generated enum; also From<&str> via the alias map (Lucide, A2UI)
    pub size: IconSize,              // Px12 | Px16 | Px20 | Px24 | Px32 | Px48
    pub style: IconStyle,            // Outline (default) | Filled | Glyph | Dot | Pro(IconStylePro)
    pub active: bool,                // Outline → Filled form
    pub badge: Option<IconBadge>,    // Check | Cross | Clock | Key | Hand | Pause | Bolt | Plus | Slash | Dots | Warn | ArrowIn
    pub tone: IconTone,              // Current (default) | Muted | Accent | Status(StatusTone) | Token(TokenId)
    pub label: Option<SharedString>, // required when the icon is the only content of a control
    pub agent: Option<AgentState>,   // optional: agent-* glyph + badge derived from AgentState
    pub surface: Surface,            // Terminal → Glyph style
    pub motion: MotionLevel,         // reduced: no scale on press/badge
}

impl Icon {
    /// `agent-*` name + badge from the shared state (Thinking → agent-thinking, AwaitingApproval → agent-waiting …).
    pub fn for_agent(state: &AgentState) -> Self { /* … */ }
}
```

## 10. Related components and references
**Related:** [icon-states.md](icon-states.md) · [plugin-icons.md](plugin-icons.md) ·
[animated-marks.md](animated-marks.md) · tool kind icons ([tools.md](tools.md) §2.4) · risk glyphs
([approvals.md](approvals.md) §2.3) · loaders size table and no-hairline rule
([loaders.md](loaders.md) §2.4) · glyph width rules ([loader-glyph.md](loader-glyph.md) §7) ·
[trace-view.md](trace-view.md) (G9) · [checkpoint-timeline.md](checkpoint-timeline.md) (G3) ·
[diff-viewer.md](diff-viewer.md) (G6) · [composer-context-meter.md](composer-context-meter.md) (G7) ·
[permission-rules.md](permission-rules.md) (G4, G5) · [subagent-tree.md](subagent-tree.md) (G2).

| Reference | What we take | Licence note |
|---|---|---|
| Lucide (research note 05 §7) | base glyphs, 24 px / 2 px / round grid, non-scaling stroke option | ISC — portable with notice kept; re-check per [../licensing.md](../licensing.md) |
| Phosphor (note 05) | duotone / fill weight idea for Pro `duotone` | MIT — portable with attribution (we draw our own) |
| Tabler (note 05) | gap-filler glyphs on the same 24 / 2 grid | MIT — portable with attribution |
| Iconoir (note 05) | 1.5 px outline reference for the 16 master | MIT — portable with attribution |
| Hugeicons (note 05) | style breadth (10 styles) as a benchmark | Free tier + paid Pro, terms not checked — study only |
| Central Icons (note 05) | stroke / radius / join axes as a style matrix idea | Commercial — study only |
| A2UI basic catalogue Icon enum (note 05 §1) | the alias names an agent can emit | Protocol names only; the full ≈ 60-name list still to be copied from the spec (gap) |
| Langfuse trace types (note 05 §5) | the ten G9 type names | Names only, no assets |
| Agent-specific gap list (note 05 §7) | the 11 groups | Our analysis; drawings are ours |
| Stagger [R109] | 24 state-machine icons as a product reference | Paid generator — study only |
| keviduk sidebar [R016], adamboyle rail [R046] | icon-anchored collapse, ≈ 160 px snap | Study only |
| supermemory pixel icons [R010] | pixel icon look for `icon/pixel` | Study only |
| affectUI Message Actions (note 11) | quiet icon bar, neutral strokes | Paid / unclear — study only |

Licences above are as recorded in research note 05 on 2026-11-11; [../licensing.md](../licensing.md)
does not yet list icon libraries and should gain rows for Lucide, Phosphor, Tabler and Iconoir before
the first port.
