**Status:** draft v0.1 (2026-11-12) · roadmap V2.5 (PR #10 `docs/site-studio-ia`) · builds in M7 (#26)

# Studio: wireframes and interaction spec

This expands [../studio.md](../studio.md) (the blueprint) into screens, interactions, a data model and
limits. Where this file **adds** to or **departs** from the blueprint it says so with **⚑ flag**.
Everything here is a proposal until both maintainers sign off (roadmap V2.5 exit).

Fixed inputs from other docs:

| From | Fixed fact |
|---|---|
| [../website-stack.md](../website-stack.md) | Dioxus web app at `/studio`; shares `oxyn-codegen` with the CLI so exports match byte for byte; full state in the URL; short-hash fallback; Cloudflare Workers + R2; no user accounts in v1 (licence key is the credential) |
| [../architecture.md](../architecture.md) | One `AgentState` enum drives every component; `Stalled` is derived by `oxyn-core`; one shared motion clock |
| [../variants.md](../variants.md) | Variants differ in structure or motion; colour / radius / font / density are **tokens**, tuned in the Studio |
| [../tokens.md](../tokens.md) §10.1 | The exact token groups the Studio exposes, and the live contrast badge on every swatch |
| [../motion.md](../motion.md) §8, §10 | Reduced-motion forms; safety timings keep their length; Studio runs a flash check |
| [../specs/README.md](../specs/README.md) | Every component has a state machine, timing table, variants and a props sketch, so controls can be generated |

## 1. Purpose and users

The Studio is where you **pick a component, tune it, watch it run through real agent states, and
take exact code away**. It is also the shop window for Pro: you can preview everything, but some
source and some exports are gated.

| User | Comes from | Wants | Leaves with |
|---|---|---|---|
| Rust dev building an agent app | gallery card "Remix", docs page "Tune in Studio" | the right variant, on-brand, in the right states | `oxyn add … --preset …` or a Rust snippet |
| Designer on a Rust team | a share link from a dev | to tune motion and colour without a toolchain | a share link, tokens JSON / CSS, PNG / video |
| Coding agent user | "Copy prompt" on docs | a paste that their agent can install from | agent prompt with every prop and allowed value |
| Launch / marketing (us and buyers) | gallery | a 10–20 s loop for X / a docs thumbnail | MP4 / WebM / GIF at a social aspect ratio |
| Evaluator deciding on Pro | pricing page | to see Pro variants moving | a licence purchase, or a free alternative |

Non-goals for v1: editing the user's running app (a dev overlay is a later Pro idea, research note 06
§2.2 item 9), composing whole pages, user accounts.

## 2. Layouts

Breakpoints (proposed): **desktop** ≥ 1280 px · **laptop** 1024–1279 · **tablet** 768–1023 ·
**mobile** < 768. All wireframes are at ≤ 100 columns.

### 2.1 Desktop default (≥ 1280)

```
┌──────────────────────────────────────────────────────────────────────────────────────────────┐
│ ◂ Gallery  Composer › Send control › ring-progress ▾   ↶ ↷   [A|B]  Copy link  Export ⤓  ⚙ │ top
├──────────────┬──────────────────────────────────────────────────────────┬────────────────────┤
│ OUTLINE      │ ◐ Dark ▾  Desktop▾  BG: surface ▾  100% ▾  Dioxus|GPUI   │ [Component] Theme  │
│ ▾ SendControl│                                                          │ ▸ Presets  ⟳ ↺     │
│   ▸ Button ◀ │                                                          │ ▸ Agent state      │
│   ▸ Glyph    │                   ┌─────────────┐                        │ ▸ Colour           │
│   ▸ Ring     │                   │  ( ■ )  ◜   │  ← live component      │ ▸ Shape            │
│   ▸ QueuePill│                   └─────────────┘                        │ ▸ Sliders          │
│ STATES       │                                                          │ ▸ Motion           │
│  ● Working   │                                                          │ ▸ Speed · seed     │
│ PRESETS      │ ┄┄ Simulate: tool + approval ▾ ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ │                    │
│  ★ pill      │ ▶ ‖  0:02.4 ━━━━━━━━━●━━━━━━━━━━━━━━━━━━ 0:08.0   1× ▾  ⟲  │                    │
│  ★ my-teal   │ state ▕idle▕sub▕thinking ▕tool ▕approval ▕stream ▕done▕     │                    │
├──────────────┴──────────────────────────────────────────────────────────┤                    │
│ ▾ CODE  Rust·GPUI  Rust·Dioxus  Props  Tokens  Prompt  CLI     ⎘ Copy   │                    │
│   SendControl::new(state).variant(SendVariant::RingProgress)            │                    │
│       .busy_action(BusyAction::Steer)          // changed               │                    │
└─────────────────────────────────────────────────────────────────────────┴────────────────────┘
  widths: outline 220 · inspector 300 (resizable 260–420) · drawer 0 / 180 / 50 % height
```

Rules:
1. Canvas is never narrower than 480 px; below that the layout drops to the next breakpoint.
2. The drawer has three snap heights (closed bar, 180 px, half) and remembers its last height.
3. Inspector tabs: **Component** (props of the selection) and **Theme** (global tokens).
   **⚑ flag (addition):** the blueprint lists only "Colour"; tokens.md §10.1 needs a home for global
   tokens (theme, neutrals, status, type, density), so they get their own tab.
4. Top-bar title is a **variant switcher**: clicking `ring-progress ▾` lists every variant of the
   component, Pro ones with a lock chip.

### 2.2 Entry / empty state (`/studio` with no component)

```
┌──────────────────────────────────────────────────────────────────────────────────────────────┐
│ ◂ Docs   Oxyn Studio                                                       Licence key ⚙     │
├──────────────────────────────────────────────────────────────────────────────────────────────┤
│                         Pick a component to tune                                             │
│              ┌──────────────────────────────────────────────────┐                            │
│              │ 🔍 Search components, variants, states…   ⌘K     │                            │
│              └──────────────────────────────────────────────────┘                            │
│   [All] Free  Pro      Family: A Composer · B Loaders · C Status text · D Containers · …     │
│   RECENT (this browser)             START FROM A SCRIPT                                      │
│   ┌────────┐ ┌────────┐ ┌────────┐  ▸ "tool + approval" on Approval card                    │
│   │ ◜ orb  │ │ ( ■ )  │ │ ▤ diff │  ▸ "long stream" on Streaming markdown                   │
│   └────────┘ └────────┘ └────────┘  ▸ "queue + steer" on Send control                        │
│   ALL COMPONENTS (posters, live on hover)                                                    │
│   ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐    │
│   │ poster │ │ poster │ │ poster │ │ poster │ │ poster │ │ poster │ │🔒 Pro  │ │ poster │    │
│   │ Send   │ │ Orb    │ │Shimmer │ │ Beam   │ │ Bubble │ │ Tool   │ │ Nebula │ │Approval│    │
│   └────────┘ └────────┘ └────────┘ └────────┘ └────────┘ └────────┘ └────────┘ └────────┘    │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
```

- Tiles use the CI posters (level 0 in website-stack.md); hover swaps in the live Dioxus demo.
- Search reuses the gallery's Pagefind facets (family, category, state, gesture). Recent is
  `localStorage` only; nothing leaves the browser.
- Other ways in: gallery card **Remix** → `/studio/<family>/<component>?v=1&var=…&p=…`; docs
  **Tune in Studio** → same plus `st=` of the docs demo; **Open in category harness** (Pro, later)
  goes the other way, into a kit shell (categories/README.md) with these settings applied.

### 2.3 Canvas: hover outline, click-to-select, breadcrumb

```
┌─ canvas ──────────────────────────────────────────────────────────────┐
│ SendControl › Button › Ring                          ✥ Select (V) ▾   │ ← breadcrumb, mode
│                                                                       │
│              ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐  Glyph  ← hover: dashed 1 px,      │
│                 ╭───────────╮         name chip, no fill              │
│              │  ┃ ╭──┬──╮   ┃  │                                      │
│                 ┃ │  ■  │◜  ┃ ← selected: solid 2 px accent box,      │
│              │  ┃ ╰──┴──╯   ┃  │   no resize handles (props only),    │
│                 ╰───────────╯       label chip "Ring · 28 px"         │
│              └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘                                    │
│                                                                       │
│  ⌥-click: select parent · Tab: next part · Esc: up one level          │
└───────────────────────────────────────────────────────────────────────┘
```

| Element | Spec |
|---|---|
| Hover outline | dashed 1 px `color.focus` at 60 %, part name chip top-left, shown after 80 ms dwell |
| Selection box | solid 2 px `color.focus`, chip "Part · W × H"; follows the part as it morphs (springs, not jumps) |
| Breadcrumb | root › … › selected; each crumb is a button; hovering a crumb outlines that part |
| Modes | **Select** (V, default): clicks select. **Interact** (I): clicks reach the component (press the send button, open the menu). Holding ⌘/Ctrl in Select mode interacts temporarily |
| Hidden parts | parts not drawn in the current state (e.g. QueuePill when idle) are greyed in the outline; selecting one jumps the simulation to the first state where it shows, with a toast "Jumped to Working with draft" |
| Overlay layer | drawn by the Studio above the preview, never inside the component, so exports never contain it |

### 2.4 Inspector, section by section

Order follows the blueprint: presets → agent state → colour → shape → sliders → motion → speed / seed.
Each section header has **↺ reset section** and **🔒 lock** (for Randomize); `•` marks changed values.

```
┌ ① PRESETS ──────────────────── 🔒 ┐   ┌ ② AGENT STATE ─────────────── • ┐
│ [ pill ▾ ]  ★ Save   ⟳ Random  ↺ │   │ idle  composing  submitted      │
│ Built-in: default · pill · ring   │   │ thinking  streaming [tool]      │
│ Saved:    my-teal · ops-dashboard │   │ approval  queued  steering      │
│ ⧉ Duplicate as B (compare)        │   │ done  error  stalled  cancelled │
└───────────────────────────────────┘   │ ─ tool ─────────────────────────│
┌ ③ COLOUR ───────────────────── 🔒 ┐   │ tool  [ bash      ▾ ]           │
│ Fill   ◼ accent.solid     ⛓ token │   │ verb  [ Running   ▾ ] verb pack │
│ Glyph  ◼ fg.on-accent     ⛓ token │   │ ─ component phase (derived) ────│
│ Ring   ◼ #0F766E          ✎ custom│   │ ComposerState::WorkingWithDraft │
│        OKLCH 0.52 0.10 185 · α 1  │   │ ☐ Pin state (pause Simulate)    │
│        4.8 : 1 on surface ✓       │   └─────────────────────────────────┘
│ [⛓ Bind to token…]  [Hex|OKLCH]   │   ┌ ⑥ MOTION ─────────────────── 🔒 ┐
└───────────────────────────────────┘   │ Level   none | subtle |expressive│
┌ ④ SHAPE ────────────────────── 🔒 ┐   │ Duration multiplier ─●─ 1.00×   │
│ Variant  [ ring-progress   ▾ ]    │   │ Ready→Sending ⛓ dur.fast 160 ms │
│ Layout   compact |inline| card    │   │   easing [ spring · critical ▾ ]│
│ Surface  chat ide desktop mobile  │   │   ╭────────╮ stiffness 420      │
│ Radius   ⛓ radius.md   [ 10 ]     │   │   │  ╭──── │ damping   41       │
│ Stop     None |DoubleEsc| Hold    │   │   │╭─╯     │ ▶ preview          │
└───────────────────────────────────┘   │   ╰────────╯                    │
┌ ⑤ SLIDERS ──────────────────── 🔒 ┐   │ Stagger ⛓ stagger.list  24 ms   │
│ ⇔ Size        ─────●──  28.00 px  │   │ ☐ Preview reduced motion (⇧M)   │
│ ⇔ Ring width  ─●──────   2.00 px  │   │ ⓘ Safety timings keep length    │
│ ⇔ Energy idle ●───────   0.15     │   └─────────────────────────────────┘
│   Budget      ☑ [ 120 ] s         │   ┌ ⑦ SPEED · SEED ────────────── 🔒 ┐
└───────────────────────────────────┘   │ Speed  0 ─────●── 2.00  ☐ rev   │
                                        │ Seed   [ 48213 ]  🎲            │
                                        │ Scale 1.00  Rotate 0°  x 0 y 0  │
                                        └─────────────────────────────────┘
```

| Section | Rules |
|---|---|
| ① Presets | apply all values at once (one undo step); ⟳ = Randomize unlocked (§7); ↺ = variant defaults; saved = this browser (§11) |
| ② Agent state | chips come from `[states]` in `meta.toml`; unsupported states are hidden, not disabled. Per-state params (tool, verb, count, risk tier, elapsed) appear under the chip. The derived component phase is read-only, from `oxyn-core`. `stalled` is selectable: the Studio fakes the event gap that makes `oxyn-core` derive it |
| ③ Colour | default is always the token (tokens.md §10 "Per-component"). Editing the swatch makes it **custom** for this component only; "Bind to token…" lists tokens of the same type. Tokens themselves are edited in the **Theme** tab (link "Edit accent.solid globally →"). The badge says "2.9 : 1 · below 4.5" in text, never colour alone |
| ④ Shape | variant switch keeps compatible props and lists dropped ones; enums ≤ 4 options are segmented; Surface shows only what the variant supports |
| ⑤ Sliders | drag the label (⇔) to scrub (§6); click the value to type; 2 decimals; `Option<T>` = checkbox + inner control |
| ⑥ Motion | rows come from the spec's timing table; duration multiplier 0.5–2× is the Astryx-style global knob (note 16 §14); reduced-motion preview affects the canvas only; safety timings (`dur.approval.arm`, `.hold`, input guard) show but cannot go below their token (motion.md §8) |
| ⑦ Speed · seed | last, speed starts at 0 with a reverse box (Paper Shaders ordering rules, note 17 §3); seed drives particles, noise and sim jitter; scale / rotate / offset are preview-only unless the prop exists |

**⚑ flag:** the blueprint's state list has 12 entries and omits **cancelled**; architecture.md's enum
has `Cancelled` and several specs draw it. Proposed: add it.

### 2.5 Timeline with Simulate scripts

```
┌ SIMULATE ─────────────────────────────────────────────────────────────────────────────────┐
│ Script [ tool + approval ▾ ]  Seed 48213 🎲   Loop ☑   Speed 0.25 0.5 [1] 1.5 2 ×          │
│ ▶  ‖  ⏮  ⏭     0:02.40 / 0:08.00                                    ✎ Edit script (Pro?) │
│ ───────────────────────────────────────────────────────────────────────────────────────── │
│ time   0s       1s       2s       3s       4s       5s       6s       7s       8s         │
│ state  ▕idle▕sub▕thinking ▕tool:bash ▕ approval (waits) ▕streaming      ▕done ▕            │
│ events  ↑send  ▲1st   ·tool start  ·tool end ▲approval  ✓allowed ┊┊┊┊ tokens ┊┊┊ ■end      │
│ draft          ▔▔▔ typing "also run tests" ▔▔▔ (queued 1)                                 │
│ playhead ───────────────────────────●─────────────────────────────────────────────────    │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

| Script (blueprint names) | Event stream fed to `oxyn-core` | Length |
|---|---|---|
| short answer | send → 1st token 400 ms → 40 tokens → done | ~3 s |
| long stream | send → thinking 1.5 s → 1,200 tokens at varied rate → done | ~12 s |
| tool + approval | send → thinking → tool start/end → approval request → allow → stream → done | ~8 s |
| error + retry | send → stream 1 s → error(network) → retry → stream → done | ~6 s |
| queue + steer | send → working → draft typed → Enter (queue 1) → ⌘↵ (steer) → done | ~9 s |
| **stall** (⚑ addition) | send → thinking → 6 s no events → stalled(soft) → events resume | ~10 s |

Rules:
1. Scripts are data (`SimScript` in `oxyn-core`), so the CLI's story tests and the Studio share them.
2. The clock is **virtual**: scrubbing sets `t`, and `oxyn-core` replays the stream from 0 to `t`
   deterministically (seeded jitter), so any frame is reproducible and exportable.
3. Clicking a state band jumps to its start. Dragging the playhead pauses play.
4. "Pin state" in the inspector stops the script and holds one `AgentState` indefinitely.
5. Approval waits in a script auto-answer after 1.2 s; in **Interact** mode the user answers instead.

### 2.6 Code drawer tabs

```
┌ CODE ─────────────────────────────────────────────────────────────────────────────────────┐
│ [Rust·GPUI] Rust·Dioxus  Props  Tokens▾  Prompt  CLI       ☑ only changed  ⎘ Copy  ⤓ .rs  │
│───────────────────────────────────────────────────────────────────────────────────────────│
│  1 use oxyn_gpui::composer::{SendControl, SendVariant, BusyAction};                        │
│  2                                                                                         │
│  3 SendControl::new(&agent_state)                                                          │
│  4     .variant(SendVariant::RingProgress)                                                 │
│  5 ▌   .busy_action(BusyAction::Steer)            ← changed lines get a left bar           │
│  6 ▌   .stop_confirm(StopConfirm::DoubleEsc)                                               │
│  7     .labels(SendLabels::default())                                                      │
│ oxyn-codegen 0.3.1 · registry 0.3 · same output as `oxyn add` ✓                            │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

| Tab | Content | Generated by |
|---|---|---|
| Rust · GPUI | builder snippet; "only changed" off shows every prop | `oxyn-codegen` (wasm) |
| Rust · Dioxus | `rsx!` usage with the same props | `oxyn-codegen` |
| Props | table: prop · type · value · default · range · token binding · Pro | registry meta |
| Tokens ▾ | sub-tabs **JSON** (DTCG, only overridden tokens) · **CSS** (`--oxyn-*` vars) · Rust const | `oxyn-tokens` |
| Prompt | install line + usage + current values + every prop with allowed values (Libraries.dev "Copy prompt" pattern, note 16 §11) | template in `oxyn-codegen` |
| CLI | `oxyn add composer/send-control --variant ring-progress --preset ox1.eNp…` and the short form `--preset oxs_7Kq2mP9aXc` | `oxyn-codegen` |

- The footer line names codegen + registry versions so "byte for byte" is checkable.
- Pro components: Rust tabs show the **usage snippet** (props are not secret). Component **source**
  is never sent to the browser; its place shows a decorative skeleton with "Pro source installs with
  `oxyn add` and a licence" (research note 06 §1: gate the source with a skeleton, not hidden state).

### 2.7 Export sheet

```
┌ Export ───────────────────────────────────────────────── ✕ ┐
│ IMAGE                                                       │
│  ◉ PNG   ○ SVG (vector variants only)                       │
│  Scale [1×] 2× 4×   Background ◉ transparent ○ canvas       │
│  Frame  ◉ current (0:02.40)  ○ poster per state (ZIP)       │
│ VIDEO                                            🔒 Pro ?   │
│  ○ MP4 (H.264)  ○ WebM (VP9)  ○ GIF                         │
│  Range  ◉ whole script  ○ one loop  ○ in/out [0:01–0:05]    │
│  Aspect 16:9  1:1  [4:5]  9:16  · Size 1080 × 1350 · 30 60fps│
│  ⚠ Flash check: pass (max 1 flash / s)                      │
│ SHARE                                                       │
│  https://<site>/studio/composer/send-control?v=1&s=eNp…   │
│  412 chars · fits  [⎘ Copy link]  [Short link ⤴]            │
│ CODE  ⎘ Rust  ⎘ CLI  ⎘ Prompt  ⤓ tokens.json  ⤓ tokens.css  │
│                                          [ Cancel ] [Export]│
└─────────────────────────────────────────────────────────────┘
```

- SVG is enabled only when the active variant renders as vector (T2 / glyph tiers); otherwise it is
  disabled with the reason "This variant draws on a canvas; export PNG or video".
- Video renders off the main thread (§12) with a progress bar and Cancel; the canvas stays usable.
- The flash check (motion.md §10 rule 7) re-runs on the user's settings; a fail **blocks video
  export** and says which setting caused it (usually speed > 1.5× on a luminance loop).

### 2.8 Compare / A-B

```
┌──────────────────────────────────────────────────────────────────────────────────────────────┐
│ Compare   [Side by side] Swipe  Overlay    Sync clock ☑   Edit: (A) B      ⇄ Swap   B→A ✓    │
├──────────────────────────────────────────────┬───────────────────────────────────────────────┤
│ A · pill (current)                           │ B · ring-progress + my-teal                   │
│                                              │                                               │
│               ( ■ )                          │                ( ■ )◜                         │
│                                              │                                               │
├──────────────────────────────────────────────┴───────────────────────────────────────────────┤
│ ▶ ‖  0:02.40 ━━━━━━━━━●━━━━━━━━━━━━━━━  shared timeline when clocks are synced              │
│ DIFF  variant pill → ring-progress · ring.colour accent.solid → #0F766E · busy_action → Steer│
└──────────────────────────────────────────────────────────────────────────────────────────────┘
```

- A and B are two full `StudioState`s; the inspector edits whichever is active (badge on the header).
- **Swipe**: a vertical divider wipes B over A at the same position (useful for colour and radius).
- **Overlay**: B at 50 % over A (useful for motion timing).
- "B→A" copies B into A as one undo step. Compare state is in the URL (`cmp=`).
- Also reachable as **"Light vs dark"** and **"Normal vs reduced motion"** quick compares (⚑ addition).

### 2.9 Tablet / narrow (768–1023)

```
┌───────────────────────────────────────────────────────────────┐
│ ◂  Send control · ring-progress ▾          ↶ ↷  ⎘ link  ⤓  ☰  │
├───────────────────────────────────────────────────────────────┤
│ ◐ Dark ▾  Mobile ▾  100% ▾                     Inspector ▸    │
│                                                               │
│                         ( ■ )◜                                │
│                                                               │
│ ▶ ‖ 0:02.4 ━━━━━━━●━━━━━━━━━━━━━━━━━ 0:08  1×  tool+approval ▾│
├───────────────────────────────────────────────────────────────┤
│ [Outline] [Inspector] [Code]          ← bottom sheet tabs     │
│ ▸ Presets  ▸ Agent state  ▸ Colour  ▸ Shape  ▸ Motion  …      │
└───────────────────────────────────────────────────────────────┘
```
Laptop (1024–1279): outline collapses to a 48 px icon rail (parts, states, presets); everything else
as desktop. Tablet: one bottom sheet with three tabs, 40 % height, draggable to 85 %. Touch: scrub by
dragging the value itself (labels are too small), 44 px targets, long-press a part = select its parent.

### 2.10 Mobile (< 768): view, play, share

```
┌─────────────────────────────────────┐
│ ◂  Send control        ⎘ Share  ⋯   │
├─────────────────────────────────────┤
│                                     │
│               ( ■ )◜                │
│                                     │
│ ▶ ‖ ━━━━━━━●━━━━━━━━━━  tool+appr ▾ │
├─────────────────────────────────────┤
│ State  idle thinking [tool] done …  │  horizontal chips
│ Preset [ pill ▾ ]  Variant [ ring ▾]│
│ Theme  ◐ dark   Reduced motion ☐    │
│ ⎘ Copy CLI   ⎘ Copy prompt          │
│ "Open on a larger screen to edit"   │
└─────────────────────────────────────┘
```
Decision (proposed): mobile is **view + light edit**: states, preset, variant, theme, reduced motion,
copy link / CLI / prompt. No sliders, colour or motion editors, no video export. Opening a link on
mobile never loses state: everything it cannot edit is kept and re-encoded unchanged.

### 2.11 Pro-locked controls

```
┌ SHAPE ──────────────────────────── ┐
│ Variant  [ nebula  🔒 Pro ▾ ]      │  preview runs fully; export gated
│ Interaction  None | Tooltip |      │
│              DragSpin 🔒           │  locked option: selectable, preview
│                                    │  works, chip "Pro" stays visible
└────────────────────────────────────┘
┌ banner above the code drawer ─────────────────────────────────────────────────────┐
│ 🔒 This setup uses Pro: variant `nebula`, prop `interaction = DragSpin`.           │
│ [Enter licence key]  [See Pro]  [Make it free: switch to `dotted-breathe` twin]    │
└───────────────────────────────────────────────────────────────────────────────────┘
```
- Locked things are **tryable, not hidden** ("preview everything, gate the source", note 06 §1).
- "Make it free" uses the variant's declared free twin / fallback from `meta.toml`
  (thinking-orb.md requires every shader variant to name a particle twin) and drops Pro-only props,
  listing what changed as one undo step.
- With a valid key (pasted once, kept in `localStorage`, validated by the registry Worker as in
  website-stack.md), locks disappear and the CLI tab shows the Pro install line.

## 3. Selection model

| Rule | Detail |
|---|---|
| Unit | `Selection` is the component root or one **part** (from `parts` in `meta.toml`). Single selection only in v1 |
| Click | selects the **deepest** part under the pointer (agent components are small; drilling down every time is slow) |
| ⌥-click · Esc · Tab | parent of what is under the pointer · up one level (stops at root, never "nothing") · next part in outline order |
| Inspector scope | shows props whose `parts` list includes the selection, plus a collapsed "Component-wide" group |
| Part geometry | renderer reports rects: Dioxus via `data-oxyn-part` attributes; GPUI-web via a hit-test map from layout (needs a hook in `oxyn-gpui`, ⚑ new requirement) |
| Morphing parts | the box follows the part every frame while playing; hover outlines hide during play to avoid flicker |
| URL, undo | saved as `sel=` so a link can point at "look at the ring"; selection and playhead are not undoable |

## 4. Undo / redo

| Item | Rule |
|---|---|
| Model | stack of `Patch { ops: Vec<Op { path, before, after }> }`; undo applies `before` in reverse |
| Granularity | one user gesture = one entry: a whole scrub drag, a preset apply, a Randomize, an "Ask" diff, "Make it free", B→A |
| Coalescing, depth | typing coalesces until blur or 800 ms idle; 200 entries, oldest dropped |
| Persistence | `sessionStorage` per tab; lost on close (the URL holds the current state, not history) |
| Branching, A/B | a new edit after undo discards the redo tail (no tree in v1); A and B have their own stacks |
| URL sync | `history.replaceState` on every change (throttled 250 ms); **no** `pushState`, so Back leaves the Studio instead of walking edits |

## 5. Keyboard shortcuts

| Keys | Action | Scope |
|---|---|---|
| ⌘/Ctrl Z · ⇧⌘Z / Ctrl Y · ⌘/Ctrl K | undo · redo · component search | global |
| Space | play / pause | not in text fields |
| ← / → · [ / ] | playhead −/+ 100 ms (⇧ = 1 s) · previous / next state band | timeline or canvas |
| 1–9 | pin the n-th supported agent state | global |
| V · I · hold ⌘/Ctrl | Select mode · Interact mode · interact temporarily | canvas |
| Esc · Tab / ⇧Tab | up one level or close sheet · next / previous part | canvas |
| R · ⇧R · L | Randomize unlocked · reset all · lock focused section | global / inspector |
| ⇧M · ⇧D | reduced-motion preview · light / dark | global |
| ⌘/Ctrl C (nothing selected in text) | copy current code tab | drawer open |
| ⌘/Ctrl ⇧C · ⌘/Ctrl E | copy share link · export sheet | global |
| ⌘/Ctrl \ · ⌘/Ctrl . | toggle code drawer · toggle inspector | global |
| A / B · ? | edit side A / B (compare) · shortcut sheet | global |

Single-letter keys are ignored while a text, number or colour field has focus, and can be turned off
in ⚙ (WCAG 2.1.4 Character Key Shortcuts).

## 6. Drag-scrub

| Item | Rule |
|---|---|
| Handle | the control's **label** (cursor ⇔); on touch, the value field |
| Rate, pointer | 1 px = 1 step; ⇧ ×10; ⌥/Alt ×0.1; Pointer Lock when available so long drags don't hit the screen edge |
| Range | clamps to `ControlDef.range`; typed values may exceed a **soft** range up to the **hard** range, shown in amber |
| Feedback | value updates live; canvas updates each frame; codegen waits for pointer-up (§12) |
| Keyboard | label focused: ↑/↓ step, ⇧ ×10, PageUp/PageDown ×10, Home/End min/max |
| Reset, undo | double-click label → default (`•` shows while changed); the whole drag is one undo entry |
| Display | 2 decimals for unitless floats; integers for px / ms; unit suffix always shown |

The scrub-area Number Field in coss ui (MIT, research note 17) is the reference for the API shape;
we implement it in Dioxus ourselves.

## 7. Randomize and locks

| Item | Rule |
|---|---|
| Scope | randomizes every control in **unlocked** sections; a single control can also be locked (🔒 on hover) |
| Never randomized | agent state, variant, labels / text, safety timings (`dur.approval.*`, `delay.approval.input-guard`), anything marked `random = false` in meta |
| Distribution | uniform within `random_range` if meta gives one, else within the soft range; enums uniform; booleans 50 % |
| Colour | samples OKLCH around the current accent (harmonies: analogous, complementary, triad); re-rolls up to 8 times until every contrast badge passes; if none pass, keeps the best and shows the badge |
| Motion | durations snap to the nearest `dur.*` token 70 % of the time, so results stay on-system |
| Seed, undo | own RNG seeded from the time; result is explicit values, so links reproduce it; one undo entry |
| Defaults | locked by default: Agent state, Colour (most people randomize motion and shape first) ⚑ proposal |

## 8. URL state

### 8.1 Shape
```
https://<site>/studio/<family>/<component>?v=1&var=<variant>&p=<preset>&st=<state>&s=<blob>
        [&sel=<part>] [&t=<ms>] [&cmp=<blob>] [&r=<registry-version>]
https://<site>/studio/s/<hash>          short link (resolved by the Worker)
```

| Param | Meaning | Example |
|---|---|---|
| path | component id | `/studio/composer/send-control` |
| `v` | **URL schema version** (required) | `1` |
| `var` | variant | `ring-progress` |
| `p` | base preset | `pill` |
| `st` | pinned agent state, or `sim:<script>` | `tool`, `sim:tool-approval` |
| `s` | compact blob: everything that differs from variant + preset defaults | `eNpLy…` |
| `sel` | selected part path | `button.ring` |
| `t` | playhead in ms | `2400` |
| `cmp` | B side blob (compare) | `eNp…` |
| `r` | registry version the link was made with (optional, for exact codegen) | `0.3` |

Hand-written links stay readable (`?v=1&var=pill&st=thinking`); the blob is only for overrides.

### 8.2 Compact encoding
1. Build `StudioDelta` (only non-default values; prop keys are the stable short `key` from meta).
2. Serialize with **postcard** (compact binary serde), then **deflate-raw**, then **base64url**
   without padding. All three steps live in `oxyn-codegen` so the CLI decodes `--preset ox1.<blob>`
   offline with the same code.
3. Canonical form: keys sorted, floats rounded to the control's step, defaults removed. Equal states
   give equal strings (needed for dedupe and short hashes).

⚑ **Departure (small):** research note 18 says "serde → compact base64url"; adding deflate and
canonicalisation is new.

### 8.3 Size limits and short-hash fallback

| Threshold | Behaviour |
|---|---|
| full URL ≤ 1,500 chars | "Copy link" copies the long link (target: most states fit) |
| 1,500 – 8,000 chars | long link still works in the address bar; Copy link offers **Short link** first, with a note that some chat apps cut long links (proposed threshold; not measured) |
| > 8,000 chars | long link not offered; short link only |
| blob > 64 KB decoded | rejected: "This setup is too large to share" (should be unreachable: a delta of one component's props) |

Short link: `POST /api/share` on a Worker with the canonical blob → content-addressed key (first 10
base62 chars of SHA-256) stored in KV or R2, immutable, no expiry in v1. Same blob → same hash, so
repeated shares cost no writes. The CLI accepts `--preset oxs_<hash>` and fetches it (online only).
Storage choice and free-tier write limits are **unverified** (open question 4).

### 8.4 Versioning

| Case | Behaviour |
|---|---|
| `v` missing | treat as `v=1` if parse succeeds, else invalid |
| `v` < current | run migrations in `oxyn-codegen` (`v1 → v2 …`); meta `renamed_from` maps renamed props; then `replaceState` to the current version |
| `v` > current | banner "This link was made with a newer Studio" + Reload; load what can be read, read-only until reload |
| unknown prop key | dropped; toast lists the dropped keys; undo restores nothing (they never applied) |
| unknown variant / component | entry screen with the closest match pre-filled ("`send-control/ring` was renamed `ring-progress`") |
| `r` ≠ current registry | banner "Made with Oxyn 0.3; code below is for 0.4" with a link to the 0.3 CLI command ⚑ (see open question 5) |

## 9. Data model (Rust sketch)

```rust
// apps/studio/src/state.rs (sketch)
pub struct StudioState {
    pub v: u16,                                  // URL schema version
    pub component: ComponentId,                  // "composer/send-control"
    pub variant: VariantId,                      // "ring-progress"
    pub preset: Option<PresetId>,                // base preset; overrides apply on top
    pub props: BTreeMap<PropKey, Value>,         // only non-default values
    pub tokens: BTreeMap<TokenPath, TokenValue>, // global overrides (Theme tab)
    pub agent: AgentDrive,
    pub view: ViewState,
    pub selection: Selection,                    // Root | Part(PartPath), e.g. ["button","ring"]
    pub locks: BTreeSet<LockKey>,                // sections or single props
    pub compare: Option<Box<StudioState>>,       // B side; never nested
}
pub enum AgentDrive {
    Pinned(AgentState),                          // oxyn-core enum incl. per-state params
    Simulate { script: ScriptId, t_ms: u32, speed: f32, seed: u64, looping: bool },
}
pub struct ViewState {
    pub theme: ThemeId, pub device: Device, pub background: CanvasBg, pub zoom: f32,
    pub renderer: Renderer,                      // Dioxus (default) | GpuiWeb
    pub reduced_motion_preview: bool, pub motion_multiplier: f32, // 0.5..=2.0
}
pub struct Preset {
    pub id: PresetId, pub label: String,
    pub source: PresetSource,                    // BuiltIn | Local (browser) | Team (Pro, later)
    pub variant: VariantId,
    pub props: BTreeMap<PropKey, Value>, pub tokens: BTreeMap<TokenPath, TokenValue>,
}

// Generated at build time from registry/*/meta.toml by `xtask build-registry`
pub struct ControlDef {
    pub key: PropKey,                            // stable short key used in URLs ("ba")
    pub rust_path: &'static str,                 // "busy_action"
    pub label: &'static str,
    pub section: Section,                        // State, Colour, Shape, Sliders, Motion, SpeedSeed
    pub kind: ControlKind,
    pub default: Value,
    pub parts: &'static [PartPath],              // shown when one of these parts is selected
    pub token: Option<TokenBinding>,             // default token + allowed token type
    pub tier: Tier,                              // Free | Pro
    pub random: RandomRule,                      // Never | Range(..) | Default
    pub visible_if: Option<Condition>,           // e.g. only when stop_confirm == Hold
    pub renamed_from: &'static [&'static str],   // URL migrations
}
pub enum ControlKind {
    Enum { options: &'static [EnumOption] },     // EnumOption { value, label, tier }
    Bool,
    Float { soft: Range<f32>, hard: Range<f32>, step: f32, unit: Unit },
    Int { soft: Range<i32>, hard: Range<i32>, step: i32, unit: Unit },
    Color, Duration { token_group: &'static str }, Easing, Spring,
    Text { max: usize }, Optional(&'static ControlKind), Labels, Seed,
}
```

## 10. Controls generated from component meta

Source of truth: `registry/<tier>/<component>/meta.toml` → `xtask build-registry` → `index.json`
and a generated Rust table the Studio, CLI and MCP read (website-stack.md data flow). ⚑ **New
requirement for `oxyn-registry-schema`:** a `[props]` entry per prop and a `parts` list.

```toml
# registry/free/send-control/meta.toml (excerpt, proposed fields)
parts = [{ id = "button", label = "Button" }, { id = "button.ring", label = "Ring" }]

[props]
busy_action = { key = "ba", section = "shape", kind = "enum", default = "queue",
                options = ["queue", "steer", "interrupt", "ask"] }
ring_width  = { key = "rw", section = "sliders", kind = "float", unit = "px", step = 0.25,
                soft = [1.0, 4.0], hard = [0.5, 8.0], parts = ["button.ring"] }
ring_color  = { key = "rc", section = "colour", kind = "color",
                token = "color.accent.solid", parts = ["button.ring"] }
```

| `ControlKind` | Widget | Notes |
|---|---|---|
| `Enum` ≤ 4 options | segmented control | Pro options carry a lock chip |
| `Enum` > 4 | select with search | variant switcher uses the same widget |
| `Bool` | switch | |
| `Float` / `Int` | slider + scrub label + number field | soft range on the track, hard range when typed |
| `Color` | swatch + Hex / OKLCH field + token binder + contrast badge | default is the token |
| `Duration` | slider in ms + token chip (`⛓ dur.fast`) | snaps to tokens unless ⌥ held |
| `Easing` | curve editor + named list (`ease.*`) | live dot preview |
| `Spring` | spring editor, two modes: stiffness / damping and duration / bounce | closed-form solver from `oxyn-core` draws the curve |
| `Text` | text field | |
| `Optional(k)` | checkbox + the inner widget | |
| `Labels` | grouped text fields per state | shows the screen-reader text per state |
| `Seed` | number + 🎲 | |
| `AgentState` (special) | state chips + per-state params | from `[states]` in meta, not from a prop |

Rules:
1. **Props = controls, one to one** (Originkit pattern, note 16 §7). No Studio-only knobs except view
   settings (zoom, background, device, transforms marked "preview only").
2. Section order inside a section follows `meta.toml` order; colours first, speed / seed last.
3. A prop without `[props.*]` metadata still appears, in an "Advanced" group with a generic widget
   inferred from its type, and CI warns.
4. Ranges come from the spec's timing / edge-case tables; the spec author writes them once in meta.

## 11. Free vs Pro (proposed, **unconfirmed**; pricing is open per vision.md)

| Capability | Free | Pro | Basis |
|---|---|---|---|
| Open the Studio, all free components and variants, every control | ✓ | ✓ | vision.md: Studio is free |
| Preview Pro variants and Pro options in the canvas | ✓ (locked chip) | ✓ | note 06 §1 "preview everything, gate the source" |
| Rust (GPUI, Dioxus) usage code, props, tokens JSON / CSS, prompt, CLI | ✓ for free setups | ✓ all | |
| Pro component source | — | via `oxyn add` + licence | website-stack.md Pro registry |
| PNG export | ✓ | ✓ | |
| SVG export | ✓ | ✓ | ⚑ proposal |
| Share link (long + short) | ✓ | ✓ | |
| Compare / A-B | ✓ | ✓ | ⚑ proposal |
| Saved presets in this browser | ✓ | ✓ | ⚑ **departs from vision.md**, which lists "saved presets" as Pro; local saves cost us nothing and a share link already stores state. Proposed: **cloud / team** presets are the Pro part |
| MP4 / WebM / GIF export | watermark? or — | ✓ | vision.md lists video export as Pro; watermarked free tier is open question 2 |
| "Ask" box (natural-language edits to the selection) | — | ✓ | studio.md Extras; has per-call model cost |
| Team libraries (shared presets, themes) | — | ✓ (needs accounts; later) | studio.md Extras; note 06 Studio add-on |
| Edit Simulate scripts / custom event streams | — | ✓ ⚑ proposal | |
| Import theme from CSS vars or screenshot | CSS vars ✓ ⚑ | screenshot ✓ | studio.md Extras |
| Open in category harness | — | ✓ (kits are Pro, per vision.md) | studio.md Extras; categories/README.md |

"Ask" box (Pro, later):
```
┌ ASK · scoped to Ring ─────────────────────────────┐
│ "pulse slower and use our teal"           [Ask ⏎] │
│ Proposed:  ring.speed  1.00 → 0.70                │
│            ring_color  accent.solid → #0F766E     │
│ [Apply (one undo step)]  [Discard]                │
└───────────────────────────────────────────────────┘
```
It returns a **patch** against `ControlDef`s only (never code), limited to props of the selected part.

## 12. Performance

| Budget | Target | How |
|---|---|---|
| Control change → canvas frame | ≤ 16 ms (one frame at 60 Hz) | props go straight to the mounted component; no codegen, no URL work in the input path |
| Codegen refresh | ≤ 50 ms after pointer-up / 150 ms idle | `oxyn-codegen` wasm call, debounced; drawer shows a 1 px progress line, never blocks input |
| First load (shell + Dioxus renderer + one family) | ≤ 1.5 MB Brotli wasm ⚑ target | per-family wasm chunks loaded on demand (Dioxus 0.7 bundle splitting, research note 18) |
| GPUI-web renderer | opt-in, loaded on toggle | multi-MB and WebGPU-only (website-stack.md); never on first load |
| Scrubbing the timeline | 60 fps for ≤ 30 s scripts | replay from the nearest cached checkpoint (every 500 ms of sim time), not from 0 |
| URL write · idle | throttled 250 ms · no frames | `replaceState` only · shared `MotionClock`, paused when hidden or off-screen (motion.md §9) |
| Video encode | off main thread | frames stepped on the virtual clock and drawn to `OffscreenCanvas` in a Web Worker; WebCodecs `VideoEncoder`; MP4 / WebM muxed in wasm; GIF quantised in the same worker. Cancel frees everything |
| Video limits | ≤ 30 s, ≤ 1080 × 1920, ≤ 60 fps ⚑ | keeps memory under ~500 MB in a tab (estimate, unmeasured) |
| Compare | two instances share one clock and one wasm | B mounts lazily |

Frame capture for video needs a canvas renderer. Dioxus renders DOM, so capture has to use either
the GPUI-web canvas (WebGPU) or a DOM-to-canvas path; see open question 1.

## 13. Accessibility of the Studio itself

| Area | Rule |
|---|---|
| Keyboard | every action reachable by keyboard (§5); focus order: top bar → outline → canvas → timeline → inspector → drawer |
| Outline | ARIA `tree`; selecting a node selects the part; the canvas box follows |
| Canvas | focusable region labelled "Preview of Send control, state Working"; parts reachable with Tab in Select mode; Interact mode passes keys to the component |
| Controls | sliders are `role="slider"` with `aria-valuetext` including the unit ("160 milliseconds, token dur.fast"); scrub labels are buttons with the same keys as the slider |
| Colour | every picker has a text field; contrast badge has text, not colour only |
| Live region | a polite region announces simulated state changes, throttled to one per second ("Agent: awaiting approval"), and copy / export results |
| Studio motion | Studio chrome follows the OS reduced-motion setting; the canvas toggle (⇧M) is separate and only affects the preview |
| Photosensitivity | the flash check (motion.md §10) runs on the user's live settings; a fail pauses playback, shows "Paused: this setup flashes more than 3 times a second", and needs an explicit Play |
| Contrast, zoom, touch | Studio UI meets WCAG 2.2 AA in both themes; usable at 200 % zoom (drops to tablet layout); 44 px touch targets; single-key shortcuts can be turned off (§5) |

## 14. Error states

| Error | What the user sees | Recovery |
|---|---|---|
| URL blob cannot be decoded | component opens with defaults; toast "Couldn't read this link's settings" + "Show raw" | edits continue; the bad blob is kept in a "Restore" menu for this session |
| Version newer / older than this Studio | newer: banner + Reload, read-only; older: silent migration, toast if props dropped | §8.4 |
| Component / variant removed or renamed | entry screen with closest match | §8.4 |
| Short hash not found / Worker down | "This short link isn't available"; entry screen | long links unaffected |
| Pro setup, no licence | full preview, Pro banner (§2.11) | enter key or "Make it free" |
| Licence invalid / expired | banner with the Worker's message (Polar status) | portal link |
| WASM fails to load (old browser, blocked, out of memory) | static fallback page rendered by Astro: posters per state, props table, CLI command, "Studio needs WebAssembly" | links still show something useful |
| iOS streaming-compile failure | loader retries without streaming compile (Cydonia's fix, note 18) | automatic |
| No WebGPU | "GPUI" renderer toggle disabled with reason; T4 shader variants show their T3 twin with chip "Showing particle twin" | Dioxus default unaffected |
| No WebCodecs | MP4 / WebM disabled with reason; GIF via wasm encoder still offered | |
| Codegen error (panic / bad meta) | drawer shows the error and "Report" (prefilled GitHub issue with the URL); canvas keeps running | |
| Flash check fail | §13 | lower speed / switch variant |
| Storage full / blocked (`localStorage`) | saved presets disabled with reason | share links still work |

## 15. Open questions

1. **Video frame capture with Dioxus.** DOM can't be read back as frames cheaply. Options: (a) use
   the GPUI-web canvas for all media export (WebGPU only, ~82–89 % of browsers per note 18);
   (b) DOM → SVG `foreignObject` → canvas (slow, font / filter fidelity issues); (c) render media in
   CI-like headless GPUI on a server (costs money, breaks "client-side"). Proposed: (a), with (b)
   for PNG only.
2. **Free video export:** none, watermarked, or 720p free / HD Pro (Animos-style tiers, note 17)?
3. **Saved presets:** keep vision.md's "Pro", or local saves free and cloud / team Pro (§11)?
4. **Short-link storage:** KV vs R2 vs D1, and their free-tier write limits (not verified this
   session). Do short links expire?
5. **Registry version pinning (`r=`):** serve old codegen versions so old links reproduce byte for
   byte, or always generate for the current registry and only warn?
6. **Mobile scope:** is view + light edit (§2.10) enough, or ship full editing on tablet only?
7. **Click semantics:** deepest-part-first (proposed) vs Figma-style top-level-first with
   double-click to drill.
8. **Default renderer:** Dioxus (fast, everywhere) is the default; GPUI posters are the reference look
   (website-stack.md risk). Should the Studio warn when Dioxus and GPUI differ visibly for a variant?
9. **Simulate script editing:** free JSON editing, Pro visual editor, or not at all in v1?
10. **Theme tab vs blueprint:** confirm the Component / Theme split (§2.1 rule 3).
11. **`Cancelled` and `stall` script:** confirm the two additions to the blueprint lists (§2.4 ②, §2.5).
12. **Accounts:** team libraries need accounts, which website-stack.md defers. Which ships first?

## 16. References (research notes opened for this file)

| Note | Used for | Licence note (from the note) |
|---|---|---|
| 06 business / Studio stack §1, §2 | Studio feature set, "preview everything, gate the source", Studio add-on idea, Leva / DialKit / tweakcn / React Bits patterns | study only unless MIT (DialKit, Leva are MIT per the note) |
| 16 niche sites A §7 Originkit, §11 Libraries.dev, §14 Astryx | props = controls, presets in URL, copy prompt, per-size tuning, free runtime / paid editor, click-to-select, motion multiplier | Originkit terms not verified → study only; Libraries.dev packages MIT, Studio Pro; Astryx licence not confirmed → study only |
| 17 niche sites B §3 Studio blueprint (Paper Shaders, Magnific), Animos, transitions.dev, coss ui | inspector order, speed / seed rules, export matrix, aspect presets, Remix → Studio, scrub field | Paper Shaders Apache-2.0; coss ui MIT; transitions.dev study only |
| 18 website stack | Dioxus Studio, codegen in wasm, URL state, GPUI-web size and WebGPU coverage, iOS wasm fix | — |
| 12 Arc site design | Arc has no generic props playground; knobs live in each demo (contrast for our approach) | study only |

External URLs are recorded in those notes; none were re-opened for this file.
