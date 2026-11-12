# Gallery and filter model

**Status:** draft v0.1 (2026-11-12) · roadmap phase V2.5 · PR #10 `docs/site-studio-ia`.
Companion: [ia.md](ia.md) (site map, page templates, demo levels, content model).

The gallery lives at **`/components`** and shows **one card per variant** (`<component>/<style>`,
[../variants.md](../variants.md)). Family index pages (`/components/<family>`) are the same gallery
with `family` locked and cards grouped by component. Data comes from a build-time `gallery.json`
(filtered client-side, instant); Pagefind handles text search and ⌘K
([../website-stack.md](../website-stack.md), research `18-website-stack`).

## 1. Layout

```text
┌ nav ───────────────────────────────────────────────────────────────────────────────────┐
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Components   912 variants · 214 components                                             │
│ [⌕ Search variants…            ] Sort [Featured ▾]  View [▦ Cozy ▾]  ◐ Dark  ☐ Reduce  │
│ Active: (State: thinking ×) (Surface: ide, terminal ×) (Tier: Free ×)  Clear all       │
├ Filters ─────────────┬ Results 37 ─────────────────────────────────────────────────────┤
│ Family          ▾    │ ┌ card ──────────┐ ┌ card ──────────┐ ┌ card ──────────┐        │
│ Category        ▾    │ │                │ │                │ │                │        │
│ State           ▾    │ └────────────────┘ └────────────────┘ └────────────────┘        │
│ Surface         ▾    │ ┌ card ──────────┐ ┌ card ──────────┐ ┌ card ──────────┐        │
│ Layout          ▸    │ │                │ │                │ │                │        │
│ Motion          ▸    │ └────────────────┘ └────────────────┘ └────────────────┘        │
│ Gesture         ▸    │                                                                 │
│ Tier            ▸    │                  [Load 24 more]  (also on scroll)               │
│ Framework       ▸    │                                                                 │
├──────────────────────┴─────────────────────────────────────────────────────────────────┤
│ Compare tray:  [ring-progress ×] [pill ×]   2 / 4        [Compare →]  [Clear]          │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

| Zone | Behaviour |
|---|---|
| Header | Totals update with filters; search is debounced 150 ms |
| Active chips | One chip per facet (values joined), × removes that facet; "Clear all" resets everything except view settings |
| Filter panel | Collapsible sections; first four open by default; each value shows a count (§3.3) |
| Results | Server-rendered first 24 cards (real HTML, posters); more on scroll or button |
| Compare tray | Appears when ≥ 1 item is picked (§10.1) |
| Mobile (< 768 px) | Panel becomes a "Filters (n)" bottom sheet; header collapses to search + sort + sheet button |

## 2. Facets

Values are exact and come from the docs named in the Source column. Keys are the URL keys (§4) and the
Pagefind filter keys (§11).

| Key | Label | Values | Source | Per item |
|---|---|---|---|---|
| `family` | Family | `composer` A, `loaders` B, `status-text` C, `container-loaders` D, `messages` E, `tools` F, `approvals` G, `checkpoints-git` H, `genui` I, `workspace-chrome` J, `live-voice` K, `icons` L, `shells` M | [../component-taxonomy.md](../component-taxonomy.md) §A; slugs per [ia.md](ia.md) §3.1 | one |
| `category` | Category | `coding`, `app-builder`, `crm`, `finance`, `video`, `support`, `research`, `data-bi`, `computer-use`, `voice`, `multi-agent-ops` | taxonomy §B, [../categories/README.md](../categories/README.md) | many |
| `state` | State | `idle`, `composing`, `submitted`, `thinking`, `streaming`, `tool-running`, `awaiting-approval`, `queued`, `steering`, `done`, `error`, `cancelled`, `stalled` | `AgentState` in [../architecture.md](../architecture.md); kebab names as `color.state.*` in [../tokens.md](../tokens.md) | many |
| `surface` | Surface | `chat`, `ide`, `desktop`, `mobile`, `voice`, `terminal` | [../variants.md](../variants.md) Axes; taxonomy §C | many |
| `layout` | Layout | `compact`, `inline`, `card`, `panel`, `floating` | variants.md Axes | many |
| `motion` | Motion | `none`, `subtle`, `expressive` | variants.md Axes; taxonomy §C | one |
| `gesture` | Gesture | `hover`, `press`, `drag`, `type`, `select`, `loading`, `scroll`, `morph`, `reveal` | taxonomy §C | many |
| `tier` | Tier | `free`, `pro` | taxonomy §C | one |
| `framework` | Framework | `gpui`, `dioxus` | registry files `gpui.rs` / `dioxus.rs` ([../architecture.md](../architecture.md)) | many |

**View settings (not filters):**

| Key | Values | Why not a filter |
|---|---|---|
| `theme` | `dark` (default), `light`, `hc-dark` | Every variant supports every theme: "same keys in every theme" ([../tokens.md](../tokens.md) principle 6). Filtering would never remove a card, so theme re-skins posters and demos instead. |
| `rm` | `1` = reduced motion | Shows every card in its reduced form ([../motion.md](../motion.md) §8); a preview of the fallback, not a filter |
| `density` | `comfy`, `cozy` (default), `compact` | §7 |

**Advanced (hidden behind "More filters", proposed):** `render` = `t0`…`t4` (render tiers,
[../specs/loaders.md](../specs/loaders.md) §2.2; only items that declare one), `status` = `new`, `beta`.

Display order of values: taxonomy order (A→M, the `AgentState` lifecycle order above), never
alphabetical, so the state facet reads like a run.

## 3. How facets combine

### 3.1 Rule

| Within one facet | Across facets | Text query |
|---|---|---|
| **OR** (`state=thinking,streaming` = supports thinking *or* streaming) | **AND** (`state=thinking&surface=ide` = both) | AND with all facets |

An item with many values matches a facet when **any** of its values is selected.

### 3.2 Opt-in AND within a facet

`+` joins values that must **all** be present. Only meaningful on many-valued facets
(`state`, `surface`, `layout`, `gesture`, `category`, `framework`).

| URL | Meaning |
|---|---|
| `framework=gpui,dioxus` | has GPUI **or** Dioxus (default) |
| `framework=gpui+dioxus` | has **both** |
| `state=thinking+error+stalled` | handles all three (useful for picking a loader that covers failure) |

UI: a small "Match any / all" toggle at the top of those facet sections. Mixing `,` and `+` in one
facet is invalid; the parser keeps the first separator it sees.

### 3.3 Counts

Disjunctive counts: a value's count is "results if this value were toggled", computed with every
*other* facet applied and this facet's own selection ignored (so OR siblings never show 0 just because
one sibling is picked). Values with 0 stay visible, dimmed, `aria-disabled`, still clickable only to
remove. This matches what Pagefind returns as `filters` vs `totalFilters`
(https://pagefind.app/docs/js-api-filtering/).

### 3.4 State filter pins the preview

When `state` is set, every card shows that state (the first selected one) instead of its default
poster. "Show me every component while it's thinking" is a first-class view. Cards whose variant does
not support the pinned state are already filtered out.

## 4. URL query format

The whole gallery view is in the query string, so any view is shareable and back/forward works.

```text
/components?q=orb&family=loaders,status-text&state=thinking&surface=ide+terminal
           &tier=free&sort=new&density=compact&theme=light&rm=1&compare=loader/glyph-braille,
           thinking-orb/dotted-breathe
```

| Rule | Decision |
|---|---|
| Keys | `q`, the nine facet keys (§2), `render`, `status`, `sort`, `seed`, `density`, `theme`, `rm`, `compare`, `page` |
| Separators | `,` OR, `+` AND (encoded `%2B` when written by code; a literal `+` is read as AND, not space) |
| Repeated keys | `state=thinking&state=error` accepted, normalised to `state=thinking,error` |
| Aliases | `family=B` → `loaders`; Studio short state names (`tool`, `approval`) → `tool-running`, `awaiting-approval` |
| Order | Normalised: `q`, facets in §2 order, then `sort`, `seed`, view keys, `compare`; values in display order |
| Defaults | Omitted (`sort=featured`, `density=cozy`, `theme=dark`, no `rm`) |
| Unknown / invalid | Dropped, with a one-line notice ("Ignored unknown state 'thinkin'") |
| History | Filter and sort changes use `replaceState` (typing doesn't flood history); opening a card or compare uses `pushState` |
| Length | Keep under 2 KB; `compare` max 4 ids |
| JS off | Facet values render as plain links to `?key=value` pages; server returns the first 24 static cards for the default view only, so JS-off filtering falls back to family index pages |
| Canonical | `rel=canonical` → `/components`; `noindex` when more than one facet is set ([ia.md](ia.md) §11) |

Family and category filters also have real paths (`/components/<family>`, `/kits/<category>`); a
query with only `family=X` shows a "Go to the Loaders page" link, and the gallery header links there.

## 5. Sort orders

| `sort=` | Order | Notes |
|---|---|---|
| `featured` (default) | Curated rank per component (`featured_rank`, proposed field), then taxonomy order, Free before Pro within a component | Stable; shows breadth first: one card per component before second variants ("diversify" pass) |
| `relevance` | Pagefind score | Auto-selected when `q` is set; reverts to the previous sort when `q` is cleared |
| `new` | `since` desc, then name | Pairs with a "New" badge for variants released in the last 30 days |
| `name` | Variant name A–Z | — |
| `installs` | 7-day `oxyn add` count from the registry Worker | Counts refreshed at build; hidden until counts are meaningful ([ia.md](ia.md) Q9). Idea from originkit "Most copied" (research 16/17) |
| `random` | Shuffle with `seed` (written to URL) | Same seed = same order for whoever opens the link. Idea from Uiverse's randomized default (research 16) |

## 6. Card anatomy and live policy

```text
┌──────────────────────────────────────┐
│                                [Pro] │   poster area, fixed 4:3 (reserves space, CLS 0)
│                                      │
│          ( ■ )  ring-progress        │   level 0 poster → level 1 live (free only)
│                                      │
│ ● thinking            ▶ Live · Dioxus│   pinned/current state · level label
├──────────────────────────────────────┤
│ ring-progress                   Free │   style name · tier badge (text, not colour only)
│ send-control · Composer              │   component · family
│ chat · ide   subtle   press loading  │   surfaces · motion · top gestures (max 3)
│ [⧉ oxyn add]  [Remix]  [☐ Compare]   │   actions (visible on hover/focus; always on touch)
└──────────────────────────────────────┘
```

| Part | Rule |
|---|---|
| Poster | AVIF still for the shown state and theme; `alt` = "<variant> in <state>: <summary>" |
| Live | Free variants only. Pro cards stay on poster + WebM loop on hover ([ia.md](ia.md) §8) |
| Script | A live card loops its `poster.script` (Studio Simulate scripts, [../studio.md](../studio.md)) unless a state is pinned (§3.4) |
| Title click | Opens the variant page (canonical); whole card is one link, actions are separate buttons |
| `⧉ oxyn add` | Copies `oxyn add <component>/<style>`; Pro copies the same command with a "needs licence key" toast |
| New / Beta | Small text badge next to the name |

**Going live (level 0 → 1):**

| Trigger | Pointer: fine (desktop) | Touch / no hover |
|---|---|---|
| Card hovered or keyboard-focused for 150 ms | live (pre-empts the oldest) | — |
| Card ≥ 60 % visible for 400 ms | live, if under the cap | live, if under the cap |
| Tap on the poster | — | live (first tap), second tap opens the page |

| Policy | Value (proposed) |
|---|---|
| Max live demos at once | **6** desktop, **3** touch / narrow, **1** with Save-Data or low power, **0** with `rm=1` (posters show the reduced form) |
| Over the cap | Evict the live card furthest from the viewport centre (LRU tie-break); it returns to its poster at the current state |
| Off-screen | Paused when < 10 % visible (IntersectionObserver), torn down after 30 s off-screen; motion.md §9 "scrolled offscreen: paused" |
| Hidden tab | All paused (`visibilitychange`), resume at the clock phase |
| Frame budget | All gallery demos together ≤ 4 ms main-thread per frame; above that for 1 s, drop the furthest live card |
| Runtime | One `gallery-dioxus` WASM instance for the page, demos are mounts inside it ([../website-stack.md](../website-stack.md)) |
| GPUI level 2 | Never in the gallery; only on detail pages |

## 7. Grid density

| `density` | Columns at ≥ 1280 / 1024 / 768 / < 768 px | Card | Live demos |
|---|---|---|---|
| `comfy` | 3 / 2 / 2 / 1 | 4:3, full meta + actions | yes |
| `cozy` (default) | 4 / 3 / 2 / 1 | 4:3, meta rows trimmed to 2 | yes |
| `compact` | 6 / 5 / 4 / 2 | 1:1 poster + name + tier only | **no** (posters only; hover plays WebM) |

Gutters from `semantic.space` tokens; cards use the site theme tokens so the frame stays matte and
colour lives inside the demo (Bencho's monochrome-chrome convention, research 14, study only).

## 8. Empty results

```text
┌ No variants match ─────────────────────────────────────────────────────────────┐
│ Nothing supports State: stalled + Surface: voice + Motion: expressive.         │
│ Try removing one:  (Motion: expressive → 6)  (Surface: voice → 11)  (State → 2)│
│ 4 Pro variants match these filters with Tier: Free removed.  [Show them]       │
│ Need it?  [Request a component ↗]  (GitHub issue template, pre-filled filters) │
└────────────────────────────────────────────────────────────────────────────────┘
```

| Rule | Decision |
|---|---|
| Relax suggestions | For each active facet, the count if that facet were removed; show the top 3 by count, skip 0 |
| Tier hint | If `tier=free` hides Pro matches (or the reverse), say how many |
| Text query | If `q` alone gives 0, show "Did you mean" from Pagefind's nearest term and the 6 most recent variants |
| Request | "Request a component" opens a GitHub issue template with the URL's filters pre-filled (idea from loading-ui, research 16) |
| Never | A blank grid, or silently dropping filters |

## 9. Keyboard and accessibility

| Key (focus in results) | Action |
|---|---|
| ← → ↑ ↓ | Move between cards (roving `tabindex`; one Tab stop for the whole grid) |
| Home / End | First / last loaded card |
| PageUp / PageDown | One screen of rows (loads more at the end) |
| Enter | Open variant page |
| Space | Play / pause this card's preview (goes live if allowed) |
| `c` | Add / remove from compare |
| `r` | Remix in Studio |
| `i` | Copy install command |
| `/` or ⌘K | Search (gallery `/` focuses the inline box; ⌘K opens the global modal) |
| `f` | Open / focus the filter panel |
| Esc | Close panel or sheet → clear search → leave grid focus, in that order |
| `?` | Shortcut sheet |

Single-key shortcuts are off while a text field is focused and can be turned off in the shortcut
sheet (WCAG 2.1.4).

| A11y rule | Decision |
|---|---|
| Card demos | `inert` and `aria-hidden`; the card's accessible name is title + tier + poster alt. Interactive demos live on detail pages ([ia.md](ia.md) §12) |
| Results count | Polite live region: "37 variants" after filters settle (debounced 500 ms) |
| Facet controls | Native checkboxes in `fieldset`/`legend`; counts in the label ("thinking, 41 results") |
| Focus | Visible `focus.ring` token; focus never moves on filter change |
| Motion | `rm=1` or OS reduced motion: no WebM autoplay, live demos in reduced form; flash limit per [../motion.md](../motion.md) §10 |
| Contrast | Badges and chips meet [../tokens.md](../tokens.md) §8 in all three themes |

## 10. Compare mode and Remix

### 10.1 Compare

Pick 2–4 variants (any component) with `c` or the card checkbox; the tray pins to the bottom; `Compare →`
opens an overlay at the same URL with `compare=` set (shareable, back closes it).

```text
┌ Compare (3) ──────────────────────────────────────────── State [Thinking ▾] ▶ ×┐
│ ring-progress (Free)    │ pill (Free)              │ liquid (Pro)              │
│ ┌ live ───────────────┐ │ ┌ live ───────────────┐  │ ┌ poster ─────────────┐   │
│ └─────────────────────┘ │ └─────────────────────┘  │ └─────────────────────┘   │
│ layout  inline, compact │ inline                   │ inline, floating          │
│ motion  subtle          │ none                     │ expressive          ≠     │
│ states  13              │ 13                       │ 11 (no stalled)     ≠     │
│ enter   dur.base 200 ms │ dur.instant 80 ms   ≠    │ spring.snap         ≠     │
│ tokens  6               │ 4                        │ 9                         │
│ [⧉ add] [Remix]         │ [⧉ add] [Remix]          │ [See Pro]                 │
└─────────────────────────┴──────────────────────────┴───────────────────────────┘
```

| Rule | Decision |
|---|---|
| Sync | One state picker + one Simulate script drive every column on the shared motion clock, so timing differences are visible side by side |
| Rows | Axes (layout, surface, motion), states supported, gestures, timing per transition (from the spec tables), tokens, render tier + fallback, tier, frameworks |
| Diff marks | `≠` on cells that differ from the first column; "Only differences" toggle |
| Pro columns | Poster per state, same as everywhere ([ia.md](ia.md) §8) |
| Narrow screens | Columns become a swipeable carousel with the rows pinned |
| Persistence | Tray survives navigation in `sessionStorage`; URL is the source of truth when present |

### 10.2 Remix in Studio

Every card, variant page and compare column has **Remix** (idea from transitions.dev "Remix → Studio",
research 17; custom licence, study only).

| Rule | Decision |
|---|---|
| Link | `/studio?from=<component>/<style>&state=<pinned state>&theme=<theme>&rm=<0|1>` |
| Seed | Studio loads the variant's registry defaults, the pinned state, and the gallery theme |
| Return | Studio top bar "◂ Gallery" returns to the exact gallery URL (`document.referrer` if same-origin, else `/components`) |
| New tab | ⌘/Ctrl-click opens a new tab like any link |
| Pro | Opens poster + locked inspector unless a licence key is stored ([ia.md](ia.md) §8, Q3) |
| Event | `remix` with `from=card|page|compare` ([ia.md](ia.md) §13) |

## 11. Pagefind facet mapping

Every **variant page** and **component page** carries filter attributes; the gallery page itself is
not indexed as a result (`data-pagefind-ignore` on the grid). Attribute syntax per
https://pagefind.app/docs/filtering/ (inline `key:value`; multiple values per page allowed). The keys
`any`, `all`, `none`, `not` are reserved by Pagefind, and none of ours collide.

| Facet | Attribute on page (example) | Pagefind query for `?key=a,b` | for `?key=a+b` |
|---|---|---|---|
| family | `data-pagefind-filter="family:composer"` | `family: { any: ["a","b"] }` | n/a (one value) |
| category | one element per value: `category:coding`, `category:crm` | `category: { any: [...] }` | `category: { all: [...] }` |
| state | one per supported state | `state: { any: [...] }` | `state: { all: [...] }` |
| surface / layout / gesture | one per value | `{ any: [...] }` | `{ all: [...] }` |
| motion | `motion:subtle` | `motion: { any: [...] }` | n/a |
| tier | `tier:free` | `tier: "free"` | n/a |
| framework | `framework:gpui`, `framework:dioxus` | `{ any: [...] }` | `{ all: [...] }` |
| type | `type:variant` / `component` / `kit` / `doc` / `changelog` | ⌘K group tabs | — |

Important: a plain array in Pagefind means **AND**, so within-facet OR must always be sent as `any`
(https://pagefind.app/docs/js-api-filtering/).

| Pagefind feature | Use |
|---|---|
| `data-pagefind-meta` | `variant`, `image` (poster URL), `image_alt`, `tier`, `family_label` → result rows and mapping hits back to gallery ids |
| `data-pagefind-sort` | `released` (from `since`), `name` |
| `data-pagefind-weight` | Title and variant name boosted over body text |
| Counts | `filters` / `totalFilters` feed the ⌘K facet chips |
| Gallery `q` | Pagefind text search → read each hit's `variant` meta → intersect with the client-side facet result |

Why two engines: `gallery.json` gives instant facet counts and card data without loading the search
index; Pagefind gives ranked text search and ⌘K. Both are generated from the same `meta.toml` in
`xtask build-registry` ([ia.md](ia.md) §14), so they cannot disagree on facet values.

## 12. `gallery.json`

```json
{
  "v": 1,
  "facets": { "family": ["composer", "loaders", "..."], "state": ["idle", "..."] },
  "items": [
    {
      "id": "send-control/ring-progress",
      "c": "send-control", "f": 0, "t": "free",
      "title": "Ring progress",
      "st": [0,1,2,3,4,5,6,7,8,9,10,11,12], "su": [0,1], "la": [0,1], "mo": 1,
      "ge": [1,5,7], "ca": [0,2], "fw": [0,1],
      "since": "0.1.0", "rank": 12, "poster": "p/9f2c…", "ps": 4
    }
  ]
}
```

Facet values are indices into `facets` to keep it small; `ps` is the default poster state. Generated by
`xtask build-registry` with a content hash in the file name; fetched once, cached immutable.

## 13. Performance budgets (proposals)

To be confirmed by the V2.6 tech spike ([../roadmap.md](../roadmap.md)). Compressed sizes are
Brotli unless noted.

| Item | Budget | Basis |
|---|---|---|
| `/components` HTML (first 24 cards) | ≤ 60 KB | static shell, Astro ships no JS by default |
| Site JS on first load (filters + `<oxyn-demo>` element) | ≤ 25 KB | vanilla TS custom element, no framework runtime |
| `gallery.json` | ≤ 40 KB | ~1,000 items × index-encoded fields |
| Poster per card (AVIF, 2× of card size) | ≤ 20 KB; first viewport ≤ 250 KB total | posters are the LCP candidates |
| Hover WebM loop | ≤ 200 KB, ≤ 4 s, loaded on hover only | — |
| `gallery-dioxus.wasm` (all free variants) | ≤ 500 KB warn, ≤ 800 KB fail | Dioxus web base is about 60 KB gz (research 18); splitting per family is Q2 |
| `gallery-gpui.wasm` (detail pages only) | ≤ 4 MB; CI fail > 20 MiB raw | a full GPUI app measured 3.9 MB Brotli; Cloudflare caps files at 25 MiB (research 18) |
| Pagefind | loaded on first search interaction only | — |
| LCP | ≤ 2.0 s p75, mid-range phone on 4G | Core Web Vitals "good" is ≤ 2.5 s; poster or H1 is the LCP element |
| CLS | ≤ 0.02 | fixed poster aspect, no layout shift on going live |
| INP | ≤ 150 ms | filter apply ≤ 50 ms for 1,000 items in JS |
| WASM mount | first demo live ≤ 1.5 s after it becomes visible (cached: ≤ 200 ms) | — |
| Live demo frame cost | ≤ 4 ms/frame total (§6) | motion.md §9 frame-rate caps |

CI checks sizes on every deploy job (no `upload-artifact`, per [../website-stack.md](../website-stack.md)).

## 14. Open questions

| # | Question | Default if undecided |
|---|---|---|
| Q1 | Posters per variant: all 13 states × 3 themes is ~39k images for ~1,000 variants. Render all states in dark + default state in light/hc-dark, and go live for the rest? | Yes; reduces CI time against 2,000 Actions min/month |
| Q2 | Can the Dioxus gallery WASM be split per family, or is one file per framework enough? | One file until it passes the warn budget |
| Q3 | Should `theme` ever be a filter (e.g. a variant that only makes sense on dark)? | No; tokens principle 6 |
| Q4 | Gesture vocabulary: taxonomy §C has 9 tags; Bencho and its Finds page use more (slide, swipe, magnetic, parallax…, research 14, study only). Add `slide`/`swipe` for touch surfaces? | Keep the 9 until a spec needs more |
| Q5 | Kit screens (`shell/<category>-<screen>`) in the main grid, or only under `/kits`? | In the grid with `family=shells`, canonical under `/kits` ([ia.md](ia.md) Q8) |
| Q6 | Show Pro cards by default, or default to `tier=free` for first-time visitors? | Show all; Pro cards are visibly badged |
| Q7 | Public install counts on cards (social proof) | Hidden in v1 |

## Sources

Opened for this file: [../component-taxonomy.md](../component-taxonomy.md) §A–C,
[../variants.md](../variants.md), [../architecture.md](../architecture.md), [../tokens.md](../tokens.md),
[../motion.md](../motion.md) §8–10, [../specs/loaders.md](../specs/loaders.md) §2.2,
[../website-stack.md](../website-stack.md), [../studio.md](../studio.md),
[../categories/README.md](../categories/README.md); project research notes `12-arc-site-design`,
`13-bench-metalforge`, `14-bencho`, `16-niche-sites-a`, `17-niche-sites-b`, `18-website-stack`;
https://pagefind.app/docs/filtering/ and https://pagefind.app/docs/js-api-filtering/.
**Study only:** Bencho, MetalForge, Arc UI, transitions.dev (custom licence) and originkit are cited for
patterns only; nothing is copied.

