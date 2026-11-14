**Status:** v1 — implemented (plan #27); the draft below is kept as the design record.

# Spec: citations, sources, confidence, memory receipts, follow-ups, empty states

Part of family E ([messages.md](messages.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Six small components that hang off an agent
answer (or stand in for one when the thread is empty):

| Component | What it is | Free | Pro |
|---|---|---|---|
| `citation` | inline marker in the answer text + hover / focus preview card | 3 | 7 |
| `source-list` | the sources an answer used: stack, cards row, list, rail — including the pre-answer "searching" state | 3 | 7 |
| `confidence-badge` | claim provenance: Sourced / Inferred / Unverified (and a calibrated level only when the host supplies one) | 3 | 5 |
| `memory-receipt` | "Saved to memory", "Updated", "Forgot", "Used 2 memories" — with Undo / Manage | 3 | 6 |
| `follow-ups` | suggestion chips after the last answer | 3 | 6 |
| `empty-state` | greeting, starter prompts, recent work when a thread is empty | 3 | 6 |
| **Total** | | **18** | **37** = **55** |

## 1. Purpose
An agent answer is only as useful as the user's ability to **check it, trust it, and continue from it**.
Inline numbered citations with hover previews and a sources strip are now common (Perplexity, ChatGPT
search, Claude Research, Le Chat, Gemini; AI Elements / assistant-ui Inline Citation — note 04 §2e);
Perplexity loads source cards **before** the answer streams (note 04); follow-up / related chips and
starter grids are everywhere (note 04 §2g, §2i). The open gaps our research found are exactly where an
agent kit should lead: **memory transparency** — only Le Chat shows "clickable receipts" (when, where
from, why), ChatGPT shows "Memory updated", nobody shows a memory diff or "memory used" on an answer —
and **claim provenance** (sourced vs inferred vs guessed), seen only in the assistant-ui catalogue
(note 04 §3, items 6 and 8). This spec makes both first-class, with the same state discipline as the
rest of Oxyn.

## 2. State machines

### 2.1 Sources and citations (one flow; markers and the list share a `SourceSet`)
```
            ToolRunning{Search|Browse|Read}            first source returned            all fetches settled
  Empty ─────────────────────────────────▶ Searching(0) ─────────────────────▶ Searching(n) ─────────────────▶ Gathered(n)
                                              │ (skeleton cards after 150 ms)    │ cards fill one by one          │
                                              │                                   │                                │ Streaming: markers
                                              ▼ no results                        ▼ fetch error per card           ▼ resolve to sources
                                           NoSources ("No sources found")      card Failed / Blocked          Cited(m of n)
                                                                                                                  │ Done
                                                                                                                  ▼
                                                                                                            Settled(m cited, n−m uncited)
  Marker:  Pending(number, unresolved) ──▶ Resolved ──hover/focus 300 ms──▶ PreviewOpen ──leave 150 ms──▶ Resolved
                       │                        │ click / Enter                 │ pin (click on touch)
                       ▼                        ▼                               ▼
                 Orphan ("source missing")  Visited (opened)               PreviewPinned (Esc closes)
```

### 2.2 Memory receipt
```
  (agent proposes a memory) ──policy Ask──▶ Proposed ("Save to memory? Save · Not now") ── Save ──▶ Saving ──▶ Saved ── Undo (10 s) ──▶ Undone
                            ──policy Auto─────────────────────────────────────────────────────────▶ Saving ──▶ Saved     └─▶ Settled (Manage only)
                                                                                                     └─ error ─▶ SaveFailed (Retry)
  Kinds: Saved · Updated(before → after) · Forgot · Used(k) (recall, opens receipts) · Off (temporary chat)
```

### 2.3 Follow-ups
```
  Hidden ──Done──▶ Pending (300 ms) ──suggestions ready──▶ Ready ──hover/focus──▶ Hot ──click/Enter──▶ Chosen ──▶ Hidden
     ▲                │ not ready by 300 ms                   │ composer has a draft ─▶ Dimmed (50 %) ─▶ (draft cleared) ─▶ Ready
     │                ▼                                       │ new turn sent ─────────────────────────────────────────▶ Hidden (moved to history: removed)
     │             Loading (3 skeleton chips) ─ fail ─▶ Hidden (silent)
```

### 2.4 Empty state
```
  Mount ──▶ Ready(static starters) ──personalised starters arrive ≤ 500 ms──▶ Ready(personalised, crossfade)
              │  (arrive later than 500 ms → keep static; use them next time)       │
              │ user types in composer ─▶ Composing (starters fade + collapse)      │ pick a starter ─▶ Chosen (fills composer or sends)
              │ first send ─▶ Docking (hero composer docks; empty state exits) ─▶ (transcript)
  Overlays: Offline (network starters disabled with reason) · Temporary (memory off notice) · Restricted (policy notice)
```

## 3. States tables

### 3.1 `citation`
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Pending` | marker "1" at 60 % (source metadata not yet resolved) | — | — | focusable, label "Citation 1, loading" |
| `Resolved` | marker (number / domain pill per variant) | open preview | open source | Tab to focus; Enter opens preview |
| `PreviewOpen` | card 320 × ≤ 220 px: favicon, title (2 lines), domain · date, quoted snippet with the cited span highlighted, "Open ↗" and "Show in sources" | Open source | Show in sources | Enter on Open; Esc closes; Tab cycles inside |
| `PreviewPinned` | same, stays until dismissed (touch tap, or click on desktop when `click_action = Preview`) | Open | — | Esc |
| `Visited` | marker in visited tone (still ≥ 3:1) | open preview | — | — |
| `Orphan` | marker with a dotted outline; preview says "Source not available" | — | report (host) | — |
| Group (adjacent markers) | "1, 3" or "1–3" merged chip; preview shows a pager "1 of 3 ‹ ›" | page | — | ←/→ inside the preview |

### 3.2 `source-list`
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Searching(n)` | "Searching · 4 sources" with the tool verb (status-text pinned activity); skeleton cards after 150 ms, filled one by one | — | Stop (composer) | — |
| `NoSources` | "No sources found" muted row | — | — | — |
| `Gathered(n)` / `Cited(m)` | stack / cards / list; cited cards carry their number badge, uncited cards dim to 70 % with "Not cited" in the tooltip | open source | filter All / Cited | Tab into list; ↑/↓ or ←/→ between cards (roving) |
| card `Loading` | skeleton (title bar + domain bar) | — | — | — |
| card `Failed` | "Couldn't open · domain" with a ⚠ glyph | retry (host) | — | — |
| card `Highlighted` | when its marker is hovered/clicked: card scrolls into view, ring 2 px accent, 600 ms flash | — | — | — |
| `Expanded` (stack → list) | popover / sheet with all sources, grouped by type (Web, Files, Memory, Tools) | open | copy all links | Esc closes |

### 3.3 `confidence-badge`
| Level (vocabulary) | Meaning (the host / adapter decides; Oxyn never computes it) | Glyph | Default copy |
|---|---|---|---|
| `Sourced` | claim directly supported by a cited source | ● filled circle with ✓ | "Sourced" |
| `Inferred` | reasoned from sources, not stated in them | ◐ half circle | "Inferred" |
| `Unverified` | no supporting source | ○ hollow circle with ? | "Unverified" |
| `Calibrated(level)` | only when the host supplies a calibrated value: High / Medium / Low (never a bare % unless the host passes one with its meaning) | segmented bar | "High confidence" |
States per badge: `Resting`, `Hover/Focus` (tooltip after 300 ms: what the level means + "Why?" if the host gives a reason), `Disputed` (host marks a claim contradicted: danger outline + "Sources disagree").

### 3.4 `memory-receipt`
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Proposed` | inline card under the answer: "Save to memory? *Prefers metric units*" with an editable text | Save | Not now; Edit | Enter save, Esc not now |
| `Saving` | chip with 12 px loader still → spinner after 300 ms | — | — | — |
| `Saved` | chip "Saved to memory · Undo · Manage" (Undo visible 10 s with shrinking underline) | Undo | Manage (host opens memory panel) | Enter on focused action |
| `Updated` | "Updated memory" ; hover/expand shows before → after (strike-through old, new) | Undo | Manage | — |
| `Forgot` | "Forgot: *old address*" | Undo | — | — |
| `Used(k)` | footer chip "Used 2 memories ›" ; popover lists each: the memory text, when saved, where from (thread title / source), why used (host-provided) — Le Chat's "clickable receipts" (note 04) | open receipts | "Don't use this" per item (host) | Enter opens, Esc closes |
| `Off` | thread-level notice "Memory is off for this chat" (temporary / incognito) | — | turn on (host) | — |
| `SaveFailed` | danger chip "Couldn't save to memory · Retry" | Retry | — | — |
| `Undone` | chip crossfades to "Removed" for 1.2 s then exits | — | — | — |

### 3.5 `follow-ups`
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Loading` | 3 skeleton chips (if not ready 300 ms after Done) | — | — | — |
| `Ready` | 3 chips (max 5), each ≤ 60 chars, single line, truncate with tooltip | send (or fill — host policy) | — | Tab after the last message; ←/→ between chips; Alt+1…5 picks (proposed) |
| `Hot` | hover/focus: chip lifts 1 px, ↗ or ↵ glyph appears | same | — | Enter |
| `Dimmed` | composer has a draft: chips at 50 %, still clickable; a click never replaces the draft — it appends the suggestion on a new line (proposed) | fill | — | — |
| `Chosen` | chip text flies into the composer (fill) or becomes the user bubble (send) | — | — | — |

### 3.6 `empty-state`
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Ready` | greeting line, optional mark/orb (family B/L, still or calm loop), 4 starter cards (icon, title ≤ 32 chars, subtitle ≤ 60), optional recent work list | pick a starter | recent thread | focus starts in the composer; Tab reaches starters after it (proposed: composer first) |
| `Composing` | starters fade and collapse; greeting stays | — | — | — |
| `Chosen` | starter text fills the composer (default) or sends (host) | — | — | — |
| `Docking` | after the first send, hero composer docks to the bottom; empty state exits | — | — | — |
| `Offline` | starters needing network show "Needs connection" and are disabled | — | — | — |
| `Temporary` | notice "Temporary chat — not saved, memory off" above starters | — | — | — |

## 4. Behaviour rules
- **Marker numbering** follows first appearance in the answer, not the source list's fetch order; the
  list re-orders to cited-first, by number, when the answer completes (cards move with `spring.snap`;
  no move while the user hovers the list).
- **Marker placement:** after the punctuation of the claim it supports, no space before; a marker never
  starts a line (non-breaking join with the previous word).
- **Preview card** positions above the marker when space allows, else below; never covers the line
  being read; max width 320 px (mobile: bottom sheet).
- **Click behaviour** (host `click_action`): `Preview` (default on touch), `ScrollToSource` (default on
  desktop: highlight the card in the list / rail), `OpenLink` (⌘/Ctrl+click always opens).
- **Snippets** are shown only if the adapter supplies them; Oxyn never fetches pages itself.
- **Source types:** Web (favicon + domain), File (file icon + path, opens in host), Memory (memory glyph;
  links to the memory receipt), Tool output (tool glyph + tool name), Transcript moment (timestamp — Pro
  `citation/timestamp`).
- **Confidence** is shown only when the host supplies it; the default is no badge. Never colour alone:
  glyph shape + word.
- **Memory policy** (host): `Ask` (default for new apps — proposed), `Auto` (save + receipt with Undo),
  `Off`. Receipts are always shown for saves and updates; `Used(k)` is opt-in per app.
- **Follow-up action** (host): `Send` when the composer is empty and idle (default), `Fill` otherwise;
  never send while a run is active (fill instead, so the composer's `busy_action` decides).
- **Starters**: static defaults render first; personalised starters may replace them only within 500 ms
  of mount (no late reshuffle under the pointer — Antigravity shows a dot rather than reshuffling, note
  08, same principle).

## 5. Micro-interactions
- **Marker resolve:** Pending → Resolved opacity 60 → 100 % (120 ms).
- **Preview open:** after `delay.preview.open` 300 ms; fade + scale 0.98 → 1 + y 4 → 0 (160 ms
  `ease.out`); the cited span inside the snippet gets a highlight sweep left → right (240 ms, once).
  Close after `delay.preview.close` 150 ms grace (moving the pointer from marker to card keeps it open).
  Moving between adjacent markers swaps content in place (120 ms crossfade, no close/open).
- **Marker ↔ card link:** hovering a marker rings its card in the list/rail; hovering a card
  underlines its markers in the answer (both 120 ms).
- **Searching:** cards fill one by one as results arrive: `dur.row.enter` 180 ms, stagger 40 ms, cap 5
  (stack variant: favicons slide into the stack with −6 px overlap, 160 ms).
- **Stack → list expand:** favicons spread into rows (200 ms `spring.snap`).
- **Re-order on done:** cards glide to cited-first order (220 ms `spring.snap`).
- **Confidence tooltip:** 300 ms delay, 120 ms fade.
- **Memory saved:** chip enters with `dur.row.enter`; memory glyph does one 400 ms pulse (scale 1 →
  1.15 → 1); Undo underline shrinks over 10 s (linear); Undone crossfade 160 ms then exit 200 ms.
- **Memory updated:** before → after shown as strike-through fading out (300 ms) while new text fades in.
- **Follow-ups:** appear 300 ms after Done; chips rise 6 px + fade (180 ms), stagger 40 ms, cap 5; on
  choose, the chip text flies to the composer (fill: 220 ms `ease.in-out`) or to the transcript (send:
  ≤ 260 ms, same as composer fly-up); the other chips fade out 120 ms.
- **Empty state:** starters stagger in on mount (180 ms, 40 ms, cap 4); on first keystroke they fade
  (160 ms) and the area collapses (200 ms); on send the hero composer docks (Pro `glass-hero`,
  `dur.travel` 260 ms `ease.in-out`).

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Marker resolve | 120 ms | 0 | `ease.out` | — | 60 Hz |
| Preview open / close | 160 / 120 ms | 300 ms / 150 ms grace | `ease.out` / `ease.in` | — | 60 Hz |
| Snippet highlight sweep | 240 ms once | on open | `ease.out` | — | 60 Hz |
| Preview content swap (adjacent markers) | 120 ms | 0 | `ease.in-out` | — | 60 Hz |
| Marker ↔ card ring / underline | 120 ms | 0 | `ease.out` | — | 60 Hz |
| Source card enter | 180 ms, y +4 → 0 | skeletons after 150 ms | `ease.out` | 40 ms, cap 5 | 60 Hz |
| Stack → list | 200 ms | 0 | `spring.snap` | — | 60 Hz |
| Re-order on done | 220 ms | 0 (held while hovered) | `spring.snap` | — | 60 Hz |
| Card highlight flash | 600 ms hold + 900 ms fade | 0 | `ease.out` | — | 30 Hz |
| Confidence tooltip | 120 ms | 300 ms | `ease.out` | — | 60 Hz |
| Memory chip enter / glyph pulse | 180 / 400 ms | 0 | `ease.out` / `spring.soft` | — | 60 Hz |
| Undo window | 10 000 ms underline | 0 | linear | — | 15 Hz |
| Updated before → after | 300 ms | 0 | `ease.in-out` | — | 60 Hz |
| Follow-ups enter | 180 ms, y +6 → 0 | 300 ms after Done | `ease.out` | 40 ms, cap 5 | 60 Hz |
| Follow-up chosen (fill / send) | 220 / ≤ 260 ms | 0 | `ease.in-out` / `ease.in` | — | 60 Hz |
| Starters enter / fade on typing / collapse | 180 / 160 / 200 ms | 0 | `ease.out` / `ease.in` / `spring.snap` | 40 ms, cap 4 | 60 Hz |
| Hero composer dock (Pro) | 260 ms | on first send | `ease.in-out` | — | 60 Hz |
| **Reduced motion** | all → 0–80 ms opacity crossfades; no sweeps, pulses, glides or staggers; previews still wait 300 ms (intent detection, not motion); Undo shown as "Undo (10 s)" text counting at 1 Hz; follow-ups and starters appear together | — | linear | none | on change |

## 7. Variants

### 7.1 `citation` — Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `citation/number` | Small pill "1" raised 0.2 em, tabular digits (default) | chat, desktop, ide |
| `citation/domain-pill` | Favicon + domain "nature.com +2" inline pill | chat, mobile |
| `citation/footnote` | Plain `[1]` text markers + numbered sources list at the end; no hover cards | terminal, plain-text exports |

### 7.2 `citation` — Pro (7)
| Name | One-line idea |
|---|---|
| `citation/highlight-span` | The claim text itself is the trigger: soft underline; hover shows the supporting source |
| `citation/favicon-dot` | 12 px favicon dots instead of numbers, stacked when several |
| `citation/sentence-pin` | Every sourced sentence gets a pin in the gutter; click opens the source at the quoted passage (Scrim UI sentence-pinned citations, note 01) |
| `citation/quote-peek` | Preview shows the exact quoted passage with surrounding context and a mini scroll position in the source |
| `citation/timestamp` | Markers point to a moment in an audio/video transcript and play from there (Notion meeting notes idea, note 04) |
| `citation/inline-card` | Click expands the source card inline under the paragraph instead of a popover |
| `citation/count-ring` | Marker ring split into segments = number of sources supporting the claim |

### 7.3 `source-list` — Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `source-list/stack` | Overlapping favicon stack + "10 sources" button → popover list (default) | chat, mobile |
| `source-list/cards-row` | Horizontal row of cards (favicon, title, domain) above or below the answer, cited ones numbered | chat, desktop |
| `source-list/list` | Numbered list under the answer; text-only fallback | ide, terminal |

### 7.4 `source-list` — Pro (7)
| Name | One-line idea |
|---|---|
| `source-list/rail` | Side rail synced with scrolling: the sources for the visible paragraph rise to the top |
| `source-list/loading-first` | Cards populate before the answer streams; the answer slides in under them (Perplexity pattern, note 04) |
| `source-list/favicon-morph` | The stack morphs into a full row on hover (SmoothUI ai-sources idea, note 01) |
| `source-list/clusters` | Sources grouped by domain / type into small clusters with counts |
| `source-list/timeline` | Sources on a date axis — for news and research recency |
| `source-list/reader` | Full-screen Sources tab with filters, search and per-source "cited in" back-links (report viewers, note 04) |
| `source-list/trust-meter` | Shows a host-supplied reliability cue per source (never computed by Oxyn) |

### 7.5 `confidence-badge` — Free (3) / Pro (5)
| Name | Tier | Idea |
|---|---|---|
| `confidence-badge/label` | Free | Text chip "Sourced" / "Inferred" / "Unverified" with glyph |
| `confidence-badge/icon` | Free | Glyph only (●✓ ◐ ○?) with tooltip — dense UIs |
| `confidence-badge/underline` | Free | Claim underline style: solid (sourced), dashed (inferred), dotted (unverified) |
| `confidence-badge/meter` | Pro | Segmented bar for a host-calibrated level, with alternatives (Beautiful UI recommendation card confidence meter, note 17) |
| `confidence-badge/heat-margin` | Pro | Paragraph-level margin strip shaded by the weakest claim in it |
| `confidence-badge/alternatives` | Pro | Low confidence opens alternatives the agent considered |
| `confidence-badge/why` | Pro | Hover explains the level with the evidence list |
| `confidence-badge/claim-map` | Pro | Report view listing every claim with its level and sources |

### 7.6 `memory-receipt` — Free (3) / Pro (6)
| Name | Tier | Idea |
|---|---|---|
| `memory-receipt/chip` | Free | Inline chip under the answer "Saved to memory · Undo · Manage" (default) |
| `memory-receipt/toast` | Free | Transient toast at the composer's top edge with Undo |
| `memory-receipt/footer-line` | Free | Dim footer text "Used 2 memories ›" / "Saved 1 memory" — terminal and IDE |
| `memory-receipt/diff` | Pro | Saved / updated / forgotten shown as a diff with before → after |
| `memory-receipt/consent-card` | Pro | Ask-before-save card with editable wording and scope ("this project only") |
| `memory-receipt/recall-trail` | Pro | Memory glyphs fly from the receipt to the sentences that used them |
| `memory-receipt/glyph` | Pro | An animated memory mark that lights when a memory is recalled (supermemory brain-to-graph idea [R010]) |
| `memory-receipt/why-personalized` | Pro | "Why this answer is personalised" explainer listing the memories and their influence |
| `memory-receipt/incognito-banner` | Pro | Distinct temporary-chat frame and banner state |

### 7.7 `follow-ups` — Free (3) / Pro (6)
| Name | Tier | Idea |
|---|---|---|
| `follow-ups/chips` | Free | Wrapped row of pill chips (default) |
| `follow-ups/list` | Free | Vertical "Related" list with ↳ glyphs |
| `follow-ups/ghost` | Free | One ghost suggestion inside the empty composer; Tab or → accepts, Enter does not send it (Claude desktop ghost prompt, note 03) |
| `follow-ups/radiate` | Pro | Chips radiate out from the end of the answer (SmoothUI ai-suggestions idea, note 01) |
| `follow-ups/marquee` | Pro | Chips scroll sideways slowly; pause on hover ([R019]) |
| `follow-ups/action-cards` | Pro | Next actions with icons ("Create PR", "Run tests") instead of questions (Codex app suggested Create PR, note 03) |
| `follow-ups/preview` | Pro | Hovering a chip previews the first line the agent would answer with (host supplies) |
| `follow-ups/voice` | Pro | Spoken-suggestion captions for voice mode, tappable |
| `follow-ups/question-card` | Pro | Structured multiple-choice clarifier instead of free chips (note 04 clarifying question card) |

### 7.8 `empty-state` — Free (3) / Pro (6)
| Name | Tier | Idea |
|---|---|---|
| `empty-state/starters-grid` | Free | Greeting + 4 starter cards + optional recent threads (default) |
| `empty-state/minimal` | Free | One greeting line + 3 starter chips; nothing else |
| `empty-state/terminal` | Free | ASCII banner + numbered tips and `/` commands list |
| `empty-state/glass-hero` | Pro | Large hero composer with an orb that docks to the bottom on first send (pairs with `composer/glass-hero`) |
| `empty-state/ascii-globe` | Pro | Rotating ASCII wireframe that slows at readable poses (Warp empty state idea, note 08; [R011]) |
| `empty-state/time-greeting` | Pro | Time-of-day greeting with ambient art (Threadbear "Good night" idea [R053]) |
| `empty-state/recent-work` | Pro | Resume cards for recent threads and runs with status and diff stats |
| `empty-state/capability-tour` | Pro | Each starter card plays a 3 s mini run (thinking → tool → answer) on hover |
| `empty-state/category-tabs` | Pro | Starters grouped by category tabs (Code, Research, Write…) per app preset |

## 8. Edge cases
- **Many citations (50+):** markers stay inline; the list collapses to the stack with "50 sources";
  numbers ≥ 10 widen the pill to the 2-digit reserved width (status-text `ReserveMagnitude`).
- **Citations streaming before sources resolve:** `Pending` markers; if a marker never resolves by
  `Done`, it becomes `Orphan` (no broken link).
- **Duplicate sources:** deduplicated by canonical URL; markers point to the single card.
- **Long titles / URLs:** titles clamp at 2 lines; domains middle-truncate; full in the tooltip.
- **RTL:** markers follow the claim at the logical end; numbers in bidi isolates; cards row scrolls
  from the right; favicon stack overlaps toward the inline-end.
- **Tiny sizes:** `citation/number` at 10 px min font with a 24 px hit area (IDE) / 44 px (touch) via
  an invisible padded target; `confidence-badge/icon` for ≤ 16 px rows.
- **Dense IDE:** `source-list/list` collapsed by default to "6 sources ›".
- **Touch:** previews are tap-to-pin sheets; follow-up chips scroll horizontally when they would wrap
  past 2 rows.
- **Slow networks:** source cards keep skeletons; favicons fall back to a monogram tile after 1 s.
- **Very fast agents:** sources and answer land together — cards render settled, no fill animation.
- **Very slow searches:** `Searching(n)` count and the status line carry liveness; skeletons stop
  sweeping after `stall_soft` and show "Still searching…".
- **Memory receipt on a branch:** receipts belong to the version that created them; switching versions
  shows that version's receipts; Undo on an old version asks for confirmation (the memory may be used
  since).
- **Follow-ups after an error or interrupt:** not shown (proposed); the message's Retry is the next step.
- **Empty state with a pre-filled draft** (deep link): starters hidden, composer focused.
- **Starter text longer than the composer limit:** rejected at authoring (codegen check, 2 000 chars).

## 9. Accessibility
- **Citation marker:** a link-styled button, name "Citation 1: nature.com, ‘Title…’" (domain + title
  when resolved); `aria-describedby` the preview when open; preview is a non-modal dialog
  (`role="dialog"`, labelled by the source title) reachable with Tab from the marker and closed with Esc;
  focus returns to the marker. Hover-only content is also available on focus (WCAG 1.4.13: dismissible,
  hoverable, persistent).
- **Sources list:** `role="list"`; cards are links with "Source 3 of 10, cited" / "not cited"; filter is
  a radio group.
- **Searching:** status updates via the composer live region at most every 10 s ("Found 6 sources").
- **Confidence:** badge text is the accessible name ("Inferred claim"); glyph `aria-hidden`; colour never
  carries meaning alone.
- **Memory receipts:** "Saved to memory: prefers metric units. Undo available for 10 seconds" announced
  politely once; Undo is a button; the 10 s window extends while the receipt has focus (WCAG 2.2.1 —
  proposed: timer pauses on focus/hover).
- **Follow-ups:** `role="group"` labelled "Suggested follow-ups"; each chip a button; not announced on
  appearance (they are optional), but the group is the next Tab stop after the last message.
- **Empty state:** greeting is a heading; starters are buttons in a group "Suggestions"; the composer
  keeps initial focus.
- **Contrast:** markers ≥ 4.5:1 (they are text); dimmed uncited cards ≥ 4.5:1 for text; skeletons
  decorative (`aria-hidden`).
- **Reduced motion:** §6 last row.

## 10. Props sketch
```rust
pub struct SourceSet {                        // shared by citations, list and badges for one agent message
    pub sources: Vec<Source>,                 // { id, kind: Web{url, domain, favicon} | File{path} | Memory{id} | Tool{name} | Moment{media, at},
                                              //   title, date: Option<Date>, snippet: Option<Snippet{text, cited_span}>, status: Loading|Ready|Failed|Blocked }
    pub phase: SourcePhase,                   // Empty | Searching { found } | NoSources | Gathered | Cited { cited } | Settled
}
impl SourcePhase {
    /// ToolRunning{Search|Browse|Read} → Searching; Streaming → Cited as markers resolve; Done → Settled.
    pub fn derive(agent: &AgentState, set: &SourceSet) -> Self { /* … */ }
}

pub struct Citation {
    pub index: u16, pub source: Option<SourceId>, pub group: Vec<SourceId>,
    pub variant: CitationVariant,             // Number | DomainPill | Footnote | Pro(..)
    pub click_action: ClickAction,            // ScrollToSource (desktop default) | Preview (touch default) | OpenLink
    pub preview: PreviewCfg,                  // { open_delay: 300 ms, close_grace: 150 ms, max_width: 320 px }
}

pub struct SourceList { pub set: SourceSet, pub variant: SourceListVariant, pub order: Order /* CitedFirst */, pub filter: SourceFilter, pub skeleton_after: Duration /* 150 ms */ }

pub struct ConfidenceBadge { pub level: Provenance /* Sourced | Inferred | Unverified | Calibrated(High|Medium|Low) */,
                             pub disputed: bool, pub reason: Option<String>, pub variant: ConfidenceVariant }

pub struct MemoryReceipt {
    pub kind: MemoryKind,                     // Proposed{text} | Saved{text} | Updated{before, after} | Forgot{text} | Used(Vec<MemoryRef>) | Off
    pub policy: MemoryPolicy,                 // Ask (default) | Auto | Off
    pub undo_window: Duration,                // 10 s, pauses on focus/hover
    pub variant: MemoryVariant, pub on_action: Callback<MemoryAction>,
}

pub struct FollowUps {
    pub items: Vec<Suggestion>,               // ≤ 5, each ≤ 60 chars
    pub agent: AgentState,                    // shown only after Done on the last turn; hidden on Error/Cancelled
    pub action: FollowUpAction,               // Auto (Send when composer empty & idle, else Fill) | Send | Fill
    pub delay: Duration,                      // 300 ms
    pub variant: FollowUpsVariant,
}

pub struct EmptyState {
    pub greeting: Option<String>, pub starters: Vec<Starter>, pub personalised: Option<StarterFuture>,
    pub swap_window: Duration,                // 500 ms
    pub recent: Vec<ThreadRef>, pub on_pick: PickAction /* Fill (default) | Send */,
    pub variant: EmptyVariant, pub mark: Option<LoaderVariant>,
}
```

## 11. Related components and references
Related: [streaming-markdown.md](streaming-markdown.md) (marker parsing) ·
[text-reveal.md](text-reveal.md) (marker appears when `]` completes) · [message-bubble.md](message-bubble.md)
(footer) · [messages.md](messages.md) (deep-link flash, transcript) · [skeletons.md](skeletons.md) ·
[status-text.md](status-text.md) (Searching verb) · [composer.md](composer.md) (`composer/glass-hero`,
busy policy for follow-ups) · family F (search tool rows) · family J (memory panel / settings) ·
family K (voice suggestions).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| Perplexity: numbered citations, favicon cards, sources load before the answer, Related questions | note 04 | `loading-first`, `follow-ups/list` | Product — study only |
| ChatGPT search / Sources button; "Memory updated" chip → Manage memories | note 04 | stack, memory chip | Product — study only |
| Le Chat memory "clickable receipts" (when, where from, why) | note 04 | `Used(k)` receipts | Product — study only |
| assistant-ui claim provenance / confidence; Inline Citation | notes 01, 04 | provenance vocabulary | MIT — may port with attribution |
| Vercel AI Elements Inline Citation, Sources, Suggestion; prompt-kit Source, Prompt Suggestion | note 01 | API shapes | Apache-2.0 / MIT — may port with attribution |
| affectUI AI Answer (citation chips preview on hover, inline source cards, follow-ups), Memory Panel | note 11 | interplay | Paid — study only |
| Beautiful UI Streaming Text (inline sources, "10 sources" favicon stack, Follow-ups) and Recommendation Card (confidence meter, alternatives) | [R133], [R106], note 17 | stack + confidence meter | MIT — may port with attribution |
| SmoothUI ai-sources (favicon morph to row), ai-suggestions (radiate) | note 01 | `favicon-morph`, `radiate` | MIT — may port with attribution |
| Scrim UI sentence-pinned citations | note 01 | `sentence-pin` | Has a Pro tier; licence not recorded — study only |
| Notion meeting notes (summary → transcript timestamp) | note 04 | `citation/timestamp` | Product — study only |
| Claude desktop ghost next-prompt (Tab accepts, Enter doesn't send) | note 03 | `follow-ups/ghost` | Proprietary — idea only |
| Codex app suggested "Create PR" | note 03 | `action-cards` | Product — idea only |
| Manus / Gemini / ChatGPT starter grids; ChatGPT Pulse finite feed | note 04 | empty state | Products — study only |
| Warp ASCII wireframe empty state | note 08 | `ascii-globe` | AGPL — study only, re-implement |
| @david_visuals_ sideways suggestion chips | [R019] | `marquee` | Study only |
| supermemory memory-extraction visual | [R010] | `memory-receipt/glyph` | Study only |
| praveenisomer ASCII globe | [R011] | `ascii-globe` | Study only |
| Threadbear "Good night" greeting | [R053] | `time-greeting` | Study only |


## Plan #27 implementation — source cards, AI / cited answer, context and recommendation cards, document extraction, research run

Implemented in `oxyn-core::chat::sources` (`SourceCards`, `Slot`, `Claim`, `Provenance`, `grounding`, `ContextCard`, `Recommendation`, `Extraction`) and `chat::thread::ResearchRun`; `oxyn-dioxus::{SourceCards, SourceCitation, AiAnswer, ContextCards, RecommendationCard, DocumentExtraction, ResearchRun}`; `oxyn-gpui::{SourceCardsEl, AnswerEl}`.

### 27.1 Purpose
Show where an answer came from and how much to trust it: sources while searching and after, claims with provenance, the context an agent used, recommendations, extracted fields and long research runs.

### 27.2 State machine
```
SourceCards: Searching(n of total, skeleton slots) ─▶ Done(n · 2.1s, stack after max) | Error(Retry) | Done(0) “No sources found”
Citation press ─▶ highlight card 2.6 s (opens the stack when needed)
AiAnswer: Searching ─▶ Streaming answer ─▶ Done (actions, follow-ups); cited: claims + grounding
Extraction: Scanning(one field per step) ─▶ Review(pending n) ─review all─▶ Confirm
Research: Planning ─▶ Searching ─▶ Reading ─▶ Writing ─▶ Done(report)
Recommendation: Open ─Accept / Dismiss─▶ Resolved ─Undo─▶ Open
```

### 27.3 States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| searching | spinner, count, skeletons | Tab | list `aria-busy` |
| done | cards, stack | Enter opens | card names: n, title, domain |
| highlighted | outlined card | — | outline, not colour only |
| claim | text + chip / provenance pill | Tab | words + icon |
| field review | amber border, Looks right | Tab | confidence label in % |

### 27.4 Micro-interactions
Cards rise 4 px in 180 ms as they arrive; skeletons pulse; hover lifts a card and shows its preview after hover; the favicon stack overlaps; grounding bar fills; extraction scan line sweeps; research favicons roll in.

### 27.5 Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Card in | 180 ms | ease-out | instant |
| Highlight | 2.6 s hold | — | same |
| Preview | 160 ms | ease-out | instant |
| Scan line | 400 ms per step | linear | static |

### 27.6 Variants
Free: source cards with stack, citations, AI answer with follow-ups.

**Oxyn plus:** cited variant with provenance (sourced · inferred · unverified) and grounding meter; Beautiful UI Context Cards (pin, remove, token meter) and Recommendation Card (confidence band, impact, evidence, Undo); document extraction with threshold; research run timeline.

### 27.7 Edge cases
Zero sources shows “No sources found” with the query. Citations to missing sources render as plain chips. Stack opens when a citation points into it. Very long titles clamp to two lines.

### 27.8 Accessibility
Sources are a labelled list; a polite region announces status once; cards say they open a new tab; previews are tooltips; provenance and confidence are text, never colour alone.

### 27.9 Props sketch
```rust
SourceCards { sources, status, total, query, duration_ms, max, elapsed_ms, step_ms, highlighted, error, expanded, children, on_retry }
AiAnswer { question, answer, sources, claims, variant: "answer"|"cited", elapsed_ms, search_ms, follow_ups, on_follow_up }
DocumentExtraction { document, fields: Vec<Field>, elapsed_ms, step_ms, threshold }
ResearchRun { question, queries, sources, elapsed_ms }
RecommendationCard { title, reason, confidence, impact, evidence }   ContextCards { cards, budget, layout }
```

### 27.10 Related components and references
`streaming-markdown.md`, `hover-card.md`. Arc UI Source cards, AI answer, Document extraction, Research run and affectUI AI Answer / Cited Answer — behaviour only; Beautiful UI Context Cards and Recommendation Card — MIT, © 2026 Shane Levine.
