**Status:** draft v0.1 (2026-11-11).

# Spec: composer tools menu (the "+" button and its menu)

Follows the 10-point standard in [README.md](README.md) at the depth of
[composer-send-control.md](composer-send-control.md). Target **8 free + 14 Pro variants** on one state
machine for the trigger button and one for each menu item.

## 1. Purpose
The "+" is where the user adds things the text cannot carry: a file from disk, an image, a repo or URL
as context, a connector (MCP server), a skill, or a mode such as Plan or Research. Research across coding
agents found **two different jobs hiding behind one "+"**: attaching a file from disk, and adding
context or capabilities (skills, connectors, plugins) (research notes 03: Claude Code desktop, Lovable,
Zed, Verdent). Consumer chat apps put modes there too (research notes 04: ChatGPT, Claude, Gemini). The
menu therefore has sections, search, nested submenus, and per-item states for tools that are on,
need sign-in, are broken, or are not allowed right now.

## 2. State machine

**Trigger button**
```
  Rest ──hover──▶ Hover ──press──▶ Pressed ──release──▶ Opening ──settle──▶ Open(root)
   ▲                                                       │                  │  type
   │                                                       │                  ▼
   │◀── Closing ◀── Esc / outside click / pick ────────────┴──────────── Searching(query)
   │                     ▲                                                    │  →/Enter on group
   │                     └──────────── ← / Esc (at root) ◀── Submenu(path) ◀──┘
  Rest(with active: n)   = closed, but n modes/tools are on → chips under the composer + dot on "+"
  Disabled(reason)       = whole menu unavailable (offline, permissions, surface does not allow it)
```

**Menu item** (each row has its own state; connectors also have a connection machine)
```
  Available ──toggle──▶ On ──toggle──▶ Available
      │                                       Connector:  Off ─▶ NeedsAuth ─▶ Authorizing ─▶ Discovering ─▶ Connected(n tools)
      ├── Conflict(with mode X)                                    ▲              │ fail           │ fail
      ├── Disabled(reason)                                         └── Error(retry) ◀──────────────┘
      ├── Hidden(surface)   (e.g. only "Add image" on a restricted environment)
      └── Loading           (async section: skills / connectors still listing)
```

## 3. States table

**Trigger**

| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Rest` | 32 px round "+" (28 px dense, 44 px touch target) | open menu | drop files on it (forwards to attachments) | Tab focus; Enter/Space opens |
| `Hover` | 6 % fill, cursor pointer, tooltip "Add files, tools and more" after 500 ms | open | — | — |
| `Pressed` | scale 0.94 | — | — | — |
| `Opening` / `Open` | "+" rotated 45° into "×"; menu above (or below if no room) | pick item | search | ↑/↓ move, Enter pick, Esc close |
| `Searching(q)` | search field (auto-shown when > 8 items, else type-to-search), flattened results with breadcrumbs "Connectors › GitHub" | pick | clear query | Esc clears query, second Esc closes |
| `Submenu(path)` | child panel slides out beside the parent (desktop) or drills in with a back row (mobile/sheet) | pick | back | → / Enter opens, ← / Esc goes back |
| `Rest(active n)` | "+" plus a 6 px accent dot; active modes/tools as removable chips under the composer | open | remove a chip (×) | Backspace on a focused chip removes it |
| `Disabled(reason)` | "+" muted; tooltip and `aria-describedby` reason | — | — | focusable, reason announced |
| busy agent | menu opens; items that cannot change mid-run are `Disabled("Can't change mode while the agent is working")` (proposed) | allowed items only | — | — |

**Items**

| Item state | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Available` | icon 16 px · label · optional description · trailing shortcut or chevron | run / toggle / open submenu | — | Enter / Space |
| `On` (toggle) | check or switch on; mode tint on the icon | turn off | — | Space toggles without closing |
| `Loading` | 3 skeleton rows under the section header (after 150 ms delay) | — | — | skipped by arrows |
| `NeedsAuth` | amber dot + word "Needs sign in" + "Sign in" pill (never colour alone) | sign in (opens auth flow) | hide | Enter = Sign in |
| `Authorizing` | shimmer label "Waiting for sign-in…" + Cancel | cancel | — | Esc cancels flow |
| `Discovering` | shimmer "Discovering tools" → settles to "Found 5 tools" (pattern from Arc MCP servers, research notes 12) | — | — | — |
| `Connected(n)` | green dot + "Connected · 5 tools" (+ optional "≈2.2k tokens") | open tool list (submenu) | disconnect | → opens |
| `Error` | danger icon + "Error" + "Retry" pill; reason on hover / second line | retry in place | details | Enter = Retry |
| `Conflict(X)` | muted + note "Turns off Research" (mode-exclusivity notice pattern from Le Chat, research notes 04) | turn on anyway (turns X off) | — | Enter |
| `Disabled(reason)` | 50 % opacity, reason as second line ("Only images are supported in this environment") | — | — | focusable but not activatable |
| `Hidden(surface)` | not rendered | — | — | — |

**Sections** (default order; apps can reorder or remove; copy is ours):
1. **Attach** — Upload file, Add image / photo, Screenshot or area, Paste from clipboard (only when the clipboard has an image).
2. **Context** — Add folder / repo, Add URL, Recent files, Previous thread.
3. **Connectors** — one row per MCP server / app with status; "Manage connectors…" at the end.
4. **Skills** — search-first list; "Browse skills…".
5. **Modes** — Plan, Research, Think, etc. as toggles; mutually exclusive groups render as radio rows.

## 4. Micro-interactions
- **+ → × morph.** The glyph rotates 45° into a cross as the menu opens (≈200 ms in [R013]); reversed on
  close. The button keeps its size; only the glyph turns.
- **Open.** The menu scales from 0.96 to 1 with origin at the "+" (bottom-left when the menu is above),
  rises 4 px and fades in. Rows do not stagger by default (a list that appears whole reads faster);
  `expressive` motion adds a 20 ms stagger capped at 6 rows.
- **Close.** Faster than open, toward the origin, no overshoot. Picking an item closes after the row's
  press feedback (80 ms), so the click is felt.
- **Highlight.** One highlight glides between rows on a snappy spring for both pointer and arrows (as in
  Arc's command palette and morph select, research notes 12); it snaps instead of gliding on jumps over
  more than 6 rows.
- **Search.** Typing filters live; matched letters render in medium weight (shared matcher with
  composer-slash-mentions.md). The panel height springs to the result count.
- **Submenu.** Opens after 120 ms hover intent; the child slides 8 px from the parent edge and fades. A
  safe-triangle (pointer heading toward the child) keeps it open for up to 300 ms.
- **Toggle on.** Switch thumb springs; a chip for the active mode grows in under the composer (scale 0.85
  → 1) at the same moment, so the user sees where the state now lives.
- **Status changes.** Connector status label and pill cross-fade (≈100 ms out, 160 ms in, the Arc MCP
  timing); discovered tools appear one by one with a 4 px fade-in.
- **Error.** The row's icon does one 4 px shake; the Retry pill fades in. No toast.
- **Drop on "+".** Files dragged onto the closed "+" make it swell to 1.12 and show a paperclip; drop
  forwards to attachments without opening the menu.

## 5. Timing (defaults)

| Transition | Duration | Delay | Easing / spring | Notes |
|---|---|---|---|---|
| Hover fill | 100 ms | 0 | ease-out | |
| Press scale 0.94 | 80 ms | 0 | ease-out | release springs back |
| + → × rotate 45° | 200 ms | 0 | spring, critically damped (stiffness 400, damping 40, proposed) | ≈200 ms observed in [R013] |
| Menu open (scale 0.96→1, y 4→0, fade) | 180 ms | 0 | ease-out cubic-bezier(0.22, 1, 0.36, 1) | proposed |
| Menu close | 120 ms | 80 ms after pick feedback | ease-in | |
| Row stagger (`expressive` only) | 20 ms per row | — | — | cap 6 rows (≤ 120 ms total) |
| Highlight glide | 140 ms | 0 | spring (stiffness 600, damping 40, proposed) | snap on jumps > 6 rows |
| Panel height on filter | 160 ms | 0 | spring, no overshoot | |
| Submenu open / close | 160 ms / 100 ms | 120 ms hover intent | ease-out / ease-in | safe triangle 300 ms |
| Loading skeleton | appears after 150 ms | 150 ms | 2 s linear sweep | never flashes for fast lists |
| Status cross-fade | 100 ms out + 160 ms in | 0 | ease-out | |
| Tool discovered row | 140 ms each | 60 ms stagger | ease-out, 4 px | |
| Active chip grow-in | 160 ms | 0 | spring | scale 0.85 → 1 |
| Error shake | 240 ms, 2 cycles, 4 px | 0 | linear | |
| **Reduced motion** | 0–80 ms opacity cross-fades | — | linear | no rotate (glyph cross-fades + → ×), no scale, no glide, no slide, no shake; skeleton static |

Sizes (proposed): menu width 280 px (min 240, max 360), max height 360 px or 60 % of viewport, row 32 px
(28 px dense, 44 px touch), section header 24 px, 8 px panel padding, 4 px gap between sections with a
1 px divider.

## 6. Variants

### Free (8)
| Name | Idea |
|---|---|
| `tools-menu/popover` | Baseline: "+" → × and a sectioned popover above the composer |
| `tools-menu/command` | Search-first panel: the search field is focused on open; sections become result groups |
| `tools-menu/sheet` | Mobile bottom sheet with drill-in submenus and a back row; 44 px rows |
| `tools-menu/split` | Two buttons: a paperclip for files and a "+" (or sliders icon) for context, skills, connectors and modes, separating the two jobs |
| `tools-menu/chip-row` | No menu: the top 3–5 actions as chips above the composer, "More" opens the popover (quick-action chips, [R047]) |
| `tools-menu/footer-tray` | A tray under the composer with folder and connected-app logos (footer tray idea from [R044]) |
| `tools-menu/terminal` | TUI list overlay with numbered rows and `[x]` toggles; arrows + digits |
| `tools-menu/minimal` | Text-only menu, no icons or descriptions, for dense IDE panels |

### Pro (14)
| Name | Idea |
|---|---|
| `tools-menu/genie` | The "+" goo-stretches into the menu and sucks back in on close [R045] |
| `tools-menu/morph-grow` | The button surface itself grows into the panel (one element, radius morphs), no separate popover |
| `tools-menu/radial` | Sections fan out in an arc around the "+" (touch and pen) |
| `tools-menu/tabbed` | Section tabs across the top (Attach · Context · Connectors · Skills · Modes) |
| `tools-menu/budget` | Each connector row shows its context cost; a segmented meter totals enabled tools against a budget |
| `tools-menu/connector-graph` | Connectors as small nodes wired to the composer; needs-auth nodes pulse amber |
| `tools-menu/stack-cards` | Skills as a fanned card stack you flick through, preview on hover |
| `tools-menu/liquid` | Gooey metaball split between the "+" and each submenu |
| `tools-menu/magnetic` | Rows lean toward the pointer by up to 2 px; the highlight has inertia |
| `tools-menu/dock` | macOS-dock magnification across a horizontal row of tool icons |
| `tools-menu/accordion` | Single column; sections expand in place on a height spring, no submenus |
| `tools-menu/pixel` | Pixel-grid icons that assemble on open (matches pixel marks) |
| `tools-menu/hold-peek` | Press-and-hold "+" to peek the menu; release over an item to pick it |
| `tools-menu/spotlight-search` | Global ⌘K-style overlay that searches files, skills, connectors and modes together |

## 7. Edge cases
- **Long labels and descriptions** ellipsize to one line each; full text in tooltip and accessible name.
  Connector names over 24 characters truncate in the middle ("Google Dr…Sheets") to keep the brand ends.
- **Many items.** 40 skills or 30 connectors: search auto-shows (threshold 8 items, the Arc morph-select
  rule), list virtualises, recent/favourite items pin to the top.
- **No room above.** Flip below the trigger; on tiny windows (height < 420 px) use the sheet.
- **RTL.** The menu anchors to the "+"'s mirrored side; submenus open to the left; chevrons and back
  arrows mirror; the + → × rotation stays 45° (symmetrical glyph).
- **Tiny sizes (16–20 px).** The "+" becomes a 16 px glyph in `minimal` and `terminal`; hit area stays
  ≥ 24 px (44 px on touch). Submenus collapse into an inline accordion below 280 px width.
- **Touch.** No hover intent: tap opens submenus; long-press on a connector shows its details.
- **Slow networks.** Async sections show skeleton after 150 ms; a section that fails shows one inline
  error row with Retry, the rest of the menu still works. OAuth tabs that never return time out after
  120 s (proposed) to `Error("Sign-in timed out")`.
- **Busy agent.** Attach and Context stay available (they add to the next or queued message). Modes that
  would change the running turn are disabled with a reason, or queued for the next turn (app choice).
- **Environment limits.** Some environments only allow some items (Claude limits WSL sessions to "Add
  image", research notes 03); items are `Hidden` or `Disabled` with a reason, never silently missing.
- **Model limits.** If the chosen model cannot use tools, connectors show "Not available with this model"
  (Zed shows a "No tools" label and warning for MCP-incompatible models, research notes 03/08).
- **Re-auth.** A connector that loses auth mid-session flips to `NeedsAuth` and the "+" gets an amber dot;
  no modal interrupts typing.
- **Very fast actions.** Upload picker returns instantly → menu is already closed; no flash of the item's
  loading state under 150 ms.

## 8. Accessibility
- Trigger: button with `aria-haspopup="menu"`, `aria-expanded`, label "Add files, tools and more" (closed)
  / "Close menu" (open). With active items: "Add files, tools and more, 2 active".
- Menu: `menu` with `group`s labelled by section headers; toggles are `menuitemcheckbox`, exclusive
  modes `menuitemradio`, submenus `aria-haspopup` + `aria-expanded`. In `command` variant the search is a
  combobox with `aria-activedescendant`, so focus stays in the field.
- Each connector row's name includes status and count: "GitHub, connected, 5 tools". Status is icon +
  word, never colour alone.
- Live region (polite): "3 results", "No results", "GitHub needs sign in", "Connected to Linear, found 5
  tools", "Research on. Turns off Think.".
- Disabled items stay focusable and expose the reason via `aria-describedby`.
- Focus order: "+" → (menu opens) search → first item; Esc returns focus to "+"; picking returns focus to
  the composer text area.
- Contrast: item text 4.5:1 minimum, descriptions 4.5:1, disabled text exempt but the reason line meets 4.5:1.
- Reduced motion per §5.

## 9. Props sketch

```rust
pub struct ToolsMenu {
    pub agent: AgentState,                   // busy → per-item mid-run rules apply
    pub variant: ToolsMenuVariant,           // Popover | Command | Sheet | Split | ChipRow | ...
    pub sections: Vec<MenuSection>,          // ordered; each { id, label, items, async_source }
    pub search: SearchMode,                  // Auto { threshold: 8 } | Always | Never
    pub open: Option<bool>,                  // controlled or uncontrolled
    pub active: Vec<ItemId>,                 // modes/tools currently on → chips + dot
    pub mid_run: MidRunPolicy,               // Disable | QueueForNextTurn
    pub surface: Surface,
    pub labels: ToolsMenuLabels,
    pub motion: MotionLevel,
    pub on_pick: Callback<ItemId>,
    pub on_toggle: Callback<(ItemId, bool)>,
    pub on_connect: Callback<ConnectorId>,   // starts auth; app drives ConnectorState
}

pub enum MenuItemKind {
    Action { shortcut: Option<Shortcut> },
    Toggle { on: bool, exclusive_group: Option<GroupId> },
    Submenu { children: Vec<MenuItem> },
    Connector { state: ConnectorState, tools: Vec<ToolInfo>, context_tokens: Option<u32> },
}

pub enum ConnectorState { Off, NeedsAuth, Authorizing, Discovering, Connected { tools: u16 }, Error(String) }

pub enum ItemAvailability { Available, Conflict(ItemId), Disabled(String), Hidden }
```

## 10. Related components and references
**Related:** composer input shell · attachments (Upload file / drop on "+") · slash and mention popovers
(shared matcher and row component) · mode switch · model picker · permission / autonomy selector · MCP /
plugin list in workspace chrome · active-mode chips · context meter.

**References** (study only unless marked MIT/Apache; see [../licensing.md](../licensing.md)):
- [R013] + morphs into × (45° rotate, ≈200 ms) opening a "Create anything…" menu with a nested Video submenu. Study only.
- [R045] Genie / goo menu morph from a "…" button. Study only.
- [R044] affectUI composer footer tray for folder / connected apps. Paid: study only.
- [R047] Composer quick-action chips that open popovers. Study only.
- [R036] BoardUI composer with approval toggle beside "+". Study only unless in the MIT tier.
- [R126] coss ui composer and menus. **MIT**: API shape portable with attribution.
- Arc UI MCP servers block, command palette and morph select (status words, 100/160 ms cross-fade, tool discovery, 8-item search threshold, gliding highlight): paid / unclear, **study only** (research notes 12).
- Claude Code desktop, Lovable, Zed, Verdent "+" menus; Conductor MCP status and per-server tool count; Le Chat mode exclusivity; Gemini unified "+": research notes 03, 04, 08. Claude, Lovable, Verdent, Conductor, Le Chat, Gemini are proprietary (reference only); Zed is mixed licence, study only by default.
