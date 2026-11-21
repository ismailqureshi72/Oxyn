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
| [admin-overview.md](admin-overview.md) | v0.1 — built (S15) |
| [audit-log.md](audit-log.md) | v0.1 — built (S15) |
| [status-board.md](status-board.md) | v0.1 — built (S15) |
| [api-keys-webhooks.md](api-keys-webhooks.md) | v0.1 — built (S15) |
| [members-roles.md](members-roles.md) | v0.1 — built (S15) |
| [billing-usage.md](billing-usage.md) | v0.1 — built (S15) |
| [agent-tool-calls.md](agent-tool-calls.md) | v0.1 — built (S15) |
| [composer.md](composer.md) | draft v0.1 — composer family overview |
| [status-indicator.md](status-indicator.md) | v1 — implemented (plan #15) |
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
| [skeletons.md](skeletons.md) | v1 — implemented (plan #26): skeleton, shimmer loader |
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
| [fanout-grid.md](fanout-grid.md) | built v0.2 (task grid, plan #32, family B part 9); lanes draft |
| [acting-indicator.md](acting-indicator.md) | draft v0.1 |
| [trace-view.md](trace-view.md) | draft v0.1 |
| [background-jobs.md](background-jobs.md) | draft v0.1 |
| [run-replay.md](run-replay.md) | draft v0.1 |
| [run-meter.md](run-meter.md) | draft v0.1 |
| [approvals.md](approvals.md) | draft v0.1 — family G overview |
| [approval-card.md](approval-card.md) | draft v0.1 |
| [plan-approval.md](plan-approval.md) | draft v0.1 |
| [permission-rules.md](permission-rules.md) | draft v0.1 |
| [takeover-banner.md](takeover-banner.md) | draft v0.1 |
| [checkpoints-git.md](checkpoints-git.md) | draft v0.1 — family H overview |
| [checkpoint-timeline.md](checkpoint-timeline.md) | draft v0.1 |
| [rollback-impact.md](rollback-impact.md) | draft v0.1 |
| [diff-viewer.md](diff-viewer.md) | draft v0.1 |
| [merge-conflict.md](merge-conflict.md) | built v0.2 (plan #31) |
| [git-chips.md](git-chips.md) | draft v0.1 |
| [pr-card.md](pr-card.md) | draft v0.1 |
| [review-card.md](review-card.md) | draft v0.1 |
| [genui.md](genui.md) | draft v0.1 — family I overview |
| [genui-renderer.md](genui-renderer.md) | draft v0.1 |
| [genui-blocks.md](genui-blocks.md) | draft v0.1 |
| [genui-forms.md](genui-forms.md) | draft v0.1 |
| [genui-skeletons.md](genui-skeletons.md) | draft v0.1 |
| [workspace-chrome.md](workspace-chrome.md) | draft v0.1 — family J overview |
| [terminal-tabs.md](terminal-tabs.md) | draft v0.1 |
| [side-panels.md](side-panels.md) | draft v0.1 |
| [mcp-plugin-list.md](mcp-plugin-list.md) | draft v0.1 |
| [command-palette.md](command-palette.md) | v1 — implemented (plan #26): palette, command menu, AI answers |
| [desktop-overlays.md](desktop-overlays.md) | draft v0.1 |
| [live-voice.md](live-voice.md) | draft v0.1 — family K overview |
| [voice-orb.md](voice-orb.md) | draft v0.1 |
| [voice-visualisers.md](voice-visualisers.md) | draft v0.1 |
| [voice-control-bar.md](voice-control-bar.md) | draft v0.1 |
| [live-captions.md](live-captions.md) | draft v0.1 |
| [icons.md](icons.md) | draft v0.1 — family L overview |
| [icon-states.md](icon-states.md) | draft v0.1 |
| [plugin-icons.md](plugin-icons.md) | draft v0.1 |
| [animated-marks.md](animated-marks.md) | draft v0.1 |
| [button.md](button.md) | v1 — implemented (plan #23): button, icon button, action button |
| [button-groups.md](button-groups.md) | v1 — implemented (plan #23): split, joined, floating, expanding groups |
| [badges-chips.md](badges-chips.md) | v1 — implemented (plan #23): badge, chip, value pill, entity chip + monogram |
| [avatar.md](avatar.md) | v1.1 — implemented (plan #23, Arc gaps closed in plan #26) |
| [display-atoms.md](display-atoms.md) | v1 — implemented (plan #23): kbd, separator, spinner, empty state |
| [number-ticker.md](number-ticker.md) | v1 — implemented (plan #23): number ticker, reaction bar |
| [toggles.md](toggles.md) | v1.1 — implemented (plan #23, Arc gaps closed in plan #25): switch, checkbox, radio group |
| [segmented-control.md](segmented-control.md) | v1 — implemented (plan #23) |
| [select.md](select.md) | v1.1 — implemented (plan #23, Arc gaps closed in plan #25) |
| [slider.md](slider.md) | v1 — implemented (plan #23) |
| [text-field.md](text-field.md) | v1 — implemented (plan #23) |
| [tooltip-popover.md](tooltip-popover.md) | v1.1 — implemented (plan #23, gaps closed in plan #26) |
| [dropdown-menu.md](dropdown-menu.md) | v1.1 — implemented (plan #23, gaps closed in plan #26) |
| [tag-input.md](tag-input.md) | v1 — implemented (plan #25): tag input |
| [combobox.md](combobox.md) | v1 — implemented (plan #25): combobox |
| [multi-select.md](multi-select.md) | v1 — implemented (plan #25): multi-select |
| [otp-input.md](otp-input.md) | v1 — implemented (plan #25): OTP input |
| [password-field.md](password-field.md) | v1 — implemented (plan #25): password field |
| [password-strength.md](password-strength.md) | v1 — implemented (plan #25): password strength |
| [inline-edit.md](inline-edit.md) | v1 — implemented (plan #25): inline edit |
| [phone-input.md](phone-input.md) | v1 — implemented (plan #25): phone input |
| [dialog.md](dialog.md) | v1 — implemented (plan #26): dialog |
| [sheet.md](sheet.md) | v1 — implemented (plan #26): side sheet |
| [sheet-stack.md](sheet-stack.md) | v1 — implemented (plan #26): sheet stack |
| [bottom-sheet.md](bottom-sheet.md) | v1 — implemented (plan #26): bottom sheet |
| [toaster.md](toaster.md) | v1 — implemented (plan #26): toaster |
| [notification-stack.md](notification-stack.md) | v1 — implemented (plan #26): notification stack |
| [alert.md](alert.md) | v1 — implemented (plan #26): alert |
| [card.md](card.md) | v1 — implemented (plan #26): card |
| [scroll-area.md](scroll-area.md) | v1 — implemented (plan #26): scroll area with stick-to-bottom |
| [pagination.md](pagination.md) | v1 — implemented (plan #26): pagination |
| [skeleton-morph.md](skeleton-morph.md) | v1 — implemented (plan #26): skeleton morph |
| [ai-command.md](ai-command.md) | v1 — implemented (plan #26): AI command |
| [ai-search-bar.md](ai-search-bar.md) | v1 — implemented (plan #26): AI search bar |
| [settings-command.md](settings-command.md) | v1 — implemented (plan #26): settings command |
| [search-field.md](search-field.md) | v1 — implemented (plan #26): search field |
| [expanding-search.md](expanding-search.md) | v1 — implemented (plan #26): expanding search |
| [shortcut-recorder.md](shortcut-recorder.md) | v1 — implemented (plan #26): shortcut recorder and cheatsheet |
| [control-center.md](control-center.md) | v1 — implemented (plan #26): control center |
| [calendar.md](calendar.md) | v1 — implemented (plan #25): calendar |
| [date-picker.md](date-picker.md) | v1 — implemented (plan #25): date picker |
| [date-range-picker.md](date-range-picker.md) | v1 — implemented (plan #25): date range picker |
| [time-picker.md](time-picker.md) | v1 — implemented (plan #25): time picker |
| [time-dial.md](time-dial.md) | v1 — implemented (plan #25): time dial |
| [date-reel.md](date-reel.md) | v1 — implemented (plan #25): date reel |
| [filter-menu.md](filter-menu.md) | v1 — implemented (plan #25): filter menu |
| [stepper-form.md](stepper-form.md) | v1 — implemented (plan #25): stepper form |
| [multi-step-form.md](multi-step-form.md) | v1 — implemented (plan #25): multi-step form |
| [decision-card.md](decision-card.md) | v1 — implemented (plan #25): decision card |
| [approval-questions.md](approval-questions.md) | v1 — implemented (plan #25): approval questions (Beautiful UI Approval Card port) |
| [smart-form-fill.md](smart-form-fill.md) | v1 — implemented (plan #25): smart form fill |
| [inline-diff.md](inline-diff.md) | built v0.2 (review mode, plan #32, family B part 9) |
| [change-summary.md](change-summary.md) | built v0.1 (plan #30) |
| [diff-table.md](diff-table.md) | built v0.1 (plan #30) |
| [diff-timeline.md](diff-timeline.md) | built v0.2 (run view, plan #32, family B part 9) |
| [code-suggestion.md](code-suggestion.md) | built v0.1 (plan #30) |
| [ai-inline-edit.md](ai-inline-edit.md) | built v0.1 (plan #30) |
| [image-compare.md](image-compare.md) | built v0.1 (plan #30) |
| [comment-thread.md](comment-thread.md) | built v0.1 (plan #30); affectUI Comment Thread added (plan #32) |
| [review-center.md](review-center.md) | built v0.1 (plan #30) |
| [conflict-radar.md](conflict-radar.md) | built v0.1 (plan #31) |
| [chat-composer.md](chat-composer.md) | built v0.2 (plan #32 fidelity: exact affectUI Chat Composer) |
| [comments-panel.md](comments-panel.md) | built v0.1 (plan #32) |
| [model-picker.md](model-picker.md) | built v0.1 (plan #32, chunk 2) |
| [run-controls.md](run-controls.md) | built v0.2 (plan #32, chunk 2; v0.1 plan #28) |
| [ai-answer.md](ai-answer.md) | built v0.1 (plan #32, overnight batch, family A) |
| [cited-answer.md](cited-answer.md) | built v0.1 (plan #32, overnight batch, family A) |
| [message-actions.md](message-actions.md) | built v0.1 (plan #32, overnight batch, family A) |
| [thinking-state.md](thinking-state.md) | built v0.1 (plan #32, overnight batch, family A) |
| [writing-assistant.md](writing-assistant.md) | built v0.1 (plan #32, overnight batch, family A) |
| [thinking-loader.md](thinking-loader.md) | built v0.2 (plan #32, overnight batch, family A; v0.1 plan #28) |
| [agent-findings.md](agent-findings.md) | built v0.1 (plan #32, overnight batch, family B) |
| [agent-inbox.md](agent-inbox.md) | built v0.1 (plan #32, overnight batch, family B) |
| [agent-crew.md](agent-crew.md) | built v0.1 (plan #32, overnight batch, family B part 2) |
| [agent-cursors.md](agent-cursors.md) | built v0.1 (plan #32, overnight batch, family B part 3) |
| [agent-graph.md](agent-graph.md) | built v0.1 (plan #32, overnight batch, family B part 3) |
| [agent-handoff.md](agent-handoff.md) | built v0.1 (plan #32, overnight batch, family B part 3) |
| [agent-lanes.md](agent-lanes.md) | built v0.1 (plan #32, overnight batch, family B part 3) |
| [agent-ledger.md](agent-ledger.md) | built v0.1 (plan #32, overnight batch, family B part 4) |
| [agent-monitor.md](agent-monitor.md) | built v0.1 (plan #32, overnight batch, family B part 4) |
| [agent-queue.md](agent-queue.md) | built v0.1 (plan #32, overnight batch, family B part 4) |
| [agent-recap.md](agent-recap.md) | built v0.1 (plan #32, overnight batch, family B part 4) |
| [agent-sequence.md](agent-sequence.md) | built v0.1 (plan #32, overnight batch, family B part 4) |
| [autonomy-dial.md](autonomy-dial.md) | built v0.1 (plan #32, overnight batch, family B part 6) |
| [away-summary.md](away-summary.md) | built v0.1 (plan #32, overnight batch, family B part 6) |
| [collaboration-stream.md](collaboration-stream.md) | built v0.1 (plan #32, overnight batch, family B part 6) |
| [context-map.md](context-map.md) | built v0.1 (plan #32, overnight batch, family B part 6) |
| [cost-waterfall.md](cost-waterfall.md) | built v0.1 (plan #32, overnight batch, family B part 6) |
| [decision-tree.md](decision-tree.md) | built v0.2 (plan #32, family B part 8) |
| [delegate-board.md](delegate-board.md) | built v0.2 (plan #32, family B part 8) |
| [fallback-chain.md](fallback-chain.md) | built v0.2 (plan #32, family B part 8) |
| [parallel-attempts.md](parallel-attempts.md) | built v0.1 (plan #32, family B part 9) |
| [judge-bracket.md](judge-bracket.md) | built v0.1 (plan #32, family B part 9) |
| [memory-panel.md](memory-panel.md) | built v0.1 (plan #32, family B part 9) |
| [running-now.md](running-now.md) | built v0.1 (plan #32, family B part 9) |
| [plan-vs-actual.md](plan-vs-actual.md) | built v0.1 (plan #32, family B part 9) |
| [root-cause-tree.md](root-cause-tree.md) | built v0.1 (plan #32, family B part 9) |
| [tool-access-map.md](tool-access-map.md) | built v0.1 (plan #32, family B part 9) |
| [scheduled-runs.md](scheduled-runs.md) | built v0.1 (plan #32, family B part 9) |
| [rate-limit-planner.md](rate-limit-planner.md) | built v0.1 (plan #32, family B part 9) |
| [trace-waterfall.md](trace-waterfall.md) | built v0.1 (plan #32, family B part 9) |
| [effort-ruler.md](effort-ruler.md) | built v0.1 (variants 1, Thinking Selector) |
| [reasoning-panel.md](reasoning-panel.md) | built v0.1 (variants 1, Thinking Selector) |
| [effort-list.md](effort-list.md) | built v0.1 (variants 1, Thinking Selector) |
| [effort-pill.md](effort-pill.md) | built v0.1 (variants 1, Thinking Selector) |
| [context-effort-menu.md](context-effort-menu.md) | built v0.1 (variants 1, Thinking Selector) |
| [model-effort-menu.md](model-effort-menu.md) | built v0.1 (variants 1, Thinking Selector) |
| [nozzle-slider.md](nozzle-slider.md) | built v0.2 (variants 1, Thinking Selector) |
| [effort-stairs.md](effort-stairs.md) | built v0.1 (variants 1, Thinking Selector) |
| [context-bar.md](context-bar.md) | built v0.1 (variants 2, Context Visualizer) |
| [context-breakdown.md](context-breakdown.md) | built v0.1 (variants 2, Context Visualizer) |
| [context-stack.md](context-stack.md) | built v0.1 (variants 2, Context Visualizer) |
| [context-tide.md](context-tide.md) | built v0.1 (variants 2, Context Visualizer) |
| [storage-meter.md](storage-meter.md) | built v0.1 (variants 2, Storage) |
| [storage-cleanup.md](storage-cleanup.md) | built v0.1 (variants 2, Storage) |
| [storage-waffle.md](storage-waffle.md) | built v0.1 (variants 2, Storage) |
| [storage-forecast.md](storage-forecast.md) | built v0.1 (variants 2, Storage) |
| [upload-dropzone.md](upload-dropzone.md) | built v0.1 (variants 2, Attachments) |
| [attachment-chips.md](attachment-chips.md) | built v0.1 (variants 2, Attachments) |
| [polaroid-stack.md](polaroid-stack.md) | built v0.1 (variants 2, Attachments) |
| [drop-lanes.md](drop-lanes.md) | built v0.1 (variants 2, Attachments) |
| [gooey-composer.md](gooey-composer.md) | built v0.1 (variants 2, Menu Motion) |
| [gooey-attach-menu.md](gooey-attach-menu.md) | built v0.1 (variants 2, Menu Motion) |
| [gooey-model-menu.md](gooey-model-menu.md) | built v0.1 (variants 2, Menu Motion) |
| [liquid-tabs.md](liquid-tabs.md) | built v0.1 (variants 2, Menu Motion) |
| [liquid-drops.md](liquid-drops.md) | built v0.1 (variants 2, Menu Motion) |
| [background-picker.md](background-picker.md) | built v0.1 (variants 3, Theme Selector) |
| [theme-grid.md](theme-grid.md) | built v0.1 (variants 3, Theme Selector) |
| [theme-split.md](theme-split.md) | built v0.1 (variants 3, Theme Selector) |
| [accent-density.md](accent-density.md) | built v0.1 (variants 3, Theme Selector) |
| [theme-schedule.md](theme-schedule.md) | built v0.1 (variants 3, Theme Selector) |
| [type-scale.md](type-scale.md) | built v0.1 (variants 3, Theme Selector) |
| [light-playground.md](light-playground.md) | built v0.1 (variants 3, Theme Selector) |
| [app-icon-picker.md](app-icon-picker.md) | built v0.1 (variants 3, Theme Selector) |
| [display-comfort.md](display-comfort.md) | built v0.1 (variants 3, Theme Selector) |
| [shell-inset.md](shell-inset.md) | built v0.1 (screens 1, Screens) |
| [shell-flat.md](shell-flat.md) | built v0.1 (screens 1, Screens) |
| [shell-three-pane.md](shell-three-pane.md) | built v0.1 (screens 1, Screens) |
| [chat-start.md](chat-start.md) | built v0.1 (screens 1, Screens) |
| [chat-overview.md](chat-overview.md) | built v0.1 (screens 1, Screens) |
| [chat-agents.md](chat-agents.md) | built v0.1 (screens 1, Screens) |
| [chat-double-sidebar.md](chat-double-sidebar.md) | built v0.1 (screens 1, Screens) |
| [chat-progress.md](chat-progress.md) | built v0.1 (screens 1, Screens) |
| [chat-run.md](chat-run.md) | built v0.1 (screens 1, Screens) |
| [folders-overview.md](folders-overview.md) | built v0.1 (screens 1, Screens) |
| [folders-detail.md](folders-detail.md) | built v0.1 (screens 1, Screens) |
| [integrations.md](integrations.md) | built v0.1 (screens 1, Screens) |
| [kanban.md](kanban.md) | built v0.1 (screens 1, Screens) |
| [crm.md](crm.md) | built v0.1 (screens 1, Screens) |
| [workflow.md](workflow.md) | built v0.1 (screens 1, Screens) |
| [settings.md](settings.md) | built v0.1 (screens 1, Screens) |
| [agent-orchestration.md](agent-orchestration.md) | built v0.1 (screens 2, Screens) |
| [agent-thread.md](agent-thread.md) | built v0.1 (screens 2, Screens) |
| [agent-timeline.md](agent-timeline.md) | v0.1 — built (C1d, AI Components, Pro) |
| [model-card.md](model-card.md) | v0.1 — built (C1d, AI Components, Pro) |
| [ai-workflow.md](ai-workflow.md) | v0.1 — built (C1d, AI Components, Pro) |
| [image-generation.md](image-generation.md) | v0.1 — built (C1d, AI Components, Pro) |
| [speech-player.md](speech-player.md) | v0.1 — built (C1d, AI Components, Pro) |
| [meeting-summary.md](meeting-summary.md) | v0.1 — built (C1d, AI Components, Pro) |
| [reply-draft.md](reply-draft.md) | v0.1 — built (C1d, AI Components, Pro) |
| [human-handoff.md](human-handoff.md) | v0.1 — built (C1d, AI Components, Pro) |
| [pii-redaction.md](pii-redaction.md) | v0.1 — built (C1d, AI Components, Pro) |
| [llm-cost-calculator.md](llm-cost-calculator.md) | v0.1 — built (C1d, AI Components, Pro) |
| [provider-keys.md](provider-keys.md) | v0.1 — built (C1d, AI Components, Pro) |
| [generation-stats.md](generation-stats.md) | v0.1 — built (C1d, AI Components, Pro) |
| [ai-translate.md](ai-translate.md) | v0.1 — built (C1d, AI Components, Pro) |
| [tldr-toggle.md](tldr-toggle.md) | v0.1 — built (C1d, AI Components, Pro) |
| [connector-setup.md](connector-setup.md) | v0.1 — built (C1d, AI Components, Pro) |
| [enrich-column.md](enrich-column.md) | v0.1 — built (C1d, AI Components, Pro) |
| [ticket-router.md](ticket-router.md) | v0.1 — built (C1d, AI Components, Pro) |
| [ask-your-data.md](ask-your-data.md) | v0.1 — built (C1e, AI Components, Pro) |
| [agent-doc-cursor.md](agent-doc-cursor.md) | v0.1 — built (C1e, AI Components, Pro) |
| [provider-status.md](provider-status.md) | v0.1 — built (C1e, AI Components, Pro) |
| [training-run.md](training-run.md) | v0.1 — built (C1e, AI Components, Pro) |
| [tone-tuner.md](tone-tuner.md) | v0.1 — built (C1e, AI Components, Pro) |
| [ai-mascot.md](ai-mascot.md) | v0.1 — built (C1e, AI Components, Pro) |
| [image-reveal.md](image-reveal.md) | v0.1 — built (C1e, AI Components, Pro) |
| [ai-spark.md](ai-spark.md) | v0.1 — built (C1e, AI Components, Pro) |
| [pixel-mascot.md](pixel-mascot.md) | v0.1 — built (C1e, AI Components, Pro) |
| [ghost-mascot.md](ghost-mascot.md) | v0.1 — built (C1e, AI Components, Pro) |
| [cube-mascot.md](cube-mascot.md) | v0.1 — built (C1e, AI Components, Pro) |
| [upgrade-prompt.md](upgrade-prompt.md) | v0.1 — built (C1e, AI Components, Pro) |
| [referral-card.md](referral-card.md) | v0.1 — built (C1e, AI Components, Pro) |
| [trial-banner.md](trial-banner.md) | v0.1 — built (C1e, AI Components, Pro) |
| [live-visitors.md](live-visitors.md) | v0.1 — built (C1e, AI Components, Pro) |
| [social-proof-toast.md](social-proof-toast.md) | v0.1 — built (C1e, AI Components, Pro) |
| Every other component in [../component-taxonomy.md](../component-taxonomy.md) | to write in V2.2 |
