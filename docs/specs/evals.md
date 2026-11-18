**Status:** built v0.1 (2026-11-18, S5: Orchestration and builders).

# Spec: Evals (`evals`)

Family: Screens. Reference: affectUI Evals (`/screens/evals`), layout and behaviour only; no source or assets copied.
Components: `EvalsScreen` (Dioxus), models in `oxyn_core::parity::buildscr`. GPUI: not built yet (see gaps).

## 1. Purpose
Know whether a prompt change made replies better before it ships: how often it passes, what changed in the prompt, and which cases broke.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Evalkit persona: Home · Chat · Evals 3 · Datasets · Analytics, then the Prompts group (Support reply, Ticket triage, Refund check, Onboarding); Evals is current |
| Top bar | Evals / Support reply prompt, a version menu (`v12 current`), a model menu with the official mark (`claude-sonnet-5-5`), "Last run 2 hours ago" and **Run eval** |
| Tiles | Pass rate, Avg score /5, Latency p50 (s) and Cost per 1k runs, each with its change against the version before (green better, red worse) |
| Pass rate by version | The picked version's rate and change, a line over v1–v12 with 0 / 50 / 100 % guides, a point per version, and "Last 12 versions · 25 test cases · model" |
| Changes in v12 | The system prompt as a diff (removed struck through on a red tint, added on a green tint), the current change outlined, a count of changes with previous / next, and buttons that settle every change one way or the other |
| Test cases | "Test cases 25", All / Pass / Partial / Fail with counts, then a table: id and source channel mark with the input, Expected, the version's output, a Pass / Partial / Fail pill with the score, latency and ↻ Re-run |

Chrome is monochrome neutral; colour only means something: official model and channel marks and status (pass green, partial amber, fail red, running blue).

## 3. Behaviour
- Scores are out of 5: Pass 4–5, Partial 2–3, Fail 0–1. Pass rate counts a partial as half.
- The version menu (and the last three chart points) switch every tile, the pill column and the deltas. Arrow keys on the chart step through versions.
- Picking another model marks the scores as not run on it; Run eval scores every case in turn (a progress bar with Stop), then the tiles and pills reflect that model.
- ↻ on a row re-runs one case; the row shows Scoring, then its latency with ↻.
- A case's input opens the row to the full input, expected reply and output.
- Accept all / Reject all settle the diff (the prompt reads as v12 or back to v11) with Undo.
- prefers-reduced-motion: the spinner slows; no other motion.

## 4. Variants
States: Data · Loading · Empty · **Running**. Frame: Inset · Flat. Oxyn extras: `fails_first` (failures and partials lead the table) and `cost_col` (a cost column per case and the run's total).

## 5. Keyboard and accessibility
The chart is a listbox of versions with aria-activedescendant (←/→, Home/End). The diff uses `del` and `ins`. The cases are a role=table with column headers; each input is a button with aria-expanded; Re-run buttons are named by case. The running eval is a progressbar. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/evals/full`) renders at the real viewport (R2). Under 1000 px of page the chart and the diff stack and Expected hides; under 700 px the tiles go two by two and only input, score and re-run stay; under 640 px the sidebar slides in from a menu button. Labels ellipsise in flex children with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
No GPUI element yet. Prompts, cases, people and the workspace are fictional; model versions are fictional, model and channel marks are the providers' official ones.
