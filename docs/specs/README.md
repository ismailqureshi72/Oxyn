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
| Every other component in [../component-taxonomy.md](../component-taxonomy.md) | to write in V2.2 |
