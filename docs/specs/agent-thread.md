**Status:** built v0.2 (2026-11-18, S3: Agent ops I; v0.1 screens 2).

# Spec: Agent · Thread (`agent-thread`)

Family: Screens. Reference: affectUI Agent / Thread (`/screens/agent-thread`), layout and behaviour only; no source or assets copied.
Components: `AgentThreadScreen` (Dioxus) · `AgentThreadEl` (GPUI), models in `oxyn_core::parity::agentscr`.

## 1. Purpose
One running conversation with the orchestrator, which splits work into tasks, runs them side by side and only interrupts when a task needs a call.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Top right | "9 running · 5 need you" counts, then the one work-area toggle (Artifacts, folded by default, 420 px when open) |
| Thread | A centred 680 px column: your messages as grey bubbles on the right; the Orchestrator with its pixel mark and time; "Ran 3 tools · 2.1s" lines with a green check that unfold into the calls; task cards with a progress ring, step, n/m and Open; file chips with toned glyphs |
| Composer | A 680 px card: the text area, then + (attach), Request approval (a toggle), the mic and a black Send that turns into Stop while a reply streams |

Chrome is monochrome neutral; colour only means something: official app marks, file-type glyphs, project hues (purple, blue, green, orange) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Send streams a reply word by word after a short Thinking and lands it in the thread; Stop keeps the words so far. Tool lines unfold. The composer + uploads a file with progress; the mic dictates with a waveform and timer and Done drops the words into the message. Offline queues messages until Reconnect. Decision answers by click or the 1–3 keys. Error offers Retry. Scrolled up shows a "2 new · Jump to latest" pill that scrolls down. Away summary sits on top and can be dismissed.
prefers-reduced-motion: no glide, spin, waveform or pulse.

## 4. Variants
States (affectUI's nine, then Empty): **Live** · **Decision** · **Scrolled up** · **Offline** · **Uploading** · **Dictating** · **Away summary** · **Error** · **Loading** · Empty. Frame: Inset · Flat. Oxyn extras: `tools_open` (tool lines start unfolded) and `stamps` (a time on each of your messages).

## 5. Keyboard and accessibility
The thread is role=log and focusable; Enter sends, Shift+Enter breaks a line; the decision card takes 1–3 (aria-keyshortcuts); tool lines report aria-expanded; the upload is a progressbar; Request approval and the mic are toggle buttons with aria-pressed; every result is announced in a polite live region.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/agent-thread/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button. Under 520 px the column and composer take the full width, Request approval and the count words keep only their icons and numbers. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional.
In a window, `ScreenView` (oxyn-gpui, `parity::screenview`) runs this screen live from `oxyn_core::parity::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
