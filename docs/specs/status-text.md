**Status:** draft v0.1 (2026-11-11).

# Spec: status-text family (family C — shimmer · verb cycler · reveal · carets · timers)

Family overview for **C. Status-text effects** in [../component-taxonomy.md](../component-taxonomy.md)
(target **100 variants + 12 verb packs**). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Motion token names are the shared ones from
[composer.md](composer.md) §"Motion tokens"; family-C tokens are added in §4 below.

Specs in this family:

| Spec | Component(s) | Variants (free + Pro) |
|---|---|---|
| this file | `status-text` (composite status lines), text-effect engine, verb packs, tense rules | 3 + 9 |
| [text-shimmer.md](text-shimmer.md) | `text-shimmer` | 4 + 18 |
| [verb-cycler.md](verb-cycler.md) | `verb-cycler` | 5 + 17 |
| [text-reveal.md](text-reveal.md) | `text-reveal`, `caret` | 5 + 13, 3 + 5 |
| [elapsed-timer.md](elapsed-timer.md) | `elapsed-timer` | 4 + 14 |
| **Total** | | **24 free + 76 Pro = 100**, plus **12 verb packs** |

## 1. Purpose
Agent apps spend most of a turn *between* user actions. The one line of text near the work —
"Reading auth.rs · 0:42", "Thought for 12s", the answer arriving word by word — is how the user
learns that the agent is **alive, what it is doing, how long it has taken, and when it is done**,
without reading the transcript. Every app we studied has its own version: Codex's model-written
header with a 1-in-4 s shimmer, Claude Code's random verb with a ±1-column glimmer, OpenCode's comet
band, Warp's `Warping... (12s)` resting as `∷ 5s • 0.5 credits`, Antigravity's "Worked for 31s"
groups (all research note 08); "Thinking 13s" rolling to "Thought for 14s" [R021]. Kits ship one to
three variants per pattern (note 02 §3). Family C makes these **one engine with one state model**, so
a shimmer, a verb swap, a token reveal and a timer all read the same `AgentState`, share one clock,
never shift layout, and degrade the same way under reduced motion.

Where it appears:
- **Status line** in the transcript under the last message (the default `status-text/line`).
- **Status tab** above the composer ([composer-status-strip.md](composer-status-strip.md)) — it embeds
  `verb-cycler` + `text-shimmer` + `elapsed-timer`.
- **Thinking header** ("Thinking 8s" → "Thought for 12s") of a collapsible reasoning block (family E).
- **Step labels** in plans and tool rows ("Reading auth.rs" → "Read auth.rs") (family F).
- **Answer body** while streaming (`text-reveal`, `caret`).
- **Run meter** totals (tokens, tok/s, cost) (`elapsed-timer`).

## 2. The text-effect engine (shared by every family-C component)

### 2.1 Pipeline
```
 text (String) ──▶ segment ──▶ shape & measure ──▶ per-glyph timeline ──▶ paint
                   │            │                   │                      │
                   │            │                   │                      └ GPUI: one text run per
                   │            │                   │                        style span; Dioxus: one
                   │            │                   │                        element + CSS vars
                   │            │                   └ f(t_clock, i, n, cfg) → {opacity, blur, dy,
                   │            │                     colour_mix, glyph_override}
                   │            └ advance widths per grapheme, line box, bidi runs,
                   │              tabular-digit cell width
                   └ grapheme clusters (UAX #29); word units; group units ("…", "12s")
```
1. **Segment.** Text is split into **grapheme clusters** (a family emoji or `é` is one unit) and, per
   variant, grouped into *words* (UAX #29 word boundaries; CJK and Thai fall back to per-character
   units, the approach LibreChat took with `Intl.Segmenter`, note 02) or *groups* (a trailing "…" and
   a duration like "12s" animate as one unit, as Warp groups "...", note 08). Rust: the
   `unicode-segmentation` crate (MIT/Apache-2.0 — verify at port time); web: `Intl.Segmenter`.
2. **Shape & measure.** Text is laid out once by the platform shaper (GPUI's text layout; the
   browser on Dioxus). We store per-grapheme x offsets and the line width. **Effects never change
   advance widths**: they modulate colour, opacity, blur and small y offsets only, so nothing reflows
   while an effect runs. (Exception: `typewriter`-class reveals append text and therefore extend the
   line; see [text-reveal.md](text-reveal.md).)
3. **Per-glyph timeline.** Every effect is a pure function
   `fn sample(t: Duration, i: usize, n: usize, cfg: &EffectCfg) -> GlyphStyle`. No per-component
   timers: `t` comes from the shared motion clock (architecture.md). This makes effects deterministic,
   testable with snapshot frames, and phase-locked across the window (Warp keeps spinner, shimmer and
   counter on one clock; OpenCode keeps all spinners in phase — note 08).
4. **Paint.** GPUI: the line is split into the minimum number of style runs (a 3–9 px shimmer band
   becomes ≤ 2 × band-width runs, not one run per glyph when glyphs share a style). Dioxus: shimmer is
   a single element with a masked gradient (compositor-only, OpenCode's opposed-translate technique,
   MIT); reveals use per-word spans that are **unwrapped once settled** (Streamdown removes its spans
   when streaming ends, note 02).

### 2.2 Measured width — no layout shift
Text that changes in place (verb swaps, past-tense rolls, timers) must not push its neighbours.

| Policy | Behaviour | Default for |
|---|---|---|
| `Width::Fit` | The slot animates from the old width to the new one with `spring.snap`; old and new are both measured *before* the swap starts, so the spring knows its target on frame 1. | verb cycler in a free-flowing line |
| `Width::ReserveLongest` | Slot = widest string in the active verb pack (measured once per font/size change) + 0 px. Never moves. | status tab, IDE rows, anything left of other content |
| `Width::ReserveMagnitude` | For numbers: width of the widest value *of the current magnitude* (`0:00`, `00:00`, `0:00:00`). Changes only at a magnitude boundary, animated with `spring.snap`. | timers, counters |
| `Width::Fixed(px)` | App-set. Overflow truncates with a middle ellipsis. | dense dashboards |

Rules (proposed): width animation never exceeds `dur.base` (200 ms); a shrink waits until the
outgoing text has faded (so letters are never clipped mid-fade); a slot never shrinks by less than
4 px (avoids sub-pixel jitter from kerning differences).

### 2.3 Tabular numerals
All digits in family C render with **tabular figures**: the font feature `tnum` where the font has it.
If the font lacks `tnum` (checked once per font at load), each digit is drawn centred in a fixed cell
whose width is the widest of `0`–`9` in that font. Colons, the thin space (U+2009) and the figure dash
(U+2012, Zed's DiffStat convention, note 08) also get fixed cells. Result: `0:09 → 0:10` never wiggles.
Arc's step card shows the same rule ("tabular numerals in a reserved width", note 12).

### 2.4 Clock and frame budget
| Situation | Repaint rate (proposed) | Note |
|---|---|---|
| Shimmer or verb transition active | 30 Hz (33 ms) | Codex repaints at 32 ms while shimmering; MonoCode leases a 33 ms clock (note 08) |
| Reveal (blur / fade per token) active | 60 Hz on desktop, 30 Hz on `Mobile` | bounded by in-flight spans (text-reveal §4) |
| Only a timer is visible | 1 Hz, aligned to the run's whole seconds | Codex drops to 1000 ms (note 08) |
| Window unfocused | effects pause; timers keep their *value* from timestamps but repaint at 0.25 Hz | architecture.md: clock slows when unfocused |
| Element off-screen | no repaint at all; resumes at the same phase | Arc pauses its sweep off-view (note 12) |

### 2.5 Tokens read by every family-C component
`text.status.fg`, `text.status.base` (derived, see [text-shimmer.md](text-shimmer.md) §contrast floor),
`text.status.crest`, `tone.warning`, `tone.danger`, `tone.success`, `font.ui`, `font.mono`,
`font.size.status` (13 px desktop / 15 px mobile / 1 cell terminal — proposed), `motion.reduced`.

## 3. AgentState mapping (what the status line shows)

| `AgentState` | Line text (neutral pack) | Shimmer | Verb cycling | Timer | Reveal / caret |
|---|---|---|---|---|---|
| `Idle`, `Composing` | hidden | — | — | — | — |
| `Submitted` | "Sending" (≤ 500 ms; architecture.md says pulse, not spinner) | off | off | not shown | — |
| `Thinking { verb }` | "Thinking" or a pack verb; model-written header pins over the pack (Codex pattern, note 08) | on | on (dwell rules in verb-cycler.md) | runs | — |
| `Streaming` | "Writing" (or hidden when the answer itself is visible on screen — proposed) | on, calm cadence | off (pinned) | runs | reveal active, caret shown |
| `ToolRunning { tool, verb }` | tool verb + target: "Reading auth.rs", "Running cargo test" | on | **pinned** to the tool verb | runs | — |
| `AwaitingApproval { request }` | "Waiting for you" | **off** (static text: the human is the bottleneck, not the model — Zed Sand idea, note 08) | off | **paused**, shows ⏸ | caret hidden |
| `Queued { count }` | current verb + " · 2 queued" suffix | unchanged | unchanged | runs | — |
| `Steering` | one-shot "Adjusting course" for ≥ 900 ms, then back | one sweep | paused | runs | — |
| `Stalled { since }` | verb freezes; after 15 s "Still working…"; after `stall_hard` "No response for 45 s" (thresholds from composer-status-strip.md) | slows to 0.5× then stops; tone ramps toward `tone.warning` | stops | runs | caret stops blinking, dims |
| `Done { elapsed }` | past tense: "Worked for 1m 12s" / "Thought for 12s" | finishes current sweep, then off | final roll to past tense | frozen into summary | caret fades out |
| `Error { kind }` | "Stopped · {kind}" e.g. "Stopped · tool error" | off | off | frozen | caret removed |
| `Cancelled` | "Interrupted after 42s" | off | off | frozen | caret removed |

The mapping is a pure function in `oxyn-core`:
`fn status_line(state: &AgentState, pack: &VerbPack, run: &RunClock) -> StatusLine`.

## 4. Family-C motion tokens (proposed; added to `oxyn-core` motion presets)
Shared tokens (`dur.instant`, `dur.fast`, `dur.base`, `dur.travel`, `ease.out`, `ease.in`,
`ease.in-out`, `ease.firm`, `spring.snap`, `spring.soft`, `stagger`) keep their composer.md values.

| Token | Value (proposed) | Used for |
|---|---|---|
| `dur.shimmer.sweep` | 1400 ms | one band crossing a 120 px verb |
| `dur.shimmer.rest` | 800 ms | gap between sweeps (default variant) |
| `dur.shimmer.delay` | 600 ms | wait after new text before the first sweep (Codex value, note 08) |
| `dur.verb.dwell` | 2800 ms (min; grows with length) | time a cycled verb stays |
| `dur.verb.swap` | 180 ms | default fade/slide swap (matches the status strip's verb swap) |
| `dur.reveal.token` | 160 ms | per-token fade / blur-in |
| `dur.digit.roll` | 280 ms | one digit rolling (must stay < 1 s tick) |
| `dur.tense.roll` | 240 ms | present → past roll at completion |
| `ease.coast` | quadratic ramps on first/last 20 %, linear between | shimmer travel (OpenCode `coast()`, MIT) |
| `ease.linear` | linear | constant-speed sweeps, marquee |
| `stagger.glyph` | 25 ms per glyph, cap 300 ms total | scramble, split-flap, reveal |
| **Reduced motion** | shimmer off (plain text at full `text.status.fg`); verb swaps → 80 ms crossfade, dwell ×2; reveals → whole block appears with an 80 ms fade; digits change without rolling; tense change → 80 ms crossfade | — |

## 5. Verb pack system

### 5.1 What a pack is
A pack is **content, not motion**: a TOML file in `oxyn-core` (`verb_packs/<id>.toml`), MIT, written
in-house (licensing.md rule 4). Any variant can use any pack.
```toml
id = "coder"            # stable
name = "Coder"
locale = "en"           # packs are written per locale, never machine-translated
order = "shuffle"       # shuffle | sequential
reasoning_done = "Thought for {d}"
turn_done = "Worked for {d}"

[[cycle]]               # 12 per pack (proposed fixed size, keeps ReserveLongest predictable)
present = "Tracing calls"
past = "Traced calls"

[activity]              # overrides of the neutral activity table (5.3); keyed by ActivityKind
search = { present = "Grepping", past = "Grepped" }
```
Rules:
1. **Both tenses are stored**, never generated. English irregulars (read → read, find → found,
   write → wrote, forecast → forecast) and every other language make conjugation by rule unsafe.
2. **Max length 24 characters** per entry in Latin scripts, 12 in CJK (proposed). Longer strings are
   rejected by `oxyn-codegen` with the measured width in the message.
3. No trailing "…" in the data: the ellipsis is a variant decoration (`verb-cycler/ellipsis`) so it
   can be grouped, animated and localised (U+2026 in most locales, see 5.6).
4. **Original words only.** Packs are checked in CI against a deny-list built from third-party verb
   lists we have studied (Claude Code's list in note 08 Appendix A is proprietary). The 2026-11-11
   draft below has **zero** overlap with it except the generic state labels *Thinking* (reasoning
   label, also used by Cline and Vercel AI Elements) and *Worked* ("Worked for 12s" is already in
   architecture.md and used by Codex/Antigravity) — both dictionary-level product conventions, kept
   on purpose.
5. Apps can `append` or `replace` a pack at runtime (`VerbPackOverride`) — a pattern Claude Code
   exposes via `spinnerVerbs` (note 08); the *idea* is common, our schema is our own.

### 5.2 The 12 packs (proposed, v0.1 copy — 12 cycle verbs each)
| # | Pack id | Voice / for | Cycle verbs (present → past stored alongside) | Turn done |
|---|---|---|---|---|
| 1 | `neutral` (default) | plain, calm, product-safe | Planning, Reading, Checking, Comparing, Organizing, Reviewing, Summarizing, Preparing, Outlining, Weighing options, Looking closer, Putting it together | Worked for {d} |
| 2 | `playful` | warm, a little silly; consumer chat | Untangling, Rummaging, Sleuthing, Juggling, Spitballing, Unboxing, Wayfinding, Knitting ideas, Daydreaming, Fizzing, Polishing, Connecting dots | Juggled for {d} |
| 3 | `coder` | coding agents, IDE, terminal | Tracing calls, Grepping, Reading the diff, Linting, Refactoring, Compiling, Profiling, Bisecting, Patching, Type-checking, Resolving imports, Stepping through | Worked for {d} |
| 4 | `finance` | sober, precise | Reconciling, Auditing, Tallying, Forecasting, Matching entries, Categorizing, Checking balances, Netting out, Reviewing ledgers, Flagging outliers, Pricing, Totting up | Reconciled for {d} |
| 5 | `crm` | sales and CRM agents | Enriching records, Qualifying leads, Deduplicating, Segmenting, Drafting outreach, Logging activity, Scoring leads, Syncing contacts, Routing, Checking history, Updating the pipeline, Finding contacts | Worked for {d} |
| 6 | `creative` | design, writing, image | Storyboarding, Moodboarding, Framing, Layering, Riffing, Drafting, Shaping, Iterating, Blocking out, Picking colours, Trying a variation, Refining edges | Shaped for {d} |
| 7 | `video` | video and media | Rendering, Encoding, Trimming, Splicing, Keyframing, Grading, Captioning, Upscaling, Stabilizing, Syncing audio, Cutting to beat, Exporting | Rendered for {d} |
| 8 | `research` | research and reports | Gathering sources, Cross-checking, Skimming, Annotating, Reading papers, Weighing evidence, Fact-checking, Following citations, Comparing claims, Taking notes, Narrowing down, Drafting findings | Researched for {d} |
| 9 | `voice` | spoken-sounding, short | Listening, Hearing you out, Catching that, Getting that down, Lining up a reply, Replying, Speaking, Tuning in, Checking that, Mapping a reply, Finding the words, Picking it up | Talked for {d} |
| 10 | `terse` | 16–20 px rows, terminals; bare stems | Busy, Plan, Read, Find, Edit, Run, Test, Fetch, Wait, Sync, Check, Write (past: Done, Planned, Read, Found, Edited, Ran, Tested, Fetched, Waited, Synced, Checked, Wrote) | Done · {d} |
| 11 | `multilingual` | originals per locale (5.6), not translations | es · fr · de · ja · zh-Hans · ar starter set | per locale |
| 12 | `oxyn` (house) | metalwork and light; the brand voice | Kindling, Sparking, Oxidizing, Aligning, Fusing, Distilling, Annealing, Smelting, Refining, Alloying, Burnishing, Cooling the metal | Refined for {d} |

Packs 2, 4, 6–9 and 12 ship **Pro**; `neutral`, `coder`, `terse`, `crm` and `multilingual` ship free
(proposed split: the free tier must cover every surface; Pro adds voice and vertical flavour).
`reasoning_done` is "Thought for {d}" in every English pack — it is a *label*, not flavour, and users
scan for it.

### 5.3 Activity table (pinned verbs from agent events; neutral pack)
When the agent emits a tool event, the cycler **pins** the activity verb instead of cycling
(verb-cycler.md §2). `ActivityKind` comes from the adapter (tool name → kind map, overridable).

| `ActivityKind` | Present | Past | Target appended? |
|---|---|---|---|
| `Think` | Thinking | Thought for {d} | no |
| `Plan` | Planning | Planned | no |
| `Read` | Reading | Read | file / URL basename |
| `Search` | Searching | Searched | query in quotes, ≤ 32 chars |
| `Edit` | Editing | Edited | file basename + `+12 −3` after done |
| `Write` | Writing | Wrote | file basename |
| `Run` | Running | Ran | command head (first token + 1 arg) |
| `Test` | Testing | Tested | suite / "3 failed" after done |
| `Browse` | Browsing | Browsed | domain |
| `Fetch` | Fetching | Fetched | domain |
| `Produce` (media) | Producing | Produced | asset kind ("image", "clip") |
| `Listen` / `Speak` | Listening / Speaking | Listened / Spoke | — |
| `WaitUser` | Waiting for you | Waited {d} for you | — |
| `Retry` | Retrying | Retried | attempt "2/3" |
| `Generic` | Working | Worked for {d} | — |

Multiple simultaneous tools: "Running 3 tools" (count, never a list — Antigravity fixed flicker
between quick tool names, note 08); a tool shorter than `pin_min` (600 ms, verb-cycler.md) never
reaches the screen.

### 5.4 Present → past tense rules
| Rule | Example | Notes |
|---|---|---|
| Reasoning completes | "Thinking 8s" → **"Thought for 12s"** | the timer value is frozen *at the event*, not at the end of the roll [R021]; Vercel AI Elements uses "Thought for N seconds" (note 02) |
| Turn completes | cycling verb → **"Worked for 1m 12s"** (pack `turn_done`) | architecture.md `Done { elapsed }` |
| Step completes | "Reading auth.rs" → **"Read auth.rs"** | present → past, target kept; per-step duration appended in detailed variants: "Read auth.rs · 2s" |
| Step fails | "Running cargo test" → **"Ran cargo test · 3 failed"** in `tone.danger` | failure is stated in words, never only colour |
| Cancelled | → **"Interrupted after 42s"** | not a past tense of the verb — the user stopped it |
| Error | → **"Stopped · {kind}"** | kind: tool error, model error, timeout, offline |
| Under 1 s | no past-tense line at all | matches composer-status-strip.md "don't show Worked for 0s" |
| Waited on user | optional suffix **"· waited 1m 04s for you"** | elapsed excludes paused time (elapsed-timer.md) |

Duration `{d}` format (shared with [elapsed-timer.md](elapsed-timer.md)): `12s`, `1m 12s`, `1h 02m`,
`1d 3h` — seconds dropped from 1 h up in summaries (proposed). The roll itself is `dur.tense.roll`
(240 ms): the verb part rolls, the shared target text stays put (morph of the common suffix).

### 5.5 Pairing with loaders
Each `ActivityKind` also names a default loader in family B (verb↔loader pairing; Circle Loaders is
the only kit doing this and only in demo copy — note 02). `status-text/semantic-verb` (Pro) uses it.

### 5.6 Localisation
- **Packs are authored per locale**, not translated word-for-word; a locale without its own pack falls
  back to the `neutral` pack of that locale, then to English `neutral`.
- **Grammar lives in templates**, not concatenation: `turn_done = "{d} gearbeitet"` (de) puts the
  duration first. Durations are formatted with the platform's unit formatting for the locale
  (ICU/CLDR-style unit patterns); plural rules come from CLDR plural categories, never `if n == 1`.
- **Ellipsis:** U+2026 by default; locales may override (e.g. Chinese often uses a doubled ellipsis —
  needs native review). Never three ASCII dots, so grouping works.
- **Starter `multilingual` pack** (present / past, *needs native-speaker review before release*):

| Locale | Think | Search | Read | Write | Turn done |
|---|---|---|---|---|---|
| es | Pensando / Pensó | Buscando / Buscó | Leyendo / Leyó | Escribiendo / Escribió | Trabajó durante {d} |
| fr | Réflexion / A réfléchi | Recherche / A cherché | Lecture / A lu | Rédaction / A rédigé | A travaillé pendant {d} |
| de | Denkt nach / Hat nachgedacht | Sucht / Hat gesucht | Liest / Hat gelesen | Schreibt / Hat geschrieben | {d} gearbeitet |
| ja | 考え中 / 考えました | 検索中 / 検索しました | 読み込み中 / 読み込みました | 執筆中 / 書きました | {d}作業しました |
| zh-Hans | 思考中 / 已思考 | 搜索中 / 已搜索 | 阅读中 / 已阅读 | 撰写中 / 已撰写 | 已工作 {d} |
| ar | يفكّر / فكّر | يبحث / بحث | يقرأ / قرأ | يكتب / كتب | عمل لمدة {d} |

- **RTL** locales: the whole line mirrors; numbers and durations sit in a bidi isolate (FSI…PDI) so
  "12s" stays LTR; shimmer and reveal direction follow the reading direction (text-shimmer.md).
- **CJK:** no word spaces, so `Word`-unit effects fall back to per-character; dwell times use
  characters × 1.6 as the length measure (CJK characters carry more meaning per glyph — proposed).

## 6. State machine (status line)
```
            send             first event                    tool start / end
  Hidden ─────────▶ Sending ─────────────▶ Active{verb} ◀─────────────────────▶ Pinned{activity}
    ▲                (≤500 ms)              │  ▲  │ approval            ▲              │
    │                                       │  │  ▼                     │              │ approval
    │                                       │  └ Waiting (timer ⏸) ◀────┼──────────────┘
    │                                       │ no event ≥ stall_soft     │ event arrives
    │                                       ▼                           │
    │                                    Stalled{level} ────────────────┘
    │
    │◀─ fade after done_hold ─ Summary("Worked for 1m 12s") ◀── Done ──────── (from any active state)
    │◀─ (stays in transcript) ─ Interrupted("after 42s") ◀───── Cancelled ───┤
    │◀─ (stays) ─────────────── Failed("Stopped · tool error") ◀── Error ────┘
  Reduced motion: same states and text; no sweeps, no rolls (80 ms crossfades).
```

| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Hidden` | nothing | — | — | — |
| `Sending` | "Sending" (no shimmer) | — | — | Esc stops (owned by send control) |
| `Active{verb}` | cycling verb, shimmer, timer | click → scroll to live step | hover → full verb + elapsed tooltip | focusable only in `status-tab` hosts |
| `Pinned{activity}` | tool verb + target, shimmer, timer | click → expand tool row | — | — |
| `Waiting` | "Waiting for you", static, ⏸ timer | click → focus approval card | — | proposed ⌘/Ctrl+Shift+A |
| `Stalled{level}` | frozen verb, warning tone, "Still working…" / "No response for 45 s" | Retry (hard level, owned by status strip) | — | — |
| `Summary` | "Worked for 1m 12s" (+ tokens/cost in detailed) | click → expand run summary | copy | Enter expands when focused |
| `Interrupted` / `Failed` | marker text, danger tone for failed | retry (failed) | details | Enter |

## 7. Micro-interactions (family-wide rules)
- **One thing moves at a time per line.** If a verb swap and a shimmer coincide, the shimmer waits for
  the swap to finish and restarts after `dur.shimmer.delay` (Codex resets its shimmer clock on each new
  header, note 08).
- **Completion has one beat.** On `Done`: shimmer finishes or fades (≤ 400 ms) → verb rolls to past
  tense (240 ms) → timer freezes into the summary. No confetti in free variants.
- **Failure has words.** Tone change (200 ms ease) + "Stopped · tool error"; never a shake on text
  (shakes belong to buttons).
- **Hover** on any live status text: tooltip with full verb, target path, elapsed with seconds, and
  the active pack's name in dev builds only.
- **Numbers roll, words swap.** Digits use `dur.digit.roll`; words use the cycler transition.

## 8. Timing table (family defaults)
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Line enter (Sending) | 160 ms, y +4 → 0 | 0 | `ease.out` | — | 60 Hz |
| Shimmer sweep / rest | 1400 / 800 ms | 600 ms after text change | `ease.coast` | — | 30 Hz |
| Verb dwell | ≥ 2800 ms | — | — | — | — |
| Verb swap (fade) | 180 ms | 0 | `ease.out` | — | 30 Hz |
| Width change | ≤ 200 ms | after outgoing fade | `spring.snap` | — | 60 Hz |
| Token reveal | 160 ms | 0 | `ease.out` | 25 ms/word cap 300 ms | 60 Hz |
| Digit roll | 280 ms | 0 | `ease.out` | 30 ms right→left | 30 Hz |
| Stall tone ramp | 2000 ms | stall_soft | linear in OKLab | — | 30 Hz |
| Present → past roll | 240 ms | after shimmer ends | `ease.out` | — | 60 Hz |
| Summary hold → fade (status tab host) | 4000 ms → 300 ms | — | `ease.in` | — | — |
| **Reduced motion** | 80 ms crossfades; no shimmer, no rolls, no blur; dwell ×2; stall shown as instant tone + text | — | linear | none | 1 Hz idle |

## 9. Variants (composite `status-text/*` lines; component variants live in each spec)

### Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `status-text/line` | Default transcript line: 12 px loader + `verb-cycler/fade` + `text-shimmer/sweep` + `elapsed-timer/units` → "Worked for 1m 12s" | chat, desktop, mobile |
| `status-text/inline-ide` | 20 px row, `terse` pack, `text-shimmer/calm`, timer right-aligned in a reserved slot | ide |
| `status-text/terminal-row` | `• Working (12s · esc to interrupt)` shape with `text-shimmer/ascii`; `└` detail line (Codex layout, Apache-2.0/MIT, note 08) | terminal |

### Pro (9)
| Name | One-line idea |
|---|---|
| `status-text/thought-header` | Collapsible reasoning header: "Thinking 8s" ticking → rolls to "Thought for 12s", auto-collapses when the answer starts [R021] |
| `status-text/step-label` | Plan/tool step label with present → past crossfade and a per-step duration chip (step card pattern, note 12) |
| `status-text/stall-ramp` | Verb and loader cool toward `tone.warning` after stall_soft; text escalates "Still working…" → "No response for 45 s" |
| `status-text/model-header` | Shows the model-written activity header ("Investigating rendering code"), pack verbs only as fallback (Codex pattern) |
| `status-text/dual-line` | Verb line + dim `└` detail line (current file, last tool output) max 1 line, middle-truncated |
| `status-text/semantic-verb` | Activity kind picks both verb and paired loader (5.5) |
| `status-text/multi-agent` | "3 agents · 1 waiting" with per-agent verbs in a hover list (lanes idea [R061]) |
| `status-text/run-meter` | Line + tokens · tok/s · cost, all tabular, rolling [R030] |
| `status-text/seasonal` | Opt-in seasonal verb pack + loader swap with a date window (Antigravity's seasonal theme idea, note 08) |

### Family catalog (all 100)
| Component | Free | Pro |
|---|---|---|
| `status-text` | line, inline-ide, terminal-row | thought-header, step-label, stall-ramp, model-header, dual-line, semantic-verb, multi-agent, run-meter, seasonal |
| `text-shimmer` | sweep, calm, pulse, ascii | comet, dual-band, spectral, chrome, aurora, glow-halo, yoyo, wave, hairline, mask-wipe, rainbow, ink-bleed, stall-ember, pointer-follow, word-pop, dither, weight-wave, outline |
| `verb-cycler` | fade, slide, blur-swap, ellipsis, static | scramble, scramble-braille, scramble-kana, decrypt-hex, split-flap, odometer, flip-3d, morph, typewriter-replace, kinetic-push, glitch-rgb, drum, stack-peek, ticker-tape, letter-shuffle, wave-in, cascade-drop |
| `text-reveal` | fade-token, word-blur, typewriter, line-commit, instant | trailing-fade, gradient-edge, drop-in, pull-up, slide-in, gradual-spacing, highlight-trail, ink-settle, cross-blur, scramble-in, dither-in, glyph-rise, focus-pull |
| `caret` | bar, block, underscore | glow, pulse-dot, comet-trail, ghost-trail, gradient |
| `elapsed-timer` | units, clock, summary, meter | odometer, split-flap, flip-card, seven-segment, dot-matrix, nixie, ring-seconds, cost-ticker, rate-sparkline, budget-countdown, laps, breakdown, eta, waited-chip |
| **Counts** | **24** | **76** |

Naming note: [../variants.md](../variants.md) uses `status-text/shimmer-comet` as an example; under this
spec that effect is `text-shimmer/comet`, and `status-text/*` is reserved for composite lines. Proposed;
variants.md is not edited here.

## 10. Edge cases (family-wide)
- **RTL:** line mirrors; numbers in bidi isolates; sweep/reveal direction = reading direction; mixed
  "Reading ملف.rs" keeps each run's own direction (shaped, not character-reversed).
- **CJK / complex scripts:** grapheme units; no word blur; per-character dwell weighting (5.6);
  Devanagari conjuncts and Thai clusters stay intact because units are graphemes.
- **Emoji / ZWJ:** a ZWJ sequence is one unit; shimmer colour does not apply to colour emoji (they
  get opacity only) — proposed.
- **Very long verbs / targets:** verb ≤ 24 chars by schema; targets middle-truncate ("src/…/auth.rs");
  line max 60 % of host width (proposed); full text in the tooltip.
- **Narrow screens (< 320 px) and tiny rows (16–20 px):** drop the timer's seconds after 1 h, then the
  target, then switch to the `terse` pack; never wrap to two lines.
- **0 s / very fast turns (< 1 s):** no verb cycle, no past-tense line, no shimmer (shimmer starts only
  after `dur.shimmer.delay` anyway).
- **Very slow turns (hours):** cycle dwell grows ×1.5 after 5 min so the line calms down (proposed);
  timer switches to `1h 02m`.
- **Font change / zoom:** re-measure all reserved widths on font or scale change; never animate that
  re-measure.
- **Terminal surface:** no blur, no sub-cell motion; effects fall back to colour tiers (DIM / normal /
  BOLD, Codex legacy shimmer, note 08) and frame-based glyph swaps.
- **Background tab / unfocused:** effects pause; on focus the timer jumps to the true value without
  rolling through skipped seconds.
- **Many status lines on screen** (sub-agents): all share one clock and phase, capped at 8 animated
  lines; the rest render static (proposed).

## 11. Accessibility (family-wide)
- Status lines are `role=status` (polite). The **visible text is the accessible text**; decorative
  glyphs (scramble noise, carets, split-flap halves) are `aria-hidden`.
- **Throttling:** verb changes announced at most once per 10 s (same as the status strip); pinned tool
  verbs announced only if they last ≥ 2 s; the timer is **never** announced per second — opt-in
  milestone announcements at 30 s, 1 min, then every 2 min (proposed; Antigravity announces thinking
  duration to screen readers, note 08). Done/Interrupted/Failed are announced once; Failed and
  Waiting-for-you are assertive.
- **Streaming answers are never announced token by token** (text-reveal.md §8).
- **Contrast:** every frame of every effect keeps ≥ 4.5:1 for status text (text-shimmer.md contrast
  floor); tone changes always come with words.
- **Reduced motion** is the global token (architecture.md); every variant documents its fallback, and
  meaning never depends on motion.
- **Focus:** status text is not in the tab order unless the host makes it actionable (status tab,
  thought header). Then it is one stop with a label like "Agent working: Reading auth.rs, 42 seconds".

## 12. Props sketch (shared types)
```rust
pub struct StatusText {
    pub state: StatusLine,               // derived: status_line(&AgentState, &VerbPack, &RunClock)
    pub variant: StatusTextVariant,      // Line | InlineIde | TerminalRow | Pro(..)
    pub pack: VerbPackId,                // "neutral" default
    pub pack_override: Option<VerbPackOverride>, // Append(Vec<VerbPair>) | Replace(Vec<VerbPair>)
    pub shimmer: Option<ShimmerVariant>,
    pub cycler: CyclerVariant,
    pub timer: Option<TimerVariant>,
    pub width: WidthPolicy,              // Fit | ReserveLongest | ReserveMagnitude | Fixed(Pixels)
    pub surface: Surface,
    pub motion: MotionLevel,             // None | Subtle | Expressive (reduced-motion token wins)
    pub labels: StatusLabels,            // localised accessible strings
}

pub struct VerbPair { pub present: SharedString, pub past: SharedString }

pub enum StatusLine {
    Hidden, Sending,
    Active   { verb: VerbPair, since: Instant },
    Pinned   { activity: ActivityKind, verb: VerbPair, target: Option<SharedString>, since: Instant },
    Waiting  { paused_at: Duration },
    Stalled  { level: StallLevel, verb: VerbPair, since: Instant },
    Summary  { text: SharedString, elapsed: Duration, tokens: Option<u64>, cost: Option<Money> },
    Interrupted { after: Duration },
    Failed   { kind: ErrorKind },
}

/// Every family-C effect implements this; `t` comes from the shared motion clock.
pub trait GlyphEffect {
    fn sample(&self, t: Duration, unit: usize, units: usize, line: &MeasuredLine) -> GlyphStyle;
    fn reduced(&self) -> GlyphStyle; // static fallback, required
    fn is_idle(&self, t: Duration) -> bool; // lets the clock drop to 1 Hz
}
```

## 13. Related components and references
Related: [composer-status-strip.md](composer-status-strip.md) (hosts the tab version) · loaders
(family B, shared clock, verb↔loader pairing) · thinking block and streaming markdown (family E) · tool
rows and plan steps (family F) · run meter (family F) · voice captions (family K).

| Ref | What we take | Licence note |
|---|---|---|
| Codex CLI (note 08) | model-written header, shimmer cadence, 32 ms / 1 s repaint, elapsed format, line-commit streaming | note 08 says **Apache-2.0**, licensing.md lists **MIT** — both allow porting with attribution; re-check before porting |
| OpenCode / Kilo (note 08) | comet shimmer, `coast` easing, text-shimmer state machine, AnimatedNumber odometer, phase sync | MIT — may port with attribution |
| Cline (note 08) | typewriter then shimmer, 500 ms grace | Apache-2.0 — may port with attribution |
| MonoCode (note 08) | 33 ms leased clock, Streamdown + `remend` repair idea | MIT — may port with attribution |
| Claude Code (note 08) | verb + past-tense completion idea, stall ramp, count-up | Proprietary — reference only; **verb list not used** |
| Warp (note 08) | phase-locked clock, grouped "...", rest line with credits | AGPL — study only |
| Zed (note 08) | Sand for awaiting, thin space + figure dash | mixed GPL/Apache — study only |
| Pitus [R021] | "Thinking 13s" → "Thought for 14s" | Study only |
| Cost waterfall [R030] | rolling cost total | Study only |
| Agent lanes [R061] | multi-agent summary | Study only |
| Arc UI (note 12) | tabular numerals in a reserved width; past-tense crossfade | Paid/unclear — study only |
| affectUI (note 11) | thinking loader that locks to a check | Paid — study only |
| Circle Loaders, Vercel AI Elements, Streamdown, LibreChat (note 02) | verb↔loader pairing; "Thought for N seconds"; span unwrap; `Intl.Segmenter` for CJK | licences not recorded in our notes — study only until checked |

## 14. Open questions / gaps
- GPUI accessibility (live regions) support needs verifying before we promise identical semantics on
  desktop; this spec defines what both runtimes must expose.
- Locale packs beyond the starter table need native-speaker authors.
- Codex licence discrepancy (MIT vs Apache-2.0) must be resolved in licensing.md before the first port.
