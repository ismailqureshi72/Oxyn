**Status:** draft v0.1 (2026-11-12).

# Spec: generative UI renderer and containers (`genui-surface`)

Part of family I ([genui.md](genui.md)); follows the 10-point standard in [README.md](README.md) at the
depth of [composer-send-control.md](composer-send-control.md). Target **4 free + 12 Pro = 16 variants**.
The schema, primitives, validation and security model are in [genui.md](genui.md) §2; skeleton shapes per
block are in [genui-skeletons.md](genui-skeletons.md); the skeleton engine and swap timing are owned by
[skeletons.md](skeletons.md). Values marked *proposed* are Oxyn's own.

## 1. Purpose
The renderer turns a stream of `GenEvent`s into a live surface and puts it somewhere sensible:
**inline** in the transcript (the default), in a **side panel** beside the conversation, **fullscreen**
for a dashboard or app preview, or in a **picture-in-picture** window that stays visible while the user
keeps chatting (display modes inline / fullscreen / PiP are what MCP Apps and the OpenAI Apps SDK define,
research note 05 §1; the side panel is the common app-builder and CRM layout, e.g. the side-panel form
in [R026]).

The user is trying to:
- see **what is being built** before it exists (the "generating this UI" wireframe in app builders,
  [../component-taxonomy.md](../component-taxonomy.md) section B, and the blueprint-to-real-UI reveal in
  [R005]);
- watch it **fill in place** without the transcript jumping;
- keep using the parts that work when one part fails;
- **regenerate** the whole thing or **edit one node in place**, and step between versions.

## 2. State machine

**Surface**
```
                  surface.begin
   (none) ───────────────────────▶ Wireframe ── first node renderable ──▶ Streaming
                                      │  (layout_hint or default shape)       │  nodes swap skeleton → content (per node, below)
                                      │                                       │  data.patch / nodes.upsert
                                      │ < 150 ms to first node: skip           │
                                      │   wireframe, go straight to Streaming  │ surface.commit / end(done)
                                      ▼                                       ▼
                                   Failed ◀── end(error) before any node    Ready ◀─────────────────────────┐
                                      │ Retry → Regenerating                  │                              │
                                                                              ├─ button / form → Submitting ─┤ intent ran (Answer/Query/Navigate)
                                                                              │        │ intent needs approval
                                                                              │        ▼
                                                                              │   AwaitingApproval ── approved → Submitting → Ready (receipt)
                                                                              │        └── denied / expired / stale → Ready (node shows outcome)
                                                                              ├─ Regenerate ─▶ Regenerating ─▶ Wireframe(ghost of vN) ─▶ … ─▶ Ready(vN+1)
                                                                              ├─ Edit ─▶ Editing(node) ─ prompt sent ─▶ Patching(node) ─▶ Ready(vN+1)
                                                                              └─ newer surface replaces this id ─▶ Stale
   Any phase: end(cancelled) / Stop ─▶ Cancelled (rendered nodes stay, skeletons fade)
              schema major mismatch ─▶ Degraded (text)        some nodes failed ─▶ PartialError overlay
   Overlays: Stalled (no event for stall_soft = 5 s, derived by oxyn-core) · Offline (intents disabled) · Offscreen (clock paused)
```

**Node** (inside an error boundary each)
```
  Referenced ─▶ Skeleton(shape) ── required props ──▶ Swapping (crossfade + height settle) ──▶ Live
      │               │ commit with props missing / validation fails                          │ data.patch
      │               ▼                                                                       ▼
      │          NodeError(kind) ── Retry (asks agent to resend node) ──▶ Skeleton        Updated (accent wash 1200 ms)
      └─ commit while never defined ─▶ Collapsed (footprint eases to 0, 200 ms)
  Render panic / custom component error inside Live ─▶ NodeError(Render) (boundary catches; siblings unaffected)
```

**Container**
```
   Inline ◀──▶ SidePanel ◀──▶ Fullscreen          Inline ──pop out──▶ PiP ──dock──▶ Inline
     ▲  agent ContainerRequest(Fullscreen|PiP|SidePanel) → RequestChip in Inline ("Open full screen") → user accepts
     └── app policy `auto_container` may accept SidePanel without asking (never Fullscreen, never PiP)
```

## 3. States table

### 3.1 Surface
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Wireframe` | wireframe of the declared layout (title bar, blocks) in skeleton tone with the shared sweep; caption "Generating interface…" under the container header | — | Stop (composer) | Esc in composer stops the run |
| `Streaming` | rendered nodes + skeleton nodes; header caption "Generating · 4 of ~9 blocks" when the hint gives a count, else "Generating…" | — | Stop | Tab reaches rendered interactive nodes only |
| `Ready` | full surface; container header: title, provenance icon, version "v2" (if > 1), Regenerate ⟲, Edit ✎, container controls | surface actions | Regenerate, Edit, Copy as text, container change | `⌘/Ctrl+Shift+R` regenerate (proposed, focus in surface), `E` edit mode, `F` fullscreen |
| `Submitting` | submitting node busy (button label morph "Sending…", loader 12 px); rest interactive | — | Cancel (if intent cancellable) | Esc cancel |
| `AwaitingApproval` | gated button / form shows amber hand + "Waiting for approval"; family-G card attached under the node (inline) or pinned in the container footer (panel / fullscreen / PiP) | answer card | — | family G keys ([approvals.md](approvals.md) §2.4) |
| `Regenerating` | previous version ghosted at 30 % opacity under the wireframe sweep (`ghost-previous` behaviour), header "Regenerating v3…" | — | Cancel (restores vN) | Esc cancel |
| `Editing(node)` | other nodes dim to 50 %; target node outlined 2 px accent with a prompt popover anchored to it: "Change this…" | Send edit | Cancel | Enter send, Esc cancel, Tab cycles nodes when picking |
| `Patching(node)` | target node in skeleton (its own shape) with the sweep; rest at full opacity | — | Cancel | Esc |
| `PartialError` | failed nodes as error tiles; header chip "2 parts failed · Retry all" | Retry all | per-tile Retry | Enter on tile |
| `Degraded` | markdown text fallback, chip "Shown as text" | — | Try again as UI | — |
| `Failed` | error card at the wireframe footprint: "Couldn't generate this interface" + reason class | Retry | Show as text (if any fallback) | Enter retry |
| `Stalled` | sweep slows to 0.5×; after 5 s caption "Still generating…" | — | Stop | — |
| `Cancelled` | rendered nodes stay; chip "Stopped"; skeletons fade out | Regenerate | — | — |
| `Stale` | dim 60 %, actions disabled with reason "Replaced by a newer version" | Go to latest | — | Enter |

### 3.2 Containers
| Container | Size (proposed) | Chrome | Enter / exit | When |
|---|---|---|---|---|
| `Inline` | transcript width; max height 640 px then inner scroll with fade mask; min height 96 px | header row 32 px (title, provenance, ⟲, ✎, ⤢ full screen, ⧉ pop out); 1 px outline | grows from its skeleton; never animates width | default |
| `SidePanel` | 360–640 px wide (default 420), full height, resizable by a 6 px edge handle | panel header 40 px with title, version switcher, ✕; transcript keeps a 1-line "Open in panel ▸" stub | slides in from the trailing edge 200 ms | dashboards next to chat, forms, CRM records |
| `Fullscreen` | window minus 24 px inset (desktop) / full viewport (mobile) | top bar 48 px: title, version, Regenerate, Edit, "Exit full screen (Esc)"; composer stays docked at the bottom (proposed) so the user can keep steering | shared-element expand from the inline frame, 260 ms | app previews, big tables, slides |
| `PiP` | 320 × 200 min, 480 × 300 default, 16:10; corner-snapped with 16 px margin; resizable | 28 px drag bar with title, ⤢, ⧉ dock, ✕ | shrinks from inline frame to nearest corner, 260 ms | keep a live chart or progress view visible while chatting |

## 4. Micro-interactions
- **Wireframe.** On `surface.begin`, the container allocates its footprint from `layout_hint` (each
  `SkeletonBlock` at its predicted size) or the default composition (1 title bar + 2 cards + 1 table,
  [skeletons.md](skeletons.md) §3) and fades in after the 150 ms show delay. The caption types nothing;
  it is a static label so the sweep is the only motion.
- **Skeleton → content per node.** Each node keeps its skeleton until its required props exist, then
  swaps with the shared crossfade (180 ms out / 180 ms in) and a height settle (200 ms `spring.snap`)
  with the **top edge fixed**. Siblings below shift once, smoothly. New nodes cascade at 30 ms stagger
  (cap 6 per frame burst).
- **Wireframe re-plan.** If arriving nodes contradict the hint (a chart where the hint said table), the
  mismatched skeleton crossfades to the right shape (120 ms); never morph unrelated shapes
  ([skeletons.md](skeletons.md) §7).
- **Error boundary.** A node that fails crossfades into an error tile of the same footprint (160 ms):
  icon + "Couldn't show this part" + Retry. No shake. Siblings never re-layout because of it beyond the
  one height settle.
- **Agent update after Ready.** Changed nodes get a 12 % accent wash fading over 1200 ms; numbers roll
  (400 ms); header tick "Updated just now" fades in and settles to relative time.
- **Regenerate.** ⟲ spins once per press (360°, 400 ms) and keeps spinning while regenerating (affectUI
  message actions behaviour, note 11); the previous version fades to 30 % and the wireframe sweep runs
  over it; new nodes replace ghosts in place.
- **Versions.** Header shows "‹ v2 of 3 ›" after the first regenerate or edit; stepping crossfades the
  body (180 ms) with height easing (200 ms) and the number rolls (affectUI version switcher, note 11).
  Older versions are read-only snapshots: their intents are disabled with reason "Older version — switch
  to latest to act".
- **Edit in place.** ✎ enters edit mode: hovering a node outlines it (1 px dashed), click selects (2 px
  solid), a popover opens with a one-line prompt "Change this…" and quick chips the app supplies ("Make
  it a bar chart", "Add a total"). Sending puts **only that node** in `Patching` with its own skeleton;
  the agent receives the node id, its current props (redacted) and the user's words.
- **Container changes.** Inline → fullscreen: the inline frame's rect expands to the fullscreen rect
  (shared-element transition, 260 ms `ease.in-out`); content re-lays out at the end of the expand (one
  re-layout, not per frame). Inline → PiP: frame shrinks to the nearest corner (260 ms) and the inline
  slot collapses to a 40 px stub "Showing in picture-in-picture · Return". PiP drag follows the pointer
  1:1 and snaps to a corner on release (`spring.soft`).
- **Container requests.** An agent's `ContainerRequest(Fullscreen)` never opens fullscreen by itself:
  a chip "Open full screen" slides in at the inline header (160 ms). `SidePanel` may auto-open when the
  app sets `auto_container = SidePanel`.
- **Submit.** Button press 0.97 scale; label morphs to "Sending…" (width springs, `spring.snap`);
  success morphs to a receipt line ("Submitted · Edit") and the form collapses (genui-forms.md); if the
  intent needs approval, the label becomes "Waiting for approval" with the amber hand.

## 5. Timing

| Transition | Duration | Delay / stagger | Easing / spring | Notes |
|---|---|---|---|---|
| Wireframe show delay / fade in | 180 ms | 150 ms | `ease.out` | skeletons.md |
| Node swap (crossfade out / in) | 180 ms / 180 ms | 30 ms stagger, cap 6 | `ease.in` / `ease.out` | skeletons.md swap |
| Height settle | 200 ms | — | `spring.snap` | top edge fixed |
| Min skeleton display | 300 ms incl. exit | content may start at +120 ms | — | skeletons.md |
| Hint mismatch reshape | 120 ms crossfade | — | `ease.out` | |
| Error tile in | 160 ms | — | `ease.out` | |
| Never-defined node collapse | 200 ms | — | `spring.snap` | |
| Agent-update wash | 1200 ms | — | `ease.out` | 12 % → 0 % |
| Regenerate spin | 400 ms per turn | — | linear | continuous while regenerating |
| Ghost previous fade | 200 ms to 30 % | — | `ease.out` | |
| Version step | 180 ms body crossfade + 200 ms height | — | `ease.in-out` / `spring.snap` | |
| Edit hover outline / select | 120 ms / 160 ms | — | `ease.out` / `spring.snap` | |
| Edit popover | 160 ms | — | `ease.out` | composer `dur.base` |
| Inline ↔ fullscreen | 260 ms | — | `ease.in-out` | `dur.travel` cap |
| Inline → PiP / dock | 260 ms | — | `ease.in-out` | |
| PiP corner snap | spring | — | `spring.soft` | |
| Side panel in / out | 200 ms / 160 ms | — | `ease.out` / `ease.in` | |
| Request chip in | 160 ms | — | `ease.out` | |
| Submit label morph | 160 ms | — | `spring.snap` | |
| Stall slow-down / caption | 2 s ramp / 160 ms | caption after 5 s | linear / `ease.out` | status strip thresholds |
| **Reduced motion** | no sweep, no cascade, no wash (2 px static accent bar for 3 s instead), no spin (⟲ swaps to a static "Regenerating…" label); every swap, version step and container change is an 80 ms opacity crossfade; heights jump once; PiP snaps without spring | — | — | |

## 6. Variants

### Free (4)
| Name | Idea |
|---|---|
| `genui-surface/inline` | Card in the transcript with header chrome, inner scroll past 640 px (default) |
| `genui-surface/side-panel` | Trailing panel beside the chat, resizable, transcript keeps a stub |
| `genui-surface/fullscreen` | Expanded surface with top bar and docked composer |
| `genui-surface/pip` | Corner-snapped floating window that survives scrolling and thread switches |

### Pro (12)
| Name | Idea |
|---|---|
| `genui-surface/carousel` | Several surfaces from one turn in a horizontal snap row (inline carousel, the OpenAI Apps SDK guidance names one, note 05) |
| `genui-surface/split-preview` | App-builder layout: prompt and log on the left, live preview right, wireframe in the preview while generating |
| `genui-surface/blueprint-build` | Dotted wireframe with mono size labels and corner handles draws in, then a scan-line reveals real nodes ([R005]) |
| `genui-surface/aurora-build` | A soft glow sweeps the region the agent is writing and content settles behind it ([R034]) |
| `genui-surface/cascade` | Nodes enter in tree order with a 6 px rise and 30 ms stagger; containers before contents |
| `genui-surface/versions-rail` | Thumbnail rail of versions along the panel edge; hover previews, two-up compare |
| `genui-surface/inspector-edit` | Edit mode opens an inspector of the selected node's props; the agent or the user tweaks them (Beautiful UI Fine-tune Card idea, [R106]) |
| `genui-surface/pinned` | Interactable surface pinned above the thread and updated in place by id, with "Updated by agent" ticks (Tambo interactables, note 05) |
| `genui-surface/diff-regenerate` | After regenerate, changed nodes carry a marker and a before / after toggle per node |
| `genui-surface/device-frames` | Desktop / tablet / mobile frame toggle around the preview (preview chrome idea, note 16) |
| `genui-surface/docked-pill` | Surface minimises to a status pill docked in the composer ("3 conflicts · +482 −37"), click to restore ([R054]) |
| `genui-surface/stage` | Centre-stage presentation with prev / next for slide-like surfaces |

## 7. Edge cases
- **No layout hint.** Default wireframe composition; reshape as nodes arrive (§4).
- **Hint much larger than result.** Never-defined nodes collapse at commit (200 ms); bottom-pinned
  transcripts compensate scroll.
- **Surface taller than 640 px inline.** Inner scroll with 24 px fade masks; a "Open in panel" button
  appears in the header; the transcript does not grow beyond that.
- **Two surfaces from one turn.** Stack inline (each with its own header) or `carousel` (Pro).
- **Surface replaced by id** (interactable update with a new `surface.begin` for the same id): the old
  one becomes `Stale` only if it was a different version; same version = in-place update.
- **User interacting while the agent patches.** Patches to a node with focus or unsaved user input are
  **held** (badge "Agent update waiting · Apply") until blur or explicit apply; never overwrite typed
  input (proposed).
- **Edit target removed** by a concurrent agent patch: edit popover closes with "That part changed —
  pick again".
- **Regenerate during approval.** Regenerating while an intent awaits approval makes the request
  `Stale` ([approvals.md](approvals.md) §3) and the card says "Interface regenerated · no longer needed".
- **PiP on small windows** (< 800 px wide): PiP disabled, request chip offers side panel instead.
- **Fullscreen on mobile:** full viewport; Esc becomes a ✕ in the top bar and the system back gesture.
- **Multiple PiPs:** at most 1 (proposed); a second pop-out replaces the first and docks it inline.
- **Window unfocused / offscreen.** Shared clock slows; offscreen surfaces stop ticking; patches still
  apply without animation.
- **Very fast agents.** Entire surface in < 150 ms: no wireframe; one fade (180 ms).
- **Very slow agents.** Wireframe up to minutes; `Stalled` caption; Stop always available in the
  composer.
- **RTL.** Side panel opens from the left (trailing edge in RTL); PiP default corner bottom-left; header
  controls mirror; version arrows mirror.
- **Touch.** Edit mode selects by tap; long-press opens the prompt popover; PiP drag by its bar only.
- **Dense IDE panel** (≤ 320 px): inline surfaces render in `genui-kit/dense`; fullscreen opens as an
  editor tab when the host supports it.
- **Terminal surface.** No PiP or fullscreen; side panel = split pane; wireframe drawn with `░` cells.

## 8. Accessibility
- **Container roles.** Inline / panel: `region` labelled "<title>, generated by <agent>"; fullscreen:
  `dialog` with `aria-modal="false"` (the composer remains reachable) and label "Full screen: <title>";
  PiP: `complementary` labelled "Picture in picture: <title>".
- **Busy.** `aria-busy="true"` during `Wireframe` / `Streaming` / `Regenerating` / `Patching`, with a
  hidden label "Generating interface"; skeletons `aria-hidden`.
- **Announcements (polite unless noted).** Ready: "Interface ready: <title>". Version step: "Version 2 of
  3". Edit sent: "Updating <node label>". Node error: "Part of the interface couldn't be shown. Retry
  available." Submit failure: assertive. Agent update held: "Agent update waiting".
- **Focus.** Container changes keep focus on the same node; entering fullscreen moves focus to the top
  bar's title only when the user triggered it; exiting returns focus to the ⤢ button. PiP never takes
  focus when it opens. Edit mode: arrow keys move between nodes in tree order; Enter selects.
- **Labels per state.** ⟲ "Regenerate interface" / "Regenerating, press Escape to cancel"; ✎ "Edit part
  of this interface"; ⤢ "Open full screen" / "Exit full screen"; ⧉ "Pop out" / "Return to conversation".
- **Contrast.** Ghosted previous version is decorative (`aria-hidden`); its 30 % opacity is not used for
  any readable text. Error tiles meet 4.5:1.
- **Reduced motion:** §5 last row.

## 9. Props sketch
```rust
pub struct GenSurfaceView {
    pub agent: AgentState,                    // Streaming / ToolRunning drive Wireframe / Streaming; Stalled → stall overlay
    pub surface: Entity<GenSurface>,          // genui.md §9
    pub container: Container,                 // Inline | SidePanel { width: Pixels } | Fullscreen | Pip { corner: Corner, size: Size<Pixels> }
    pub auto_container: Option<Container>,    // app policy; only SidePanel honoured automatically
    pub kit: GenKit,                          // Plain | Dense | Terminal | Pro(..)
    pub variant: SurfaceVariant,              // Inline | SidePanel | Fullscreen | Pip | Pro(..)
    pub skeleton: GenSkeletonCfg,             // genui-skeletons.md §9
    pub max_inline_height: Pixels,            // 640 (proposed)
    pub versions: VersionCfg,                 // { keep: 10, show_switcher_after: 2 }
    pub edit: Option<EditCfg>,                // None hides ✎; { quick_chips: Vec<SharedString> }
    pub hold_patches_on_input: bool,          // true (proposed)
    pub intents: Arc<IntentRegistry>,         // genui.md §2.6; side-effect intents create ApprovalRequests
    pub approvals: Entity<ApprovalQueue>,     // family G
    pub on_intent: Callback<IntentInvocation>,// app performs Answer / Query / Navigate; others come back via approvals
    pub on_regenerate: Option<Callback<RegenerateRequest>>,
    pub on_edit: Option<Callback<EditRequest>>,     // { node: NodeId, props_redacted: Value, prompt: SharedString }
    pub on_container_change: Callback<Container>,
    pub labels: SurfaceLabels,
    pub motion: MotionLevel,
}

pub enum NodeView {
    Skeleton { shape: SkeletonBlock },
    Swapping { since: Instant },
    Live,
    Updated { at: Instant },
    Error { kind: NodeErrorKind },            // MissingRequired | Invalid | UnknownType | Render | Data
    Collapsed,
}
```

## 10. Related components and references
Related: [genui.md](genui.md) (schema, security) · [genui-skeletons.md](genui-skeletons.md) ·
[skeletons.md](skeletons.md) · [container-loaders.md](container-loaders.md) (app-builder "generating this
UI" container, shared clock) · [border-beam.md](border-beam.md) (optional edge trace on the inline frame
while generating) · [approval-card.md](approval-card.md) · [composer-status-strip.md](composer-status-strip.md)
(stall thresholds) · [message-bubble.md](message-bubble.md) (regenerate / versions at message level).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| MCP Apps / OpenAI Apps SDK display modes inline / fullscreen / PiP; inline card and carousel guidance | research note 05 §1 | container set; `carousel` | not recorded in [../licensing.md](../licensing.md): concepts only |
| GenUISurface states building / partial / ready / validation-error / retrying → degrade to prose (Tambo, AG2 via note 05) | research note 05 §1 | surface machine | concepts only |
| Tambo interactables, "updated by agent" highlight and version tick | research note 05 §1 | `pinned`, update wash | concepts only |
| Jakub Antalik skeleton-blueprint → real UI with scan-line reveal | [R005] | `blueprint-build` | Study only; transitions.dev licence forbids competing kits |
| Aurora sweep over the region the agent is changing | [R034] | `aurora-build` | Study only |
| Merge-conflict gen-UI with status pill docked in the composer | [R054] | `docked-pill` | Study only |
| affectUI message actions: regenerate spin, version switcher crossfade with height easing | research note 11 | regenerate and versions motion | Paid / unclear: study only |
| Beautiful UI Fine-tune Card | [R106], [R133] | `inspector-edit` | MIT: port with attribution |
| Twenty CRM side-panel form with ⌘↵ submit | [R026] | side-panel container | Study only |
| Preview chrome Desktop / Tablet / Mobile toggles (BoardUI, Lightswind) | research note 16 | `device-frames` | BoardUI free tier MIT, Pro paid: study the Pro parts only |

**Gaps.** Whether adapters can stream per-node `done` markers (A2UI, AG-UI) was not researched; without
them nodes finalise at `surface.commit`. GPUI multi-window PiP (a real OS window vs an in-window overlay)
needs a spike.
