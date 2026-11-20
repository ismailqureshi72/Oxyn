**Status:** built v0.1 (2026-11-20, S16b: Agent apps IV, part b).

# Spec: Hangar (`hangar`)

Family: Agent apps (pages at `/screens/hangar/` and `/full`). Reference: the agent-manager pattern (Google Antigravity's agent manager with its editor and browser surfaces and artifacts, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken. Brand marks appear only where a real model is named (Lobe Icons, credited in `THIRD_PARTY_NOTICES.md`), never as this app's logo, which is our own drawn mark.
Components: `HangarScreen` (Dioxus) · `HangarEl` (GPUI), rules in `oxyn_core::parity::hangar`.

## 1. Purpose
Hand a task to an agent, follow its task list, review the artifacts it makes (comment on the plan line by line before it goes on), and look at its work in the editor and the browser it drives.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name, Hide sidebar; **New conversation** ⌘N; **Inbox** (a toggle with the count of conversations waiting on review); Search conversations; conversations grouped by workspace (atlas-store, tidepool-docs) with a spinner (working), an amber dot (needs review), a green check (done) or a chat glyph, each with its ISO date. Account row with the account menu |
| Top bar (48) | Conversation title, a status pill (Working, Needs review, Done, Idle) and the Surfaces toggle |
| Conversation | Your prompt; the **Task list** card (2 of 5, each task with its state in text); **Artifacts** (Implementation plan, Walkthrough recording, Checkout at 390 px screenshot) as cards that open the matching surface; the agent's note; a shimmer while working |
| Composer | Text area, Add context, **Mode** (Planning · Fast), **Model** with coloured marks (Gemini, Claude, OpenAI), Send. Start screen: "What should the agent build?" with three suggestions |
| Surfaces pane (440) | Tabs **Editor** (the edited file with line numbers and + lines tinted green), **Browser** (the address, an "Agent driving" pill, the checkout at 390 px with its sticky footer, and the agent's actions with times) and **Plan** (the plan's lines as buttons; a picked line opens a comment box; comments sit under their line; **Proceed** with no comments, **Send n comments** with some) |

## 3. Behaviour
A working conversation finishes on its own after about 3 s. Inbox shows only conversations that wait on review. A comment needs text (at most 280 characters); Comment adds it under its line, Cancel closes the box. The review button reads Proceed / Send 1 comment / Send n comments; pressing it locks the plan and the agent's note changes. Artifact cards open the Plan or the Browser. Sending starts a conversation at the top of the first workspace. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty · **Artifact review** (the pane on Plan, open on narrow screens too) · **Browser surface** (the pane on Browser). Frame: Inset · Flat. Oxyn extras: `rail` and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; the open conversation has aria-current=page and its state in its label. Inbox is aria-pressed. Editor / Browser / Plan are a tablist with roving tabindex (← →). Each plan line is a button (aria-pressed when its box is open) labelled with its number and text; the comment box is a labelled group with errors in an alert. Mode and Model are menus of menuitemradio. Enter sends; Shift + Enter breaks the line.

## 6. Responsive
1440 × 900 stage; `/full` at the real viewport. Under 960 px the sidebar is a drawer and the pane opens over the conversation from the top bar. Under 620 px chip labels shorten and dates hide in rows. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`HangarScreen` takes `data: Option<HangarData>` (`oxyn_core::parity::hangar`): `convs: Vec<HgConv>` (id, title, workspace, status, at) and `notes: Vec<HgNote>` (line, text). Start from `HangarData::default()`.

## 8. Known gaps
No backend: the task list, artifacts, editor lines and browser actions are fixed fictional data; the walkthrough is a card, not a video. The GPUI view draws the plan's comments and Proceed as a press (`proceed`) for the host; comment boxes are Dioxus only.

## 9. GPUI
`AgentAppView::new(AgentApp::Hangar, window, cx)` (oxyn-gpui, `parity::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Hangar app, drawn from the same screen code as the snapshot element `HangarEl` and driven by `oxyn_core::parity::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views (the conversation with the Editor, the Plan, the Browser) and Escape closes the menu, then the rail. App presses: Editor, Plan and Browser switch the pane (`view:`, `view:artifact`, `view:browser`); Proceed fires `proceed`; a row press returns to the Editor. Call `bind_ime_keys(cx)` once at start; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` (from the repo root) opens every agent app in one window.
