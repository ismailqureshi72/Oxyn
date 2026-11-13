---
title: Build a composer
description: Send, stop, queue and steer, with model, effort and context in reach.
---

The [composer](/components/composer/composer/) is the most-used surface in an agent app. Oxyn's comes in six layouts (chat, desktop, IDE panel, terminal, mobile and node) and is built from parts you can also use on their own.

- **[Send control](/components/composer/send-control/)**: send when idle, stop while working, queue or steer when you type during a run. Twelve styles.
- **[Model picker](/components/composer/model-picker/)**: providers, favourites, search, models that need a key or are rate limited, and a change that waits for the next turn.
- **[Effort control](/components/composer/effort-control/)**: how hard the model thinks; a mid-run change applies to the next message.
- **[Context meter](/components/composer/context-meter/)**: how full the context window is, with a breakdown and a compact action.
- **[Attachments](/components/composer/attachments/)**: upload progress, failures with retry and context chips for files, repos, selections and URLs.

While the agent works, Enter queues the draft (or steers, if you choose `BusyAction::Steer`). When sending is off (offline, quota, policy) the composer says why and keeps the draft.
