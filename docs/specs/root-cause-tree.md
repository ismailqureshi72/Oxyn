**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 9 – against the ref-2 capture notes).

# Spec: Root Cause Tree (`root-cause-tree`)

Family: Agents. Closes the parity row: affectUI · Root Cause Tree (Pro). Behaviour-only rebuild; layout, sizes,
colours, states, motion and keyboard follow the ref-2 capture notes (approximate); the failure, hypotheses, evidence
and copy are our own. Related Oxyn component: trace-view.
Components: `RootCauseTree` (Dioxus) · `RootCauseTreeEl` (GPUI), model in `oxyn_core::parity::rootfid`.

## 1. Purpose
Show how an agent works out why a run failed: the failure on top, candidate causes below joined by OR / AND gates,
each checked in turn and ruled out or confirmed, with the verdict and evidence for whichever node you pick.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Root | no outer card in Clean (nodes are their own cards), up to 720 px; Classic frames everything in one card with a header band |
| Header | branch mark, "Root cause" 13/600, a violet "Investigating" pill until a cause is found; right: "n hypotheses · n ruled out" 12 muted |
| Gate | uppercase 9 px chip with a hairline under a node with children (OR / AND); violet on the found route |
| Node card | 124 px (failure 150 px), 58 px min, 10 px radius, white / #1B1B1B, hairline #ECECEC / #2E2E2E: brand mark + title 12/600, a mono file chip (+n for more), status line 11 px – "Failed · …" red, "Checking" violet with a spinning arc, "Not checked" muted with a hollow dot, "Ruled out · m:ss" muted with ×, "Confirmed · m:ss" red dot, "Root cause · m:ss" red chip, "Cause found" violet chip on the failure |
| Connectors | 1 px elbows #D9D9DC / #3A3A3E; the confirmed route draws 2 px violet #6E56CF / #9D8CF5 and off-route cards fade to 40 % |
| Selected | violet border, 3 px soft violet ring, 1 px lift |
| Evidence | full-width card, 12 px radius: mark + title 13/600, status right; the verdict 12 px; file chips; a "Tool call" or "Diff" label with a mono code block (#F6F6F7 / #232323, − red, + green); on a root cause, two actions: Retry from here as an outlined button, Fix it as a filled pill |

## 3. Behaviour
- Each hypothesis has a check window (start, end) and an outcome: `rc_status` gives Not checked → Checking →
  Ruled out / Confirmed at run time `t`. `rc_counts` feeds the header; `rc_found` is true once any hypothesis directly
  under the failure is confirmed (each node's own confirmation already reflects its gate in the data); `rc_roots` lists the deepest confirmed nodes and
  `rc_path` the route from them to the failure.
- The evidence card shows the picked node, else the newest node being or having been checked, else the failure.
- Clicking a node (or arrow keys in the tree) selects it and calls `on_select`; Retry from here / Fix it call
  `on_retry` / `on_fix` with the node id.
- Live demo: run seconds advance with the clock; checks land one after another, then the route lights and the demo
  loops.
- prefers-reduced-motion: no spinner, fades or scale-in.

## 4. Variants
live · checking · ruled out (tool call) · found (diff + actions) · pending branch · failure picked · narrow · classic.
Controls: style, state (6), width.

## 5. Keyboard and accessibility
The tree is role=tree; nodes are treeitem buttons with aria-level and aria-selected, named with title and status. ← ↑
and → ↓ move to the previous / next node depth-first, Home / End jump to the failure / last node; Tab then reaches the
evidence buttons. Status is always in words; the header counts are a status region.

## 6. Responsive
Wide: a top-down tree centred in the width (scrolls sideways if a very wide tree does not fit). Under 600 px
(container query) it becomes an indented outline: full-width cards, gate chips inline, left rails for the connectors.

## 7. Known gaps
- Only the vertical connector segments light up on the found route; the horizontal sibling bar stays grey.
- No "Read only" example (the component works without the handlers, but the demo always passes them).
- GPUI: a static snapshot – no spinner or fades; Classic only squares the cards and frames the root.
