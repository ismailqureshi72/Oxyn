---
title: Ask before acting
description: Approval cards, risk tiers and checkpoints to undo.
---

Agents that send email, delete files or spend money should ask first, and make the answer quick and safe.

## Match the friction to the risk

The [approval card](/components/approvals/approval-card/) has four risk tiers:

- **Low**: Allow is the default button; one press.
- **Medium**: an optional countdown that pauses while you read.
- **High**: Allow *arms* first and Deny is the default, so a stray Enter can't approve.
- **Critical**: press and hold to allow.

It renders inline in a transcript, as a footer inside a tool call, as a terminal prompt or as a modal. Several open requests page through an [approval pager](/components/approvals/approval-pager/).

## Make it undoable

Approvals reduce mistakes; checkpoints fix the ones that get through. [Checkpoint markers](/components/checkpoints-git/checkpoint-marker/) sit between chat turns and restore files, the conversation or both; [checkpoint lanes](/components/checkpoints-git/checkpoint-lanes/) show a rewound run with each attempt in its own lane; a [restore toast](/components/checkpoints-git/restore-toast/) reports what happened with an undo window.
