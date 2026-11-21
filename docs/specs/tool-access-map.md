**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 9 – against the ref-2 capture notes).

# Spec: Tool reach (`tool-access-map`)

Family: Agents. Closes the parity row: affectUI · Tool Access Map (Pro). Behaviour-only rebuild; layout, sizes,
colours, states, motion and keyboard follow the ref-2 capture notes (approximate); agents, tools, calls and copy are
our own. Related Oxyn components: mcp-plugin-list, plugin-icons.
Components: `ToolAccessMap` (Dioxus) · `ToolAccessMapEl` (GPUI), model in `oxyn_core::kit::toolmapfid`.

## 1. Purpose
Show which of a run's tools each agent can reach and what it did with them, and let you read any tool's calls and
replay one – unchanged or with an edited input.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Header | tool mark, "Tool access" 13/600; right: blue "n running" pill (shown only when some call is still going), "reach/total tools · n calls" 12 muted tabular |
| Agent tabs | segmented track #F1F1F3 / #1F2024, 28 px tabs with the model's mark; the picked one white with a soft shadow |
| Map | three columns: tools (first half of the list) · agent card (190 px) · tools (rest), 44 px apart |
| Agent card | 10 px radius hairline card: mark + name 13/600, model 11 muted, status badge (blue Running, amber Needs you, grey Queued / Done, red Failed), task 12 px |
| Tool card | fixed 46 px tall: mark, name 12/600, "n×" right, and one line – "Last call 520ms", a blue "Running" pill, red "Failed", "Not used yet" or "No access" (card at 50 %, disabled, mark greyed) |
| Wires | a 22 px stub from every card to a column spine, then on to the agent: solid blue where the agent has access, grey dashed where not, red after a failed call; a brighter glint runs along a busy tool's wire |
| Calls view | replaces the map in a framed card: back arrow, mark, tool name, scope, "n calls"; left a list (newest first: status glyph, one-line input, Replay chip, duration); right the picked call's Input (with its run time) and Output (with its duration; "Waiting for the result" while in flight; errors in red), Edit input and a filled Replay button |

## 3. Behaviour
- `tm_state` reads one tool for one agent: unlisted = no access, no calls = unused, any call running = running, last
  call failed = failed, else used. `tm_counts` feeds the header; `tm_sides` splits the layout; `tm_dur`, `tm_clock`
  and `tm_summary` format.
- Every agent shares the same tool layout; switching tabs only changes the states, so nothing moves.
- Opening a tool shows its calls with the newest picked. Replay (`tm_replay`) adds a running call marked Replay with
  the same or the edited input and calls `on_replay(agent, tool, input)`; in the component the replay lands after a
  short wait (`tm_settle` is there for hosts that settle it themselves). Both buttons are disabled while it is in
  flight.
- prefers-reduced-motion: no glint, slide or fade.

## 4. Variants
map · other agent (failed call, query in flight) · needs you · calls · replaying · editing · failed call · narrow ·
classic. Controls: style, state (7), width.

## 5. Keyboard and accessibility
← / → move between agent tabs (tablist, roving focus); Tab reaches each tool, Enter opens its calls; the call list is
one tab stop with ↑ / ↓; Escape goes back to the map; Cmd or Ctrl + Enter runs an edited input. Tool buttons are named
with state and call count; out-of-reach tools are disabled; states are always words; the counts are a status region.

## 6. Responsive
Fixed card sizes. Under 540 px (container query) the agent card leads, the tools sit in two columns below with no
wires, and the calls view stacks the list above the call.

## 7. Known gaps
- Switching agents does not re-run the wires in from the agent; cards change state in place.
- The map does not stay mounted (inert) under the calls view; it is replaced and comes back on Escape, without
  returning focus to the tool that was opened.
- GPUI: a static map snapshot – no calls view, no glint.
