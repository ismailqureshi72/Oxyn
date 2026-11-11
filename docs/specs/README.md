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
| [composer-send-control.md](composer-send-control.md) | draft v0.1 — worked example |
| Every other component in [../component-taxonomy.md](../component-taxonomy.md) | to write in V2.2 |
