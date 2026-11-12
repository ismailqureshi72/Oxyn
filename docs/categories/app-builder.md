**Status:** draft v0.1 (2026-11-12).

# Category kit: App builder

A prompt-to-app agent: the user describes an app, the agent writes it, runs it in a live preview,
fixes its own failures and publishes it. This kit is the **App builders** row of
[../component-taxonomy.md](../component-taxonomy.md) section B ("Prompt → preview split, version
history, deploy panel"; "web preview frame, element picker, 'generating UI' wireframe, publish flow").
It follows the 10-section standard in [README.md](README.md). Registry ids: `shell/app-builder-<screen>`
(family M). Values marked *proposed* are Oxyn's own starting points, to be tuned in the Studio.

Everything named below (product, people, business, hosts, providers) is **invented** for demos.

## 1. Who it's for

| | |
|---|---|
| **Job** | "Turn my idea into a working web app I can put in front of customers, without me writing code, and let me change it by pointing at it." |
| **Primary user** | Non-developer founder or designer; a developer collaborator drops into the code view now and then |
| **What they fear** | Losing a working version; shipping something broken; a key leaking; being charged or going live by accident |
| **What the UI must prove** | What is being built right now, what changed, what is live, and that nothing reaches production or money without a hold-to-confirm |

**Demo world (fictional).**

| Thing | Name | Notes |
|---|---|---|
| App-builder product | **Brickwell** | The harness app this kit ships as; its agent is called **Brick** |
| User | **Ines Okafor** | Owner-baker, first-time builder |
| Business | **Quillby Bakehouse** | Small bakery that sells out on weekends |
| App being built | **Quillby Preorders** | Weekend menu, pickup-slot booking, card payment, order emails, a staff page |
| Collaborator | **Tomás Reyes** | Freelance developer, edits code twice a month |
| Payments provider | **PayRail** | Invented; keys `PAYRAIL_SECRET_KEY`, `PAYRAIL_WEBHOOK_SECRET` |
| Email provider | **Mailkite** | Invented; key `MAILKITE_API_KEY` |
| Hosts | `quillby-preorders.brickwell.example` (default) · `order.quillby.example` (custom) | `.example` TLD so no real host is shown |

## 2. Shell

Three panes in one window: **conversation** (left), **workspace** (right, tabbed: Preview, Code,
Database, Env, Deploy, History) and a collapsible **console drawer** under the workspace. Built on
`panel-split` ([../specs/workspace-chrome.md](../specs/workspace-chrome.md)) with the
`genui-surface/split-preview` layout idea ([../specs/genui-renderer.md](../specs/genui-renderer.md)).

### 2.1 Desktop (GPUI)

```text
┌ ● ● ●  Quillby Preorders ▾   ⎇ v14 · Draft   ◐ Brick is building…        👥2  Share  [Publish ▾]┐
├─────────────────────────────────┬────────────────────────────────────────────────────────────────┤
│ CONVERSATION                    │ [Preview] Code  Database  Env  Deploy  History           ⌘1–6  │
│                                 │ ┌ ⟳  ‹ ›  /menu                    🖥 ▭ ▯  ⌖ Select  ⤢  ↗ ┐ │
│  Ines: Add a pickup-slot step   │ │                                                          │ │
│  after the cart, 15-min slots,  │ │   QUILLBY BAKEHOUSE            Menu  Pickup  Cart (2)    │ │
│  Sat and Sun only.              │ │   ┌────────────┐ ┌────────────┐ ┌────────────┐           │ │
│                                 │ │   │ Sourdough  │ │ Cardamom   │ │ Rye loaf   │           │ │
│  Brick                          │ │   │ €6.50  [+] │ │ bun  €3.20 │ │ €5.80  [+] │           │ │
│  ▾ Plan · 4 steps    ✓✓◐○       │ │   └────────────┘ └────────────┘ └────────────┘           │ │
│  ✓ Edit  src/cart.rs   +42 −3   │ │                                                          │ │
│  ✓ Edit  src/slots.rs  +118     │ │   ┌ GENERATING · PickupSlots ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐        │ │
│  ◐ Run   cargo build   0:07     │ │   ╎ ░░░░░░░░░  ░░░░░░░░░  ░░░░░░░░░             ╎        │ │
│    │ Compiling quillby v0.3.0   │ │   └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘        │ │
│                                 │ └──────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────┐ ├─ Console ── 0 errors · 2 warnings ── Logs  Problems  ▾ ─────┤
│ │ @PickupSlots  Ask Brick…    │ │ 12:04:11  ready on :5173                                     │
│ │ + Build ▾  Auto-edit ▾   ■  │ │ 12:04:12  warn: slot list has no key                         │
│ └─────────────────────────────┘ │                                                              │
├─────────────────────────────────┴────────────────────────────────────────────────────────────────┤
│ ● Dev  ⎇ main  v14 (draft)   Live: v12 · 2 h ago   ◷ 0:41   ⛁ 1.2k tok · €0.04       Problems 0  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

| Region | Parts | Spec |
|---|---|---|
| Top bar | project switcher, version chip, run status, presence, Share, Publish split-button | `git-chips/pill` ([git-chips](../specs/git-chips.md)), `acting-indicator/chip` ([acting-indicator](../specs/acting-indicator.md)), `presence-stack` (new, §6) |
| Conversation | transcript, plan, tool rows, composer | [message-bubble](../specs/message-bubble.md), [plan-todo](../specs/plan-todo.md), [tool-call](../specs/tool-call.md), [composer](../specs/composer.md) |
| Workspace tabs | Preview · Code · Database · Env · Deploy · History | `side-panel/tabs` ([side-panels](../specs/side-panels.md)) |
| Preview toolbar | reload, back / forward, route field, device toggles, Select, fullscreen, open in tab | `preview-frame` (new, §6) |
| Console drawer | Logs (live tail), Problems (errors with "Fix with Brick") | [terminal-output](../specs/terminal-output.md), `problem-list` (new) |
| Status bar | environment, branch, draft vs live version, elapsed, run meter, problem count | `status-bar/run-strip` ([side-panels](../specs/side-panels.md)), `run-meter/footer-line` ([run-meter](../specs/run-meter.md)), `env-badge` (new) |

Rules (proposed):
1. **The preview never goes blank while the agent works.** It keeps the last good build; regions being
   rewritten show the `genui-surface/blueprint-build` wireframe over them (§3.3).
2. **Live is always visible.** The status bar always names the live version ("Live: v12"), so a draft
   is never mistaken for production.
3. **One Publish button**, top-right, with the target in its label once chosen ("Publish to
   production"). It is the only entry to the Critical publish card (§4 F4).
4. The composer stays docked bottom-left in every tab, so the user can steer from Code or Database.

### 2.2 Web (Dioxus)
Same layout. Differences: the preview is an `<iframe>` with `sandbox` and a strict origin (the picker
talks over `postMessage`); the title bar becomes a 48 px app bar; panel sizes persist in local storage;
"Open in tab" opens the dev URL, never the production one.

### 2.3 Narrow and mobile

```text
 Tablet 600–959 px                          Phone < 600 px
┌ Quillby Preorders ▾   [Chat|Preview] ⋯ ┐  ┌ ≡  Quillby Preorders   ⋯ ┐
│ ┌ ⟳  /menu         ▯ ▭  ⌖  ⤢         ┐ │  │ ┌ preview (phone frame) ┐ │
│ │                                    │ │  │ │  QUILLBY BAKEHOUSE    │ │
│ │  preview, full width               │ │  │ │  Sourdough   €6.50 +  │ │
│ │                                    │ │  │ │  ┌ GENERATING ─ ─ ┐   │ │
│ └────────────────────────────────────┘ │  │ │  ╎ ░░░░░░░░░░░   ╎   │ │
│ ◐ Building · Run cargo build 0:07   ▴ │  │ └───────────────────────┘ │
│ ┌──────────────────────────────────┐   │  │ ◐ Building… 0:07   Chat ▴ │
│ │ Ask Brick…                    ■  │   │  │ ┌ Ask Brick…          ■ ┐ │
│ └──────────────────────────────────┘   │  │ └───────────────────────┘ │
└────────────────────────────────────────┘  └───────────────────────────┘
```

| Breakpoint | Layout | Notes |
|---|---|---|
| ≥ 1280 px | three panes as 2.1; conversation 360–480 px | console drawer open by default |
| 960–1279 px | conversation collapses to a 48 px rail (`panel-split/rail-snap`) on demand | workspace tabs become an overflow menu after 4 |
| 600–959 px | segmented **Chat / Preview** switch; composer docked under both | running step shows as a one-line strip above the composer |
| < 600 px | preview first; chat is a bottom sheet (`panel-split/mobile-sheets`) | Code, Database, Env open as full-screen sheets; Publish lives in `⋯` and still opens the Critical card |

## 3. Screens

Every screen lists its Oxyn parts. "(new)" points to §6.

### 3.1 Home and project gallery (dashboard) — `shell/app-builder-home`

```text
┌ Brickwell                                   ⌘K Search projects        Ines ▾ ┐
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ What do you want to build?                                               │ │
│ │ A preorder page for my bakery with pickup slots and card payments…       │ │
│ │ + Attach   Template ▾   Build ▾                                     ↑   │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│  Recent                                              Grid ▦  List ☰  Sort ▾  │
│ ┌ thumb ──────────┐ ┌ thumb ──────────┐ ┌ thumb ──────────┐ ┌ + New ──────┐ │
│ │ Quillby Preord. │ │ Staff Rota      │ │ Wholesale form  │ │  Blank       │ │
│ │ ● Live v12  2h  │ │ ◐ Building 0:41 │ │ ✕ Build failed  │ │  From temp.  │ │
│ │ 👥 2            │ │                 │ │   Fix with Brick│ │              │ │
│ └─────────────────┘ └─────────────────┘ └─────────────────┘ └──────────────┘ │
│  Activity  ✓ v12 published to production · 2 h   ⟲ Staff Rota rolled back …  │
└──────────────────────────────────────────────────────────────────────────────┘
```

| | |
|---|---|
| Purpose | Start a new app from a prompt or template; see which projects are live, building or broken |
| Parts | hero composer `composer-input/hero-dock` ([composer-input-shell](../specs/composer-input-shell.md)), `attachment/chip-bar` ([composer-attachments](../specs/composer-attachments.md)), `project-card` (new), `background-jobs/popover` for builds running elsewhere ([background-jobs](../specs/background-jobs.md)), activity list `notification-center/timeline` ([command-palette](../specs/command-palette.md)), `command-palette/centered` for ⌘K |
| Empty | No projects: hero composer + 3 template chips + "Start blank"; no illustration needed |
| Loading | `genui-skel/card-shimmer` cards (`oxyn-skel-02`, [genui-skeletons](../specs/genui-skeletons.md)), 6 tiles |
| Error | Gallery fetch failed: inline banner with Retry; cached thumbnails stay, marked "offline" |

### 3.2 Template picker — `shell/app-builder-templates`

```text
┌ Start from a template                                         Search ⌕  ✕  ┐
│ All · Store · Booking · Internal tool · Landing · Form                      │
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐   ┌ Preview ───────────┐ │
│ │ ▓▓ thumb ▓▓  │ │ ▓▓ thumb ▓▓  │ │ ▓▓ thumb ▓▓  │   │ 🖥 ▭ ▯              │ │
│ │ Preorder shop│ │ Slot booking │ │ Staff portal │   │  live preview of   │ │
│ │ DB · Payments│ │ DB · Email   │ │ DB · Auth    │   │  selected template │ │
│ └──────────────┘ └──────────────┘ └──────────────┘   │                    │ │
│                                                       └────────────────────┘ │
│ Needs: database, PAYRAIL_SECRET_KEY (you'll add it later)   [Use template] │
└──────────────────────────────────────────────────────────────────────────────┘
```

| | |
|---|---|
| Purpose | Pick a starting point and see what it needs (database, secrets) before choosing |
| Parts | filter chips `genui-question/chips` ([genui-forms](../specs/genui-forms.md)), `template-card` (new), right pane `preview-frame` (new) in read-only mode, needs line from `env-badge` + secret names (never values) |
| Empty | Filter returns nothing: "No templates for *Form* yet · Describe it instead" opens the hero composer with the query |
| Loading | Thumbnails use `skeleton/shimmer`; live preview shows `gen-placeholder/blueprint` ([media-generation-placeholder](../specs/media-generation-placeholder.md)) until the template boots |
| Error | Template preview fails to boot: static thumbnail + "Preview unavailable · Use anyway" |

### 3.3 Builder (prompt + live preview) — `shell/app-builder-builder`

The shell in 2.1 with the Preview tab active. Three preview phases:

```text
 First build (nothing yet)         Patching (region rewritten)       Ready
┌ ⟳ /                    🖥 ▭ ▯ ┐ ┌ ⟳ /menu               🖥 ▭ ▯ ┐ ┌ ⟳ /menu              🖥 ▭ ▯ ┐
│ ┌ HEADER  1280×64 ─ ─ ─ ─ ─ ┐ │ │ QUILLBY        Menu  Cart     │ │ QUILLBY       Menu  Cart     │
│ ╎ ░░░░░░        ░░  ░░  ░░  ╎ │ │ ┌──────┐ ┌──────┐ ┌──────┐    │ │ ┌──────┐ ┌──────┐ ┌──────┐   │
│ └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘ │ │ │ loaf │ │ bun  │ │ rye  │    │ │ │ loaf │ │ bun  │ │ rye  │   │
│ ┌ GRID · 3 CARDS ─ ─ ─ ─ ─ ─┐ │ │ └──────┘ └──────┘ └──────┘    │ │ └──────┘ └──────┘ └──────┘   │
│ ╎ ░░░░░░  ░░░░░░  ░░░░░░    ╎ │ │ ┌ PICKUP SLOTS ─ ─ ─ ─ ─ ┐    │ │ Pickup  Sat 09:00 09:15 …    │
│ └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘ │ │ ╎ ░░░░  ░░░░  ░░░░       ╎◐   │ │          ✦ changed           │
│ AGENT.LOG // writing App.rs   │ │ └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘    │ │                              │
└───────────────────────────────┘ └───────────────────────────────┘ └──────────────────────────────┘
```

| | |
|---|---|
| Purpose | Watch the app take shape; steer mid-run; see what changed |
| Parts (conversation) | `message/flat` ([message-bubble](../specs/message-bubble.md)), `reasoning-block/inline-summary` ([reasoning-block](../specs/reasoning-block.md)), `plan-card/versioned` + `todo-list/checklist` ([plan-todo](../specs/plan-todo.md)), `tool-call/row` ([tool-call](../specs/tool-call.md)), `file-edit-chip/row` ([file-edit-chip](../specs/file-edit-chip.md)), `terminal-output/tail` ([terminal-output](../specs/terminal-output.md)), `verb-cycler/fade` + `elapsed-timer/summary` ([verb-cycler](../specs/verb-cycler.md), [elapsed-timer](../specs/elapsed-timer.md)) |
| Parts (composer) | `composer-input/rounded-box`, `mode-switch/segmented` Plan / Build / Ask ([composer-mode-switch](../specs/composer-mode-switch.md)), `permission-selector/segmented` ([composer-permission-selector](../specs/composer-permission-selector.md)), `queue-stack/chips` ([composer-queue-stack](../specs/composer-queue-stack.md)), `send-control/steer-slide` ([composer-send-control](../specs/composer-send-control.md)), `@` element chips from the picker ([composer-slash-mentions](../specs/composer-slash-mentions.md)) |
| Parts (preview) | `preview-frame` (new) hosting the dev server; wireframe over rewritten regions = `genui-surface/blueprint-build` look with `genui-skel/surface-blueprint` (`oxyn-skel-44`) ([genui-renderer](../specs/genui-renderer.md), [genui-skeletons](../specs/genui-skeletons.md)); changed-region markers from `genui-surface/diff-regenerate`; reduced-motion fallback `skeleton/static` ([skeletons](../specs/skeletons.md)) |
| Empty | New blank project: preview shows a centred "Your app will appear here" with three starter prompts as chips |
| Loading | Dev server booting: `preview-frame` *Booting* (route field shimmer `text-shimmer/calm`, [text-shimmer](../specs/text-shimmer.md)); first build: full-page blueprint wireframe with mono labels ("HEADER 1280×64") per [R005] |
| Error | Build failed: last good build stays, dimmed to `opacity.stalled`, with a top banner "Build failed · v14 · Fix with Brick / Show log" (flow F3); runtime crash: in-frame error card + Problems count in console |

Wireframe rules (proposed): the wireframe only covers regions the agent's `layout_hint` names; it
fades in after `delay.skeleton` and is replaced region by region with a crossfade at `dur.version.swap`;
changed regions keep a ✦ marker for `dur.highlight.fade`. A region the agent abandons collapses
(`genui-renderer` §2 *Collapsed*).

### 3.4 Visual edit (element picker) — `shell/app-builder-visual-edit`

```text
┌ ⟳ /menu                                  🖥 ▭ ▯  [⌖ Select ✓]  ⤢ ┐ ┌ Inspector · Button ───────┐
│ QUILLBY BAKEHOUSE                    Menu  Pickup  Cart (2)       │ │ src/menu.rs : 88          │
│ ┌────────────┐ ┌────────────┐ ┌────────────┐                      │ │ Text   [ Add to bag    ]  │
│ │ Sourdough  │ │ Cardamom   │ │ Rye loaf   │                      │ │ Fill   ■ accent  ▾        │
│ │ €6.50      │ │ bun €3.20  │ │ €5.80      │                      │ │ Radius ○ 4 ● 8 ○ full     │
│ │╔══════════╗│ │ [ + ]      │ │ [ + ]      │                      │ │ Pad    8 · 16             │
│ │║  [ + ]   ║│ └────────────┘ └────────────┘                      │ │ ───────────────────────── │
│ │╚══════════╝│  button.add · 96×32 · 3 matches                    │ │ Applies to: ● this ○ all 3│
│ └────────────┘                                                    │ │ ↶ Undo   Before/After ◑   │
└───────────────────────────────────────────────────────────────────┘ │ [Ask Brick…]   [Apply ⏎]  │
 Composer: [⌖ button.add ×] "make these say Add to bag and round them" └───────────────────────────┘
```

| | |
|---|---|
| Purpose | Point at a thing instead of describing it; small style / text tweaks without a prompt; bigger changes by prompt with the element attached |
| Parts | `element-picker` (new) overlay: hover outline, name tag with size and match count, multi-select with ⌘/Ctrl-click; selection becomes a composer chip `attachment/context-chip` ([composer-attachments](../specs/composer-attachments.md)); `style-inspector` (new, Pro) builds on `genui-surface/inspector-edit`; Apply produces a new version with `file-edit-chip/row` in the transcript |
| Empty | Picker on with nothing hovered: hint chip "Click any element · Esc to stop" |
| Loading | Apply sent: selected region gets `border-beam/hairline` ([border-beam](../specs/border-beam.md)) until the patch lands |
| Error | Element not traceable to source ("generated at runtime"): inspector shows "Can't edit directly · Ask Brick" and keeps the prompt route; preview reloaded under the selection: selection clears with a toast "Selection lost after reload" |

### 3.5 Code view — `shell/app-builder-code`

```text
┌ [Preview] [Code ✓] …                                         Read-only ▾  ⎘ Download ┐
│ ┌ Files ─────────────┐ ┌ src/slots.rs ─────────────────────────── +118 ─ Brick ✎ ──┐ │
│ │ ▾ src                │ │ 12  pub fn slots_for(day: Day) -> Vec<Slot> {           │ │
│ │   menu.rs        M   │ │ 13      let open = day.open_at();                       │ │
│ │   cart.rs        M   │ │ 14 +    (0..12).map(|i| open + 15.min() * i)            │ │
│ │   slots.rs       A ● │ │ 15 +        .filter(|s| !s.is_full())                   │ │
│ │ ▸ migrations         │ │ …                                                       │ │
│ │ ▸ public             │ │ [Changes in v14 ▾]  Unified · Split                     │ │
│ └──────────────────────┘ └────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

| | |
|---|---|
| Purpose | Let a developer (Tomás) read or edit code; let anyone see what a version changed |
| Parts | `file-tree/changed` ([side-panels](../specs/side-panels.md)); editor body `code-block/file` ([streaming-markdown](../specs/streaming-markdown.md)); "Changes in vN" toggles `diff-viewer/unified` / `diff-viewer/split` ([diff-viewer](../specs/diff-viewer.md)); "Brick ✎" owner tag while the agent writes the file = `file-edit-chip/writing-sweep` |
| Empty | Nothing generated yet: "Code appears after the first build" |
| Loading | Large file: `skeleton/code-gutter` ([skeletons](../specs/skeletons.md)) |
| Error | User edits a file the agent is writing: the editor goes read-only with "Brick is editing this file · Take over" (`takeover-banner/strip`, [takeover-banner](../specs/takeover-banner.md)) |

### 3.6 Version history — `shell/app-builder-history`

```text
┌ History   Bookmarks                                     Compare ⇄   ✕ ┐
│ Today                                                                  │
│ ┌ ▓thumb▓ ┐ v14  Pickup slots step          ◐ Draft    +160 −3   now  │
│ ┌ ▓thumb▓ ┐ v13  Fix: slot list key         ✓          +2 −1   12:01  │
│ ┌ ▓thumb▓ ┐ v12  Card payments (PayRail)    ● Live     +240    10:12  │
│ ┌ ▓thumb▓ ┐ v11  Menu photos                 ★ Bookmark +31 −8  09:40  │
│ Yesterday                                                              │
│ ┌ ▓thumb▓ ┐ v10  First build from prompt                       17:22  │
│ ─────────────────────────────────────────────────────────────────────  │
│ v12 selected:  Preview ↗   View changes   Go to message   [Restore]   │
└────────────────────────────────────────────────────────────────────────┘
 Snapshot banner (when previewing): Viewing v11 from 09:40 · Back to latest · Restore
```

| | |
|---|---|
| Purpose | Find a known-good state, compare, restore it (dev) or roll production back to it (flow F5) |
| Parts | `checkpoint-timeline/filmstrip` + `checkpoint-timeline/milestone-cards` + `checkpoint-timeline/compare-pair` ([checkpoint-timeline](../specs/checkpoint-timeline.md)); `env-badge` (new) for Live / Draft / Preview; restore sheet `rollback-impact/sheet` with `rollback-impact/compensations` and `rollback-impact/receipt` ([rollback-impact](../specs/rollback-impact.md)); compare opens [diff-viewer](../specs/diff-viewer.md) |
| Empty | One version only: list with a single row and "Every change Brick makes is saved here" |
| Loading | Thumbnails: `genui-skel/media-shimmer` (`oxyn-skel-12`) |
| Error | Version too old to restore: Restore disabled-with-reason "Can't restore this far back · Preview only" |

### 3.7 Database / backend — `shell/app-builder-database`

```text
┌ Database   ● Dev ▾ (Production is read-only here)                 + Table  ⋯ ┐
│ ┌ Tables ───────┐ ┌ orders · 128 rows ─────────────────────────────────────┐ │
│ │ products   12 │ │ id   customer      slot          total   status        │ │
│ │ orders    128 │ │ 128  A. Lindqvist  Sat 09:15     €12.90  paid          │ │
│ │ slots      96 │ │ 127  K. Mensah     Sat 09:00     €6.50   paid          │ │
│ │ staff       3 │ │ 126  R. Duarte     Sun 10:30     €19.10  refunded      │ │
│ ├ Schema ───────┤ │ …                                       ‹ 1 2 3 … 6 › │ │
│ │ ◇ orders ─┐   │ └────────────────────────────────────────────────────────┘ │
│ │ ◇ slots ◀─┘   │ Pending migration 0007_add_slot_capacity   [Review] [Run] │
│ └───────────────┘                                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

| | |
|---|---|
| Purpose | See the data the app stores, the schema the agent made, and approve migrations |
| Parts | `genui-table/data-grid` ([genui-blocks](../specs/genui-blocks.md)); `genui-table/changed-cells` after the agent edits seed data; `schema-graph` (new, Pro); migration row is a `tool-call/card` whose run goes through [approval-card](../specs/approval-card.md) (`approval-card/split-preview` with the SQL on the left); `env-badge` switches Dev / Production |
| Empty | No database yet: "This app doesn't store anything yet · Add a database" (Medium approval: provisions a resource) |
| Loading | `genui-skel/table-shimmer` (`oxyn-skel-04`) |
| Error | Connection lost: grid keeps last page, header turns `color.status.warning.*` "Reconnecting…", Retry |

Rule: production data is **read-only** in this screen; any production write or migration is a
separate, Critical request (§5).

### 3.8 Environment and secrets — `shell/app-builder-env`

```text
┌ Environment   Dev  Preview  [Production]                               + Add ┐
│ Name                       Scope        Value            Used by      Updated │
│ PAYRAIL_SECRET_KEY    🔒   Production   •••• ·· set      payments.rs  10 Nov  │
│ PAYRAIL_WEBHOOK_SECRET 🔒  Production   •••• ·· set      webhook.rs   10 Nov  │
│ MAILKITE_API_KEY      🔒   All          •••• ·· set      email.rs      9 Nov  │
│ PUBLIC_SHOP_NAME           All          Quillby Bakeh…   menu.rs       8 Nov  │
│ ⚠ PAYRAIL_SECRET_KEY missing in Preview          [Add value]                   │
│ ─────────────────────────────────────────────────────────────────────────────  │
│ Values are never shown, copied, or sent to Brick. Brick sees names and scopes. │
└───────────────────────────────────────────────────────────────────────────────┘
```

| | |
|---|---|
| Purpose | Add, replace and scope keys; see which code uses them; never expose a value |
| Parts | `secret-list` (new, family G) with masked value cells (status only: *set*, *missing*, *expired*); add / replace uses a write-only field in `genui-form/side-panel` ([genui-forms](../specs/genui-forms.md)) that clears on submit; agent requests to use a secret render as `approval-card/secret` ([approval-card](../specs/approval-card.md)); `permission-selector/secret-guard` idea for the composer |
| Empty | "No environment variables · Brick will ask when the app needs one" |
| Loading | `genui-skel/list-shimmer` (`oxyn-skel-03`) |
| Error | Save failed: field keeps focus, value is **not** kept in memory after failure (user re-enters); scope conflict: inline "Already set for Production · Replace?" |

Hard rules: no reveal button, no copy button, no value in logs (console masks known values as
`••••`), no value in screenshots or session replay. The agent's tool results carry `SecretRef` names only.

### 3.9 Deploy / publish panel with logs — `shell/app-builder-deploy`

```text
┌ Deploy                                                       Preview ▾  Production ┐
│ Live in production:  v12 · order.quillby.example · 2 h ago · ● healthy              │
│ ┌ Publishing v14 to production ────────────────────────────────── 0:48 ─ Cancel ─┐ │
│ │ ✓ Build        0:21     ✓ Checks  3/3     ◐ Upload  62%     ○ Switch traffic   │ │
│ │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━░░░░░░░░░░░░░░░░░ │ │
│ │ 12:20:31  upload  assets/menu-photos  14/22                                    │ │
│ │ 12:20:33  upload  server bundle 2.1 MB                                         │ │
│ └────────────────────────────────────────────────────────────────────────────────┘ │
│ History   v12 ● live   v11 superseded   v10 superseded            Roll back… ↶    │
│ Domains   quillby-preorders.brickwell.example ✓   order.quillby.example ✓ TLS     │
└────────────────────────────────────────────────────────────────────────────────────┘
```

| | |
|---|---|
| Purpose | Ship a version to Preview (shareable, not public) or Production (public), watch it go, roll back |
| Parts | `deploy-pipeline` (new, family F) built from `step-timeline/metro` ([plan-todo](../specs/plan-todo.md)) + `terminal-output/section-fold` logs ([terminal-output](../specs/terminal-output.md)) + `elapsed-timer/eta`; checks as `review-card/verdict-chip` ([review-card](../specs/review-card.md)); background copy `background-jobs/dock-pill` when the user leaves the tab; production publish card `approval-card/modal` + `approval-card/hold-ring` (Critical); domain rows use `genui-table/compact-records` |
| Empty | Never published: "Not live yet · Publish a preview link first" with a Preview button (Low) |
| Loading | Pipeline steps as `todo-list/checklist` pending rows; logs `skeleton/line-by-line` |
| Error | Step failed: that step turns `color.status.danger.*`, log auto-scrolls to first error line, "Fix with Brick" (F3) and "Keep live on v12" note — **a failed publish never touches traffic** |

### 3.10 Share and collaborate — `shell/app-builder-share`

```text
┌ Share Quillby Preorders                                               ✕ ┐
│ People                                                                   │
│  Ines Okafor (you)          Owner                                        │
│  Tomás Reyes                Can edit ▾        ● in Code · slots.rs       │
│  [ name@example …        ]  Can view ▾   [Invite]                        │
│ Links                                                                    │
│  Preview link (v14)   quillby-preorders-v14.brickwell.example  ⎘  Off ◯  │
│  Remix link           anyone can copy, no secrets, no data       ⎘  Off ◯│
│ Comments  3 open on preview · Show pins                                  │
└──────────────────────────────────────────────────────────────────────────┘
```

| | |
|---|---|
| Purpose | Invite collaborators, share a preview, let others remix without secrets or data |
| Parts | `genui-form/stacked` ([genui-forms](../specs/genui-forms.md)); `presence-stack` (new) with live location ("in Code · slots.rs"); comment pins in the preview reuse `element-picker` anchoring; human vs agent edits distinguished by `acting-indicator/avatar-ring` ([acting-indicator](../specs/acting-indicator.md)) |
| Empty | Only owner: "Just you so far" + invite field focused |
| Loading | Invite sending: row with `loader/dots-typing` ([loader-vector](../specs/loader-vector.md)) |
| Error | Invite failed / address invalid: inline field error; link toggle failed: toggle snaps back with toast |

## 4. Agent flows

States are [../architecture.md](../architecture.md) `AgentState`. "Shows" names the component carrying
the step. Approvals per [../specs/approvals.md](../specs/approvals.md).

### F1. First build from a prompt
Ines types on Home: "A preorder page for my bakery. Weekend menu, pickup slots, pay by card."

| # | AgentState | What happens | Shows | User can steer |
|---|---|---|---|---|
| 1 | `Composing` → `Submitted` | Prompt sent from the hero composer; app shell opens with the builder | `composer-input/hero-dock` morphs to the docked composer (`dur.layout.switch`); `send-control` submitted pulse | — |
| 2 | `Thinking { "Planning" }` | Brick drafts a plan: pages, data, integrations | `reasoning-block/inline-summary`; `verb-cycler/fade` | Stop |
| 3 | `AwaitingApproval { Plan }` | Plan card: 5 steps, "will create a database", "will need PAYRAIL keys later" | `plan-card/versioned` + `plan-approval` ([plan-approval](../specs/plan-approval.md)); risk Low | Edit plan, approve, or reply "no payments yet" |
| 4 | `ToolRunning { write }` | Files written; the preview shows the full blueprint wireframe from the plan's layout hint | `file-edit-chip/row` per file; `preview-frame` *Wireframe* | Type to queue ("make it green") → `queue-stack/chips` |
| 5 | `ToolRunning { "db.create" }` | Dev database provisioned | `tool-call/row`; Medium approval if level is Ask (resource created) | Allow / Deny |
| 6 | `ToolRunning { "cargo build" }` | Build + dev server | `terminal-output/tail`; preview *Booting* | Stop |
| 7 | `Streaming` | Regions swap from wireframe to real UI one by one | `genui-surface/blueprint-build` swaps; `stagger.row` | Click a region to queue an edit |
| 8 | `ToolRunning { "app.test" }` | Brick clicks through menu → cart → slots in the preview | `acting-indicator/cursor-flag` over the preview | **Take over** (`takeover-banner/strip`) |
| 9 | `Done { 2m 41s }` | v10 saved; summary with what to try next | `elapsed-timer/summary` "Worked for 2m 41s"; `checkpoint-timeline/inline-marker`; follow-up chips | Pick a follow-up |

Errors: build failure → F3. `Stalled` after 5 s without events → verb text warms per `dur.stall.ramp`;
the preview keeps the wireframe still (no fake progress).

### F2. Visual edit via the element picker
Ines wants the "+" buttons to say "Add to bag" and be rounder.

| # | AgentState | What happens | Shows | User can steer |
|---|---|---|---|---|
| 1 | `Idle` | `S` or ⌖ turns on the picker; hover outlines elements | `element-picker` *Hovering* | Esc exits |
| 2 | `Idle` | Click selects; "3 matches" offers all three cards | `element-picker` *Selected*; inspector opens | ⌘-click to add / remove |
| 3 | `Idle` (local) | Radius 8 in the inspector previews live, no agent call | `style-inspector` pending edit; Before / After toggle | Undo / Reset |
| 4 | `Composing` | Text change typed as a prompt with the element chip attached | `attachment/context-chip` "⌖ button.add ×3" | Remove chip |
| 5 | `Submitted` → `ToolRunning { write }` | Pending style edits + prompt sent as one request | selected regions `border-beam/hairline`; `file-edit-chip/row` `src/menu.rs +3 −3` | Stop |
| 6 | `Done { 9s }` | v15; changed regions marked ✦; picker stays on for the next edit | `genui-surface/diff-regenerate` markers fade over `dur.highlight.fade` | Undo = restore v14 (Low) |

Edge: the agent is mid-run when Ines applies — the request queues ("Runs after the current task")
in `queue-stack/chips`; the selection is stored as a source location, not a pixel position, so it
survives the reload.

### F3. Failing build → auto-fix

| # | AgentState | What happens | Shows | User can steer |
|---|---|---|---|---|
| 1 | `ToolRunning { "cargo build" }` | Build fails: `E0308 mismatched types` in `slots.rs:14` | `terminal-output/tail` row turns ✗ with exit code; preview keeps v13, dimmed; banner "Build failed" | — |
| 2 | `Thinking { "Reading the error" }` | Brick parses the log; Problems tab gets 1 entry | `problem-list` row with file:line link; `tool-call/retry-ladder` shows attempt 1 of 3 | "Stop fixing" |
| 3 | `ToolRunning { write }` | Patch to `slots.rs` | `file-edit-chip/row` `+2 −1`; code tab shows `diff-viewer/unified` if open | Open diff |
| 4 | `ToolRunning { "cargo build" }` | Rebuild passes | ✓ row; banner morphs to "Fixed · 1 attempt" | — |
| 5 | `ToolRunning { "app.test" }` | Re-runs the step that broke | `acting-indicator/cursor-flag` | Take over |
| 6 | `Done { 38s }` | v14 saved with tag "auto-fix" | `checkpoint-timeline/milestone-cards` collapses the failed attempt between markers | — |

Limits (proposed): **3 attempts** per failure; attempts 2–3 show the diff before building. After the
third failure → `Error { kind: BuildFailed }`: the banner offers "Restore v13", "Show log",
"Tell Brick what's wrong". A fix that needs a new package or a network host raises its own Medium
approval; a fix that would touch a secret or production stops at Critical (never auto).

### F4. Publish to production (Critical)

| # | AgentState | What happens | Shows | User can steer |
|---|---|---|---|---|
| 1 | `Idle` | Ines presses **Publish ▾ → Production** (or asks Brick to publish) | Publish split-button | — |
| 2 | `ToolRunning { "deploy.checks" }` | Pre-publish checks: build, tests, secret scope check ("PAYRAIL_SECRET_KEY set for Production ✓"), DB migration plan | `deploy-pipeline` checks stage; `review-card/verdict-chip` per check | Cancel |
| 3 | `AwaitingApproval { Custom "Publish" }` | **Critical** card: version v14, target `order.quillby.example`, what changes for visitors, migration `0007` (adds a column, no data loss), payments **live**, rollback target v12 | `approval-card/modal` + `approval-card/hold-ring` (hold `dur.approval.hold`, 1200 ms); header hatch `color.risk.critical-hatch`; `rollback-impact/blast-radius` summary row | Hold to publish · Deny · Deny and tell Brick… |
| 4 | `ToolRunning { "deploy" }` | Build → upload → switch traffic | `deploy-pipeline` metro steps + log; `background-jobs/dock-pill` if Ines leaves the tab | Cancel (until "Switch traffic" starts) |
| 5 | `ToolRunning { "deploy.verify" }` | Health check on the live URL | step 5 spinner; `elapsed-timer/eta` | — |
| 6 | `Done { 1m 12s }` | "v14 is live" with link; status bar → "Live: v14" | `rollback-impact/receipt`-style receipt; `env-badge` Live moves to v14 in History | Open site, Roll back |

Rules: the card appears **even in Bypass** (`bypass_includes_critical = false`); "Always allow" is not
offered; a timeout never approves; if Brick asks to publish, the same card appears (the agent can only
*request*). A first publish that also connects `order.quillby.example` or turns on live payments shows
each as its own page of a paged card (`approval-card/paged`), each Critical, each held separately.

### F5. Roll back to a version

| # | AgentState | What happens | Shows | User can steer |
|---|---|---|---|---|
| 1 | `Idle` | Visitors report a broken slot picker; Ines opens History, picks v12 (● previously live) | `checkpoint-timeline/filmstrip`; snapshot banner "Viewing v12" | Preview v12 first |
| 2 | `Idle` | **Roll back production to v12** opens the impact sheet | `rollback-impact/sheet`: code → v12; DB migration `0007` **not** reverted (column stays, harmless); orders placed since v14 stay; compensations: "Unpublish v14 preview link" | Choose "Also restore dev to v12" |
| 3 | `AwaitingApproval` | Critical (changes production) | `approval-card/hold-ring` inside the sheet | Hold · Cancel |
| 4 | `ToolRunning { "deploy.switch" }` | Traffic switched to the stored v12 build (no rebuild) | `deploy-pipeline` with a single "Switch traffic" step | — |
| 5 | `Done { 6s }` | "Production is on v12"; v14 stays in history marked *rolled back*; dev keeps v14 unless chosen | `rollback-impact/receipt`; History badges update with `dur.version.swap` | "Ask Brick to fix v14" |

Restoring **dev only** (no production effect) is Low and needs no hold: it creates a new version
(v16 = copy of v12), like a revert commit, so nothing is lost.

## 5. Approvals and risk

Tiers per [../specs/approvals.md](../specs/approvals.md) §2.3. **Critical always stops, at every
permission level including Bypass.** Default permission level for this kit: **Auto-edit**
(`AutoEdit` auto-runs Low `Edit`s only).

| Action in Brickwell | Request kind | Tier | Card variant | Notes |
|---|---|---|---|---|
| Write / edit project files | `Edit` | Low | `tool-call/row` footer | auto-runs at Auto-edit |
| Restore dev to an older version | `Custom "Restore"` | Low | `rollback-impact/inline` | creates a new version; reversible |
| Publish a **Preview** link (not public, no live payments) | `Custom "Publish preview"` | Low | `approval-card/inline` | test keys only |
| Run build / test / dev server | `Command` | Low (sandboxed) | `approval-card/footer` | Medium if outside sandbox |
| Install a package | `Command` | Medium | `approval-card/footer` | shows package name + licence |
| Fetch from a new host / call an MCP tool | `Network` / `McpTool` | Medium | `approval-card/site-grant` | |
| Create dev database / storage bucket | `Custom "Provision"` | Medium | `approval-card/inline` | |
| Dev migration that drops a column or table | `Command` | High | `approval-card/split-preview` (SQL left) | arms for `dur.approval.arm` |
| Invite an editor / turn on a remix link | `Custom "Share"` | High | `approval-card/inline` | only by user action; agent cannot invite |
| Use a secret in a tool call (dev / preview scope) | `Credential` | High | `approval-card/secret` | name + destination only |
| **Publish to production** | `Custom "Publish"` | **Critical** | `approval-card/modal` + `hold-ring` | F4 |
| **Roll back / unpublish production** | `Custom "Publish"` | **Critical** | `rollback-impact/sheet` + hold | F5 |
| **Connect or change a custom domain / DNS** | `Custom "Domain"` | **Critical** | `approval-card/modal` | shows record diff; affects live traffic |
| **Turn on live payments / change payout account / any charge or refund** | `Spend` | **Critical** | `approval-card/amount` | amounts in tabular figures |
| **Add, replace, re-scope or use a production secret** | `Credential` | **Critical** | `approval-card/secret` | value never shown, never sent to the agent |
| **Production DB migration or write; delete production data** | `Command` | **Critical** | `approval-card/split-preview` + hold | migration plan + rows affected |
| **Delete project** | `Custom "Delete"` | **Critical** | `approval-card/modal` | type-to-confirm project name *and* hold |

Kit rules (proposed):
1. The agent can **request** any Critical action; only the user's hold completes it. No rule, session
   grant or level can pre-approve Critical (`AllowAlways` and `AllowSession` hidden).
2. Batching never includes Critical: a paged card shows one Critical item per page.
3. Every resolution writes an audit row (`audit-log/timeline`, [permission-rules](../specs/permission-rules.md)),
   shown in Deploy → History.
4. Permission level lives in the composer (`permission-selector/segmented`); the level's reach is
   shown with `sandbox-badge/chip`.

## 6. New components

Proposed; each needs its full 10-point spec ([../specs/README.md](../specs/README.md)) before build.

| Proposed name | Family | Tier | Brief |
|---|---|---|---|
| `preview-frame` | I (surface container, sibling of `genui-surface`) | Free (3) + Pro | A live web-app preview: toolbar (reload, back / forward, route field, device toggles desktop / tablet / phone + rotate, zoom, fullscreen, open in tab), states *Booting*, *Wireframe*, *Live*, *Patching*, *Stale* (last good build, dimmed), *Crashed*, *Offline*. Hosts a webview (GPUI) or sandboxed iframe (Dioxus). Reuses `genui-surface/device-frames` for the frame art and the blueprint wireframe for patching regions. Free: `plain`, `device-frames`, `minimal-toolbar`; Pro: `hardware-bezel`, `side-by-side-devices`, `before-after-split`. |
| `element-picker` | I | Free | Overlay over `preview-frame`: hover outline + name tag (tag · size · match count), click to select, ⌘/Ctrl-click multi-select, Esc clears, `S` toggles. Emits a `SelectionRef` (source file + span + DOM path) that becomes a composer chip. Keyboard mode: Tab walks elements in DOM order, arrows move to parent / child. |
| `style-inspector` | I | Pro | Property panel for the selected element (text, fill, radius, spacing, type, shadow) with live pending edits, undo / redo, Before / After, "this / all matches"; Apply sends edits + optional prompt as one request. Extends `genui-surface/inspector-edit` to arbitrary app nodes. |
| `deploy-pipeline` | F | Free (3) + Pro | Publish run as a metro of stages (Build, Checks, Upload, Switch traffic, Verify) with per-stage timers, folded logs, cancel boundary ("can't cancel after traffic switch"), failure that never touches live, and a receipt. Free: `metro`, `checklist`, `terminal`; Pro: `canary-bar` (traffic % slider shown read-only), `region-map`, `diff-summary`. |
| `env-badge` | H | Free | Small chip naming an environment or version role: Dev, Preview, Production, Live, Draft, Rolled back. Shape + label carry meaning (filled dot = live), colour is redundant. Used in status bar, History, Env, Database. |
| `secret-list` | G | Free | Table of secret **names** with scope, status (*set / missing / expired / unused*), "used by" file links, updated date. No reveal, no copy; write-only add / replace field that clears on submit; missing-in-scope warnings. Pairs with `approval-card/secret`. |
| `problem-list` | F | Free | Console tab of build and runtime problems: severity glyph, message, file:line link, occurrence count, "Fix with Brick" (sends the problem as context), "Ignore". Counts mirror to the status bar. |
| `schema-graph` | I | Pro | Read-only table / relation graph of the app's database with changed tables marked after a migration; click a table opens it in the grid. Reuses `genui-kit/canvas` layout. |
| `project-card` | M | Free | Gallery tile: thumbnail, name, status (`env-badge` Live vN, *Building* with `elapsed-timer`, *Failed* with "Fix"), collaborators, last edit. States: idle, hover (thumbnail scrubs versions, Pro), building, failed, archived. |
| `template-card` | M | Free | Template tile with thumbnail, tags, and "Needs" line (database, secret names) before choosing. |
| `presence-stack` | J | Pro | Collaborator avatars in the top bar with live location ("in Code · slots.rs"), click to follow; distinct from the agent's `acting-indicator`. |

## 7. Data model and fixtures

```rust
// Sketch only (oxyn-kit-app-builder, proposed). All demo data is invented.
pub struct Project {
    pub id: ProjectId, pub name: String, pub owner: PersonId,
    pub template: Option<TemplateId>, pub live: Option<VersionId>, pub draft: VersionId,
    pub collaborators: Vec<Collaborator>, pub thumbnail: Option<ImageRef>,
}
pub struct Version {
    pub id: VersionId, pub number: u32, pub title: String,        // "Pickup slots step"
    pub created_at: Timestamp, pub by: Actor,                      // Agent | Person(PersonId)
    pub message: Option<MessageId>, pub files: Vec<FileChange>,    // +N −N per file
    pub role: VersionRole,          // Draft | Saved | Live | Superseded | RolledBack
    pub bookmarked: bool, pub tags: Vec<String>,                   // "auto-fix"
    pub restorable: bool, pub screenshot: Option<ImageRef>,
}
pub enum Environment { Dev, Preview, Production }
pub struct Deployment {
    pub id: DeployId, pub version: VersionId, pub env: Environment,
    pub stages: Vec<DeployStage>,
    pub status: DeployStatus,       // Queued | Running | Live | Failed | Cancelled
    pub url: String, pub approval: Option<ApprovalId>, pub started: Timestamp,
}
pub struct DeployStage {
    pub kind: StageKind, pub state: StepState, pub elapsed: Duration, pub log: LogRef,
}
pub struct SecretRef {               // never holds a value
    pub name: String, pub scopes: Vec<Environment>,
    pub status: SecretStatus,       // Set | Missing | Expired | Unused
    pub used_by: Vec<SourceSpan>, pub updated_at: Timestamp,
}
pub struct DbTable {
    pub name: String, pub rows: u64, pub columns: Vec<Column>, pub changed_in: Option<VersionId>,
}
pub struct Migration {
    pub id: String, pub sql: String, pub destructive: bool, pub env: Environment,
}
pub struct SelectionRef {
    pub source: SourceSpan, pub dom_path: String, pub label: String, pub matches: u32,
}
pub struct Problem {
    pub severity: Severity, pub message: String, pub at: Option<SourceSpan>, pub count: u32,
}
pub struct Collaborator { pub person: PersonId, pub role: Role, pub location: Option<String> }
```

| Fixture set | Contents | Drives |
|---|---|---|
| `fx/quillby-project` | 1 project, 16 versions (v1–v16) over 2 days, v12 live, v14 draft, 2 bookmarks, 1 rolled back | Home, History, status bar |
| `fx/quillby-files` | ~24 source files (Rust + assets) with realistic sizes; diffs for v10→v14 | Code, diff, file chips |
| `fx/quillby-db` | `products` 12, `orders` 128, `slots` 96, `staff` 3; invented customer names and € totals; one pending migration | Database |
| `fx/quillby-env` | 4 secret refs (names only), 1 missing in Preview, 1 plain var | Env |
| `fx/deploys` | 3 deployments: one live, one failed at Checks, one in progress at Upload 62% | Deploy |
| `fx/runs` | Event streams for F1–F5 as `AgentState` transitions with timestamps (real-time and 4× speed) | Agent flows, Studio demos |
| `fx/gallery` | 6 projects across states (live, building, failed, archived, empty, long name) | Home |
| `fx/templates` | 8 templates in 5 categories with "needs" lines | Template picker |
| `fx/people` | Ines (owner), Tomás (editor), 1 pending invite | Share, presence |

Preview content is a small invented app rendered by the kit itself (no third-party site), screenshot
thumbnails are generated from it, and no real logo, brand or person appears.

## 8. Variants

### 8.1 Shell variants

| Variant | Layout | Surfaces | Motion | Tier |
|---|---|---|---|---|
| `shell/app-builder-split` | chat left, preview right, console drawer (2.1) | desktop, web | subtle | Free |
| `shell/app-builder-stacked` | Chat / Preview segmented switch (tablet) | web, mobile | subtle | Free |
| `shell/app-builder-mobile` | preview first, chat sheet | mobile | subtle | Free |
| `shell/app-builder-canvas` | infinite canvas of screens (`genui-kit/canvas`), chat floating | desktop, web | expressive | Pro |
| `shell/app-builder-focus` | full preview, composer as floating pill (`composer-input/floating`), panels on demand (`panel-split/focus-mode`) | desktop, web | expressive | Pro |
| `shell/app-builder-ide` | adds file tree + editor as a centre column (`panel-split/presets`) for developer users | desktop | none / subtle | Pro |
| `shell/app-builder-tui` | terminal look: preview as screenshot pane, logs first (`panel-split/tui`, `genui-kit/terminal`) | terminal | none | Pro |

### 8.2 Screen variants

| Screen | Free | Pro |
|---|---|---|
| Home | grid gallery, list gallery | hover-scrub thumbnails, bento dashboard (`genui-kit/bento`) |
| Builder preview loading | `genui-skel/surface-shimmer` (`oxyn-skel-15`) | `oxyn-skel-44` blueprint, `oxyn-skel-45` aurora, `oxyn-skel-46` glyph |
| Visual edit | picker + prompt only | `style-inspector` panel, annotation drawing |
| History | `checkpoint-timeline/side-list` | `filmstrip`, `compare-pair`, `milestone-cards` |
| Deploy | `deploy-pipeline/metro` | `canary-bar`, `region-map` |
| Database | `genui-table/data-grid` | `schema-graph`, `genui-table/inline-edit` (dev only) |

### 8.3 Motion levels
- **none**: no wireframe animation; regions swap instantly; `motion.reduced` fallbacks throughout.
- **subtle** (default): blueprint wireframe with `skeleton/shimmer`; swaps at `dur.version.swap`;
  panels at `dur.surface.enter`; sheets at `dur.sheet.enter` (`spring.snap`).
- **expressive**: blueprint draw-in then `loop.scan` reveal; `genui-surface/aurora-build` glow over the
  region being written; publish success uses the `mark/ring-step` success state
  ([animated-marks](../specs/animated-marks.md)).

## 9. Accessibility and edge cases

| Case | Behaviour (proposed) |
|---|---|
| Keyboard-only | ⌘/Ctrl+1–6 switch workspace tabs; `S` toggles picker; in picker Tab / Shift+Tab walk elements, ↑ parent, ↓ first child, Enter selects, Esc exits; ⌘/Ctrl+Shift+A jumps to a pending approval; Critical hold works with Space held |
| Screen readers | Preview tab has its own landmark; picker announces "Button, Add to bag, 96 by 32, 3 matches"; deploy stages announced on change via polite live region, failures assertive; secret cells read "value hidden, set" |
| Focus inside preview | The preview is a separate focus scope; ⌘/Ctrl+Esc returns focus to the composer from inside the app |
| Colour independence | `env-badge`, risk tiers, deploy stages all carry shape + text; Live = filled dot + "Live" |
| Reduced motion | No wireframe shimmer or scan; aurora / beam replaced by static outline; hold-to-confirm keeps its duration with a stepped fill (motion.md safety rule) |
| Very fast agent (< 150 ms) | Skip the wireframe entirely (`genui-renderer` rule); no flash of skeleton |
| Very slow agent / `Stalled` | Wireframe goes still, verb text warms; after 30 s a "Still building · Show log" link appears |
| Huge projects | File tree and data grid virtualised; logs folded by phase; History groups by day, loads 50 at a time |
| Long names | Project and version titles truncate middle in chips ("Quillby Pre…orders"), full name in tooltip and accessible name |
| RTL | Shell mirrors (chat right, preview left); preview content follows the app's own `dir`; route field and code stay LTR |
| Narrow / touch | 44 px targets (`size.touch-target`); picker uses long-press to select, tap-and-hold for parent; Critical hold works by touch |
| Preview crash / infinite loop | `preview-frame` *Crashed* with "Reload" and "Restore last good version"; the shell never freezes (preview runs in its own process / iframe) |
| Offline | Preview and Code keep working on cached build; Publish, Database and Env disabled-with-reason "Offline" |
| Two editors on one file | Agent holds a write lock per file; human edit shows `takeover-banner/strip`; collaborator cursors via `presence-stack` |
| Secrets in output | Console and tool results mask known secret values; screenshots and `run-replay` exclude the Env screen |
| Mid-run restore | Restore disabled while a run is active: "Stop or wait for Brick to finish" (queued prompts pause, as Bolt does) |

## 10. References

Studied 2026-11-12. Behaviour only; no code, assets or copy taken. Licence column per
[../licensing.md](../licensing.md).

| Product | What we looked at | Source (opened) | Licence / use |
|---|---|---|---|
| Lovable | Preview toolbar modes (Select `S`, inline text `T`, draw `D`, comment `C`), multi-select with Cmd/Ctrl, edits sent while the agent works "run after the current task" | https://docs.lovable.dev/features/visual-edit | Proprietary · study only |
| Lovable | Version history: every change is a version, snapshot view with "Back to latest", Published badge, bookmarks, revert restores code not database data | https://docs.lovable.dev/features/projects/history | Proprietary · study only |
| v0 | Design mode: hover-highlight then select, design panel (type, colour, layout, border, shadow), pending edits with undo / reset / before-after, Apply creates a new version | https://v0.app/docs/design-mode | Proprietary · study only |
| Replit Agent | App Testing: agent drives a real browser in the preview with a visible cursor, take-over prompt, video replay | https://docs.replit.com/features/agent/app-testing | Proprietary · study only |
| Replit | Publishing pane: domain, access, production database, monitoring, security scan before publish, deployment secrets synced on publish | https://docs.replit.com/features/publishing/overview | Proprietary · study only |
| Bolt | Select a UI element into the chat, prompt queue (edit / reorder / pause), restoring a version requires stopping the current prompt | https://support.bolt.new/building/chat-tools | Proprietary · study only (the open-source bolt.new repo was not reviewed for this kit) |
| Figma Make | Chat + workspace with Browser / Files / Review tabs, edit tool with properties panel, draw and annotations, templates, backend for secrets, publish with custom domain | https://help.figma.com/hc/en-us/articles/31304412302231-Explore-Figma-Make | Proprietary · study only |
| Research note 03 | App-builder rows: dev-server preview card, built-in browser, visual / design edit modes, plan with versions | project research folder `03-coding-agent-apps.md` | notes |
| Research note 05 | Checkpoint / version history patterns (Replit rollback with database opt-in, Lovable history), A2UI catalog for gen UI | `05-workflow-genui-icons.md` | notes |
| Research notes 11, 12 | affectUI approval card and permission request; Arc UI status morph ("Publish → Publishing → Published"), toasts | `11-affectui-catalog.md`, `12-arc-deep-dives.md` | Paid · study only |
| Research notes 16, 17 | Preview chrome (device toggles, canvas background, fullscreen); preview context controls | `16-niche-sites-a.md`, `17-niche-sites-b.md` | mixed; ideas only |
| [R005] | Dotted blueprint wireframe with mono size labels revealing into real UI | [../research/sources.csv](../research/sources.csv) | X post · idea only |
| [R034] | Aurora glow sweep over the region the agent fills | [../research/sources.csv](../research/sources.csv) | X post · idea only |
| [R049] | "Secrets stay out of the chat" masked key card | [../research/sources.csv](../research/sources.csv) | X post · idea only |
| [R107] signoff-ui | Approval card + policy model | https://agent-ui-kit-demo.vercel.app | MIT · may port with attribution |

