**Status:** built v0.1 (2026-11-20, S16a: Agent apps IV).

# Spec: Preview Desk (`preview-desk`)

Family: Agent apps (pages at `/screens/preview-desk/` and `/full`). Reference: the agent-previews pattern (Factory's desktop app: sessions, preview tabs, Design mode and inline diff comments, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken. Brand marks appear only where a real model, agent or code host is named (Lobe Icons / Simple Icons, credited in `THIRD_PARTY_NOTICES.md`), never as this app's logo.
Components: `PreviewDeskScreen` (Dioxus) · `PreviewDeskEl` (GPUI), rules in `oxyn_core::kit::previewdesk`.

## 1. Purpose
See what a coding agent made as a running preview, point at an element or a line and tell the agent what to change, then approve.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name; **New session** ⌘N; **Sessions** with a spinner or chat glyph, the title, the machine (this computer or a persistent one) and an amber count of open comments. Account row |
| Top bar (48) | Session title and Working / Ready to review; the Preview toggle |
| Session | Your prompt, the agent's steps (Read, Wrote, Edited, Ran, Opened), while working a shimmer, then the summary with **Open preview** and **Review the diff**. Composer: **Machine** (This Mac · build-box-2 · gpu-box-1), the working folder, **Model** with coloured marks, Stop, Send |
| Preview pane (440) | Tabs **localhost:3000** (the address and **Design mode**; the live pricing page, always light, with a working Monthly / Yearly switch and three plan cards, Team featured), **rollout.md** (the doc with ISO dates), **Diff** (PlanCard.tsx with line numbers, + and − tinted; a line press opens a comment box under it; comments sit under their line; **Approve changes**) |
| Design mode | Elements outlined dashed; a press picks one (solid outline) and opens "Comment on <element>" with Cancel and **Send to agent**; sent comments list as "Comments for the agent" and count on the session row |

## 3. Behaviour
The open session finishes after about 3 s. Design mode on: pick the heading, the switch, the Team card or its Start trial button and comment on it. A blank comment shows an inline error. Diff: press a line, comment, send; Approve changes confirms. Sending starts a session on the chosen machine. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty · **Design mode** (Team card picked, one comment) · **Diff review** (the diff with a comment on line 9). Frame: Inset · Flat. Oxyn extras: `rail` and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; the open session has aria-current=page. Preview tabs are a tablist (← →). Design mode is an aria-pressed toggle. Each diff line is a button named with its number and text. The comment box is a labelled group; Enter sends it. Enter sends the composer; Shift + Enter breaks the line.

## 6. Responsive
1440 × 900 stage; `/full` at the real viewport. Under 960 px the sidebar is a drawer and the pane opens over the conversation from the top bar. Under 620 px chip labels shorten and rows stack. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`PreviewDeskScreen` takes `data: Option<PreviewDeskData>` (`oxyn_core::kit::previewdesk`): `sessions: Vec<PdSession>` (id, title, machine, running, notes), `folder: String`. Start from `PreviewDeskData::default()`.

## 8. Known gaps
No real browser or machine: the site, doc and diff are fixed fictional data drawn by Oxyn. In the GPUI view Design mode shows the picked card and its comment; picking other elements and typing comments are Dioxus-only for now.

## 9. GPUI
`AgentAppView::new(AgentApp::PreviewDesk, window, cx)` (oxyn-gpui, `kit::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Preview Desk app, drawn from the same screen code as the snapshot element `PreviewDeskEl` and driven by `oxyn_core::kit::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views (the session, Design mode, the diff) and Escape closes the menu, then the rail. App presses: the Site / Diff tabs and Design mode (`view:`, `view:design`, `view:diff`); a row returns to the session. Call `bind_ime_keys(cx)` once at start; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` (from the repo root) opens every agent app in one window.
