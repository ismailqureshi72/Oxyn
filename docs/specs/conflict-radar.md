**Status:** built v0.1 (2026-11-15, plan #31).

# Spec: Conflict radar (`conflict-radar`)

Family H, "Checkpoints, diff & git" (overview in [checkpoints-git.md](checkpoints-git.md)). Closes the
affectUI **Conflict Radar** (Pro, Components · Agents). Behaviour-only rebuild from its public docs
page; nothing is copied. Dioxus `ConflictRadar` / GPUI `ConflictRadarEl` over
`oxyn_core::kit::merge::{Radar, RadarAgent, RadarEdit, RadarFile}`. Styles: `merge.css`, classes
`oxyn-crd-*`. Sibling: [merge-conflict.md](merge-conflict.md) (the line-level editor a settled
conflict can open).

## 1. Purpose
Several coding agents work on their own branches of the same repo at once. Before anything lands,
the user needs to see **where they overlap**, pick **one agent's way** for each file where they
clash, and merge only when every clash is settled and every agent has stopped writing. When the
agents simply try the same task and the best one wins, use a best-of-N view instead; when the
timeline matters, use agent lanes; for one agent's changes, use a change summary.

## 2. Anatomy
```
┌ ⑂ Parallel work   5 files · 2 conflicts · 0 settled        2 conflicts · 1 working  [⑂ Merge] ┐
├ File                      │ A Atlas          │ B Birch          │ C Cove ●                       │ header row
├ src/auth/session.rs ⚠ ▾   │  ▬▬▬▬━━━━━━━━━━━━━━━━━━━━▬▬▬▬▬    │                                 │ clash line
│  ◉ A Atlas  Refresh tokens in middleware before every request.                                  │ ways
│  ○ B Birch  Refresh tokens on demand when a call returns 401.                                   │ (radiogroup)
├ src/api/routes.rs         │  ▬                │                  │            ▬▬▬▬ (sheen)        │
├ Suggested merge order  Cove → Atlas → Birch                                       (Oxyn plus)   │
└ Point at a bar or an agent for details.                                       (polite status)   ┘
```
A **bar** sits in each cell an agent touched, placed and sized by the changed line range against the
longest file. A **conflict** is a file where two or more agents' ranges overlap (`RadarFile::clashes`);
a line joins the clashing agents' columns.

## 3. State machine
**Radar:** `Working{n}` (some agent running) → `Settling{open}` (all done, conflicts left) →
`Ready` (no open conflict, no agent running) → `Merged` (Merge pressed). Conflicts can be settled
while agents still work; Merge stays disabled with the reason ("2 conflicts · 1 working",
"1 agent still working", "1 conflict to settle").

**File row:** `Clean` (no overlap) · `Conflict{agents}` → (pick) → `Settled{agent}`; the pick can change
any time before Merge. `Collapsed` ⇄ `Expanded` (ways shown) on click / Enter / Space.

**Cell:** `Empty` · `Edited{from, to}` · `Writing` (edit still active: sheen) · `Kept` · `NotKept`
(faded to 28 %) · `Open` (in an unsettled clash: amber ring).

**Agent column:** `Working` (pulsing dot) · `Done`. Hover / focus shows its numbers in the status line.

## 4. States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| Working | pulsing dot on the agent, sheen on its active bars, Merge disabled with reason | — | settle conflicts | Tab to agents reads numbers |
| Conflict row | amber "Conflict" chip, amber ring on clashing bars, glowing clash line | open ways | — | Enter / Space toggles ways |
| Ways open | radio list: mark, name, one-sentence way | keep a way | close | ↑/↓/←/→ move and keep, Enter / Space keep |
| Settled row | "✓ <agent>" chip, the other agents' bars fade, line turns resolved and stops glowing | change pick | — | as above |
| Ready | Merge enabled (solid) | Merge | — | Tab, Enter |
| Merged | "✓ Merged" replaces the button; status reads "Merged every branch" | — | — | — |
| No conflicts (filter) | "No conflicts: every agent changed different lines." | — | — | — |

## 5. Micro-interactions
- Bars grow in from the left (`scaleX 0 → 1`) when the row appears; a sheen sweeps an active bar
  while the agent is still writing.
- A conflict draws a line between the clashing agents' columns with a glow running along it until
  the file is settled; then the line turns `merge.resolved` at 50 % and stops.
- Pointing at a bar shows "Birch changed lines 30 to 64, kept" in the status line; pointing at or
  focusing an agent shows "Atlas: 3 files · 1 open conflict · 0 kept".
- Opening a row folds the ways in under it (fade + 3 px rise); the chevron turns 180°.
- Keeping a way fills its radio (4 px ring), the chip flips Conflict → ✓ name with a small pop.
- Merge morphs into "✓ Merged" with a pop.

## 6. Timing table
| Transition | Duration | Easing | Notes |
|---|---|---|---|
| Bar grow-in | 500 ms | `cubic-bezier(.2,.8,.2,1)` | on mount |
| Active sheen | 1600 ms loop | linear | only while `active` |
| Clash glow | 1800 ms loop | linear | stops when settled |
| Working dot | 1400 ms loop | ease-in-out | opacity 1 → .25 |
| Ways open | 200 ms | `cubic-bezier(.2,.8,.2,1)` | fade + 3 px |
| Chevron turn | 180 ms | ease | — |
| Chip / Merged pop | 160–200 ms | `cubic-bezier(.2,.8,.2,1)` | scale .9 → 1 |
| **Reduced motion** | — | — | no grow, sheen, glow or pulse; static bars, a static line and text carry the state |

## 7. Variants
| Variant | What differs |
|---|---|
| `conflict-radar/radar` | default: one agent still writing |
| `conflict-radar/settled` | everyone done; pick ways, then Merge |
| `conflict-radar/heat` (Oxyn plus) | clashing cells tinted by overlap size (`overlap_lines`, 20 lines = full) |
| `conflict-radar/merge-order` (Oxyn plus) | "Suggested merge order": agents with no open clashes first, then by most lines changed (`Radar::merge_order`) |
| `conflict-radar/conflicts-only` (Oxyn plus) | only the rows that clash; four agents |
| `conflict-radar/collapsed` | ways stay folded until a file is opened |

## 8. Edge cases
- **Three-way clash:** every clashing agent is a way; the line spans the leftmost to rightmost clashing column.
- **Agent not in the clash:** its bar in that row stays at full strength ("kept") and it is not offered as a way; `resolve` ignores it.
- **Controlled picks:** pass `resolved`; the component then only reports `on_resolve` and never stores a pick.
- **Many agents:** columns share the width evenly; below 440 px names drop to monograms.
- **Long paths:** folder and name wrap; below 440 px folders drop.
- **Zero-line edits:** a bar is at least 6 px wide.
- **No conflicts:** Merge enables as soon as every agent is done.
- **Touch:** rows and ways are full-width targets ≥ 34 px tall.
- **RTL:** chrome mirrors; bars still read by line number from inline-start.

## 9. Accessibility
- The grid is `role=table`: agents are `columnheader`s, files `rowheader`s, every cell has a label
  ("Birch changed lines 30 to 64, still writing, conflict").
- Conflicts and settled files are named in text, not only by colour.
- The ways are a `radiogroup` with roving `tabindex`; arrows move and keep.
- The status line is a polite live region (bar and agent details, Merge result).
- Merge is a real button; when disabled its reason is visible beside it.

## 10. Props sketch
```rust
#[component]
pub fn ConflictRadar(
    agents: Vec<RadarAgent>,            // { id, name, model, running }
    files: Vec<RadarFile>,              // { path, edits: [RadarEdit { agent, from, to, summary, active }] }
    title: String,                      // "Parallel work"
    resolved: Option<BTreeMap<String, String>>, // controlled path → agent id
    heat: bool, order: bool, conflicts_only: bool, default_open: bool,
    on_resolve: EventHandler<(String, String)>,
    on_merge: EventHandler<()>,
) -> Element
```
GPUI: `ConflictRadarEl::new(radar).title(..).heat(..).order(..).expanded(Some(path)).on_resolve(..).on_merge(..)`.

## 11. Related components and references
[merge-conflict.md](merge-conflict.md) · [git-chips.md](git-chips.md) · [agent-fleet.md](agent-fleet.md) ·
[change-summary.md](change-summary.md).

| Reference | What we take | Licence note |
|---|---|---|
| affectUI Conflict Radar (public docs page) | props (agents, files, title, resolved, onResolve, onMerge), keyboard, motion and a11y rules | Paid; behaviour only, no source |
