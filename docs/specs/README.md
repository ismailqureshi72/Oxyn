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
This table lists the specs in the public repo: the ones the Free items are built from (a few of
them also cover Pro variants of the same component). Specs for Oxyn Pro items (Pro components and
families, screens, agent-app templates, blocks, AI special components, kits) live with their
sources in the private repo `ismailqureshi72/oxyn-pro`, at the same paths
(`docs/specs/<name>.md`); a link to one of them elsewhere in these docs is written as plain text
marked "Pro (private)".

| Spec | Status |
|---|---|
| [composer.md](composer.md) | draft v0.1 — composer family overview |
| [composer-send-control.md](composer-send-control.md) | draft v0.2 — worked example |
| [composer-attachments.md](composer-attachments.md) | draft v0.1 |
| [composer-model-picker.md](composer-model-picker.md) | draft v0.1 |
| [composer-effort-control.md](composer-effort-control.md) | draft v0.1 |
| [composer-context-meter.md](composer-context-meter.md) | draft v0.1 |
| [composer-voice-input.md](composer-voice-input.md) | draft v0.1 |
| [composer-queue-stack.md](composer-queue-stack.md) | draft v0.1 |
| [loader-glyph.md](loader-glyph.md) | draft v0.1 |
| [text-shimmer.md](text-shimmer.md) | draft v0.1 |
| [verb-cycler.md](verb-cycler.md) | draft v0.1 |
| [text-reveal.md](text-reveal.md) | draft v0.1 |
| [elapsed-timer.md](elapsed-timer.md) | draft v0.1 |
| [skeletons.md](skeletons.md) | v1 — implemented (plan #26): skeleton, shimmer loader |
| [messages.md](messages.md) | draft v0.1 — family E overview |
| [message-bubble.md](message-bubble.md) | draft v0.1 |
| [streaming-markdown.md](streaming-markdown.md) | draft v0.1 |
| [reasoning-block.md](reasoning-block.md) | draft v0.1 |
| [tool-call.md](tool-call.md) | draft v0.1 |
| [subagent-tree.md](subagent-tree.md) | draft v0.1 |
| [fanout-grid.md](fanout-grid.md) | built v0.2 (task grid, plan #32, family B part 9); lanes draft |
| [approval-card.md](approval-card.md) | draft v0.1 |
| [diff-viewer.md](diff-viewer.md) | draft v0.1 |
| [workspace-chrome.md](workspace-chrome.md) | draft v0.1 — family J overview |
| [command-palette.md](command-palette.md) | v1 — implemented (plan #26): palette, command menu, AI answers |
| [button.md](button.md) | v1 — implemented (plan #23): button, icon button, action button |
| [button-groups.md](button-groups.md) | v1 — implemented (plan #23): split, joined, floating, expanding groups |
| [badges-chips.md](badges-chips.md) | v1 — implemented (plan #23): badge, chip, value pill, entity chip + monogram |
| [avatar.md](avatar.md) | v1.1 — implemented (plan #23, Arc gaps closed in plan #26) |
| [display-atoms.md](display-atoms.md) | v1 — implemented (plan #23): kbd, separator, spinner, empty state |
| [toggles.md](toggles.md) | v1.1 — implemented (plan #23, Arc gaps closed in plan #25): switch, checkbox, radio group |
| [segmented-control.md](segmented-control.md) | v1 — implemented (plan #23) |
| [select.md](select.md) | v1.1 — implemented (plan #23, Arc gaps closed in plan #25) |
| [slider.md](slider.md) | v1 — implemented (plan #23) |
| [text-field.md](text-field.md) | v1 — implemented (plan #23) |
| [dropdown-menu.md](dropdown-menu.md) | v1.1 — implemented (plan #23, gaps closed in plan #26) |
| [tag-input.md](tag-input.md) | v1 — implemented (plan #25): tag input |
| [combobox.md](combobox.md) | v1 — implemented (plan #25): combobox |
| [otp-input.md](otp-input.md) | v1 — implemented (plan #25): OTP input |
| [password-strength.md](password-strength.md) | v1 — implemented (plan #25): password strength |
| [inline-edit.md](inline-edit.md) | v1 — implemented (plan #25): inline edit |
| [phone-input.md](phone-input.md) | v1 — implemented (plan #25): phone input |
| [dialog.md](dialog.md) | v1 — implemented (plan #26): dialog |
| [card.md](card.md) | v1 — implemented (plan #26): card |
| [calendar.md](calendar.md) | v1 — implemented (plan #25): calendar |
| [date-reel.md](date-reel.md) | v1 — implemented (plan #25): date reel |
| [inline-diff.md](inline-diff.md) | built v0.2 (review mode, plan #32, family B part 9) |
| [diff-timeline.md](diff-timeline.md) | built v0.2 (run view, plan #32, family B part 9) |
| [code-suggestion.md](code-suggestion.md) | built v0.1 (plan #30) |
| [chat-composer.md](chat-composer.md) | built v0.2 (plan #32 fidelity: exact affectUI Chat Composer) |
| [model-picker.md](model-picker.md) | built v0.1 (plan #32, chunk 2) |
| [run-controls.md](run-controls.md) | built v0.2 (plan #32, chunk 2; v0.1 plan #28) |
| [agent-crew.md](agent-crew.md) | built v0.1 (plan #32, overnight batch, family B part 2) |
| [agent-cursors.md](agent-cursors.md) | built v0.1 (plan #32, overnight batch, family B part 3) |
| [autonomy-dial.md](autonomy-dial.md) | built v0.1 (plan #32, overnight batch, family B part 6) |
| [collaboration-stream.md](collaboration-stream.md) | built v0.1 (plan #32, overnight batch, family B part 6) |
| [context-map.md](context-map.md) | built v0.1 (plan #32, overnight batch, family B part 6) |
| [cost-waterfall.md](cost-waterfall.md) | built v0.1 (plan #32, overnight batch, family B part 6) |
| [decision-tree.md](decision-tree.md) | built v0.2 (plan #32, family B part 8) |
| [fallback-chain.md](fallback-chain.md) | built v0.2 (plan #32, family B part 8) |
| [judge-bracket.md](judge-bracket.md) | built v0.1 (plan #32, family B part 9) |
| [tool-access-map.md](tool-access-map.md) | built v0.1 (plan #32, family B part 9) |
| [scheduled-runs.md](scheduled-runs.md) | built v0.1 (plan #32, family B part 9) |
| [trace-waterfall.md](trace-waterfall.md) | built v0.1 (plan #32, family B part 9) |
| [code-block.md](code-block.md) | v1 — implemented (plan #27) |
| [copy-button.md](copy-button.md) | v1 — implemented (plan #27) |
| [tabs.md](tabs.md) | v1 — implemented (plan #27) |
| [confirm-popover.md](confirm-popover.md) | built v0.1 (plan #28) |
| [stepper.md](stepper.md) | built v0.1 (plan #28) |
| [file-dropzone.md](file-dropzone.md) | built v0.1 (plan #28) |
| [agent-trail.md](agent-trail.md) | built v0.2 (S15: full affectUI Agent Trail spec; v0.1 plan #28) |
| Every other component in [../component-taxonomy.md](../component-taxonomy.md) | to write in V2.2 |
