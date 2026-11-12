**Status:** draft v0.1 (2026-11-12).

# Category kit: video / media agents

Kit for section B row **Video / media agents** of [../component-taxonomy.md](../component-taxonomy.md):
*timeline editor with agent panel, render queue; frame scrubber, generation placeholders, render
progress, shot list, asset bin.* Follows the 10-section standard in [README.md](README.md). Registry
names are `shell/video-<screen>` (family M). Values marked *proposed* are starting points for the Studio.

## 1. Who it's for

**The job.** A small creator or studio hires the agent to do the slow parts of making a video: turn a
script into a shot list, generate the shots it cannot film, cut long recordings down by reading the
transcript, caption and translate, render, and publish on schedule. The human keeps taste and the
final say; the agent does the fetching, cutting, generating and checking.

**Made-up company (all fixtures invented).**

| Item | Fixture |
|---|---|
| Studio | **Kestrel & Fern**, a two-person documentary studio |
| Series | *Low Tide Notes*, a web series about coastal trades |
| Project | Episode 7, *The Lamp Keeper* (target 6:00, 16:9, 24 fps, plus a 9:16 cut-down) |
| People | Ama Ruiz-Hale (director, user), Theo Lindqvist (editor), Noor Siddiqui (client reviewer) |
| Agent | **Cutter**, the studio's editing agent; sub-agents *Scout* (asset analysis), *Lens* (generation) |
| Footage | 42-min interview with a retired keeper, 31 B-roll clips, 3 archive stills, 1 music bed |
| Credits | Fictional unit "cr". Plan 1,200 cr / month; project budget 600 cr; ~40 cr per 5 s shot |
| Publish targets | `Harbour Channel` (public, 48k followers), `Patron feed` (members), `Review link` (private) |

No real people, brands, platforms or logos. Thumbnails are generated abstract stills.

## 2. Shell

The shell is a non-linear editor (NLE) with the agent as a docked panel, not a chat with a video
attached. The timeline is the source of truth; every agent edit lands on it as a visible, undoable
change.

### 2.1 Desktop (GPUI), ≥ 1440 px

```text
┌ ◐ Kestrel & Fern ▸ Low Tide Notes ▸ Ep 7 ▾ ───── ⌘K ── ◔ 3 jobs 41% ── ◈ 412/600 cr ── Export ┐
├──┬─────────────────┬──────────────────────────────────────────┬──────────────────────────────┤
│▣ │ BIN        ⌕  ▾ │ ┌ Source ───────────┐ ┌ Program ───────────┐ │ Cutter ● Generating 4/12   │
│☰ │ ▸ Interview (1) │ │                   │ │                    │ │ ▸ Plan · 12 shots · 480 cr │
│¶ │ ▸ B-roll (31)   │ │  [ clip preview ] │ │ [ sequence frame ] │ │ ⚙ gen_shot ×12   ◔ 4/12   │
│CC│ ▾ Generated (8) │ │                   │ │                    │ │   ▒▒ ▒▒ ✓ ✓ ✕ ▒▒ …        │
│⇪ │   ▪ s04 harbour │ └ 00:01:12:08 ─────┘ └ 00:03:41:02 ◀ ▶ ▶▶ ┘ │ ▲ Spend 80 cr more? [..]   │
│✎ │   ▪ s07 lamp ✕  ├──────────────────────────────────────────┤ │                            │
│⤴ │ tags: storm ×6  │ 0:00   0:30   1:00   1:30  ▼  2:00   2:30   │ │                            │
│  │       keeper ×9 │ V2 ·[s04▒▒]···[s07 ✕]·····│·············    │ │                            │
│  │                 │ V1 [interview A ][broll 3][interview B   ]  │ │ ┌ Ask Cutter… @clip /cut ┐ │
│  │                 │ A1 ▁▂▅▇▅▂▁▁▂▅▇▇▅▂▁▁▁▂▅▇▅▂▁│▁▂▅▇▅▂▁▁▂▅      │ │ │                    ⏎ ■ │ │
│  │                 │ CC [The lamp…][We kept…]  │ [It was…]       │ │ └────────────────────────┘ │
├──┴─────────────────┴──────────────────────────────────────────┴──────────────────────────────┤
│ 00:02:01:14 · 24 fps · sel 0:04:06 · Ep7 main ▾ · ◇ Ask before edits · checkpoint 14 ↶      │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
```

| Region | Size (proposed) | Built from |
|---|---|---|
| Top bar | 40 px | [workspace-chrome](../specs/workspace-chrome.md); job pill [`background-jobs/dock-pill`](../specs/background-jobs.md); credits [`run-meter/credits`](../specs/run-meter.md); ⌘K [`command-palette/agent-scope`](../specs/command-palette.md) |
| Left rail + panel | 48 px rail, 280 px panel (resizable 220–400) | [`side-panel/rail`](../specs/side-panels.md); panel hosts Bin, Shots, Transcript, Captions, Queue, Review, Publish |
| Viewers | flexible, 16:9 boxes | `genui-media/player` ([genui-blocks](../specs/genui-blocks.md)) + **flash-guard** (new) |
| Timeline | 240 px default, drag to 160–480 | **media-timeline** + **frame-scrubber** (new) |
| Agent panel | 360 px (320–480) | [message-bubble](../specs/message-bubble.md), [tool-call](../specs/tool-call.md), [plan-todo](../specs/plan-todo.md), [approval-card](../specs/approval-card.md), [composer](../specs/composer.md) |
| Status bar | 24 px | [`status-bar/ide`](../specs/side-panels.md), [permission selector](../specs/composer-permission-selector.md) chip, [checkpoint](../specs/checkpoint-timeline.md) chip |

### 2.2 Web (Dioxus) and breakpoints

| Width | Layout |
|---|---|
| ≥ 1440 | as desktop; two viewers |
| 1024–1439 | one viewer with Source / Program toggle; left panel becomes an overlay drawer |
| 640–1023 | timeline full width; agent panel becomes a bottom sheet (peek 96 px, half, full) |
| < 640 | **mobile review mode**: no free-form editing (see 2.3) |

The web build streams proxies (low-res previews), never the masters. Waveforms and thumbnails come
precomputed from the server; the canvas only draws.

### 2.3 Narrow / mobile

Mobile is for **approving, reviewing and watching jobs**, not cutting. Trim is limited to nudging a
selected clip ±1 s.

```text
┌ Ep 7 · The Lamp Keeper     ◈ 412 cr ┐
│ ┌────────────────────────────────┐  │
│ │      [ program preview ]       │  │
│ └ 02:01 ━━━━━━━●──────── 06:04 ┘    │
│ V ▮▮▮▯▮▮▮▮▯▮  CC ▮▮▮▮▮▮▮▮▮▮         │  timeline as a read-only strip
├─────────────────────────────────────┤
│ ⬣ Publish to Harbour Channel        │  Critical card pinned on top
│   Public · Fri 18:00 · EN ES UR     │
│   [ Hold to publish ●──────── ]     │
├─────────────────────────────────────┤
│ ◔ Render master 4K   41% · ~6m      │
│ ✕ s07 lamp  Failed · Retry          │
│ ✎ 3 comments need you               │
├─────────────────────────────────────┤
│ Ask Cutter…                    🎤 ⏎ │
└─────────────────────────────────────┘
```

## 3. Screens

Components are linked to their specs; **bold** names are new (section 6).

### S1. Dashboard (`shell/video-dashboard`)

```text
┌ Kestrel & Fern · Studio ───────────────────────────────────── ◈ 412 / 600 cr this project ┐
│ NEEDS YOU (3)                     │ RUNNING (4)                     │ CREDITS (Nov)          │
│ ⬣ Publish Ep 7 · Fri 18:00        │ ◔ Render Ep 7 master   41% ~6m  │ ███████▓░░  842/1200   │
│ ▲ Spend 80 cr over plan           │ ◔ Lens: s07 retry      Step 9/30│ by project:            │
│ ✎ Noor: 3 comments on v4          │ ⠿ Scout: index B-roll  18/31    │  Ep 7  412 · Ep 6 301  │
├───────────────────────────────────┴─────────────────────────────────┴────────────────────────┤
│ PROJECTS                                                                                     │
│ ▣ Ep 7 The Lamp Keeper   cut v4 · 6:04 · 12 shots (10 ✓ 2 ✕)  edited 4m ago     [Open]      │
│ ▣ Ep 6 Net Menders       published · 5:48 · 2 languages        3 days ago        [Open]      │
│ ▣ Ep 8 (untitled)        script only                           —                 [Start]     │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
```

Purpose: what needs me, what is running, what is it costing. Uses
[`fanout-grid/inbox-board`](../specs/fanout-grid.md) (Needs you / Running / Done columns),
[`background-jobs/render-queue`](../specs/background-jobs.md),
[`run-meter/budget-burn`](../specs/run-meter.md), [`approval-card/inbox`](../specs/approval-card.md),
[`genui-list/rich-rows`](../specs/genui-blocks.md).

| State | Shows |
|---|---|
| Empty | "No projects yet" + Start from a script / Import footage; credits card still shown |
| Loading | [`skeleton/blueprint`](../specs/skeletons.md) rows; credits as `—` (never 0) |
| Error | credits source down: card says "Balance unavailable"; generation actions are disabled with that reason |

### S2. Editor (`shell/video-editor`)

The shell in 2.1. Agent edits arrive as **ghost clips** (dashed outline, 40 % fill) until applied;
applied edits create a checkpoint ([`checkpoint-timeline/filmstrip`](../specs/checkpoint-timeline.md)).
Who is acting on the timeline is shown by [`acting-indicator/timeline-strip`](../specs/acting-indicator.md);
when the agent holds the playhead, the user's first scrub hands control back
([takeover-banner](../specs/takeover-banner.md)).

| State | Shows |
|---|---|
| Empty | empty tracks with drop zone "Drop footage or ask Cutter to start from a script" ([`attachment/drop-overlay`](../specs/composer-attachments.md)) |
| Loading | timeline ruler + track headers draw first; clips as flat blocks; thumbnails and waveforms fill in by visible range |
| Error | missing media: clip hatched with "Offline · relink"; proxy failed: "Preview unavailable", edits still allowed |

### S3. Storyboard and shot list (`shell/video-storyboard`)

```text
┌ Shots · Ep 7 ──────────── 12 shots · 0:58 total · est 480 cr ── [Grid ▪ List] ── Generate all ┐
│ ┌ s01 ──────────┐ ┌ s02 ──────────┐ ┌ s03 ──────────┐ ┌ s04 ──────────┐                       │
│ │ ░░ noise ░░░  │ │ ✓ [still]     │ │ ▒▒ Step 12/30 │ │ ✓ [still]     │                       │
│ │ Queued · 2nd  │ │ 5 s · 40 cr   │ │ 5 s · 40 cr   │ │ 5 s · 40 cr   │                       │
│ │ Wide, harbour │ │ Lamp room,    │ │ Storm surge   │ │ Keeper's hands│                       │
│ │ at dawn, fog  │ │ brass, dust   │ │ on rocks      │ │ winding clock │                       │
│ └ filmed? ○ gen ┘ └ ref: still 2 ─┘ └───────────────┘ └───────────────┘                       │
│ ┌ s05 ──────────┐ ┌ s06 ──────────┐ ┌ s07 ──────────┐ ┌ s08 ──────────┐                       │
│ │ ◇ from B-roll │ │ ✓ [still]     │ │ ✕ Timed out   │ │ ⊘ Not shown   │                       │
│ │ broll_12 3.2s │ │ 4 s · 32 cr   │ │ Retry · 40 cr │ │ Rewrite prompt│                       │
│ └───────────────┘ └───────────────┘ └───────────────┘ └───────────────┘                       │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
```

Purpose: prompt or script → numbered shots the user can reorder, edit, mark "use filmed B-roll" vs
"generate", then send. Each card is a **shot-board** card wrapping a
[`gen-placeholder/video-strip`](../specs/media-generation-placeholder.md) tile; the batch runs as
[`fanout-grid/media-contact-sheet`](../specs/fanout-grid.md). The plan header and its approval use
[`plan-card/document`](../specs/plan-todo.md) and
[`plan-approval/scope-preview`](../specs/plan-approval.md). `⊘ Not shown` is the placeholder's
`Blocked(policy)` state: it never shows what was blocked.

| State | Shows |
|---|---|
| Empty | composer prompt "Paste a script or describe the video" + example chips |
| Loading | cards appear as [`skeleton/blueprint`](../specs/skeletons.md) ("SHOT 16:9 · 5 s") while the agent streams the list |
| Error | parse failed: script shown with the unparsed span highlighted, "Split manually" |

### S4. Asset bin (`shell/video-bin`)

```text
┌ Bin ─────────────────────────────── ⌕ "storm"  ▾ Type ▾ Tag ▾ Used ─ 37 items ┐
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐                          │
│ │▓▓▓▓▒▒░░░░│ │ [thumb]  │ │ [thumb]  │ │ [thumb]  │                          │
│ │ Indexing │ │ 0:42:10  │ │ 0:00:12  │ │ still    │                          │
│ │ 18/31 fr │ │ ✓ text   │ │ ✓        │ │ ⚠ faces  │                          │
│ └ broll_19 ┘ └ intvw_A ─┘ └ broll_03 ┘ └ arch_02 ─┘                          │
│ broll_03 · 12 s · 4K · tags: storm 0.91  rocks 0.88  night 0.74  [+ tag]    │
│ Used in: Ep7 main @ 01:12 · Scout note: "lens flare 00:07–00:09, bright"    │
└─────────────────────────────────────────────────────────────────────────────┘
```

Purpose: find footage by what is in it. Import triggers *Scout*: each item shows
[`analysis-scan/frame-index`](../specs/media-analysis-scan.md) (video keyframe strip, `Indexing(frame
k/n)`), stills use [`analysis-scan/region-hunt`](../specs/media-analysis-scan.md), faces of
non-consenting people can be pre-blurred with [`analysis-scan/redact`](../specs/media-analysis-scan.md).
Auto tags are chips with confidence; the user can accept, remove or add. Built on **asset-bin**
(new), whose list mode extends [`file-tree`](../specs/side-panels.md).

| State | Shows |
|---|---|
| Empty | "Drop footage" + supported formats |
| Loading | per-item scan; bin stays usable; tags appear as each item reaches `Done(summary)` |
| Error | `Unreadable(reason)` per item ("Codec not supported"); `Partial` shows which frames failed |

### S5. Generation and render queue (`shell/video-queue`)

```text
┌ Queue ─────────────────────── running 3 · queued 2 · done 9 · failed 1 ── ◈ 412/600 cr ┐
│  #  Job                      Kind    Progress            ETA     Cost     Actions      │
│  1  Render Ep7 master 4K     render  ▕████▍     ▏ 41%    ~6m     —        ■ Cancel     │
│  2  Lens · s07 lamp (retry)  gen     Step 9/30           —       40 cr    ■            │
│  3  Scout · index broll_19   scan    18/31 frames        —       —        ■            │
│  4  Lens · s11 tide (var 2)  gen     Queued · 1 ahead    —       40 cr    ↑ ■          │
│  5  Captions · UR translate  text    Queued              —       6 cr     ■            │
│  ✕  Lens · s08 cliff         gen     Not shown (policy)  —       0 cr     Rewrite      │
│  ✓  Lens · s04 harbour       gen     Done 3m ago         —       40 cr    Open ↗       │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│ This run: 360 cr of 480 approved · Project: 412 of 600 · Plan left: 358 cr               │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

Purpose: one list for every slow thing (generate, render, scan, transcribe, translate), with cost per
job and honest progress. This is [`background-jobs/render-queue`](../specs/background-jobs.md) with
[`background-jobs/budget-meter`](../specs/background-jobs.md) and
[`background-jobs/batch-retry`](../specs/background-jobs.md); ETA only when the provider supplies one
(background-jobs §1 rule 2); costs use [`run-meter/credits`](../specs/run-meter.md). Cancelling a
costly job asks first ("Cancel render? 41 % will be lost"). Policy-blocked jobs cost 0 cr only if the
provider refunds; otherwise the row says what was charged.

| State | Shows |
|---|---|
| Empty | "Nothing running" + last 24 h finished |
| Loading | rows with `—` progress; never a fake 0 % |
| Error | `Unknown` after restart: "Lost track · Check"; provider down: queued jobs `Paused(provider)` |

### S6. Transcript edit (`shell/video-transcript`)

```text
┌ Transcript · intvw_A (42:10 → 4:06 proposed) ─────── Show: ◉ proposed ○ applied ── Apply ┐
│ 00:03:12  KEEPER   The lamp never went out. Not once in thirty years.                      │
│ 00:03:18  KEEPER   ~~Um, well, there was the — no, wait, let me start again.~~  ⓘ retake     │
│ 00:04:01  AMA      ~~Take your time.~~                                       ⓘ interviewer │
│ 00:11:40  KEEPER   The night of the storm, the glass cracked. I held the mantle             │
│                    with my coat sleeve until dawn.                           ★ key line     │
│ 00:12:02  KEEPER   ~~[long pause 6.2 s]~~                                    ⓘ gap > 2 s    │
│ ── 37:44 removed in 41 spans · reasons: off-topic 22 · retakes 9 · gaps 10 ──               │
├────────────────────────────────────────────────────────────────────────────────────────────┤
│ V1 [■■][■][■■■■][■][■■]  ← ghost cuts mirror the strikes                                     │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

Purpose: edit video by editing text; the agent proposes strikes with a reason per span, the user
keeps or restores any span before applying. Built on **transcript-cut** (new), which reuses the
`Word` model and scroll rules of [`voice-transcript`](../specs/live-captions.md) (word-level timing,
`Estimated` timing marked) and the per-hunk keep / undo grammar of [diff-viewer](../specs/diff-viewer.md).
Clicking a word seeks the playhead ([`voice-transcript/scrub`](../specs/live-captions.md) idea).
Nothing is deleted from media: cuts are non-destructive and restorable.

| State | Shows |
|---|---|
| Empty | "No transcript yet · Transcribe (≈2 cr)" |
| Loading | transcription streams in; words appear with [`text-reveal`](../specs/text-reveal.md) fade, low-confidence words dotted |
| Error | alignment failed for a range: words shown with `Estimated` timing and a "timing approximate" band |

### S7. Captions and translation (`shell/video-captions`)

```text
┌ Captions ── Track: EN v2 ▾ ── + Translate ▾ (ES ✓  UR ◔ 63 %) ── Style ▾ ── Burn-in ○ ──┐
│  #   In          Out         EN                                    CPS   UR (RTL)      │
│  41  02:01:14    02:03:20    The night of the storm,               14   ‏طوفان کی رات، │
│  42  02:03:20    02:06:02    the glass cracked.                    9    ‏شیشہ ٹوٹ گیا۔ │
│  43  02:06:02    02:07:10    I held the mantle with my coat sleeve 24 ⚠ ‏…ترجمہ جاری   │
│                                                         ⚠ reading speed > 17 cps · Split │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│ CC  [The night…][the glass…][I held the mantle…]          ← cue blocks on the timeline    │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

Purpose: auto-captions, fix, style, translate, and keep cues in sync with the cut. Built on
**caption-editor** (new); the preview overlay uses [`captions/cinema`](../specs/live-captions.md) and
bilingual preview uses [`captions/translate`](../specs/live-captions.md). The translation job runs in
the queue (S5); rows fill as cues arrive ([`skeleton/line-by-line`](../specs/skeletons.md)). Reading
speed (characters per second, CPS) over a threshold (17, proposed) is flagged amber with a fix.

| State | Shows |
|---|---|
| Empty | "No captions · Generate from transcript" (free if transcript exists) |
| Loading | cues fill top to bottom; untranslated cells say "Translating…", never blank |
| Error | language unsupported: listed with reason; cue drift after a re-cut: "12 cues out of sync · Re-align" |

### S8. Review (`shell/video-review`)

```text
┌ Review · cut v4 ── Noor Siddiqui (client) · 3 open · 5 resolved ─────── Compare v3 ▸ v4 ┐
│ [ program viewer with comment pin at 01:12 ● ]                                          │
│ 0:00 ─────●(01:12)────────●(02:40)──────────────●(04:55)───────────── 6:04               │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│ ● 01:12:08  Noor   "Shot feels too dark, can we see her face?"                           │
│             Cutter  Proposed: +0.4 EV on s04 · ghost on V2  [Apply] [Variation…] [Reply] │
│ ● 02:40:00  Noor   "Music too loud under the key line"                                   │
│             Cutter  Proposed: duck A2 −6 dB 02:38–02:52           [Apply] [Reply]        │
│ ● 04:55:12  Noor   "Spelling: 'mantle' not 'mantel'"   → caption #43   [Fix] [Resolve]   │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

Purpose: reviewers leave comments pinned to a timecode (and optionally a region of the frame); the
agent turns each into a proposed fix the editor applies or rejects. Built on **timecode-review**
(new), which borrows severity, thread and fix-popover patterns from
[`review-card/inline-thread`](../specs/review-card.md) and [`review-card/fix-popover`](../specs/review-card.md).
Version compare uses `genui-media/compare-slider` ([genui-blocks](../specs/genui-blocks.md)) and
[`checkpoint-timeline/compare-pair`](../specs/checkpoint-timeline.md).

| State | Shows |
|---|---|
| Empty | "No comments yet · Share review link" (High approval, section 5) |
| Loading | markers appear on the ruler first, thread bodies after |
| Error | comment on a range that was since cut: "This moment was removed in v5 · Show in v4" |

### S9. Export and publish (`shell/video-publish`)

```text
┌ Export & publish · Ep 7 cut v5 ───────────────────────────────────────────────────────┐
│ OUTPUTS                         │ CHECKS                         │ DESTINATION           │
│ ☑ Master 4K 24 fps ProRes       │ ✓ Flash check (3/s limit)      │ ☐ Save to drive  Low  │
│ ☑ Web 1080p H.264  ~410 MB      │ ✓ Captions EN ES UR            │ ☐ Review link   High  │
│ ☑ Vertical 9:16 cut-down 0:58   │ ⚠ Music licence: not recorded  │ ☑ Harbour Channel     │
│ ☑ Captions .srt × 3 (sidecar)   │ ✓ Loudness −14 LUFS            │   Public · ⬣ Critical │
│ Thumbnail [ still s04 ▾ ]       │ ✓ No offline media             │   Fri 18:00 PKT ▾     │
├─────────────────────────────────┴────────────────────────────────┴───────────────────────┤
│ Title  The Lamp Keeper · Low Tide Notes Ep 7          Render cost: 0 cr (local)          │
│                                     [ Render only ]  [ Render & schedule publish ▸ ]     │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

Purpose: pick outputs, see pre-flight checks, choose destinations. Each destination shows its risk
tier before the user commits. The form is [`genui-forms`](../specs/genui-forms.md); the checks list is
`genui-list/checklist` ([genui-blocks](../specs/genui-blocks.md)); the publish step is an
[`approval-card`](../specs/approval-card.md) at **Critical** (hold 1200 ms, Allow once only, stops in
Bypass). The flash check is **flash-guard** run on the rendered file.

| State | Shows |
|---|---|
| Empty | no outputs ticked: primary button disabled with reason "Pick at least one output" |
| Loading | checks run one by one; publish disabled until all have a result (pass, warn or fail) |
| Error | a failed check (e.g. flash) blocks publish; warn-level checks need an explicit "Publish anyway" inside the Critical card |

## 4. Agent flows

States are the `AgentState` enum in [../architecture.md](../architecture.md). Approval behaviour is
[../specs/approvals.md](../specs/approvals.md) §2. "Steer" means the user acts mid-run (`Steering`) or
queues a message ([composer-queue-stack](../specs/composer-queue-stack.md)).

### F1. Script → storyboard → generate shots (fan-out, some fail, retry)

| # | `AgentState` | What the user sees | Steer |
|---|---|---|---|
| 1 | `Composing` | script.md as an [`attachment/chip`](../specs/composer-attachments.md); "Storyboard this, 16:9, moody" | — |
| 2 | `Submitted` | send-control pulse < 500 ms ([composer-send-control](../specs/composer-send-control.md)) | stop |
| 3 | `Thinking { "Breaking down the script" }` | [verb-cycler](../specs/verb-cycler.md), collapsible [reasoning-block](../specs/reasoning-block.md) | queue "keep it under a minute" |
| 4 | `Streaming` | shot cards fill S3 as blueprints; plan card counts 12 shots, est 480 cr | edit, reorder, mark s05 "use B-roll" |
| 5 | `AwaitingApproval { Plan + Spend }` | [`plan-approval/scope-preview`](../specs/plan-approval.md) with a Spend line: "Generate 11 shots · up to 480 cr · 168 cr left after" (High) | Allow once, Allow session up to N cr, Edit and allow (fewer shots), Deny with reason |
| 6 | `ToolRunning { gen_shot, "Generating" }` ×11 | [`fanout-grid/media-contact-sheet`](../specs/fanout-grid.md); each tile [`gen-placeholder/video-strip`](../specs/media-generation-placeholder.md); queue pill "◔ 11 jobs"; credits meter ticks per settled job | cancel one tile, cancel all |
| 7 | (partial) | s07 `Failed(timeout)`; s08 `Blocked(policy)` ("Not shown"); grid `Partial`, 9 ✓ | Retry s07; Cutter offers a rewritten s08 prompt for the user to accept |
| 8 | `ToolRunning` (retry) | retry is a new job with `retry_of`; inside the approved envelope, no new ask | — |
| 9 | `AwaitingApproval { Spend }` *(only if over budget)* | retries would pass the 600 cr project budget → **Critical** card, hold to allow | Edit and allow (lower count), Deny |
| 10 | `Done { 6m 12s }` | [`run-meter/receipt`](../specs/run-meter.md) "11 shots · 440 cr · 6m 12s"; clips drop on V2 as ghosts, then apply → checkpoint | Undo via checkpoint |

Stalled: no provider event for the stall window → tile and lane shift to `Stalled` tone; the job is
not auto-cancelled.

### F2. Cut a 42-minute interview by transcript

| # | `AgentState` | What the user sees | Steer |
|---|---|---|---|
| 1 | `ToolRunning { transcribe }` | S4 item shows [`analysis-scan/frame-index`](../specs/media-analysis-scan.md) + transcript streaming in S6 | cancel |
| 2 | `Composing` → `Submitted` | "Cut to about 4 min, the storm night, keep his voice not mine" | — |
| 3 | `Thinking { "Reading the interview" }` | [`text-shimmer/calm`](../specs/text-shimmer.md) status; [tool-call](../specs/tool-call.md) rows for `search_transcript` | — |
| 4 | `Streaming` | strikes appear span by span in **transcript-cut**, each with a reason; ghost cuts mirror on V1; duration counter 42:10 → 4:06 ([`elapsed-timer/odometer`](../specs/elapsed-timer.md) style digits) | restore any span → `Steering` ("keep 11:40–12:02") |
| 5 | `AwaitingApproval { Edit }` | [`approval-card/split-preview`](../specs/approval-card.md): transcript left, timeline right; "Apply 41 cuts to *Ep7 main*" (Medium) | Allow once; Edit and allow (open in S6) |
| 6 | `Done { 48s }` | cuts applied as one checkpoint "Interview cut v1"; [`checkpoint-timeline/filmstrip`](../specs/checkpoint-timeline.md) | Restore checkpoint |

### F3. Regenerate one shot with a variation

| # | `AgentState` | What the user sees | Steer |
|---|---|---|---|
| 1 | `Composing` | clip s04 selected → context chip `@s04`; "lower camera, more fog, 3 options" | — |
| 2 | `AwaitingApproval { Spend }` *or skipped* | "3 variations · 120 cr" (High). Skipped if a session allowance from F1 still covers it | Allow once / session |
| 3 | `ToolRunning { gen_variation }` | 1×3 [`gen-placeholder/candidate-race`](../specs/media-generation-placeholder.md) in the agent panel; `Previewing` frames under thinning noise | cancel one |
| 4 | `Done` | user picks candidate 2; s04 swaps on V2 with a **take stack** badge "3 takes" (old take kept) | Swap back any time |
| — | `Error { policy }` | a candidate `Blocked`: tile says "Not shown"; no preview, prompt, or reason text that would reveal content | Rewrite prompt |

### F4. Render and publish (Critical)

| # | `AgentState` | What the user sees | Steer |
|---|---|---|---|
| 1 | `Composing` | "Render v5 and publish to Harbour Channel Friday 6 pm, with all captions" | — |
| 2 | `Thinking` → `AwaitingApproval { Plan }` | [`plan-card/document`](../specs/plan-todo.md): render 3 outputs → checks → schedule publish (Low; the plan itself does nothing) | edit steps |
| 3 | `ToolRunning { render }` | S5 row + [`gen-placeholder/timeline-render`](../specs/media-generation-placeholder.md) fills rendered ranges on the clip track | cancel (confirm, costly) |
| 4 | `ToolRunning { preflight }` | S9 checks tick in order; **flash-guard** scans the render; music licence warns | fix and re-run |
| 5 | `AwaitingApproval { Publish }` | **Critical** [`approval-card/hold-ring`](../specs/approval-card.md): thumbnail, title, Public, Harbour Channel, Fri 18:00 PKT, captions EN ES UR, warnings listed. Hold 1200 ms. Stops even in Bypass | Deny with reason; Edit (schedule, visibility → Patron feed lowers to High) |
| 6 | `Done` | "Scheduled for Fri 18:00 PKT" receipt with Unschedule (Low before the time) and Unpublish after (High) | — |

If the window is unfocused when step 5 is reached, the card also goes to
[`island/approvals`](../specs/desktop-overlays.md) and mobile (2.3); it never auto-expires into a
yes. A timed-out Critical request resolves as denied.

### F5. Auto-captions and translation

| # | `AgentState` | What the user sees | Steer |
|---|---|---|---|
| 1 | `ToolRunning { caption }` | S7 cues fill from the applied cut's transcript (no new transcription cost) | — |
| 2 | `Streaming` | Cutter flags 6 cues over 17 CPS and proposes splits | accept per cue |
| 3 | `AwaitingApproval { Spend }` | "Translate to ES, UR · 12 cr" (High; usually covered by session allowance) | pick languages |
| 4 | `ToolRunning { translate } ×2` | two queue rows; UR column renders RTL; untranslated cells say "Translating…" | cancel one language |
| 5 | `AwaitingApproval { Edit }` | "Replace track EN v1 with EN v2" → **High** because 14 manual edits on v1 would be overwritten; offers "Save as new track" (Low) instead | choose new track |
| 6 | `Done { 1m 20s }` | three tracks; preview toggles [`captions/translate`](../specs/live-captions.md) | — |

## 5. Approvals and risk

Tiers per [../specs/approvals.md](../specs/approvals.md) §2.3. Spend uses the `Spend` request kind
(floor High, Critical above the app threshold). **Critical always stops, including in Bypass**
(`bypass_includes_critical = false`).

| Action | Kind | Tier | Notes |
|---|---|---|---|
| Analyse, tag, transcribe a clip already in the bin | `Custom(Analyse)` | Low | free or a few cr; within session allowance |
| Propose cuts, ghost clips, markers, comments | — | none | proposals change nothing until applied |
| Apply edits to a sequence (cuts, reorder, swap take) | `Edit` | Low (scratch sequence) / Medium (main sequence) | always one checkpoint per apply |
| Replace a caption track or audio mix that has manual edits | `Edit` | High | "Save as new" offered as the Low path |
| Spend credits within an approved envelope / session allowance | `Spend` | (auto) | the envelope was itself a High approval |
| Start a paid generation batch | `Spend` | High | shows count × unit cost, total, budget left |
| Spend that would pass the project or monthly budget | `Spend` | **Critical** | hold to allow; Allow once only |
| Download from a URL / stock source new to the project | `Network` | Medium | licence field required on import |
| Share a private review link | `Custom(Share)` | High | names who can open it, expiry |
| Publish or schedule to a members-only feed | `Custom(Publish)` | High | |
| **Publish or schedule to a public channel** | `Custom(Publish)` | **Critical** | thumbnail, title, visibility, time, warnings |
| Unpublish a live public video | `Custom(Publish)` | High | |
| **Delete source footage or originals** (not proxies) | `Custom(Delete)` | **Critical** | lists files, size, "used in 3 sequences" via [rollback-impact](../specs/rollback-impact.md) |
| **Clone a real person's voice or likeness** | `Custom(Likeness)` | **Critical** | needs a recorded consent asset; card names it |
| Use a stored platform credential to post | `Credential` | High | secret name only, never value |

Rules for this vertical (proposed):
1. **Envelopes, not drips.** A batch is approved once with a maximum; jobs inside it never ask again.
   Retries spend from the same envelope. Crossing it raises a new request.
2. **No "Always allow" for Spend or Publish** (approvals.md §2.4).
3. **Warnings travel into the Critical card.** Every pre-flight warning is listed in the card body;
   publishing past a warning needs no extra click, but the warning text must be on screen.
4. **Policy blocks are never approval prompts.** A blocked generation is a result, not a question.

## 6. New components

Eight components A–L does not cover. Each needs its full 10-point spec before it is built.

| Proposed name | Family | Tier | Brief |
|---|---|---|---|
| `media-timeline` | J Workspace chrome | free baseline (1) + Pro | Multi-track timeline: ruler, track headers (V / A / CC / Gen), clips with thumbnails and waveforms, playhead, in/out, snapping. Agent-specific states: **ghost clip** (proposed, dashed), **generating clip** (hosts a gen placeholder at the clip's exact length), **failed clip** (hatched, retry), **rendered-range bar**, per-clip acting owner. Virtualised by visible range. |
| `frame-scrubber` | J Workspace chrome | free | Ruler + filmstrip you drag through; hover shows a frame peek with timecode; frame-step keys; scrub audio optional. Throttles screen-reader timecode announcements. Also used standalone on mobile and in review. |
| `shot-board` | F Tool calls, plans & orchestration | free baseline + Pro | Shot list / storyboard: numbered shot cards (prompt, duration, source = filmed / generated, ref images, cost), grid ↔ list, drag to reorder, bulk select. Each card hosts a gen placeholder; the board is a plan the agent streams and the user edits before approval. |
| `asset-bin` | J Workspace chrome | free | Media bin: grid / list, search by tag and transcript text, smart bins ("unused", "has faces"), analysis scan per item, tag chips with confidence and accept / remove, "used in" back-links. List mode extends `file-tree`. |
| `transcript-cut` | K Live & voice | Pro (free read-only view) | Text-based editing on the `Word` model: strike / restore spans, reasons per span, speaker labels, word-click seek, proposed vs applied view, removed-time summary, ghost sync to the timeline. Keep / undo grammar from diff-viewer. |
| `caption-editor` | K Live & voice | free baseline + Pro | Caption cue table + timeline track: in / out, text, CPS meter with threshold, line-break hints, multi-language columns with RTL cells, drift detection after re-cuts, style presets, burn-in vs sidecar. |
| `timecode-review` | H Checkpoints, diff & git | Pro | Review comments pinned to a timecode (and optional frame region): markers on the ruler, threads, agent-proposed fix per comment with Apply, resolve, version compare, "moment was cut" state. |
| `flash-guard` | D Container & media loaders | free | Photosensitivity guard for **user or generated video**, which Oxyn's own motion rules can't control: runs a flash analysis on a clip or render (≥ 3 flashes / s, saturated red), marks risky ranges on the timeline, dims and pauses autoplay over them in previews, and feeds the publish check. Off-switch is per user, never per clip. |

Not new (variants that already exist cover it): render queue → `background-jobs/render-queue`;
credits meter → `run-meter/credits` + `run-meter/budget-burn`; candidate picking →
`gen-placeholder/candidate-race`; player → `genui-media/player`; checkpoints →
`checkpoint-timeline/filmstrip`.

Proposed tokens (to add to [../tokens.md](../tokens.md) after review): `color.media.track.video /
audio / caption / gen` (aliases of existing hue steps), `color.media.ghost` (accent border at 40 %),
`color.media.playhead` (accent.solid), `color.media.flash-risk` (status.warning, never red).

## 7. Data model and fixtures

```rust
pub struct Project { id: ProjectId, name: String, fps: Fps, aspect: AspectRatio,
    sequences: Vec<SequenceId>, budget: Credits, spent: Credits }

pub struct Sequence { id: SequenceId, name: String, tracks: Vec<Track>,
    duration: Timecode, is_main: bool, checkpoint: CheckpointId }

pub struct Track { id: TrackId, kind: TrackKind /* Video | Audio | Caption(LangTag) | Gen */,
    clips: Vec<Clip>, locked: bool, muted: bool }

pub struct Clip { id: ClipId, asset: Option<AssetId>, shot: Option<ShotId>,
    start: Timecode, src_in: Timecode, src_out: Timecode,
    state: ClipState, /* Applied | Ghost{proposal} | Generating{job} | Failed{job} | Offline */
    takes: Vec<AssetId>, owner: Actor /* User | Agent(AgentId) */ }

pub struct Asset { id: AssetId, name: String, kind: MediaKind, duration: Option<Timecode>,
    proxy: Option<Url>, analysis: AnalysisState, tags: Vec<Tag>,
    transcript: Option<TranscriptId>, licence: Option<String>, flash_ranges: Vec<Range> }

pub struct Tag { label: String, confidence: f32,
    source: TagSource /* Agent | User */, accepted: bool }

pub struct Shot { id: ShotId, n: u16, prompt: String, duration: Duration,
    source: ShotSource /* Filmed(AssetId) | Generate{model, refs: Vec<AssetId>} */,
    est_cost: Credits, job: Option<JobId> }

pub struct GenJob { id: JobId,
    kind: JobKind, /* Gen | Variation | Render | Scan | Transcribe | Translate */
    phase: GenPhase /* from media-generation-placeholder §9 */, cost: Credits,
    eta: Option<Duration>, retry_of: Option<JobId>, envelope: Option<EnvelopeId> }

pub struct SpendEnvelope { id: EnvelopeId, max: Credits, used: Credits, approval: ApprovalId }

pub struct TranscriptSpan { words: Range<WordIdx>, speaker: SpeakerId,
    proposal: Option<CutProposal> /* Strike{reason} | Keep */, applied: bool }
// Word = live-captions.md "Word model" (start_ms, end_ms, timing: Measured | Estimated)

pub struct CaptionCue { id: CueId, track: TrackId, t_in: Timecode, t_out: Timecode,
    text: String, lang: LangTag, dir: TextDir, cps: f32, edited_by_user: bool }

pub struct ReviewComment { id: CommentId, author: PersonId, at: Timecode,
    region: Option<Rect>, version: SequenceVersion, body: String,
    fix: Option<ProposalId>, status: CommentStatus /* Open | Resolved | Orphaned */ }

pub struct PublishTarget { id: TargetId, name: String, visibility: Visibility,
    /* Private | Members | Public */ schedule: Option<DateTimeTz>, tier: RiskTier }
```

Fixture plan (`fixtures/video/`, invented content only):

| Fixture | Contents | Drives |
|---|---|---|
| `kestrel-fern.toml` | studio, people, budget 600 cr, plan 1,200 cr | dashboard, meters |
| `ep7-sequence.json` | 3 sequences, 6 tracks, 48 clips incl. ghosts, 2 failed, 1 offline | editor |
| `ep7-shots.json` | 12 shots: 9 done, 1 queued, 1 timeout, 1 policy-blocked, 1 filmed | storyboard, F1 |
| `intvw-a-transcript.json` | 42:10 interview, ~6,300 words, 3 speakers, 41 proposed strikes, one `Estimated` range | S6, F2 |
| `captions-ep7.json` | EN (61 cues, 6 over 17 CPS), ES complete, UR partial (RTL) | S7, F5 |
| `review-v4.json` | 8 comments, 1 orphaned by a later cut | S8 |
| `jobs-replay.ndjson` | timed event stream for F1–F5 at 1×, 4× and "very slow" (20 min render) | gallery replay |
| `stills/` | 40 abstract generated stills and gradient "frames"; no faces, no logos | thumbnails |
| `flash-test.webm` | synthetic clip with a 5 flashes / s range at 00:07 | flash-guard demo |

## 8. Variants

### Shell variants

| Variant | Surface | Motion | Tier |
|---|---|---|---|
| `shell/video-editor` | desktop (GPUI) | subtle | free |
| `shell/video-editor-web` | web (Dioxus), proxies only | subtle | free |
| `shell/video-review-mobile` | mobile: review, approvals, queue | subtle | free |
| `shell/video-storyboard-first` | prompt-first: storyboard full screen, timeline collapsed | expressive | Pro |
| `shell/video-transcript-first` | doc-style: transcript centre, timeline as a strip | subtle | Pro |
| `shell/video-studio-queue` | dashboard + multi-project render farm view | subtle | Pro |
| `shell/video-terminal` | TUI: queue + transcript only, [`background-jobs/ascii-bars`](../specs/background-jobs.md) | none | Pro |

### Screen variants

| Screen | Free | Pro |
|---|---|---|
| Dashboard | inbox board | studio wall with live render thumbnails ([`background-jobs/corner-tile`](../specs/background-jobs.md)) |
| Storyboard | grid of cards, `noise-resolve` / `video-strip` placeholders | `blueprint`, `scan-car`, `dot-noise` placeholders; animatic playback |
| Asset bin | grid + `frame-index` scan | `region-hunt`, `heatmap`, `redact` scans; smart bins |
| Queue | `render-queue` list | swimlanes on a time axis, batch retry, sound cues |
| Transcript | read-only + manual strikes | agent proposals with reasons, speaker filters |
| Captions | single track editor | bilingual, karaoke style preview, burn-in styles |
| Review | — | full `timecode-review` with region pins and version compare |
| Publish | export + local save | scheduled publish, multi-destination checks |

Motion levels: **none** (static states, 80 ms crossfades), **subtle** (default; per
[../motion.md](../motion.md) durations), **expressive** (noise reveals, candidate spotlight). Playback
and scrubbing are never eased: the playhead tracks the pointer with `dur.instant`; easing applies only
to chrome (panels `dur.travel` 240 ms, ghost → applied clip morph `dur.base` 180 ms, completion ✓ hold
`dur.beat` 600 ms).

## 9. Accessibility and edge cases

### Keyboard (proposed, NLE conventions)

| Key | Action |
|---|---|
| Space / K | play-pause / pause |
| J / L | shuttle back / forward (repeat to speed up) |
| ← / → | step one frame; ⇧ ten frames |
| I / O | mark in / out |
| ⌘/Ctrl+Shift+A | jump to the pending approval (approvals.md) |
| Tab | moves rail → panel → viewer → timeline → agent panel → composer; inside the timeline ↑ / ↓ change track, ← / → move between clips when a clip is focused |
| Enter on a ghost clip | open its proposal; `Y` apply, `N` reject (same input guard as approval cards) |

### Screen readers
- Timeline is a `grid` (tracks = rows, clips = cells); each clip reads "s04 harbour, generated, 0:05,
  starts 01:12, proposed by Cutter".
- Scrubbing announces timecode at most every 500 ms (polite); stopping announces it once.
- Job completions are polite; failures and approval requests are assertive, one at a time.
- Credits are read as text ("412 of 600 credits used"), not as a ring alone.

### Photosensitivity
Oxyn's chrome follows [../motion.md](../motion.md) §10 (≤ 3 flashes / s, no saturated red flashing).
**User footage and generated video are outside that guarantee**, so: previews never autoplay under
`motion.reduced`; `flash-guard` marks risky ranges and dims them in previews by default; gen
placeholders and scan layers over video obey the area rule and never strobe; the flash check is a
blocking pre-flight check before publish (overridable only inside the Critical card, with the warning
shown).

### Edge cases

| Case | Behaviour |
|---|---|
| 300 clips, 12 tracks | timeline virtualises by visible range; thumbnails decode lazily; zoom-to-fit keeps clip min width 2 px with merged "+14" blocks |
| Very long names | clip labels truncate middle (`interview_A…_take3`); full name in tooltip and SR label |
| RTL locale | chrome mirrors; **time still runs left → right** on the ruler (open question); RTL caption cells use `dir="rtl"` with timecodes kept LTR |
| Mixed fps / aspect | clip badge "25→24 fps"; gen placeholders reserve the shot's aspect, not the sequence's |
| Very fast agent (captions in 2 s) | skip the noise phase; cues appear in one `dur.base` fade; no flash of skeleton < 300 ms |
| Very slow agent (20-min render) | job pill only; no looping animation speeds up or "gives up"; `Stalled` after the stall window with "Still waiting on provider" |
| Offline | edits continue locally; generation and publish disabled with reason; queued jobs `Paused(offline)` |
| Budget exhausted mid-batch | running jobs finish; queued jobs `Paused(budget)`; one Critical request to continue |
| Agent and user edit the same clip | user wins; agent's proposal turns stale ("clip changed · re-propose") |
| Policy block | tile says "Not shown"; no blurred preview, no echoed prompt |
| Caption drift after re-cut | cues shift with their words; orphaned cues listed for review |
| Touch | clip handles 44 px hit area; long-press for clip menu; no hover-only actions |

## 10. References

Study only unless noted. Nothing copied; behaviour is re-implemented from our own specs
([../licensing.md](../licensing.md)). Research note numbers refer to the project research folder;
`[Rxxx]` IDs are in [../research/sources.csv](../research/sources.csv).

| Source | What we took (idea only) | Licence note |
|---|---|---|
| Descript, "Edit like a doc" — https://help.descript.com/getting-started/edit-like-a-doc | edit media by editing the transcript; non-destructive deletes with "restore removed media"; inline notes vs comments | Proprietary · study only |
| Descript, "Correct your transcript" — https://help.descript.com/hc/en-us/articles/10119613609229-Editing-in-Descript | fixing transcript text without changing media; "correct all"; re-alignment shown as a dotted underline | Proprietary · study only |
| Runway, "How do credits work?" — https://help.runwayml.com/hc/en-us/articles/15124877443219-Credits-Overview | credits as a per-second rate (cost = rate × duration), shared workspace pool, spend order | Proprietary · study only |
| CapCut, "How to Recognize Bilingual Subtitles?" — https://www.capcut.com/help/bilingual-subtitles | auto captions with a "translate to" target and bilingual display | Proprietary · study only |
| seanaiux studio job pill "Image to video · Rendering 4%" [R013] | docked render pill | Study only |
| twelvelabs scan highway [R009] | keyframe strip lit by a travelling light (`frame-index`, `scan-car`) | Study only |
| fal.ai queue "Up next" [R084] | queued generations lane | Study only |
| ElevenLabs transcript viewer + scrub bar (research note 04) | word-click seek, scrub bar | Check per component; study only |
| Manus session replay with scrubber (research note 04) | scrubbable agent history beside media | Study only |
| Animos export presets (research note 17) | aspect-ratio presets for social exports | Study only |
| Arc UI "Audio and video" group (research note 12) | catalog coverage check (all Pro) | Paid · study only |

Kling and Captions were in the brief but not opened for this draft; nothing here is attributed to them.

## Open questions
1. **Family placement.** Four new components land in J (timeline, scrubber, bin) and K (transcript,
   captions). Should media editing be its own family (e.g. "N Media") instead of stretching J and K?
2. **RTL timelines.** We keep time flowing left → right in RTL locales (as most NLEs do). Confirm with
   RTL users before the spec.
3. **Spend floor.** approvals.md puts `Spend` at High. For cheap jobs (2 cr transcription) is a
   per-project "small spend" threshold (auto under N cr) acceptable, or must it always ride an envelope?
4. **Likeness / voice clone** is listed Critical here; approvals.md has no kind for it. Add a
   `Likeness` kind with a consent-asset field?
5. **flash-guard accuracy.** Do we ship our own analyser (luminance per frame, 1 s window, as the
   Studio check does) or only a host hook? False negatives on generated video are the risk.
6. **GPUI video decode.** Desktop playback and frame peeks need a decode path (platform decoders vs a
   bundled one); out of scope for the kit but blocks `media-timeline` and `frame-scrubber`.
7. **Credits vs currency.** Show credits only, or credits with a currency hint when the app supplies a
   price (as `run-meter` does for tokens)?
