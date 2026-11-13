---
title: Accessibility
description: How Oxyn components handle keyboard, screen readers, contrast and motion.
---

Each component page has **Keyboard** and **Accessibility** sections, and quotes the accessibility section of its spec. The rules all components share:

- **Real semantics.** Buttons are buttons; switches are `role=switch`; menus are `role=menu`; selects follow the combobox and listbox pattern; trees are `role=tree`.
- **Names that say the state.** "Stop generating", "Running cargo test, 12 seconds", "Coder, thinking".
- **Calm announcements.** State changes go through polite live regions with throttling; only failures and approvals that block the user are assertive. Streamed tokens are never announced one by one.
- **Not colour alone.** Status and risk always carry text or shape too.
- **Focus is never lost.** Loading buttons keep focus; disabled controls with a reason stay focusable and explain why.
- **Reduced motion** is honoured everywhere (see [Motion](/docs/motion/)).
