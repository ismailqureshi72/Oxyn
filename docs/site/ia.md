# Site information architecture

**Status:** draft v0.1 (2026-11-12) · roadmap phase V2.5 (`site/ia.md`) · PR #10 `docs/site-studio-ia`.
Companion: [gallery.md](gallery.md) (gallery and filter model). Studio layout is in [../studio.md](../studio.md).

Stack is fixed by [../website-stack.md](../website-stack.md): Astro 7 + Starlight static shell,
`<oxyn-demo>` WASM demos, Dioxus Studio at `/studio`, Pagefind, `starlight-llms-txt`, free registry
as static JSON, Pro registry Worker (Bearer licence key), Polar, no accounts in v1, remote MCP +
`oxyn mcp`, Cloudflare. This file decides **what pages exist, what is on them, and how they connect**.

## 1. Goals

| # | Goal | Measured by (see §13) |
|---|---|---|
| G1 | A Rust dev sees a component **running through real agent states** within 5 s of landing | `demo_live` on first page view |
| G2 | Every component and variant installs in one copy: CLI, source, prompt or MCP | `copy_install`, registry Worker installs |
| G3 | Coding agents can find, read and install Oxyn without a browser | `/llms.txt`, `.md`, MCP tool calls |
| G4 | Pro is visible everywhere but never blocks reading the docs | `pro_cta_click` → Polar checkout |
| G5 | Every page is real HTML: fast, indexable, readable with JS off | LCP, Pagefind coverage, crawl |
| G6 | Every variant has exactly one canonical page that links Studio, spec and code | 0 duplicate canonicals in CI |

## 2. Audiences

| Audience | Arrives from | Wants | Primary path | Must never hit |
|---|---|---|---|---|
| **Rust devs** (GPUI desktop, Dioxus web) | GitHub, crates/Reddit, search | "Does it run in my stack, is it deep, is it MIT?" | Landing → Components → component page → install tab | A React snippet, a demo that needs WebGPU to see anything |
| **Agent-app builders** (vertical teams) | Search "CRM agent UI", X posts | A whole shell fast | Landing → Kits → kit page → Studio / Pricing | Pro wall before seeing the kit |
| **Coding agents / LLMs** | `llms.txt`, MCP, `.md` URLs, `oxyn mcp` | Names, props, install command, rules | `/llms.txt` → `/components/…​.md` → `/r/{name}.json` | Canvas-only content, JS-only text |
| **Pro buyers** | Pricing link, locked variants, kits | What exactly Pro adds, one price, a key | Locked variant → Pricing → Polar checkout → key → `oxyn add` | Accounts, fake or blurred code |
| Designers (secondary, [../vision.md](../vision.md)) | Gallery, Studio share links | Tune motion and tokens, hand over code | Gallery card → Remix in Studio → share link | Losing state on reload |

## 3. Site map

Canonical paths. `<family>` = family slug (§3.1), `<component>` = registry component name,
`<style>` = the style-family part of a variant name (`send-control/ring-progress` → `ring-progress`).

```text
/                                         Landing
/components                               Gallery: every variant, faceted (gallery.md)
/components/<family>                      Family index (13 pages, A–L + shells redirect)
/components/<family>/<component>          Component page (default variant + all variants)
/components/<family>/<component>/<style>  Variant page (canonical home of one variant)
/components/<family>/<component>.md       Markdown twin of a component page (agents)
/components/<family>/<component>/<style>.md  Markdown twin of a variant page
/c/<component>/<style>                    Short alias → 301 to the variant page (CLI/MCP links)
/kits                                     Kit index (category kits, family M)
/kits/<category>                          Kit page (shell + dashboard + screens)
/kits/<category>/<screen>                 Screen page = canonical home of shell/<category>-<screen>
/studio                                   Studio (Dioxus app); state in query (?s=…) 
/studio/s/<hash>                          Short share link (KV) when URL state > budget
/pricing                                  Free vs Pro, FAQ, Polar checkout links
/account                                  302 → Polar customer portal (keys, invoices, devices)
/docs                                     Docs home (Starlight)
/docs/getting-started/{install,gpui,dioxus,first-agent-screen}
/docs/concepts/{agent-state,variants,tokens,theming,motion,accessibility,adapters}
/docs/cli/{init,add,diff,update,mcp}
/docs/registry/{free,pro,schema}
/docs/ai/{llms-txt,mcp,copy-prompt,rules-for-agents}
/docs/studio/{tour,simulate-scripts,export}
/docs/pro/{licence-key,activation,teams}
/docs/contributing/{specs,variants,credits}
/changelog                                Release notes (CLI, registry, site); RSS at /changelog.xml
/blog                                     Optional, not in v1 (open question Q6)
/legal/{licence,pro-licence,terms,privacy,refunds,third-party-notices}
/r/index.json                             Free registry index (static)
/r/<component>.json                       Component item: variants list + shared files
/r/<component>/<style>.json               Variant item (shadcn-like shape)
/r/pro/<component>/<style>.json           Pro item, Worker, Authorization: Bearer <licence>
/mcp                                      Remote MCP (Worker, Streamable HTTP)
/llms.txt  /llms-full.txt  /llms-small.txt
/og/<family>/<component>[/<style>].png    OG images (built from posters)
/sitemap-index.xml  /robots.txt  /404
```

Static assets: posters and WebM loops on R2 behind an assets host (`/p/<hash>.{avif,png,webm}`);
WASM at `/wasm/gallery-dioxus.<hash>.wasm`, `/wasm/gallery-gpui.<hash>.wasm` (immutable cache).

### 3.1 Family slugs

Slugs reuse the spec overview file names in [../specs/README.md](../specs/README.md), so a family
page and its spec share a name. Family letters stay as aliases (`/components/B` → 301).

| Letter | Family ([../component-taxonomy.md](../component-taxonomy.md) §A) | Slug |
|---|---|---|
| A | Composer | `composer` |
| B | Loaders & thinking indicators | `loaders` |
| C | Status-text effects | `status-text` |
| D | Container & media loaders | `container-loaders` |
| E | Messages & reasoning | `messages` |
| F | Tool calls, plans & orchestration | `tools` |
| G | Approvals & safety | `approvals` |
| H | Checkpoints, diff & git | `checkpoints-git` |
| I | Generative UI | `genui` |
| J | Workspace chrome | `workspace-chrome` |
| K | Live & voice | `live-voice` |
| L | Icons & animated marks | `icons` |
| M | Shells, dashboards & screens | `shells` → 301 to `/kits` |

**Routing rule (decision):** the brief's `/components/<c>/<variant>` pattern is kept as the **tail**
of the variant URL, so the path ends in the exact variant name (`…/send-control/ring-progress`).
Putting the family first avoids the two-segment clash between `<family>/<component>` and
`<component>/<style>`. Build fails if a component slug equals a family slug.

### 3.2 Category kit slugs

From [../component-taxonomy.md](../component-taxonomy.md) §B and [../categories/README.md](../categories/README.md).

| Kit | Slug | v1 status |
|---|---|---|
| Coding agents | `coding` | draft spec |
| App builders | `app-builder` | draft spec |
| CRM agents | `crm` | draft spec |
| Finance agents | `finance` | draft spec |
| Video / media agents | `video` | draft spec |
| Support, Research, Data/BI, Browser/computer-use, Voice/live, Multi-agent ops | `support`, `research`, `data-bi`, `computer-use`, `voice`, `multi-agent-ops` | later; kit page shows "Planned" + tagged components |

## 4. Global navigation

```text
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ ◆ oxyn   Components  Kits  Studio  Docs  Pricing      [⌘K Search…]  ◐  ★ GitHub  Get Pro │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

| Item | Target | Notes |
|---|---|---|
| Logo | `/` | Animated mark idles; static under reduced motion |
| Components | `/components` | Mega-menu on hover/focus: 12 families with counts + "All variants" |
| Kits | `/kits` | Menu: 5 kits + "More coming" |
| Studio | `/studio` | Full-page app, own chrome (keeps this bar collapsed to a 32 px strip) |
| Docs | `/docs` | Starlight sidebar takes over below the bar |
| Pricing | `/pricing` | — |
| ⌘K | Pagefind modal (§6) | `⌘K` / `Ctrl K` / `/` |
| ◐ | Theme: dark (default) → light → hc-dark | Stored in localStorage; also sets demo theme |
| GitHub | repo | Star count fetched at build, not live |
| Get Pro | `/pricing#pro` | Text link, not a filled button (Arc pattern, study only) |

Mobile (< 768 px): logo · ⌘K icon · menu sheet (same items, families nested). No hover menus.

Docs pages use Starlight's left sidebar + right "On this page"; component pages use the same
three-zone frame (sidebar = family tree with variant counts and Free/Pro tier toggle).

## 5. Footer

```text
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ Product            Developers            AI & agents           Legal                     │
│ Components         Docs                  llms.txt              Licence (MIT)             │
│ Kits               CLI                   llms-full.txt         Pro licence               │
│ Studio             Registry (/r)         MCP server            Terms · Privacy           │
│ Pricing            GitHub                Copy-prompt guide     Refunds                   │
│ Changelog (RSS)    Contributing          Rules for agents      Third-party notices       │
│                                                                                          │
│ Manage licence ↗ (Polar)   ·   Theme ◐   ·   Reduce motion ☐   · © 2026 Oxyn · MIT core  │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

"Reduce motion" is a site switch that sets `data-oxyn-reduced` (same flag as
[../motion.md](../motion.md) §8 "Pause animations"); it overrides the OS setting only towards *less*.

## 6. ⌘K search (Pagefind)

One Pagefind index over all HTML pages. Text search plus facets; facet attributes are listed in
[gallery.md](gallery.md) §11. Pagefind filters default to AND, OR needs `any`
(https://pagefind.app/docs/js-api-filtering/).

```text
┌ Search Oxyn ───────────────────────────────────────────────────────────────┐
│ ⌕ thinking orb                                                      esc    │
│ [Type: All ▾] [Family ▾] [Tier: All ▾] [State ▾]                           │
├────────────────────────────────────────────────────────────────────────────┤
│ VARIANTS                                                                   │
│ ▣ loader/particle-orb          Loaders · thinking · streaming       Free   │
│ ▣ thinking-orb/nebula        Loaders · thinking                     Pro    │
│ COMPONENTS                                                                 │
│ ▣ thinking-orb               Loaders · 14 variants                         │
│ DOCS                                                                       │
│ ▤ Reduced motion             Concepts › Motion                             │
├────────────────────────────────────────────────────────────────────────────┤
│ ↵ open  ⌘↵ copy install  ⇧↵ open in Studio  ↑↓ move             Pagefind   │
└────────────────────────────────────────────────────────────────────────────┘
```

| Rule | Decision |
|---|---|
| Result row | Poster thumb (meta `image`) · name · family · top states · tier badge (affectUI row pattern, study only) |
| Groups | Variants, Components, Kits, Docs, Changelog — by `type` filter |
| Ranking | Pagefind default; `data-pagefind-weight` boosts title and variant name |
| Actions | `⌘↵` copies `oxyn add <variant>`; `⇧↵` opens Studio; Pro rows copy the Pro command |
| JS off | Search button links to `/components` (filters as plain links) and `/docs` |

## 7. Page templates

Width budget: 1280 px content grid; wireframes below are schematic, ≤ 100 columns.

### 7.1 Landing `/`

```text
┌ nav ─────────────────────────────────────────────────────────────────────────────────────┐
├──────────────────────────────────────────────────────────────────────────────────────────┤
│  The agent-UI layer for Rust apps.                     ┌ live hero stage ─────────────┐  │
│  Composer, thinking, tools, approvals, diffs —         │ composer + tool row + orb    │  │
│  in every state, for GPUI and Dioxus.                  │ running "tool + approval"    │  │
│  [Browse components]  [Open Studio]                    │ script on loop               │  │
│  $ cargo binstall oxyn && oxyn add send-control  ⧉     │ [Idle][Think][Tool][Approve] │  │
│  MIT core · 2 frameworks · N variants                  └──────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│  One state, every component:  AgentState timeline scrubber driving 6 small demos         │
│  idle ─ composing ─ submitted ─ thinking ─ streaming ─ tool ─ approval ─ done ─ error    │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│  Families (12 tiles, poster + count)                        [All components →]           │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│  Kits: Coding · App builder · CRM · Finance · Video  (wide posters, "Pro" where gated)   │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│  For coding agents:  llms.txt · MCP · Copy prompt     (code sample of an MCP call)       │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│  Studio teaser (WebM)  ·  Free vs Pro strip  ·  FAQ (5)  ·  footer                       │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

Hero stage is the only demo that goes live without interaction (level 1, after LCP). LCP element is
the H1 or the hero poster, never the WASM canvas.

### 7.2 Family index `/components/<family>`

```text
┌ nav ────────────────────────────────────────────────────────────────────────────────────┐
├ sidebar ──────────┬ main ───────────────────────────────────────────────────────────────┤
│ [All|Free|Pro]    │ Components › Loaders                                                │
│ ▾ Composer   61   │ Loaders & thinking indicators          180 variants · 9 comps       │
│ ▾ Loaders   180   │ One busy signal per region; T0–T4 render tiers.  Spec ↗             │
│   loader-glyph    │ ┌ filter bar (gallery.md, family pre-set) ───────────────────────┐  │
│   loader-vector   │ └────────────────────────────────────────────────────────────────┘  │
│   thinking-orb    │ ── loader-glyph (24) ─────────────────────────── View component →   │
│   loader-mascot   │ [card] [card] [card] [card]                                         │
│ ▸ Status text     │ ── thinking-orb (14) ─────────────────────────────────────────────  │
│ ▸ …               │ [card] [card] [card] [card]                                         │
└───────────────────┴─────────────────────────────────────────────────────────────────────┘
```

A family index is the gallery with `family` locked and cards grouped by component. Its canonical is
the path, not the filtered `/components?family=…` view.

### 7.3 Component page `/components/<family>/<component>`

```text
┌ nav ───────────────────────────────────────────────────────────────────────────────────┐
├ sidebar ──────┬ main ─────────────────────────────────────────────────────┬ on page ───┤
│ family tree   │ Components › Composer › Send control                      │ Demo       │
│               │ Send control  [Free · 12] [Pro · 20]   ⧉ Copy MD  ⧉ Prompt│ States     │
│               │ Send / stop / queue / steer in one control.   Spec ↗      │ Variants   │
│               │ ┌ hero demo   [Fit|Mobile|⤢] ────────────────────────────┐│ Install    │
│               │ │                                                        ││ Props      │
│               │ │            ( ■ )  Queue   ▸ live Dioxus demo           ││ Timing     │
│               │ │                                                        ││ A11y       │
│               │ │ Live · Dioxus  [GPUI-accurate · WebGPU]  ◐  ⏸          ││ Credits    │
│               │ └────────────────────────────────────────────────────────┘│            │
│               │ State (●Idle)(Composing)(Submitted)(Thinking)(Streaming)  │            │
│               │       (Tool)(Approval)(Queued)(Steering)(Done)(Error)…    │            │
│               │ Script ▶ [tool + approval ▾] ━━━━●━━━━━  00:04 / 00:11    │            │
│               │ Component states: Empty·Ready·Sending·Working·Queued(n)   │            │
│               │ Variants ◂ [pill][ring-progress][split][liquid·Pro] ▸ All→│            │
│               │ [Open in Studio]  [Compare variants]                      │            │
│               │ ── Install ── [CLI] [Copy source] [Prompt] [MCP]          │            │
│               │ $ oxyn add send-control/ring-progress            ⧉        │            │
│               │ ── Props ── name · type · default · description           │            │
│               │ ── States ── AgentState → shows · actions · keys          │            │
│               │ ── Timing ── transition · token · ms · easing · reduced   │            │
│               │ ── Accessibility ── labels, live region, focus            │            │
│               │ ── Usage (GPUI / Dioxus tabs) ──  ── Related ──           │            │
│               │ ── Licence & credits ── MIT · Codex CLI (MIT) ↗           │            │
│               │ ◂ prev component                    next component ▸      │            │
└───────────────┴───────────────────────────────────────────────────────────┴────────────┘
```

| Block | Source | Behaviour |
|---|---|---|
| Free/Pro badge | count of variants per tier in registry | Pro count links to the variant strip filtered to Pro |
| Hero demo | default variant (`meta.toml` `default = true`) | Level 0 → 1 per §9; `⤢` fullscreen dialog; Fit / Mobile frame |
| State switcher | `AgentState` (13 values, [../architecture.md](../architecture.md)); only states in `states` are enabled | Click sets state on the demo; JS off → per-state poster strip; URL `?state=thinking` |
| Script scrubber | Studio Simulate scripts ([../studio.md](../studio.md)): short answer, long stream, tool + approval, error + retry, queue + steer | Plays a fake event stream through `oxyn-core`; the state chips follow it |
| Component states row | spec state machine (e.g. send-control `Empty…Dictating`) | Read-only chips mapped from AgentState; hover shows which AgentState drives it |
| Variant strip | all variants of the component, Free first then Pro | Poster thumbs; selecting swaps the hero demo in place (URL `?v=`); "All →" opens variant pages |
| Install tabs | §10 | Tab choice remembered (localStorage) |
| Props table | `oxyn-registry-schema` reads the Rust props struct doc comments | Name, type, default, description; `AgentState`-driven fields marked |
| Timing table | spec timing table, resolved to motion tokens ([../motion.md](../motion.md)) | Columns: transition, token, value, easing/spring, reduced form |
| A11y notes | variant §3 of [../variants.md](../variants.md) | Labels per state, live-region text, keyboard map, contrast result |
| Licence & credits | `[[credits]]` in `meta.toml` | Licence badge; port credits link to `/legal/third-party-notices` |

### 7.4 Variant page `/components/<family>/<component>/<style>`

Same frame as 7.3, focused on one variant. It is the **canonical** URL for that variant.

```text
┌ main (sidebar and on-page column as 7.3) ──────────────────────────────────────────────┐
│ Components › Composer › Send control › ring-progress    ⧉ Copy MD  ⧉ Prompt            │
│ send-control/ring-progress  [Free]                                                     │
│ layout: inline · surface: chat, ide · motion: subtle · gestures: press, loading        │
│ ┌ demo (this variant only) ────────────────────────────────┐  [Open in Studio]         │
│ └──────────────────────────────────────────────────────────┘  [Compare +]              │
│ State chips · script scrubber                                                          │
│ Install  [CLI] [Copy source] [Prompt] [MCP]                                            │
│ What differs from the default variant (2–4 lines) · Supported states table             │
│ Timing (this variant) · Reduced-motion fallback (poster of reduced form)               │
│ Tokens it reads · Category tags (→ kits) · Gesture tags (→ gallery)                    │
│ Render tier T2 → fallback: send-control/pill                                           │
│ Licence & credits · Siblings strip (other variants) · ◂ prev / next ▸                  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

Every field in [../variants.md](../variants.md) "Every variant documents" (1–8) has a block here.

### 7.5 Pro (locked) variant: same template, different surfaces

```text
┌ main ───────────────────────────────────────────────────────┬ Pro card ────────────────┐
│ send-control/liquid  [Pro]                                  │                          │
│ ┌ poster (WebM of the GPUI render) ────────────────────────┐│ Pro                      │
│ │                                                          ││ Source + CLI install     │
│ │               ◉  liquid morph                            ││ for this and every       │
│ │                                                          ││ Pro variant.             │
│ │ GPUI render · recorded loop                [Pro]         ││                          │
│ └──────────────────────────────────────────────────────────┘│ [See Pro →]              │
│ State (Idle)(Thinking)(Tool)… → swaps per-state posters     │ Have a key? →            │
│ Install  [CLI] [Prompt] [MCP]     (no Copy source tab)      │                          │
│ $ OXYN_LICENSE_KEY=… oxyn add send-control/liquid  ⧉        │                          │
│ Props (public API) · States · Timing · A11y · Credits       │                          │
└─────────────────────────────────────────────────────────────┴──────────────────────────┘
```

### 7.6 Kit page `/kits/<category>`

```text
┌ main ──────────────────────────────────────────────────────────────────────────────────┐
│ Kits › CRM agents                                  [Pro kit]  [Open in Studio]         │
│ Tallyfern (fictional seller, crm.md) — inbox + record view, pipeline board             │
│ ┌ shell demo:  Desktop (GPUI poster) | Web (Dioxus live) | Narrow ────────────────────┐│
│ │ rails · record view · composer · approval card                                      ││
│ │                                                                                     ││
│ │                                                                                     ││
│ └─────────────────────────────────────────────────────────────────────────────────────┘│
│ Agent flows ▶ [Enrich lead] [Draft outreach → approval] [Bulk update]                  │
│ Screens  [inbox] [record] [pipeline] [dashboard]  → /kits/crm/<screen>                 │
│ Components used (grouped by family, each links to its page, tier badge)                │
│ New components this kit adds · Approval risk table · Fixtures (all invented)           │
│ Install: $ oxyn add shell/crm-inbox  (Pro) · Variants: layout × surface × motion       │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

Sections mirror the 10 points of [../categories/README.md](../categories/README.md).

### 7.7 Pricing `/pricing`

Prices are not decided ([../vision.md](../vision.md)); wireframe shows structure only.

```text
┌ main ──────────────────────────────────────────────────────────────────────────────────┐
│             Free forever for the core. Pro for depth.                                  │
│ ┌ Free (MIT) ──────────────┐ ┌ Pro ─────────────────────┐ ┌ Team ────────────────────┐ │
│ │ $0                       │ │ $— once / $— year (TBD)  │ │ $— up to N devs (TBD)    │ │
│ │ core crate, tokens       │ │ every Pro variant        │ │ Pro × N seats            │ │
│ │ base variants (≥3 each)  │ │ category kits            │ │ shared activations       │ │
│ │ CLI, Studio, MCP, llms   │ │ motion packs, Studio+    │ │ invoice                  │ │
│ │ [Get started]            │ │ [Get Pro → Polar]        │ │ [Buy for team]           │ │
│ └──────────────────────────┘ └──────────────────────────┘ └──────────────────────────┘ │
│ Compare table (feature × tier) · What a licence key does (CLI, MCP, Studio)            │
│ FAQ: What do I get? · Once or yearly? · Ship in client apps? · Refunds?                │
│      How do agents use Pro (MCP + key)? · Need an account? (no: key + Polar)           │
│ Merchant of record: Polar · VAT handled · Manage licence ↗                             │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### 7.8 Docs page (Starlight)

```text
├ sidebar ─────────────┬ article ─────────────────────────────────────────┬ on this page ┤
│ Getting started      │ Concepts › Agent state        ⧉ Copy MD  ⧉ Prompt│ States       │
│ Concepts             │ # Agent state                                    │ Rules        │
│   · Agent state      │ prose · tables · inline <oxyn-demo> (level 0/1)  │ Adapters     │
│   · Variants         │ code blocks (Rust, copy)                         │              │
│ CLI · Registry · AI  │ ◂ prev                                 next ▸    │              │
│ Studio · Pro         │ Edit on GitHub · Last updated                    │              │
└──────────────────────┴──────────────────────────────────────────────────┴──────────────┘
```

## 8. Pro gating UX

| Rule | Decision | Why |
|---|---|---|
| Preview | Locked variants show the **poster** (static PNG + WebM loop per state) and a "Pro" badge. No live WASM demo. | Pro code compiled into public WASM would ship the implementation. Peers show live Pro previews (affectUI, Arc — study only, research 11/12); we trade that for not leaking source. |
| Code | **Never fake, blurred or placeholder code.** The Copy source tab is absent; the public props API, states, timing and a11y are shown in full. | Honest docs; agents must not ingest fake code. |
| Install | CLI tab shows the real command with `OXYN_LICENSE_KEY`; without a key the CLI prints a pricing link. | One command for both tiers. |
| State switcher | Works on Pro pages (swaps per-state posters). | Buyers still see behaviour per state. |
| Gallery | Pro cards are poster-only, badge top-right, never auto-live. | Same as above; also caps WASM work. |
| Nudge | One "Pro" side card per Pro page; a "More in Pro" strip (≤ 3) at the end of Free pages. No modal, no dock that follows scroll. | Calm by default ([../vision.md](../vision.md) principle 4). |
| Key holders | Pasting a key in Studio or `/docs/pro/licence-key` stores it in localStorage; pages then show "Pro unlocked" and enable Copy source via the Worker. | No accounts in v1. |
| Studio | Pro variants open in Studio as poster + locked inspector until a key is present (Q3). | — |

## 9. Demo loading levels and fallbacks

Levels from [../website-stack.md](../website-stack.md). `<oxyn-demo id props level>` decides per mount.

| Level | What | Loads | When |
|---|---|---|---|
| 0 | Poster: AVIF/PNG still per state + optional WebM loop, rendered headlessly from GPUI | `<img>`/`<video>` | Always first; the only level for Pro, JS off, crawlers |
| 1 | Live Dioxus web demo | one cached `gallery-dioxus.wasm` per site, `mount(id, props)` | Visible ≥ 50 % + idle, or hover/focus/tap (gallery.md §6) |
| 2 | GPUI-on-web, desktop-accurate | `gallery-gpui.wasm` into **one shared stage** per page | Only on click "GPUI-accurate"; needs WebGPU |

| Condition | Detected by | Result |
|---|---|---|
| JS off | no custom element upgrade | Level 0 still; state switcher becomes a static per-state poster strip (`loading=lazy`) |
| Reduced motion (OS or site switch) | `prefers-reduced-motion`, `data-oxyn-reduced` | Still posters (no WebM autoplay); level 1 mounts with `motion.reduced = true` ([../motion.md](../motion.md) §8) |
| No WebGPU | `!navigator.gpu` or adapter request fails | Level 2 button disabled: "Needs WebGPU — showing GPUI recording"; plays the WebM |
| Low power / Save-Data | `navigator.connection.saveData`, `deviceMemory ≤ 2`, site sets `data-oxyn-low-power` | Level 0 only until click; once live, low-power fps column of motion.md §9 |
| Hidden tab / off-screen | `visibilitychange`, IntersectionObserver | Pause (motion.md §9: offscreen loops paused); resume at clock phase |
| WASM fails or > 4 s | load error / timeout | Keep poster; inline "Demo didn't load · Retry"; event `demo_fallback` |
| iOS WebKit streaming-compile bug | UA check | Non-streaming instantiate (research 18) |
| GPU context lost | `devicelost` | Drop to level 1 (or 0) for that stage |

Labels are honest: level 0 says "GPUI render", level 1 "Live · Dioxus", level 2 "Live · GPUI (WebGPU)".
GPUI posters are the reference look; Dioxus may differ slightly ([../website-stack.md](../website-stack.md) Risks).

## 10. Install tabs and agent-facing surfaces

| Tab | Content (free) | Content (Pro, no key) |
|---|---|---|
| CLI | `oxyn add <component>/<style>` (+ `--framework gpui|dioxus`, `--preset <hash>` from Studio) | same command + "needs `OXYN_LICENSE_KEY`" |
| Copy source | GPUI and Dioxus source tabs from `/r/<component>/<style>.json`, crate deps listed | absent |
| Prompt | Agent prompt: what it is, install command, props, states it needs, tokens, rules | same, without source |
| MCP | Snippet to add the remote MCP / `oxyn mcp` + example call | same; note key header |

Per-page actions: **Copy Markdown** (fetches the `.md` twin) and **Copy prompt** (short, task-shaped).

| Surface | Path | Built by |
|---|---|---|
| `llms.txt` / `llms-full.txt` / `llms-small.txt` | `/llms*.txt` | `starlight-llms-txt`, plus a generated components section from `index.json` |
| Markdown twins | `<page>.md` | Astro endpoint from the same MDX + `meta.toml` |
| Registry | `/r/index.json`, `/r/<component>.json`, `/r/<component>/<style>.json` | `xtask build-registry` |
| Pro registry | `/r/pro/<component>/<style>.json` | `workers/registry-pro`, Bearer key → Polar validate |
| MCP | `/mcp` (remote), `oxyn mcp` (stdio) | `workers/mcp`, reads `index.json` |

MCP tool sketch (first six from research 18; the rest proposed):

| Tool | Input | Returns |
|---|---|---|
| `search_components` | `query`, facet filters (gallery.md §2) | variants with name, family, tier, states, url |
| `get_component` | `name` | component meta, variants, props, states, `.md` url |
| `get_variant` *(proposed)* | `component/style` | variant meta, timing, a11y, tokens, credits |
| `get_install_command` | `name`, `framework`, `preset?` | CLI line + crate deps |
| `list_themes` / `get_theme` | `theme?` | dark / light / hc-dark token sets |
| `get_pro_component` | `name` + key header | Pro source (Worker validates) |
| `list_states` *(proposed)* | — | `AgentState` enum + which variants support each |
| `compose_screen` *(proposed)* | `category`, `screen` | component list + layout for a kit screen (idea from Watermelon's `compose_page`, research 16) |
| `studio_link` *(proposed)* | `name`, props, state | `/studio?...` URL for a human to tune |

## 11. SEO

| Rule | Decision |
|---|---|
| Canonical | One per variant: `/components/<family>/<component>/<style>`. Component pages canonical to themselves. `/c/…` aliases 301. Gallery query views `rel=canonical` → `/components`, `noindex` when > 1 facet set. |
| Indexable text | Every page is static HTML: description, props, states, timing, poster `<img alt>` (research 18 risk 8: crawlers can't see canvas) |
| OG images | `/og/…png` 1200×630 built in CI from the default-state GPUI poster + name + family + tier badge |
| Titles | `<Variant name> — <Component> · Oxyn` / `<Component> for GPUI & Dioxus · Oxyn` |
| Structured data | `SoftwareSourceCode` (free) / `Product` + `Offer` (pricing page only) |
| Sitemap | All canonical pages + `.md` twins excluded (`X-Robots-Tag: noindex`) to avoid duplicates |
| Compare pages | Later: honest "Oxyn vs …" pages for Rust UI options (idea from Watermelon, research 16) |

## 12. i18n and accessibility

**i18n stance:** English only in v1, root locale (no `/en/` prefix). Starlight i18n stays configured so
locales can be added later under `/<lang>/`. UI strings live in one file. Component demos get a
`dir=rtl` toggle in the demo toolbar because specs require RTL ([../specs/README.md](../specs/README.md) §7).
Verb packs and copy are in-house ([../licensing.md](../licensing.md)) and English for now.

| A11y rule | Decision |
|---|---|
| Target | WCAG 2.2 AA for the site; hc-dark theme meets the stricter contrast in [../tokens.md](../tokens.md) |
| Motion | Respect OS reduced motion + site switch; flash limit of [../motion.md](../motion.md) §10 holds for every poster and demo |
| Demo focus | Detail-page demos are focusable and keyboard-operable; gallery card demos are `inert` (gallery.md §9) |
| Announcements | Detail demo live regions on by default; only one demo announces at a time |
| State switcher | `role=radiogroup`, arrows move, labels = state names + disabled reason |
| Posters | `alt` from meta: "<variant> in <state>: <one line>"; WebM has no audio, `aria-hidden` with the alt on its wrapper |
| Skip links | "Skip to demo", "Skip to install" on component pages |
| Colour | Status never colour alone (tokens principle 3); Free/Pro badges carry text |

## 13. Analytics (cookieless)

Cloudflare Web Analytics for page views; custom events go to a tiny beacon endpoint on the existing
Worker (no cookies, no IDs, no IP stored; aggregated counts only). Sales from Polar, installs from the
registry Worker ([../website-stack.md](../website-stack.md)). Sample events to stay inside the free
100k requests/day.

| Event | Props |
|---|---|
| `demo_live` | variant, level (1/2), trigger (visible/hover/click) |
| `demo_fallback` | variant, reason (no-webgpu, wasm-error, timeout, reduced, save-data) |
| `state_switch` | variant, state, source (chip/script) |
| `copy_install` / `copy_source` / `copy_prompt` / `copy_markdown` | variant, framework, tier |
| `open_studio` / `remix` | variant, from (page/card/compare) |
| `filter_change` / `search` | facet keys only (no free text stored), result count bucket |
| `compare_open` | count |
| `pro_cta_click` / `checkout_start` | placement (side card, strip, pricing) |
| `mcp_tool` | tool name (server-side) |

## 14. Content model

Source of truth: `registry/{free,pro}/<component>/<style>/meta.toml` (variant) and
`registry/{free,pro}/<component>/component.toml` (shared), parsed by `oxyn-registry-schema`, emitted by
`xtask build-registry` into `index.json`, `r/*.json`, `gallery.json`, MDX stubs and Pagefind
attributes. Exact file split is proposed (Q1).

```toml
# registry/free/send-control/ring-progress/meta.toml
name        = "send-control/ring-progress"     # stable once released
component   = "send-control"
family      = "composer"                      # slug, §3.1
title       = "Ring progress"
summary     = "Stop square with a progress ring for the running turn."
tier        = "free"                          # free | pro
status      = "stable"                        # draft | beta | stable | deprecated
since       = "0.1.0"
default     = false
layout      = ["inline", "compact"]
surface     = ["chat", "ide"]
motion      = "subtle"                        # none | subtle | expressive
gestures    = ["press", "loading", "morph"]
categories  = ["coding", "crm"]
states      = ["idle","composing","submitted","thinking","streaming","tool-running",
               "awaiting-approval","queued","steering","done","error","cancelled","stalled"]
frameworks  = ["gpui", "dioxus"]
render_tier = "T2"                            # T0..T4 (loaders.md §2.2)
fallback    = "send-control/pill"            # proposed field (loaders.md)
tokens      = ["color.accent.solid", "color.state.streaming", "dur.base"]
spec        = "specs/composer-send-control.md#variants"

[poster]
state  = "streaming"                          # default poster/OG state
aspect = "4:3"
script = "tool-approval"                      # Simulate script for the WebM loop

[a11y]
labels = { idle = "Send", streaming = "Stop generating", queued = "Queue message" }

[[credits]]
source  = "Codex CLI"
licence = "MIT"
kind    = "port"                              # port | study
```

```rust
// oxyn-registry-schema (sketch)
pub struct VariantMeta {
    pub name: VariantName,            // "<component>/<style>"
    pub component: String,
    pub family: Family,               // Composer | Loaders | … | Shells
    pub title: String,
    pub summary: String,
    pub tier: Tier,                   // Free | Pro
    pub status: Status,
    pub since: semver::Version,
    pub default: bool,
    pub layout: Vec<Layout>,          // Compact | Inline | Card | Panel | Floating
    pub surface: Vec<Surface>,        // Chat | Ide | Desktop | Mobile | Voice | Terminal
    pub motion: Motion,               // None | Subtle | Expressive
    pub gestures: Vec<Gesture>,
    pub categories: Vec<Category>,
    pub states: Vec<AgentStateKind>,  // payload-free mirror of oxyn_core::AgentState
    pub frameworks: Vec<Framework>,   // Gpui | Dioxus
    pub render_tier: Option<RenderTier>,
    pub fallback: Option<VariantName>,
    pub tokens: Vec<TokenPath>,
    pub spec: String,
    pub poster: PosterSpec,
    pub a11y: A11yMeta,
    pub credits: Vec<Credit>,
}
```

| Field | Page block | Gallery | Pagefind | Registry JSON | MCP |
|---|---|---|---|---|---|
| `name`, `title`, `summary` | H1, subtitle | card title/line | title, weight | `name`, `description` | ✓ |
| `family`, `component` | breadcrumb, URL | facet | `family` filter | `meta.family` | ✓ |
| `tier` | badge, install tabs, gating | facet, badge | `tier` | path (`/r` vs `/r/pro`) | ✓ |
| `layout`, `surface`, `motion`, `gestures`, `categories` | meta row, tag links | facets | filters | `meta.*` | filters |
| `states` | state switcher enablement | facet | `state` | `meta.states` | `list_states` |
| `frameworks` | install `--framework`, source tabs | facet | `framework` | `files[]` | ✓ |
| `render_tier`, `fallback` | "Render tier" line | (advanced) | — | `meta` | ✓ |
| `tokens` | Tokens block | — | — | `meta.tokens` | ✓ |
| `poster` | hero level 0, OG | card poster | meta `image` | — | thumbnail url |
| `a11y` | A11y block, poster alt | card alt | — | — | ✓ |
| `credits` | Licence & credits | — | — | `meta.credits` | ✓ |
| `since`, `status` | badge "New"/"Beta", changelog link | sort "New" | sort `released` | ✓ | ✓ |

Props tables and timing tables are not in `meta.toml`: props come from the Rust props struct doc
comments; timing comes from the spec table resolved against `tokens/oxyn.motion.json`.

## 15. Open questions

| # | Question | Default if undecided |
|---|---|---|
| Q1 | One `meta.toml` per variant, or one per component with `[[variants]]`? | Per variant (stable URL = folder) |
| Q2 | Registry names with `/` (`/r/send-control/ring-progress.json`) vs flat (`send-control--ring-progress`) for shadcn compatibility | Nested path; flat alias later if needed |
| Q3 | Can Studio run Pro variants live for key holders (fetch a Pro WASM chunk via the Worker)? | Poster + locked inspector in v1 |
| Q4 | Studio's state list has 12 states and no `cancelled`; architecture has 13. Site uses the 13 from [../architecture.md](../architecture.md) / tokens. Align Studio? | Use 13 |
| Q5 | Domain and whether `/mcp` and `/r/pro` live on the main host or a subdomain | Same host, Worker routes |
| Q6 | Ship `/blog` in v1, or changelog only? | Changelog only |
| Q7 | Prices and Team seat count | Pricing page ships with structure, prices TBD |
| Q8 | Kit screens: are `shell/<category>-<screen>` variants also listed in `/components` gallery, or only under `/kits`? | Both, canonical under `/kits` |
| Q9 | Do we show install counts publicly (social proof) or keep them internal? | Internal until numbers are meaningful |

## Sources

Opened for this file: [../website-stack.md](../website-stack.md), [../studio.md](../studio.md),
[../vision.md](../vision.md), [../component-taxonomy.md](../component-taxonomy.md),
[../variants.md](../variants.md), [../architecture.md](../architecture.md),
[../specs/README.md](../specs/README.md), [../specs/loaders.md](../specs/loaders.md) §2.2,
[../specs/composer-send-control.md](../specs/composer-send-control.md) §2,
[../categories/README.md](../categories/README.md), [../tokens.md](../tokens.md),
[../motion.md](../motion.md) §8–10, [../licensing.md](../licensing.md); project research notes
`11-affectui-catalog`, `12-arc-site-design`, `13-bench-metalforge`, `14-bencho`, `16-niche-sites-a`,
`17-niche-sites-b`, `18-website-stack`; https://pagefind.app/docs/js-api-filtering/ and
https://pagefind.app/docs/filtering/. **Study only** (no code, assets or copy): affectUI, Arc UI,
Bencho, MetalForge, transitions.dev — patterns cited as ideas, re-implemented from our own specs.
