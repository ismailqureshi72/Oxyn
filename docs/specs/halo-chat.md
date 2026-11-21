**Status:** built v0.1 (2026-11-20, S16b: Agent apps IV, part b).

# Spec: Halo Chat (`halo-chat`)

Family: Agent apps (pages at `/screens/halo-chat/` and `/full`). Reference: the desktop assistant pattern (the Gemini desktop app: chat with tool cards, a mini chat from a shortcut, Deep research plans, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken. Brand marks appear only where a real model is named (Lobe Icons, credited in `THIRD_PARTY_NOTICES.md`), never as this app's logo, which is our own drawn mark.
Components: `HaloChatScreen` (Dioxus) · `HaloChatEl` (GPUI), rules in `oxyn_core::kit::halochat`.

## 1. Purpose
Ask an assistant anything, see what it searched, ran and built as cards, keep sources and a canvas draft beside the chat, ask quick questions from a mini chat over any app, and shape a research plan before it runs.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name, Hide sidebar; **New chat** ⌘N; **Helpers** (Trip planner, Code reviewer, Study partner as toggles); **Pinned** and **Recent** chats with ISO dates. Account row with the account menu |
| Top bar (48) | Chat title, the active helper as a pill, **Pin** (aria-pressed) and **Mini chat** |
| Chat | Your prompt; tool cards that open and close: **Searched the web** (the query and five sources), **Ran code** (Python and its output, dollars per km of range from the core rule), **Made a table** (bike, price, range, torque, $ per km); the answer; Copy, Regenerate and **Open in Canvas** |
| Composer | Text area, Add files or share a window, **Tools** (No tool, Deep research, Canvas, Create image, Run code), **Mode** (Fast · Thinking · Deep think), **Model** with coloured marks (Gemini, Claude), Dictate, Send. Start screen: "Hello, Rowan" with three suggestions |
| Pane (440) | Tabs **Sources** (numbered, with site and ISO date) and **Canvas** (the shortlist draft with Copy and Share) |
| Mini chat | A small dialog over a striped desktop: the app mark, the shortcut (Alt + Space), Open full chat, one exchange and a one-line input |

## 3. Behaviour
Pin moves a chat between Pinned and Recent (pinned first, newest first). Helpers toggle; the active one shows in the bar. Sending starts a chat at the top of Recent and closes the mini chat. In Research plan, steps switch on and off, the estimate is 2 min per step (at least 3), Start research with no step on shows an error, and starting locks the plan. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty · **Mini chat** · **Research plan** (Deep research on, the plan in the chat). Frame: Inset · Flat. Oxyn extras: `rail` and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; the open chat has aria-current=page; helpers and Pin are aria-pressed. Tool card headers are buttons with aria-expanded; the table has a header row. Sources / Canvas are a tablist with roving tabindex (← →). Plan steps are labelled checkboxes. The mini chat is a labelled dialog. Tools, Mode and Model are menus of menuitemradio. Enter sends; Shift + Enter breaks the line.

## 6. Responsive
1440 × 900 stage; `/full` at the real viewport. Under 960 px the sidebar is a drawer and the pane opens over the conversation from the top bar. Under 620 px chip labels shorten and dates hide in rows. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`HaloChatScreen` takes `data: Option<HaloChatData>` (`oxyn_core::kit::halochat`): `chats: Vec<HcChat>` (id, title, pinned, at). Start from `HaloChatData::default()`.

## 8. Known gaps
No backend: the search, code output, table, sources and canvas are fixed fictional data; the mini chat is drawn inside the app frame, not as a separate OS window; Create image and Run code only set the tool. The GPUI view draws Start research and Open full chat as presses (`start`, `full`).

## 9. GPUI
`AgentAppView::new(AgentApp::HaloChat, window, cx)` (oxyn-gpui, `kit::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Halo Chat app, drawn from the same screen code as the snapshot element `HaloChatEl` and driven by `oxyn_core::kit::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views (the chat, the research plan, the mini chat) and Escape closes the menu, then the rail. App presses: Start research fires `start`; Open full chat fires `full` and returns to the chat; a row press returns to the chat. Call `bind_ime_keys(cx)` once at start; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` (from the repo root) opens every agent app in one window.
