**Status:** built v0.1 (2026-11-20, S16b: Agent apps IV, part b).

# Spec: Courier (`courier`)

Family: Agent apps (pages at `/screens/courier/` and `/full`). Reference: the thread-first coding-agent pattern (the Amp app: a sidebar of all active threads, suggested replies, model and effort, a cloud sandbox per thread with Desktop / Portals / Changes, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken. Brand marks appear only where a real model is named (Lobe Icons, credited in `THIRD_PARTY_NOTICES.md`), never as this app's logo, which is our own drawn mark.
Components: `CourierScreen` (Dioxus) · `CourierEl` (GPUI), rules in `oxyn_core::parity::courier`.

## 1. Purpose
Run several coding agents as threads, see which ones need you, review each turn's edits in one card, reply with a suggestion, and look into the thread's own cloud machine (its desktop, ports and changes).

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name, Hide sidebar; **New thread** ⌘N; **All · Running · Needs you** (a radiogroup); threads with a spinner (running), an amber dot (needs you) or a green check (done), the repository and ISO time. Account row with the account menu |
| Top bar (48) | Thread title, a status pill and the Pod toggle |
| Thread | Your prompt; tool lines (Searched, Read, Ran); the **edits card** ("Edited 3 files +71 −3", Review) with each file as a button that opens its diff; the agent's closing line. A thread that needs you shows the agent's question |
| Composer | **Suggested replies** above it (three buttons that fill the box); text area, Attach, **Model** with coloured marks (Claude, OpenAI, Gemini), **Effort** (Low · Medium · High), Send. Start screen: "Start a thread" with three suggestions |
| Pod pane (440) | Header with the pod's name, size and a Running pill; tabs **Desktop** (the pod's browser on the product page with the cached price), **Ports** (3000, 5432, 6006 with Open or Forward) and **Changes** (files with +/− and **Revert**, totals) |

## 3. Behaviour
Filters show all, running or waiting threads. A running thread finishes on its own after about 3 s. Revert removes a file from Changes and from the edits card (core `revert`, `cr_totals`). Forward opens a port. Review opens the pod on Changes. Sending starts a thread at the top and resets the filter. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty · **Changes** (the pod on Changes) · **Pod desktop** (the pod on Desktop). Frame: Inset · Flat. Oxyn extras: `rail` and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; filters are a radiogroup; the open thread has aria-current=page and its state in its label. Edit-card files are buttons with aria-expanded. Suggested replies are a labelled group. Desktop / Ports / Changes are a tablist with roving tabindex (← →). Model and Effort are menus of menuitemradio. Enter sends; Shift + Enter breaks the line.

## 6. Responsive
1440 × 900 stage; `/full` at the real viewport. Under 960 px the sidebar is a drawer and the pane opens over the conversation from the top bar. Under 620 px chip labels shorten and dates hide in rows. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`CourierScreen` takes `data: Option<CourierData>` (`oxyn_core::parity::courier`): `threads: Vec<CrThread>` (id, title, status, repo, at) and `files: Vec<CrFile>` (path, add, del, diff). Start from `CourierData::default()`.

## 8. Known gaps
No backend: threads, edits, ports and the desktop are fixed fictional data; the desktop is drawn, not streamed. The GPUI view's pod opens on Ports, draws Revert as a press (`revert`) and fills the composer from a suggestion (`suggest-N`).

## 9. GPUI
`AgentAppView::new(AgentApp::Courier, window, cx)` (oxyn-gpui, `parity::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Courier app, drawn from the same screen code as the snapshot element `CourierEl` and driven by `oxyn_core::parity::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views (the thread with Ports, Changes, Desktop) and Escape closes the menu, then the rail. App presses: Ports, Changes and Desktop switch the pane (`view:`, `view:changes`, `view:desktop`); Review fires `view:changes`; a suggested reply fills the composer (`suggest-N`); Revert fires `revert`. Call `bind_ime_keys(cx)` once at start; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` (from the repo root) opens every agent app in one window.
