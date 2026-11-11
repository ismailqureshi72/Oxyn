**Status:** draft v0.1 (2026-11-11).

# Spec: composer model picker (pill + popover)

Follows the 10-point standard in [README.md](README.md) at the depth of the worked example
[composer-send-control.md](composer-send-control.md). Target **6 free + 14 Pro variants**, all driven by
one state machine. Every number below is an Oxyn **proposal** unless a source is cited next to it.

## 1. Purpose
The model pill sits in the composer's control row (usually right of `+`, left of the send control). The
user is choosing **who does the next piece of work**: a cheaper/faster model for a quick question, a
reasoning model for a hard refactor, a vision model because an image is attached, or "Auto" so the app
routes for them. The picker must answer three questions at a glance: *what will run next*, *can it do
what my message needs* (images, tools, context length), and *what will it cost me in time and money*.
It also has to be honest when a model **cannot** run right now (no key, rate-limited, down) and behave
predictably when the user switches while an agent is mid-run.

Model pickers are table stakes: research note 03 found one in essentially every coding-agent app
(Zed provider logos + ★ favourites + cycle shortcut, Conductor model search, Pi `/model` / Ctrl+L /
Ctrl+P favourites, Codex combined "Select Model and Effort" list), and research note 01 counts it among
the atoms shipped by 6–12 libraries. Our differentiation is the **state machine** (availability states,
mid-run policy, per-message model, router transparency) and the motion, not the existence of a list.

## 2. State machine

Two layers: the **pill** (what is selected and whether it can run) and the **popover** (browsing).

```
                       hover                  click / Enter / Mod+Shift+M
  Closed(model) ───────────────▶ Hover ─────────────────────────────────▶ Opening ─▶ Browsing
     ▲  ▲  ▲                       │                                                 │  │  │
     │  │  └──── Esc / click-out ──┴────────── Closing ◀── Esc / click-out ──────────┘  │  │
     │  │                                                                type ▼          │  │
     │  │                                                           Searching ─ no hits ─▶ NoResults
     │  │                                                                │ ↑↓ / hover       │
     │  │                                                                ▼                  │
     │  │                                                           Highlighted(row) ◀──────┘
     │  │                                     Enter / click on available row │
     │  │                       ┌─────────────────────────────────────────────┤
     │  │        agent idle     ▼                                 agent busy ▼
     │  └──── Applied ◀─── Switching ──────┐                  MidRunChoice ── policy ──┐
     │         (label roll)                │                (NextTurn | ApplyNow | Ask) │
     │                                     │                                            ▼
     │                                     │                               PendingNext(model)
     │                                     │                      (pill shows "→ Opus" badge)
     │                                     │                                  │ run ends
     └─────────────────────────────────────┴──────────────────────────────────┘
  Row-level / pill-level special states:
     NeedsKey(provider) ─ Enter ─▶ KeyFlow (app callback) ─ key saved ─▶ Available
     RateLimited(until) ─ timer reaches 0 ─▶ Available          Unavailable(reason)
     Deprecated(sunset)  · Incompatible(reason: no vision / context too long / no tools)
     CatalogLoading ─▶ CatalogError(retry) · Offline · Locked(policy) · Routing (Auto resolving)
```

Transitions worth stating:
1. `Switching` is instant for the app (selection is a local setting); there is no spinner unless the
   adapter must call the provider to validate (`validate_on_select`), in which case the pill shows a
   2 px underline progress and stays interactive.
2. Selecting a row in `NeedsKey` does **not** select the model; it fires `on_request_key(provider)` and
   leaves the popover open in a `KeyFlow` row state ("Waiting for key…"). When the key lands the row
   re-renders as available and the model is selected automatically (the user's intent was clear).
3. A `RateLimited` model stays selectable when the app sets `allow_queued_on_limit` (the message waits
   until the reset); otherwise it is disabled-with-reason and shows a live countdown.
4. `Auto` is a normal row. While `Submitted`/`Thinking` the pill is in `Routing` and shows `Auto`; on
   the first event that names the resolved model it rolls to `Auto · Sonnet` (secondary label).

## 3. States table

| State | Pill shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Closed` | provider mark 14 px + short name ("Sonnet 5") + `⌄` 10 px; caps hint dots optional | open popover | — | Tab focus, Enter/Space open, `Mod+Shift+M` open (proposed), `Alt+M` / `Alt+Shift+M` cycle favourites (proposed) |
| `Hover` | bg → `chip.hover`, chevron nudges 1 px down | open | tooltip after 500 ms: full name, context size, "Mod+Shift+M" | — |
| `Focus` | 2 px focus ring, 2 px offset | open | — | Enter/Space/↓ open |
| `Opening` / `Browsing` | popover above the composer: search, Auto, Recent (≤ 3), Favourites, provider groups | highlight + select | ★ toggle on row, "Manage models" footer link | type to search, ↑↓ move, Enter select, `Mod+F` focus search, Esc close |
| `Searching` | filtered rows, matched chars bold; filter chips (vision · tools · reasoning · 1M) | select highlighted | clear (×) | Esc clears query first, second Esc closes |
| `NoResults` | "No model matches 'gpt-9'" + "Clear search" | clear | — | Esc clear |
| `Highlighted(row)` | row bg `list.highlight` (one gliding highlight), detail tray updates | select | ★ (`Mod+D` proposed), info (→ opens detail side card) | ← / → toggle detail card |
| `Switching` | label rolls old → new; width springs | — | — | — |
| `Applied` | new model; 1 subtle accent flash on the mark | — | undo toast 4 s optional ("Switched to Opus · Undo") | `Mod+Z` while toast visible |
| `MidRunChoice` (policy `Ask`) | inline 2-button strip inside popover row: "Next turn" · "Restart turn with Opus" | next turn (default) | apply now | Enter = next turn, `Mod+Enter` = apply now |
| `PendingNext(model)` | current model + small "→ Opus" badge with dashed outline | — | cancel pending (× on badge) | Esc on focused badge cancels |
| `Routing` (Auto) | "Auto" + 3-dot breathe in the mark slot | — | — | — |
| `Auto · resolved` | "Auto · Sonnet" (secondary label at 70 % opacity) | open | "Pin Sonnet" in popover | — |
| `NeedsKey(provider)` | row: lock glyph + "Add API key" link text; pill (if selected elsewhere): warning dot | start key flow | "Use another model" | Enter starts flow |
| `RateLimited(until)` | row: clock glyph + "Resets in 4:12" live; pill: amber dot + countdown in tooltip | select (if `allow_queued_on_limit`) | "Use fallback" (app-provided) | — |
| `Unavailable(reason)` | row dimmed 50 %, reason text ("Provider outage", "Not on your plan") | — (disabled) | "Learn more" link if provided | row is focusable, Enter announces reason |
| `Incompatible(reason)` | row normal opacity + warning glyph + reason ("Can't read the 2 attached images", "Conversation is 240K, model holds 200K") | select with confirm, or blocked per `incompatibility` prop | "Compact first" (context case) | — |
| `Deprecated(sunset)` | row badge "Retires Dec 1" | select | "Switch to successor" | — |
| `CatalogLoading` | skeleton rows after 150 ms grace; pill keeps last known model | — | — | Esc close |
| `CatalogError` | "Couldn't load models" + Retry; cached list shown below if any | retry | — | Enter retry |
| `Offline` | pill dims 70 %, tooltip "Offline · model list may be stale" | open (cached) | — | — |
| `Locked(policy)` | pill shows lock 10 px, no chevron; tooltip "Set by your workspace" | — | — | Enter announces reason |
| `Disabled(reason)` | pill muted, tooltip with reason | — | — | — |

Rules:
1. The pill **never changes the composer layout height**; width changes animate and are capped
   (`max_pill_width`, default 180 px, then the name truncates).
2. The selected model is always the first thing read: pill label = selected model, never the
   highlighted row while browsing.
3. Recently used: last 3 distinct models in this workspace, newest first, excluding the current one.
4. Favourites (★) appear above provider groups; `Alt+M` cycles favourites without opening the popover
   (pattern from Zed / Pi / OpenCode per research 03; key binding is ours).
5. Per-message model: a queued message chip and a sent user bubble carry a `via <model>` meta label
   when it differs from the thread default; "Retry with…" in message actions opens this same popover
   in `scope = Message(id)` mode (header reads "Retry this message with").
6. Mid-run default policy is **NextTurn**: the running turn is never silently re-routed.
7. A model that fails the incompatibility check is **never auto-selected** by cycling; cycling skips it.

## 4. Micro-interactions
- **Hover:** chip bg fades to `chip.hover`; chevron translates +1 px y. No lift/shadow (control row stays flat).
- **Press:** chip scales to 0.97 on pointer-down, springs back on release.
- **Focus ring:** 2 px `focus.ring`, offset 2 px, appears without animation on keyboard focus only.
- **Open:** the popover grows out of the pill: origin = pill's top-centre, scale 0.96 → 1, y +4 px → 0,
  opacity 0 → 1. Variant `morph-list` instead grows the pill surface itself into the list (corner
  radius morphs with size), an idea studied from Arc UI's morph select (research 12; study only).
- **Rows enter:** first 8 rows stagger 18 ms each (cap 144 ms); rows beyond 8 appear with the list.
- **Highlight:** one shared highlight rect glides between rows (spring) instead of per-row hover fades.
- **Detail tray** (footer strip, 44 px): shows context size, speed, cost for the highlighted row;
  numeric values roll (tabular digits) when the highlight moves.
- **Capability badges:** 16 px square icons with 1 px stroke; on search-filter toggle, matching badges
  pulse once (opacity 1 → 0.6 → 1).
- **Select:** the chosen row's name lifts and flies to the pill (shared-element, ≤ 260 ms), pill width
  springs to fit, label rolls **up** if the new model is above the old one in list order, **down** if
  below (direction idea from Arc UI morph select; study only).
- **Close:** reverse of open, faster, no overshoot.
- **PendingNext badge:** "→ Opus" slides out of the pill's right edge (x −6 → 0, opacity) with a dashed
  1 px outline; on run end it merges into the pill (badge collapses, label rolls).
- **Routing (Auto):** three 3 px dots in the mark slot breathe in sequence; on resolve, dots collapse
  to the provider mark (scale 0 → 1) and the secondary label types in.
- **RateLimited countdown:** `m:ss` digits roll each second; at 0 the row's clock glyph morphs to the
  provider mark and the row un-dims.
- **Error (CatalogError):** retry button shakes 2 cycles × 4 px only after a failed retry.

## 5. Timing table

| Transition | Duration | Delay | Easing / spring | Notes |
|---|---|---|---|---|
| Hover bg | 80 ms | 0 | ease-out | — |
| Tooltip | 120 ms fade | 500 ms | ease-out | — |
| Press scale 0.97 ↔ 1 | ~120 ms | 0 | spring stiffness 600, damping 30 | close to button springs reported for Claude desktop [research 08 §1.4, RE] |
| Popover open (scale/opacity/y) | ~200 ms settle | 0 | spring 380 / 32 | no visible overshoot |
| Popover close | 120 ms | 0 | ease-in | — |
| Row stagger | 18 ms × index, cap 8 rows | 40 ms | ease-out 140 ms each | — |
| Highlight glide | ~120 ms | 0 | spring 500 / 40 | — |
| Detail tray digit roll | 320 ms | 0 | cubic-bezier(0.22, 1, 0.36, 1) | shorter cousin of OpenCode AnimatedNumber 560 ms [research 08 §1.5, MIT] |
| Name fly to pill | ≤ 260 ms | 0 | ease-in-out | — |
| Pill width | ~240 ms | 0 | spring 300 / 30 | — |
| Label roll | 220 ms | 0 | cubic-bezier(0.22, 1, 0.36, 1) | direction by list order |
| PendingNext badge in | 180 ms | 0 | ease-out | — |
| Routing dots | 1.2 s loop, 150 ms stagger | 0 | sine | shared motion clock |
| Skeleton rows | 1.6 s sweep | 150 ms grace | linear | — |
| Search filter | 0 ms (instant) | debounce 60 ms | — | list reorders without animation |
| Countdown digit | 200 ms | 0 | ease-out | 1 Hz |
| Frame rate | display rate (GPUI) / rAF (Dioxus); idle = no frames | — | — | — |
| **Reduced motion** | open/close = 100 ms opacity only; no stagger, no fly, no roll (labels swap); routing dots static `…`; highlight jumps | — | linear | — |

## 6. Variants

### Free (6)
| Name | Idea |
|---|---|
| `model-picker/pill-popover` | Baseline: pill + floating popover with search, Auto, Recent, ★, provider groups, detail tray |
| `model-picker/morph-list` | The pill surface itself grows into the list; chosen label flies back and rolls (radius morphs with size) |
| `model-picker/command` | Full-width command-palette sheet over the composer; best for 50+ models; filter chips row |
| `model-picker/glyph` | Terminal look: numbered two-column list `1. sonnet-5   Balanced, 200K` with `›` cursor (TUI surface) |
| `model-picker/compact` | 20 px provider mark only, name in tooltip; for dense IDE panels |
| `model-picker/favorites-segmented` | 2–4 favourite models as a segmented control; "More…" opens the popover |

### Pro (14)
| Name | Idea |
|---|---|
| `model-picker/tradeoff-bars` | Highlighting a row fills speed / quality / cost bars in the tray (bars from app-supplied scores) |
| `model-picker/spec-sheet` | Right-side detail card per model: context, modalities, price per Mtok, knowledge note, all digits rolling |
| `model-picker/router-trace` | Auto row shows a light travelling along a tiny branch diagram to the resolved model |
| `model-picker/fallback-chain` | Shows the app's fallback order; a failed primary breaks red and the light continues to the next |
| `model-picker/provider-orbit` | Provider marks orbit the pill while open; picking one zooms its group in |
| `model-picker/capability-matrix` | Grid view: models × capabilities, toggling a column filters live |
| `model-picker/price-ticker` | Split-flap price per Mtok and first-token time for the highlighted model |
| `model-picker/arena-split` | Pick two models; the next message runs both (A/B) and the pill shows `A|B` |
| `model-picker/liquid-lens` | Glass lens magnifies the highlighted row as it glides |
| `model-picker/card-stack` | Favourites as a fanned card stack; swipe/scroll to flip (mobile-first) |
| `model-picker/dial` | Rotary picker for 3–6 models with detents (desktop notch / voice surfaces) |
| `model-picker/budget-aware` | Rows tint by projected cost of the current draft + context (app supplies estimator) |
| `model-picker/notch` | Desktop island: current model in the notch, expands into a horizontal strip |
| `model-picker/voice-chip` | Voice surface: spoken "use Opus" confirmation chip that morphs from the transcript |

## 7. Edge cases
- **Long names** ("Claude Sonnet 5.5 Thinking 1M"): pill uses `short_name` if supplied, else end-ellipsis
  at `max_pill_width`; popover rows wrap to 2 lines max, then ellipsis with full name in tooltip.
- **Many models** (OpenRouter-style 300+): virtualised list (row height fixed 36 px / 48 px two-line),
  search auto-focused when `models.len() > 12`, provider groups collapsible, group headers sticky.
- **Duplicate names across providers**: always show provider mark + provider name in the row's second line.
- **RTL:** chevron moves to the left; list and tray mirror; digits stay LTR in tabular figures.
- **Tiny sizes (16–20 px row):** `compact` variant only: 16 px mark, no label, badge dots 4 px; the
  warning/limit state becomes a 4 px corner dot (amber/red) plus tooltip text.
- **Dense IDE panels:** popover max height = min(420 px, 60 % of panel); opens below if above-space < 240 px.
- **Touch:** popover becomes a bottom sheet (snap points 50 % / 90 %), rows 44 px, ★ needs long-press.
- **Slow networks:** catalog fetched once and cached with timestamp; stale list usable, "Updated 3 h ago"
  in footer; validation calls never block selection.
- **Very fast agents:** if the run ends within 300 ms of a mid-run selection, `PendingNext` is skipped
  and the switch applies directly (no badge flash).
- **Very slow agents:** `PendingNext` badge persists for the whole run; hovering it explains "Applies to
  your next message".
- **Selected model removed from catalog:** pill keeps the old label with warning dot and "No longer
  available · Choose another"; send control goes `Disabled(reason)` until a choice is made.
- **Key added in another window:** catalog event updates rows live; `NeedsKey` rows animate to available.
- **Attachments added after selection** that the model can't read: pill shows warning dot; send control
  shows the reason; no automatic switch (we never change the model behind the user's back).
- **Auto never resolves** (adapter doesn't report): pill stays "Auto"; no fake model name.

## 8. Accessibility
- Pill: `button`, `aria-haspopup="listbox"`, `aria-expanded`. Label per state, e.g.
  "Model: Claude Sonnet 5. Change model", "Model: Auto, routed to Sonnet 5", "Model: Opus, rate limited,
  resets in 4 minutes", "Model locked by workspace policy".
- Popover: search field is a `combobox` controlling a `listbox`; groups are `group` with labels; rows are
  `option` with `aria-selected` and `aria-disabled` + `aria-describedby` → reason text.
- Capability badges are icons **with text alternatives** ("vision", "tools", "reasoning", "200 thousand
  token context", "fast", "cost: high"); never colour-only.
- Live region (polite): "12 models match", "No matches", "Switched to Opus", "Opus will be used from your
  next message", "Opus available again". Countdown is **not** announced every second; only at 60 s and 0.
- Focus order: search → Auto → Recent → Favourites → groups → footer link. Esc returns focus to the pill.
- Contrast: row text ≥ 4.5:1; dimmed unavailable rows keep reason text ≥ 4.5:1 (only the name dims).
- Reduced motion honoured from the global token.

## 9. Props sketch
```rust
pub struct ModelPicker {
    pub agent: AgentStateRef,              // &AgentState: Idle/Composing = free switch; busy states = mid-run policy
    pub catalog: ModelCatalog,             // groups, models, cached_at, loading/error
    pub selected: ModelSelection,          // Model(id) | Auto { resolved: Option<ModelId> }
    pub scope: PickerScope,                // Thread | Message(MessageId) | Queued(QueueId)
    pub variant: ModelPickerVariant,
    pub favorites: Vec<ModelId>,
    pub recent: Vec<ModelId>,              // app-owned, newest first; picker shows ≤ 3
    pub mid_run: MidRunPolicy,             // NextTurn (default) | ApplyNow | Ask
    pub incompatibility: IncompatPolicy,   // Warn | ConfirmToSelect | Block
    pub draft_needs: DraftNeeds,           // has_images, needs_tools, context_tokens: u32
    pub allow_queued_on_limit: bool,
    pub max_pill_width: Px,                // default 180
    pub shortcuts: PickerShortcuts,        // open: Mod+Shift+M, cycle: Alt+M / Alt+Shift+M (proposed)
    pub labels: ModelPickerLabels,         // i18n + screen-reader strings
    pub motion: MotionLevel,
    pub on_select: Callback<ModelSelection>,
    pub on_request_key: Callback<ProviderId>,
    pub on_toggle_favorite: Callback<ModelId>,
}

pub struct ModelInfo {
    pub id: ModelId, pub provider: ProviderId, pub name: SharedString, pub short_name: Option<SharedString>,
    pub caps: Capabilities,                // vision, tools, reasoning: ReasoningSupport, audio
    pub context_tokens: u32,               // shown as "200K" / "1M"
    pub speed: Option<Tier3>,              // Slow | Medium | Fast (app-supplied)
    pub cost: Option<CostInfo>,            // per-Mtok in/out or Tier3; never invented by Oxyn
    pub status: ModelStatus,
}
pub enum ModelStatus { Available, NeedsKey, RateLimited { until: Instant }, Unavailable { reason: SharedString },
                       Deprecated { sunset: Date }, Locked { reason: SharedString } }

/// Pure derivation: picker phase from agent state + own state.
fn mid_run(agent: &AgentState) -> bool {
    matches!(agent, AgentState::Submitted | AgentState::Thinking { .. } | AgentState::Streaming
        | AgentState::ToolRunning { .. } | AgentState::AwaitingApproval { .. } | AgentState::Steering)
}
```
`Routing` is derived: `selected == Auto { resolved: None }` and `mid_run(agent)`. `Offline` comes from the
app's connection state, not a hidden timer.

## 10. Related components and references
**Related:** effort control ([composer-effort-control.md](composer-effort-control.md)) — shown inside the
popover for reasoning models in the Codex-style combined layout; context meter
([composer-context-meter.md](composer-context-meter.md)) — the incompatibility check reads its token
count; send control (disabled-with-reason when the model can't run); queued-message chips (per-message
model); message actions "Retry with…"; API-key secure form; provider marks (icon set).

**References** (study only unless MIT/Apache):
- Research note 03: Zed provider logos, ★ favourites, cycle-favourites shortcut, "No tools" warning for
  MCP-incompatible models; Conductor model search; Pi `/model`, Ctrl+L, Ctrl+P; Codex combined model +
  effort list (Codex: permissive — licensing.md lists MIT, research note 08 reads Apache-2.0; re-check
  before porting, port with attribution either way); Zed is mixed GPL/Apache → study only.
- Research note 04: Raycast "Auto" (Lightweight / Balanced / Flagship), ChatGPT simplified picker, Manus
  one-line descriptions + Pro badges — closed products, study only.
- [R044] affectUI composer: model pill "Claude Haiku 5.5 ⌄" or "Auto" — paid, study only. affectUI Model
  Picker (hover fills speed/quality/cost bars, effort metric unfolds, tray rolls first-token time and
  price; research note 11) — Pro, study only; our `tradeoff-bars` / `price-ticker` are re-designed from
  this spec, no assets or code.
- [R036] BoardUI composer bottom row with model pill "GPT-5.6 Mini ⌄"; [R095] BoardUI hero model picker
  with "Effort Medium / Faster–Smarter" — free tier MIT (portable with attribution), Pro study only.
- Arc UI morph select (pill grows into list, fly-back, direction-aware roll; research note 12) — paid /
  unclear, study only.
- [R106]/[R133] Beautiful UI Prompt Bar model picker — MIT, port with attribution.
- [R126] coss ui Composer API — MIT, reference design for composition.
- [R078] Replit queue with per-task model and settings — closed, study only.
- Vercel AI Elements Model Selector (research note 01; Apache-2.0, deprecated end of Nov 2026) — portable
  with attribution.
- [R030] run-cost rows naming the model per step — study only.
