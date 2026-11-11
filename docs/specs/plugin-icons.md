**Status:** draft v0.1 (2026-11-12).

# Spec: plugin and MCP server icons

Part of family L ([icons.md](icons.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). This spec owns the **small square that stands for
a plugin, MCP server, connector or skill** wherever it appears: tool rows ("server mark or plug" in
[tools.md](tools.md) §2.4), approval cards (server identity badge for `McpTool`,
[approvals.md](approvals.md)), the context meter's per-server rows
([composer-context-meter.md](composer-context-meter.md)), the tools menu, and the MCP / plugin list in
family J ([../component-taxonomy.md](../component-taxonomy.md)). Family J owns the list rows and their
actions; this spec owns the **icon, its fallback, and its status badge**. Target **4 free + 12 Pro =
16 variants**.

## 1. Purpose
Users connect many servers and need to tell them apart at 16–24 px and see at a glance which one is
broken, waiting for sign-in or waiting for approval. Products show real brand marks (Arc's MCP card
uses real favicons and pairs each status icon with a word, never colour alone; research note 12) and
coloured status dots (Cursor: green connected, yellow connecting / needs connect, red failed; note 05
§6, where third-party docs disagree on yellow). Claude Code lists nine-plus states (`✔ Connected`,
`! Connected · tools fetch failed`, `! Needs authentication`, `✘ Failed to connect`, `⏸ Pending
approval`, `✘ Rejected`, `⊘ Disabled`, `cached`; note 05 §6).

Oxyn cannot ship those brand marks (§6.1), so the component must look good **with no logo at all**:
a stable, legible **monogram** is the default, and the user's own image is an upgrade.

Rules:
1. **Oxyn ships no third-party logos.** Not in the registry, the gallery, the docs or the Studio.
   Demos use made-up companies (taxonomy §B "one made-up company per category").
2. **Monogram first, image when supplied.** The monogram renders on the first frame; a supplied
   image crossfades in when decoded; a broken image never shows a broken box.
3. **Same id → same tile, everywhere.** Colour and letters derive from the stable server id, so one
   server looks identical in the tool row, approval card and settings.
4. **Status by badge shape + host word.** Each status has a distinct badge shape; colour is redundant.
5. **The library never fetches.** Favicon or logo fetching is the app's decision; the component takes
   bytes.

## 2. State machine
Two independent machines: **source** (what the tile shows) and **status** (the badge).
```
 SOURCE
                      source = Image/Svg                       decoded + sanitised
  Init ─▶ Monogram ─────────────────────────▶ Loading(monogram shown) ─────────────────▶ Image
   │   (first frame)                             │ decode error / unsafe / timeout 5 s       │ source removed
   │                                             ▼                                           ▼
   │                                       Monogram(fallback, dev warning)              Monogram
   └─ name empty / only symbols ─▶ GenericGlyph (`plug`)

 STATUS (badge)
             connect                 handshake ok                tools listed
  Unknown ─────────▶ Connecting ──────────────▶ (tools…) ──────────────▶ Connected (badge hidden by default)
                       │  │  └─ needs OAuth ─▶ NeedsAuth ─ signed in ─▶ Connecting
                       │  └─ policy asks ───▶ PendingApproval ─ allowed ─▶ Connecting
                       │                                       └ denied ─▶ Rejected
                       └─ error ─▶ Failed ─ retry ─▶ Connecting
  Connected ─ tools fetch fails ─▶ ToolsWarning ─ refetch ok ─▶ Connected
  Connected ─ offline, using cached tool list ─▶ Cached ─ back online ─▶ Connected
  any ─ user disables ─▶ Disabled ─ enable ─▶ Connecting
  plugin only: UpdateAvailable (overlay on Connected) · ConfigNeeded (before Connecting)
  Overlays: Reduced (no spin, no breathe) · Terminal (bracket monogram + status char) · Selected / Hover (host)
```

## 3. States table
| Status | Badge shape (24 px tile) | Tone | Tile treatment | Host word (family J) | Host actions | Keyboard |
|---|---|---|---|---|---|---|
| `Unknown` | none | — | normal | — | — | host's |
| `Connecting` | 3/4 arc, rotating | `fg-muted` | normal | "Connecting…" | Cancel | host's |
| `Connected` | none (opt. solid dot) | success | normal | "Connected" | Disable, Logs | host's |
| `NeedsAuth` | key | waiting (amber) | normal | "Needs sign-in" | Sign in | Enter → Sign in |
| `PendingApproval` | raised hand | waiting (amber) | normal | "Waiting for approval" | Review | Enter → Review |
| `ToolsWarning` | `!` in a small triangle | warn | normal | "Connected · tools failed to load" | Retry, Logs | — |
| `Cached` | counter-clockwise arrow (history) | `fg-muted` | normal | "Offline · cached tools" | Reconnect | — |
| `Failed` | ✕ | danger | normal | "Failed to connect" | Retry, Logs | Enter → Retry |
| `Rejected` | ⊘ (slashed circle) | danger | 60 % opacity | "Blocked by policy" | Details | — |
| `Disabled` | pause bars `‖` | `fg-muted` | greyscale, 40 % opacity (letters keep 3:1) | "Disabled" | Enable | — |
| `UpdateAvailable` (plugin) | ↑ arrow | accent | normal | "Update available" | Update | — |
| `ConfigNeeded` (plugin) | cog | waiting (amber) | normal | "Needs configuration" | Configure | — |
| Reduced motion | `Connecting` arc static at 3/4 | — | same | same | same | same |

Source states: `Monogram` (letters on a tinted tile), `Loading` (= Monogram, no spinner), `Image`
(user image inside the tile safe area), `GenericGlyph` (`plug` glyph from [icons.md](icons.md) G8).

## 4. Micro-interactions
- **Image arrives:** monogram → image crossfade 160 ms, no scale, so rows don't twitch. If decode
  takes > 5 s (proposed) the tile stays on the monogram for the session.
- **Badge enter / change:** 160 ms `spring.snap` scale 0.6 → 1; badge-to-badge changes use the
  icon-state engine ([icon-states.md](icon-states.md) pair #8 for pending → connected: the plug slides
  in, the clock exits).
- **Connected moment:** when the badge is hidden for `Connected`, the outgoing badge morphs into a ✓
  for 600 ms then fades (240 ms), so success is seen once and then gets out of the way.
- **Failure:** ✕ badge enters; the **host row** nudges 2 × 3 px (icons never shake).
- **Needs sign-in:** badge key is static; the host's "Sign in" button is the attention point. Pro
  `breathing-auth` adds a slow tile breathe.
- **Hover (host row):** tooltip "GitHub · Needs sign-in · 12 tools" (name, status word, tool count);
  the tile itself doesn't change.
- **Press:** handled by the host; tile scale 0.92 when the tile is the control (e.g. a launcher grid).
- **Drag (reordering in settings):** tile lifts with a 4 px shadow; badge stays attached.

## 5. Timing, sizes and drawing rules

### 5.1 Timing table
| Transition | Duration | Delay | Easing / spring | Frame rate |
|---|---|---|---|---|
| Monogram → image crossfade | 160 ms | 0 (after decode) | `ease.out` | 60 Hz |
| Badge enter / exit | 160 ms / 120 ms | 0 | `spring.snap` / `ease.in` | 60 Hz |
| Badge change (morph) | 160 ms | 0 | per [icon-states.md](icon-states.md) style | 60 Hz |
| Connected ✓ hold / fade | 600 ms / 240 ms | 0 | — / `ease.in` | 60 Hz |
| Connecting arc rotation | 1000 ms per turn (`loop.base`) | `show_delay` 300 ms | linear | ≤ 60 fps, 30 fps unfocused |
| Disabled greyscale / opacity | 200 ms | 0 | linear | 60 Hz |
| Status-change coalescing | last state within 160 ms wins | — | — | — |
| **Reduced motion** | crossfades 0–80 ms; arc static at 3/4; no ✓ morph (the ✓ appears and fades in 0–80 ms); no breathe; greyscale instant | 0 | linear | repaint on change only |

### 5.2 Sizes (proposed)
| Tile | Corner radius | Monogram | Max letters | Badge | Knock-out ring |
|---|---|---|---|---|---|
| 12 px | 3 px | 7 px semibold | 1 | none: host shows a status dot beside the tile | — |
| 16 px | 4 px | 9 px | 1 | 6 px solid dot in status tone (shape lost; host word required) | 1 px surface colour |
| 20 px | 4.5 px | 10 px | 2 | 8 px, shape-only glyph (✕ ! key hand) | 1.5 px |
| 24 px | 5 px | 12 px (1) / 10 px (2) | 2 | 10 px | 1.5 px |
| 32 px | 7 px | 15 / 13 px | 2 | 12 px | 2 px |
| 48 px | 10 px | 22 / 19 px | 2 | 16 px | 2 px |

Badge sits bottom-trailing, its centre inset 1 px inside the tile corner, ring drawn in the host
surface colour (unlike [icons.md](icons.md) composites, a user image underneath is arbitrary, so the
ring must be painted, not clipped).

### 5.3 Monogram rules (proposed)
1. **Input:** the display name; if empty, the server id.
2. **Strip** (case-insensitive) a leading `@scope/`, a leading `mcp-` / `mcp_` / `server-`, and a
   trailing `-mcp` / `-server` / `_mcp`, so `mcp-server-github` → `github`.
3. **Split** on spaces, `-`, `_`, `.`, `/` and camelCase boundaries; drop words that are only digits
   unless nothing else remains.
4. **Letters:** first grapheme cluster of the first word, plus the first of the second word when
   size ≥ 20 px and a second word exists (`google-drive` → `GD`; `github` → `G`). Uppercase only where
   the script has case. CJK, Arabic, Devanagari: one grapheme, never two; Arabic uses the isolated form.
5. **Skip emoji and symbols;** if nothing remains → `GenericGlyph` (`plug`).
6. **Colour:** hue slot = FNV-1a 32-bit hash of the **stable id** mod 12; tile background and letter
   colour are fixed OKLCH lightness/chroma pairs per theme chosen so letters are ≥ 4.5:1 on the tile.
   The 12 hues are tokens (`color.monogram[0..12]`), tuned in the Studio.
7. **Collision nudge:** a list host may pass `avoid: &[HueSlot]`; the tile moves to the next free
   slot (stable for that list order).

### 5.4 User-supplied images
| Source | Handling |
|---|---|
| `Svg(bytes)` | Sanitised at load: allowed elements `svg g path rect circle ellipse line polyline polygon defs clipPath linearGradient radialGradient stop`; removed: `script`, `foreignObject`, `image`, external or `data:` `href`, `style`/CSS, animation elements, event attributes. Caps: 64 KB, 2,000 path commands (proposed). Fails → Monogram |
| `Svg` with `template: true` | single-colour mark tinted with `currentColor` (status tones and dark mode work) |
| `Raster(bytes)` | decoded off the UI thread, downsampled to 2× the largest requested size, max 512 × 512 input (proposed); first frame only for animated formats; supported formats follow the GPUI / browser image decoders (to confirm) |
| `Glyph(IconName)` | any Oxyn or base-map icon on the tinted tile |
| Safe area | image inset 12.5 % of the tile on each side; transparent images sit on the neutral tile |
| Low-contrast logos | if the image's mean luminance contrast against the tile is < 1.5:1, the tile flips to the opposite neutral (light ↔ dark) and gets a 1 px inner outline (proposed) |

## 6. Variants and brand policy

### 6.1 Brand-logo policy
- **Never shipped:** third-party logos, favicons or wordmarks in any Oxyn package, registry item,
  story, screenshot, poster or docs page. This includes the AI-vendor icons in itshover: its code is
  MIT (research note 05 §7) but the marks are their owners' trademarks, and an MIT notice does not
  grant trademark rights.
- **User-supplied only:** apps pass their own image bytes (`PluginIconSource::Svg | Raster`) and are
  responsible for having the right to show them. Docs say so next to the prop.
- **Gallery / demos:** monograms and made-up brands only.
- **Studio:** a user can upload a logo to preview it; uploads stay in the user's project and are never
  added to shared presets.

### 6.2 Variant catalogue
**Free (4)**
| Name | Idea | Surfaces |
|---|---|---|
| `plugin-icon/tile` | Default: rounded tile with monogram or user image + status badge | all GUI |
| `plugin-icon/monogram` | Forces the monogram even when an image exists (uniform, calm lists) | all GUI |
| `plugin-icon/glyph` | Terminal: bracket monogram `[gh]` + status char `✓ … ! ✕ ⊘ ‖` | terminal, ide |
| `plugin-icon/dot` | No tile: status dot + name, for 12 px dense rows | ide, chat |

**Pro (12)**
| Name | One-line idea |
|---|---|
| `plugin-icon/stack` | Overlapping tiles (−25 % overlap) with a "+N" tile, for "used 5 servers" summaries |
| `plugin-icon/ring-status` | Status as a 2 px ring around the tile (arc spins while connecting, fills on connect) instead of a badge |
| `plugin-icon/tool-count` | Corner pill with the server's tool count (`12`), rolls on change |
| `plugin-icon/transport-corner` | Tiny top-trailing tag for transport (`stdio`, `http`, `sse`) |
| `plugin-icon/scope-corner` | Top-leading notch for scope (project / user / managed), shape-coded |
| `plugin-icon/health-spark` | 3-bar latency mini-meter under the tile from the last 10 calls |
| `plugin-icon/breathing-auth` | `NeedsAuth` tile breathes amber (2.4 s) until signed in |
| `plugin-icon/connect-beam` | A beam travels the tile border while connecting (reuses `busy-glow` drawing) |
| `plugin-icon/duotone-monogram` | Two-tone monogram: first letter solid, second outline |
| `plugin-icon/pixel-monogram` | Letters drawn on a 5 × 7 pixel font, integer scaling (pairs with `icon/pixel`) |
| `plugin-icon/glass-tile` | Frosted tile with a 1 px highlight for liquid-glass shells |
| `plugin-icon/orbit-badge` | Badge orbits once around the tile corner when the status changes, then settles |

## 7. Edge cases
- **Same first letters** (`GitHub`, `GitLab`, `Gmail`): different hue slots from the id hash; at ≥ 20
  px two-letter monograms (`GH`, `GL`) disambiguate; the host always shows the name.
- **Very long or odd names** (`my_super_long_internal_tools_server_v2`): rules §5.3 give `MS`; names
  that are only digits (`42`) keep the digits.
- **Name changes:** colour is from the id, so renaming keeps the colour; letters update.
- **RTL:** badge moves bottom-left; monogram letters are not reordered (each grapheme is taken in
  logical order).
- **Light / dark:** monogram pairs are per theme; template SVGs follow `currentColor`; full-colour
  logos use the low-contrast rule (§5.4).
- **Hostile or huge SVG:** sanitiser limits (§5.4); parse runs off the UI thread with a 50 ms budget
  (proposed), then Monogram.
- **Slow networks:** the app's fetch is outside the component; Monogram shows instantly, the image
  crossfades in later (or never).
- **Offline:** `Cached` status; images already decoded stay; nothing re-fetches.
- **Many servers** (40+ in a settings list; Cursor soft-caps around 40 enabled tools per note 05):
  tiles are cheap (one path + text); images are decoded once per (source, size) and shared.
- **Status flapping** (connect / fail every second): coalesced (§5.1); after 3 failures in 30 s the
  badge stays `Failed` until the next manual retry (proposed; family J decides retries).
- **Tiny sizes:** below 20 px the badge loses its shape, so the host **must** show the status word
  (props enforce `show_status_text` at ≤ 16 px).
- **Touch:** tile is not the hit target unless the host makes it one (≥ 44 px area).

## 8. Accessibility
- **Name:** the tile is decorative next to a visible name (`aria-hidden`); standalone (launcher grid)
  it gets "GitHub, needs sign-in".
- **Status in text:** every status has a host word (§3); badge shape is the second channel, colour the
  third. Daltonized themes keep shapes distinct.
- **Announcements:** status changes go to the app's polite live region via family J ("GitHub
  connected"); `Failed`, `NeedsAuth` and `PendingApproval` are assertive. The icon itself announces
  nothing.
- **Contrast:** monogram letters ≥ 4.5:1 on the tile; badge glyph ≥ 3:1 on its disc; the knock-out
  ring separates the badge from any image.
- **Motion:** only `Connecting` spins; `breathing-auth` breathes ≤ 0.42 Hz; no flashing.
- **Forced colours:** tile becomes an outlined square in the system text colour; letters remain.

## 9. Props sketch
```rust
pub struct PluginIcon {
    pub id: SharedString,                  // stable server / plugin id: drives hue and cache key
    pub name: SharedString,                // display name: drives letters and labels
    pub source: PluginIconSource,          // Monogram (default) | Svg { bytes, template: bool } | Raster(bytes) | Glyph(IconName)
    pub status: PluginStatus,              // Unknown | Connecting | Connected | NeedsAuth | PendingApproval
                                           // | ToolsWarning | Cached | Failed | Rejected | Disabled
                                           // | UpdateAvailable | ConfigNeeded
    pub kind: PluginKind,                  // Mcp | Plugin | Skill | Connector
    pub size: IconSize,                    // Px12..Px48
    pub variant: PluginIconVariant,        // Tile (default) | Monogram | Glyph | Dot | Pro(PluginIconPro)
    pub shape: TileShape,                  // RoundedSquare (default) | Circle | Squircle
    pub show_connected_badge: bool,        // default false (✓ moment only)
    pub avoid_hues: Vec<HueSlot>,          // collision nudge from a list host
    pub tool_count: Option<u32>,           // Pro tool-count
    pub label: Option<SharedString>,       // standalone use
    pub motion: MotionLevel,
}

impl PluginIcon {
    pub fn monogram(name: &str, size: IconSize) -> Monogram { /* §5.3, pure, unit-tested */ }
}
```

## 10. Related components and references
**Related:** [icons.md](icons.md) (G8 plug glyphs, grid) · [icon-states.md](icon-states.md) (badge
morphs) · [tools.md](tools.md) §2.4 and [tool-call.md](tool-call.md) (MCP rows) ·
[approvals.md](approvals.md) and [approval-card.md](approval-card.md) (server identity badge) ·
[composer-context-meter.md](composer-context-meter.md) (per-server rows) ·
[composer-tools-menu.md](composer-tools-menu.md) · family J MCP / plugin list (9 states) ·
[loader-mascot.md](loader-mascot.md) (`busy-glow` drawing reused by `connect-beam`).

| Reference | What we take | Licence note |
|---|---|---|
| MCP server states: Claude Code `mcp list`, Agent SDK statuses (research note 05 §6) | the status set and words | Proprietary product — reference only; our wording |
| Cursor Tools & MCP (note 05 §6) | status dot + "Needs login" badge pattern | Proprietary — reference only |
| Arc UI MCP servers card (note 12) | real favicons + status icon with a word, never colour alone | Paid — study only |
| itshover (note 05 §7) | shows that vendor-logo icon packs exist; we deliberately don't ship one | MIT code; logos are trademarks — not used |
| Lucide `plug` and related glyphs (note 05) | base for `GenericGlyph` | ISC — portable with notice |
| A2UI / MCP Apps (note 05 §1) | embedded-app context for `app-frame` | Protocol reference only |
