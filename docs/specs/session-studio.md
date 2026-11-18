**Status:** built v0.1 (2026-11-18, S10: Agent apps II).

# Spec: Session Studio (`session-studio`)

Family: Agent apps (pages at `/screens/session-studio/` and `/full`). Reference: the session-and-tools pattern of desktop coding-agent apps (the Claude Code desktop app and others, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken.
Components: `SessionStudioScreen` (Dioxus) · `SessionStudioEl` (GPUI), rules in `oxyn_core::parity::cmdstudio`.

## 1. Purpose
Work with a coding agent in a session: see every file it reads and edits and every command it runs, answer its permission prompts, plan first when you want to, and check the diff.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name, Hide sidebar; New session ⌘N; Sessions: status mark, title, "folder · age". Account row |
| Top bar (48) | Session title, folder chip, the Diff toggle with its total |
| Chat | Your messages as bubbles; replies as text; tool blocks with a header (glyph, **Read** / **Edit** / **Bash**, path or `$ command` in mono, lines read or a spinner): Edit shows the inline diff, Bash its output on the inset grey |
| Permission prompt | Inside the Bash block, amber edge: "Run this command?" with **Allow once** (1), **Always allow** (2) and **Deny** (3) |
| Plan | In Plan mode a reply is a numbered plan with **Approve plan** and Keep planning; approving turns it into the todo list and switches to Accept edits |
| Todos | A disclosure above the composer, "Todos d/n": checkboxes, the running one with a spinner, the waiting one with an amber dot, done ones struck through |
| Composer | Text area; the **permission mode** picker (Ask before edits, Accept edits, Plan mode; the border dashes in Plan mode), the model picker with the official coloured mark, Send |
| Side panes (440) | A bar of pane chips (Diff, Side chat, Context: show or hide each) and **Rearrange**. The panes stack: **Diff** (the shared review pane: files, hunks, Accept / Revert per hunk and file, Accept all; each session its own changes), **Side chat** (ask about the session without interrupting it: messages, a field and Ask) and **Context** (folder, mode, context used meter, files changed); each has × to hide |
| Rearrange | The panes become dashed cards with a grip; drag one onto another to move it there (native pointer drag; the target highlights), or use ↑ ↓ on each card; Alt + ↑ / ↓ inside a pane moves it outside Rearrange too. Done keeps the order |

## 3. Behaviour
In Data the test command runs and finishes after about 3 s; in **Permission** it waits for you. Allow once or Always allow runs it (9 tests pass), ticks the todo and the session replies; Deny skips it and says so. Shift + Tab in the composer cycles the permission mode. Sending in Plan mode returns a plan to approve; otherwise the session answers that it is on it. A blank or too-short message shows an inline error. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty (start: "What are we changing today?" with Local, folder and mode) · **Permission** · **Side chat** (the side chat pane shown under the diff) · **Rearrange** (the panes as draggable cards). Frame: Inset · Flat. Oxyn extras: `rail` and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; the open session has aria-current=page. Permission prompts and plans are labelled groups; with focus in a prompt, 1, 2 and 3 answer it (Allow once, Always allow, Deny). Todos are native checkboxes. Pickers are menus of menuitemradio closing on Escape.

## 6. Responsive
As Command Center: drawer sidebar and the diff pane over the chat under 960 px; shorter chips under 620 px; no text overlap at 1440, 768 and 390 in light and dark (scripts/ovl.py).

## 7. Known gaps
GPUI is a static snapshot (`.extra("permission" | "rail" | "hidden")`). No backend: the blocks, test output, plan and diff are fictional, and other sessions open empty.
