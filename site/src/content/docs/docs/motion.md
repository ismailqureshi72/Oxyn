---
title: Motion
description: Durations, easings, springs and reduced motion.
---

All motion reads its timing from `tokens/oxyn.motion.json`: durations (`dur.fast`, `dur.base`, `dur.travel`…), easings, springs, loop periods and staggers. The [Motion page](/motion/) lists the core tokens with their values and shows the live motion components.

## A shared clock

Components that animate over time take `now: ClockMs` from the host. One clock per window keeps loops in phase: two glyph loaders on the same screen turn together, and a paused loop resumes at the current phase instead of catching up.

## Reduced motion

- Every component honours `prefers-reduced-motion`.
- Several take a `reduced` prop for an explicit static form (text shimmer, verb cycler, text reveal, number ticker, attachments).
- This site has a **Reduce motion** switch in the footer; it only ever reduces.
