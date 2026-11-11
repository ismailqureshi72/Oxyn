# Component specs

**Rule: every component gets the same micro-level depth.** The composer send control is only the first
worked example. A loader, a tool-call row, a diff hunk, a model picker, a sidebar item or an icon is
specified just as deeply. When we point at one component as an example, the standard applies to all.

## Every spec covers
1. **Purpose** — the moment in an agent app where it appears, and what the user is trying to do.
2. **State machine** — every state it can be in, including the in-between and failure states
   (empty, hover, focus, pressed, loading, streaming, working, queued, awaiting approval, success,
   error, stalled, cancelled, disabled-with-reason, offline), and every transition.
3. **States table** — what it shows, its primary and secondary actions, and its keyboard behaviour per state.
4. **Micro-interactions** — hover, press, focus ring, drag, enter/exit, morph between states, number and
   text changes, completion and failure moments.
5. **Timing table** — duration, delay, easing or spring, stagger and frame rate for each transition,
   plus the reduced-motion version.
6. **Variants** — free and Pro sets named per [../variants.md](../variants.md) (target ≥ 3 free; 8–20 for
   high-traffic components).
7. **Edge cases** — long text, overflow, RTL, tiny sizes (16–20 px), dense IDE panels, touch, slow
   networks, very fast and very slow agents.
8. **Accessibility** — labels per state, live-region announcements, contrast, focus order.
9. **Props sketch** — Rust struct, driven by `AgentState`.
10. **Related components and references** — with licence notes.

## Specs
| Spec | Status |
|---|---|
| [composer.md](composer.md) | draft v0.1 — composer family overview |
| [composer-send-control.md](composer-send-control.md) | draft v0.2 — worked example |
| [composer-input-shell.md](composer-input-shell.md) | draft v0.1 |
| [composer-tools-menu.md](composer-tools-menu.md) | draft v0.1 |
| [composer-attachments.md](composer-attachments.md) | draft v0.1 |
| [composer-slash-mentions.md](composer-slash-mentions.md) | draft v0.1 |
| [composer-model-picker.md](composer-model-picker.md) | draft v0.1 |
| [composer-effort-control.md](composer-effort-control.md) | draft v0.1 |
| [composer-context-meter.md](composer-context-meter.md) | draft v0.1 |
| [composer-mode-switch.md](composer-mode-switch.md) | draft v0.1 |
| [composer-permission-selector.md](composer-permission-selector.md) | draft v0.1 |
| [composer-voice-input.md](composer-voice-input.md) | draft v0.1 |
| [composer-queue-stack.md](composer-queue-stack.md) | draft v0.1 |
| [composer-status-strip.md](composer-status-strip.md) | draft v0.1 |
| [loaders.md](loaders.md) | draft v0.1 — family B overview |
| [loader-glyph.md](loader-glyph.md) | draft v0.1 |
| [loader-vector.md](loader-vector.md) | draft v0.1 |
| [thinking-orb.md](thinking-orb.md) | draft v0.1 |
| [loader-mascot.md](loader-mascot.md) | draft v0.1 |
| [status-text.md](status-text.md) | draft v0.1 — family C overview |
| [text-shimmer.md](text-shimmer.md) | draft v0.1 |
| [verb-cycler.md](verb-cycler.md) | draft v0.1 |
| [text-reveal.md](text-reveal.md) | draft v0.1 |
| [elapsed-timer.md](elapsed-timer.md) | draft v0.1 |
| [container-loaders.md](container-loaders.md) | draft v0.1 — family D overview |
| [border-beam.md](border-beam.md) | draft v0.1 |
| [media-generation-placeholder.md](media-generation-placeholder.md) | draft v0.1 |
| [media-analysis-scan.md](media-analysis-scan.md) | draft v0.1 |
| [skeletons.md](skeletons.md) | draft v0.1 |
| [messages.md](messages.md) | draft v0.1 — family E overview |
| [message-bubble.md](message-bubble.md) | draft v0.1 |
| [streaming-markdown.md](streaming-markdown.md) | draft v0.1 |
| [reasoning-block.md](reasoning-block.md) | draft v0.1 |
| [citations-sources.md](citations-sources.md) | draft v0.1 |
| [tools.md](tools.md) | draft v0.1 — family F overview |
| [tool-call.md](tool-call.md) | draft v0.1 |
| [file-edit-chip.md](file-edit-chip.md) | draft v0.1 |
| [terminal-output.md](terminal-output.md) | draft v0.1 |
| [plan-todo.md](plan-todo.md) | draft v0.1 |
| [subagent-tree.md](subagent-tree.md) | draft v0.1 |
| [fanout-grid.md](fanout-grid.md) | draft v0.1 |
| [acting-indicator.md](acting-indicator.md) | draft v0.1 |
| [trace-view.md](trace-view.md) | draft v0.1 |
| [background-jobs.md](background-jobs.md) | draft v0.1 |
| [run-replay.md](run-replay.md) | draft v0.1 |
| [run-meter.md](run-meter.md) | draft v0.1 |
| Every other component in [../component-taxonomy.md](../component-taxonomy.md) | to write in V2.2 |
