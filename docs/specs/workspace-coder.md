**Status:** built v0.1 (2026-11-19, S11: Agent apps III).

# Spec: Workspace Coder (`workspace-coder`)

Family: Agent apps (pages at `/screens/workspace-coder/` and `/full`). Reference: the chat-plus-computer pattern of agent apps (Kimi's agent mode and others, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken.
Components: `WorkspaceCoderScreen` (Dioxus) · `WorkspaceCoderEl` (GPUI), rules in `oxyn_core::parity::wscoder`.

## 1. Purpose
Ask for something to be researched and built, watch the agent work in its own computer, then preview and share what it made.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name, Hide sidebar; **New chat** ⌘K; Search chats; chats under Today and Earlier with their mode glyph (a spinner while working). Account row with the account menu |
| Top bar (48) | Chat title and a pill (Working, or the chat's mode) and the Workspace toggle |
| Conversation | Your prompt; the agent's opening line; **Progress** (a disclosure: six todos, done green, running spinner, to do ring, "n of 6"); the tool log (Searched, Read, Wrote, Ran with the object in mono); while working a shimmer "Building the page · step n of 6"; when done the summary and a deliverable card (thumbnail, title, "Website · 5 files", Preview) |
| Composer | Text area, attach, **Mode** (Chat · Researcher · Workspace), **Thinking** toggle, **Model** with the official coloured marks, Stop while working, Send. Start screen: "What can I build for you?" with three suggestions that fill the composer |
| Workspace pane (440) | Header "Workspace" with live status (Writing app.js, or Idle); tabs **Files** (file list + viewer with line numbers), **Terminal** (commands bold, output, a blinking cursor while working), **Browser** (address and results), **Preview** (Desktop / Phone toggle, Publish, the built page: title, bars per city with the cheapest in green) |

## 3. Behaviour
The open chat finishes on its own after about 3 s: Progress completes, the last log line appears and the deliverable card shows. Preview opens the pane on Preview (over the chat when narrow). Publish turns into Published with the link (`move-budget.pages.example`). Sending starts a chat in the chosen mode (Workspace chats start working). A blank or too-short prompt shows an inline error. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty · **Preview** (finished, pane on Preview) · **Files** (finished, pane on Files with app.js open). Frame: Inset · Flat. Oxyn extras: `rail` and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; the open chat has aria-current=page. The Workspace tabs are a tablist with roving tabindex (← →). Progress is a labelled group with an expandable header. Desktop / Phone are aria-pressed toggles; Thinking too. The composer sends on Enter and breaks the line on Shift + Enter.

## 6. Responsive
1440 × 900 stage; `/full` at the real viewport. Under 960 px the sidebar is a drawer and the Workspace opens over the chat. Under 620 px the Thinking label hides. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`WorkspaceCoderScreen` takes `data: Option<WorkspaceCoderData>` (`oxyn_core::parity::wscoder`): `chats: Vec<Chat>` (id, title, mode, running, day), `files: Vec<WsFile>` (path and lines; Files lists them and Preview reads the CSV), `deliverable: String`. Start from `WorkspaceCoderData::default()`.

## 8. Known gaps
GPUI is a static snapshot (`.extra("preview" | "files" | "rail" | "hidden")`). No backend: progress, log, terminal, browser and the page are fixed fictional data; Publish only shows the link.
