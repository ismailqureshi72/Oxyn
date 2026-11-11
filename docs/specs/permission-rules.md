**Status:** draft v0.1 (2026-11-11).

# Spec: permission rules, sandbox badge & audit log (`permission-rules`, `sandbox-badge`, `audit-log`)

Part of family G ([approvals.md](approvals.md)). Same depth as
[composer-send-control.md](composer-send-control.md). Target **5 free + 12 Pro = 17 variants** across
three components that share one data model: the **rules** that decide whether a request asks, the
**sandbox** that bounds what an allowed action can reach, and the **audit log** of every decision.
"Always allow" on an approval card ([approval-card.md](approval-card.md) `PatternEditing`) writes a rule
here; the permission selector shows the rule count and links here
([composer-permission-selector.md](composer-permission-selector.md) §3 "rules row", Pro
`permission-selector/rules-drawer`). Numbers are Oxyn **proposals** unless a source is cited next to them.

## 1. Purpose
Over a week of use the user accumulates dozens of "don't ask again" answers. They need to see **what is
allowed without asking, where that came from, and how wide each permission really is**, and change it:
narrow a pattern, move it from global to one project, delete it, or add a deny. They need to see **when two
rules disagree** and which one wins. While the agent works they need a small, always-visible answer to
"**what can an allowed command actually reach?**" (the sandbox badge). And after the fact they need a
**trustworthy record**: what was asked, what was decided, by whom (you, a rule, the level, a reviewer, a
timeout), and how long it waited (the audit log).

What we saw: Claude Code rules like `Bash(npm test *)` with allow / ask / deny and scopes from user,
project and managed settings; Codex "always allow commands starting with `X`", sandbox
`read-only | workspace-write | danger-full-access`, network off by default in workspace-write, `.git`
protected (notes 03, 05); Claude desktop browser per-site grants revocable in Settings (note 03); Cursor
allowlists and team MCP allowlisting (note 05); signoff-ui `useApprovalPolicy` with rules by tool and
argument pattern and an audit trail [R107]; research note 05's RuleEditor / AllowlistRow (pattern, scope,
source, allow / ask / deny) and SandboxIndicator (3 levels × network on / off + protected-paths tooltip);
affectUI's Tool Access Map and Agent Ledger (names only, note 11).

Anatomy (`permission-rules/table`, project scope filter):
```
Rules · 14            [ All scopes ⌄ ] [ Any tool ⌄ ] [ Search patterns…        ] [ Test a command ] [ + Add rule ]
──────────────────────────────────────────────────────────────────────────────────────────────────────────────────
 Effect   Tool       Pattern                          Scope        Source            Used        
 ✓ Allow  Command    starts with  cargo test          This project from approval 2d  41 × · 3m ago   ⋯
 ✓ Allow  Command    starts with  npm *               All projects you, manual        2 × · 5d ago   ⋯  ⚠ overlaps "Deny npm publish"
 ⊘ Deny   Command    starts with  npm publish         All projects you               0 ×             ⋯
 ? Ask    Network    host         *.stripe.com        This project 🔒 organisation    —               ⋯  managed · read-only
 ✓ Allow  Edit       path glob    src/**              Session      from approval     7 ×  ⏲ ends with session
 ░ Allow  Command    exact        cargo test --lib    This project you               never      ⋯  shadowed by row 1 (never applies)
```

## 2. Data model and precedence (proposed)

### 2.1 Rule
- **Effect**: `Allow` (auto-approve), `Ask` (always show a card, even if the level would auto-run),
  `Deny` (block without asking; the agent receives `denied` with the rule's note).
- **Matcher** (no regex in v0.1, so overlaps stay computable): `Exact(args)`, `Prefix(tokens)` (command
  tokens, Codex-style), `PathGlob` (`*`, `**`, relative to the workspace root), `Domain { host,
  include_subdomains }`, `McpTool { server, tool, args: Vec<(key, glob)> }`, `Payee { id, max_amount }`
  (session scope only), `Secret { name, destination }`.
- **Scope**: `Session` (dies with the session; created by "For this session"), `Project` (this workspace),
  `Global` (all projects for this user). **Source**: `User`, `ProjectFile` (checked into the repo),
  `Managed(org)` (read-only here).
- **Provenance**: `FromApproval(ApprovalId)` | `Manual` | `Import(file)`, created_at, last_used, hit
  count, optional `expires_at`, optional note (shown to the agent on Deny).

### 2.2 Precedence
1. **Managed beats everything** (organisation policy cannot be overridden from this UI).
2. Otherwise **Deny > Ask > Allow**, whatever the scope (a narrow project allow cannot punch through a
   global deny). Rationale: the safe answer wins when the user's own rules disagree.
3. Among rules of the winning effect, the **most specific** matcher is shown as "the rule that matched"
   (exact > longer prefix / glob > shorter > domain wildcard).
4. Rules decide before the permission level: `Allow` auto-runs even at `Ask` level; `Ask` shows a card
   even at `FullAccess`; the level applies only when no rule matches. Critical requests ignore `Allow`
   rules (approvals.md §2.3).

### 2.3 Conflicts
| Kind | Detected when | Shown as |
|---|---|---|
| `Shadowed` | a rule can never win (a broader rule of a stronger effect or same effect covers it entirely) | row at 50 % with ░ fill, "shadowed by row n (never applies)", Remove suggestion |
| `Overlap` | an Allow and a Deny / Ask both match some inputs | ⚠ chip on both rows "overlaps …"; tooltip with one sample input and which wins |
| `Duplicate` | same matcher + effect at the same scope | "duplicate of row n", Merge |
| `PolicyOverride` | a user rule is overridden by a managed rule | 🔒 chip "organisation rule wins" |
| `Stale` | rule unused for 90 days (proposed) | muted "unused 90 d" tag; never auto-deleted |

### 2.4 Sandbox status
`level: ReadOnly | WorkspaceWrite | FullAccess` × `network: Off | On | Allowlist(n hosts)`, plus
`protected_paths` (e.g. `.git`, per Codex note 05), `status: Starting | Active | Unavailable(reason) | Off`,
and `violations` (count of blocked attempts this turn). Derived tier for the badge: ReadOnly → 0,
WorkspaceWrite + network off → 1, WorkspaceWrite + network on → 2, FullAccess → 3, Off (no sandbox) → 4,
matching the selector's tier colours.

### 2.5 Audit
`AuditEntry` as in [approvals.md](approvals.md) §2.7. The log is append-only; the UI offers filter, search,
open-request, open-rule and export (callback to the app); it has no edit and no delete.

## 3. State machines

### 3.1 Rules list / editor
```
  Loading ──▶ Empty ("No rules yet · Rules appear when you choose Always allow") ──add──▶ Editing(new)
     │
     └──────▶ Ready(rules) ──type / filter──▶ Filtering ──clear──▶ Ready
                 │  │  │  │
                 │  │  │  └─ Test a command ──▶ Testing(input) ── result: Matched(rule) | Level(L) | Denied(rule) ─▶ Ready
                 │  │  └──── row ⋯ / Enter ──▶ Editing(rule) ── save ─▶ Validating ─┬─ ok ─▶ Saving ─▶ Ready (row flash)
                 │  │                                │ Esc (unsaved? confirm)        ├─ conflict ─▶ ConflictWarning ─ save anyway / adjust
                 │  │                                ▼                               └─ invalid ─▶ Invalid(reason)
                 │  │                              Ready
                 │  └─── Delete ─▶ Deleted(undo 5 s) ── timeout ─▶ Ready      Saving ── fail ─▶ SaveFailed (Retry)
                 └────── approval card "Always allow…" confirmed ─▶ Adding(from approval) ─▶ Ready (new row enters, highlighted)
  Row overlays: Managed (read-only, 🔒) · Shadowed · Overlap · Expiring(⏲) · RecentlyUsed (dot 1.6 s)
  Session grants: "Revoke" on a session row ─▶ Revoked (immediately applies to queued requests)
```

### 3.2 Sandbox badge
```
  Off ──app starts sandbox──▶ Starting ──ok──▶ Active(level, network) ──level change──▶ Active(level', network')
   ▲                             │ fail                 │ blocked attempt        │ sandbox dies
   │                             ▼                      ▼                        ▼
   └──────────────────── Unavailable(reason) ◀── Violation(n) (flash, 1.2 s) ── Unavailable(reason)
  Hover / focus ─▶ Popover(details: level, network, protected paths, last violations, "Change in permissions")
```

### 3.3 Audit log
```
  Loading ─▶ Empty | Live(entries, follow = true) ── user scrolls up ─▶ Paused("4 new ↓") ── click / End ─▶ Live
               │  filter / search ─▶ Filtered(n of m) ── clear ─▶ Live
               │  row Enter ─▶ Detail(entry) ─ open request / open rule / copy ─▶ Live
               │  Export ─▶ Exporting ─▶ Exported("Saved 214 entries") | ExportFailed
               └  source error ─▶ Error(reason, Retry)
```

## 4. States table

### 4.1 Rules
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Loading` | 4 skeleton rows ([skeletons.md](skeletons.md)) | — | — | — |
| `Empty` | glyph + "No rules yet" + how rules are made; Add rule | Add rule | Import | Enter add |
| `Ready` | table (effect glyph + word, tool, matcher kind + pattern in mono, scope, source, used count + last used, actions) grouped by scope with counts | open row | filter, test, add | ↑/↓ rows, Enter edit, Del delete (with undo), `/` search, `T` test, `N` new |
| `Filtering` | matching rows, "6 of 14", highlights in patterns | — | clear | Esc clear |
| `Editing(rule)` | inline sheet: effect segmented (Allow · Ask · Deny), tool picker, matcher kind + pattern field with live token highlight, scope radio (Session · This project · All projects), expiry (never / 1 h / 1 day / custom), note to agent (Deny); live preview "Would match 41 past calls · 0 denied" from the audit log | Save | Cancel, Delete | Tab fields, ⌘/Ctrl+Enter save, Esc cancel |
| `Validating` / `Invalid(reason)` | inline error under the field ("Pattern matches every command") | fix | — | — |
| `ConflictWarning` | "This allow overlaps your deny `npm publish`; deny wins for those commands" with sample | Save anyway | Adjust | Enter save, Esc adjust |
| `Adding(from approval)` | new row enters at the top of its scope group with a 1.6 s highlight and "from your approval just now" | — | Edit, Undo | — |
| `Deleted` | row collapses; toast "Rule deleted · Undo" 5 s | — | Undo | ⌘/Ctrl+Z |
| `Testing(input)` | field "Try a command, path or host"; result line: "Allowed by row 1 `cargo test`" / "Asks (no rule; level Ask)" / "Denied by row 3" and the winning row flashes | — | — | Enter test, Esc close |
| Row `Managed` | 🔒 + "organisation"; no edit; tooltip shows source | view | — | Enter view |
| Row `Shadowed` / `Overlap` / `Duplicate` | as §2.3 | fix suggestion | dismiss | Enter opens suggestion |
| Row `Expiring` | ⏲ "ends with session" / "ends in 42 m" | revoke | extend | R revoke |
| `SaveFailed` | inline "Couldn't save · Retry"; edits kept | Retry | Cancel | Enter |

### 4.2 Sandbox badge
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Active(ReadOnly)` | box glyph with closed lid + "Read-only" | open popover | — | Enter / Space opens |
| `Active(WorkspaceWrite, Off)` | box + pencil + "Workspace · no network" | popover | — | as above |
| `Active(WorkspaceWrite, On / Allowlist)` | box + pencil + network dot + "Workspace · network" / "· 3 hosts" | popover | — | — |
| `Active(FullAccess)` | open box + ▲ corner + "Full access" in tier-3 tone | popover | — | — |
| `Off` | box with slash + "No sandbox" in tier-4 tone, static hatch | popover | — | — |
| `Starting` | box glyph with a 2-frame still ("Starting sandbox") — no spinner loop beyond 2 s | — | — | — |
| `Unavailable(reason)` | box with ! + "Sandbox unavailable"; reason in popover | popover | "Learn more" (app) | — |
| `Violation(n)` | badge flashes once in tier tone; counter "2 blocked" for the turn | popover lists blocked attempts with "Allow…" (opens an approval-style rule editor) | dismiss | Enter |
| `Popover` | level, network (with hosts), protected paths list, violations this turn, link "Change in permissions" | — | — | Esc close |

### 4.3 Audit log
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Live` | rows newest at the bottom (timeline) or top (table variant): time, kind glyph, target (mono, truncated), risk glyph, decision glyph + word, by (You / Rule n / Level / Reviewer / Policy / Timeout), waited (0:07), agent swatch | open detail | filter | ↑/↓, Enter detail, `/` search, `F` filters, End follow |
| `Paused` | "4 new ↓" pill at the edge | resume follow | — | End |
| `Filtered` | filter chips (Decision, By, Risk, Kind, Agent, Date) + "12 of 214" | — | clear | Esc clear |
| `Detail(entry)` | side sheet: full request snapshot (redacted), risk reasons, reason quote, decision, deny text, rule link, origin, run / turn links | open rule / open run | copy JSON | Esc close |
| `Empty` | "Nothing decided yet in this range" | — | change range | — |
| `Exporting` / `Exported` / `ExportFailed` | inline progress line / toast / error with Retry | — | — | — |
| `Error` | "Couldn't load the log · Retry" | Retry | — | Enter |

## 5. Micro-interactions
- **Rule born from an approval.** When the approval card resolves with "Always allow", a ghost of the
  pattern chip lifts from the card and travels to the permission chip in the composer (≤ 260 ms
  `dur.travel`), whose rule count rolls up by one; if the rules view is open, the new row enters with
  `dur.row.enter` and a 1.6 s highlight (`dur.highlight.fade`).
- **Pattern field.** Tokens covered by the pattern highlight as you type (80 ms); a `*` token draws a
  dashed underline to show "anything here"; broadening past the first argument tints the border with the
  Medium tone and shows "Matches commands you haven't seen yet".
- **Live match preview.** The "Would match N past calls" count rolls (280 ms) after a 200 ms debounce.
- **Conflict chip.** Appears with a 160 ms scale 0.9 → 1; hovering it draws a 1 px connector to the other
  row (240 ms) so the pair is visible.
- **Tester.** The winning row flashes its background once (400 ms); shadowed rows briefly dim further.
- **Delete / undo.** Row collapses (180 ms `ease.in`); Undo re-expands it (`dur.row.enter`) in place.
- **Revoke session grant.** Row strikes through (200 ms), then collapses; any queued request that relied on
  it becomes a card again (approval-card.md).
- **Sandbox level change.** Glyph morphs (box lid closes / opens, 220 ms), tone crossfades (240 ms OKLab);
  label rolls.
- **Violation.** One 600 ms flash of the badge border in tier tone (no loop), counter pops in
  (`spring.snap`); repeated violations within 2 s only bump the counter.
- **Audit follow.** New entries enter with `dur.row.enter`; when paused, the "n new" pill counts with a
  roll; jump-to-latest scrolls with `ease.out` 240 ms (instant if > 50 rows away).

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Row enter (rule / audit) | 200 ms height + 160 ms fade | 0 | `spring.snap` / `ease.out` | 40 ms cap 6 | 60 Hz |
| New-rule highlight | 1600 ms | 0 | `ease.out` | — | 30 Hz |
| Pattern chip travel to composer | ≤ 260 ms | resolve end | `ease.in-out` | — | 60 Hz |
| Rule count roll | 280 ms | 0 | `ease.out` | — | 30 Hz |
| Token highlight | 80 ms | 0 | `ease.out` | — | 60 Hz |
| Match preview roll | 280 ms | 200 ms debounce | `ease.out` | — | 30 Hz |
| Conflict chip / connector | 160 ms / 240 ms | 0 / hover 300 ms | `spring.snap` / `ease.out` | — | 60 Hz |
| Tester row flash | 400 ms | 0 | `ease.out` | — | 60 Hz |
| Delete collapse / undo toast | 180 ms / 5 s | 0 | `ease.in` | — | 60 Hz |
| Revoke strike + collapse | 200 + 180 ms | 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Editor sheet open / close | 200 / 140 ms | 0 | `spring.snap` / `ease.in` | — | 60 Hz |
| Sandbox glyph morph / tone | 220 / 240 ms | 0 | `spring.snap` / `ease-in-out` | — | 60 Hz |
| Violation flash / counter pop | 600 ms / 160 ms | 0 | `ease.out` / `spring.snap` | — | 60 Hz |
| Audit jump to latest | 240 ms | 0 | `ease.out` | — | 60 Hz |
| **Reduced motion** | rows appear with 80 ms fade, no height spring; no chip travel (count swaps); highlights become a static 2 px inline-start bar for 4 s; connector drawn instantly; no flash (tester shows "← matched" text on the row); sandbox glyph swaps with 80 ms fade; violation shows a static ! dot until opened; audit jumps instantly | — | linear | none | 1 Hz idle |

## 7. Variants

### Free (5)
| Name | Idea | Surfaces |
|---|---|---|
| `permission-rules/table` | Sortable table grouped by scope with effect, tool, pattern, scope, source, usage, conflict chips, tester and add | desktop, ide (settings), chat (sheet) |
| `permission-rules/editor` | The rule editor sheet on its own (also opened from approval-card "Always…"): effect, matcher, scope, expiry, note, live match preview | all |
| `sandbox-badge/chip` | Chip with box glyph + level + network word; popover with protected paths and violations | chat, desktop, ide, mobile |
| `sandbox-badge/glyph` | TUI status segment `[sandbox: workspace · net off]` / `[no sandbox]` with tier glyph | terminal, ide |
| `audit-log/timeline` | Chronological list with decision glyphs, by-whom, waited time, follow / pause, filters, detail sheet, export callback | desktop, ide, chat (sheet) |

### Pro (12)
| Name | One-line idea |
|---|---|
| `permission-rules/rules-drawer` | Rules as editable chips in a drawer off the permission chip (pattern, scope, source) (used by `permission-selector/rules-drawer`) |
| `permission-rules/tool-map` | Grid of tools × effect with counts; click a cell to see its rules (affectUI Tool Access Map idea, note 11) |
| `permission-rules/pattern-ladder` | Pattern editor shows narrower → broader steps as a ladder with match counts per rung |
| `permission-rules/simulator` | Replays the last run against draft rules: "With these rules, 12 of 15 prompts would not have asked" |
| `permission-rules/suggestions` | Suggests rules from repeated identical approvals ("You allowed `cargo test` 9 times · Make it a rule?") |
| `permission-rules/diff-review` | Changes to rules shown as a diff before saving (for project rule files checked into the repo) |
| `permission-rules/site-grants` | Per-site / per-app grants list with favicons, subdomains toggle and Revoke (Claude browser idea, note 03) |
| `sandbox-badge/cube` | Isometric box whose walls fade as access widens; network as a port on the wall (pairs with `permission-selector/sandbox-cube`) |
| `sandbox-badge/reach` | Concentric reach rings: workspace · machine · network, filled per level |
| `audit-log/table` | Dense sortable table with column chooser, saved filters, CSV / JSON export callback |
| `audit-log/ledger` | Ledger view grouped by run with totals: asked, auto-allowed, denied, waited (affectUI Agent Ledger name, note 11) |
| `audit-log/trace-lane` | Approvals drawn as hatched waiting segments on the trace waterfall lane ([trace-view.md](trace-view.md)) |

## 8. Edge cases
- **Hundreds of rules**: virtualised rows (render ~2× viewport); group headers stick; search matches
  pattern and note text.
- **Very long patterns / paths**: middle-truncate in the table (`src/…/handlers/**`), full in tooltip and
  editor; mono font, never wrapped mid-token.
- **Pattern that matches everything** (`*`, `**`, empty prefix): blocked as `Invalid` for Allow at Global
  scope; allowed for Deny / Ask (proposed).
- **Project rule file changed on disk** (checked-in rules): rows update with highlight and "changed in
  rules file"; local unsaved edits show a conflict prompt.
- **Managed rule appears mid-session** and blocks a user allow: row gets 🔒 "organisation rule wins";
  queued requests re-evaluate.
- **Session ends**: session rows disappear with a one-time note "3 session permissions ended".
- **Secrets**: credential rules show the secret **name** only; audit snapshots store masked values.
- **Audit at high volume** (fan-out runs): entries coalesce per 50 ms batch into one row "Allowed 6 ·
  by rule `cargo test`" with expand; follow mode caps at 30 new rows per second visually (counter keeps
  exact count).
- **Clock skew / remote agents**: audit shows the server's timestamp; the waited time is computed from the
  run clock's paused interval, not wall clocks on two machines.
- **Sandbox flapping** (dies and restarts): badge stays `Unavailable` for at least 2 s before returning to
  `Active` (no flicker); violation counters reset per turn.
- **RTL**: tables mirror; patterns, paths and hosts stay LTR in isolated spans.
- **Tiny sizes (16–20 px)**: sandbox badge as glyph only, tier-3 and tier-4 add a 4 px corner dot; rules
  and audit are not offered below 280 px width (open as a sheet).
- **Touch**: rows 44 px; swipe-to-delete is not offered (proposed — destructive); Delete is in the row menu
  with undo.
- **Slow network / remote rule store**: optimistic row with "Saving…" underline; on failure the row
  reverts with SaveFailed; a rule is not used for decisions until saved (fail safe: until then requests
  still ask).

## 9. Accessibility
- Rules table: `grid` with column headers; each row's accessible name reads as a sentence: "Allow commands
  starting with cargo test, this project, created from an approval 2 days ago, used 41 times". Conflict
  chips are part of the name ("overlaps deny npm publish; deny wins").
- Effect is a word plus a glyph (✓ / ? / ⊘), never colour only; shadowed rows say "never applies".
- Editor: labelled fields; the live match preview is `aria-live="polite"` (debounced 500 ms); validation
  errors are linked with `aria-describedby` and announced.
- Delete: polite "Rule deleted. Undo available for 5 seconds"; Undo reachable by keyboard and ⌘/Ctrl+Z.
- Tester result: polite "Allowed by rule: commands starting with cargo test".
- Sandbox badge: `button` named "Sandbox: workspace write, network off. Show details"; violations are
  polite ("Sandbox blocked a write to /etc/hosts"), at most one announcement per 5 s.
- Audit log: `log` role for the live timeline (polite, new entries announced only when follow is on and
  at most one per 2 s, summarised); table variant uses `grid`; Detail sheet is a `dialog`.
- Focus order (rules view): filters → search → test → add → table; after save focus returns to the row.
- Contrast: text 4.5:1, glyphs and borders 3:1 in both themes and daltonized themes.

## 10. Props sketch
```rust
pub struct Rule {
    pub id: RuleId,
    pub effect: Effect,                     // Allow | Ask | Deny
    pub matcher: Matcher,                   // Exact(Value) | Prefix(Vec<Token>) | PathGlob(Glob) | Domain { host, include_subdomains }
                                            // | McpTool { server, tool, args: Vec<(SharedString, Glob)> } | Payee { id, max: Money } | Secret { name, destination }
    pub tool: ToolKindFilter,               // Command | Edit | Network | Mcp(server) | Spend | Credential | Any
    pub scope: RuleScope,                   // Session | Project(ProjectId) | Global
    pub source: RuleSource,                 // User | ProjectFile(path) | Managed(OrgRef)
    pub provenance: RuleProvenance,         // FromApproval(ApprovalId) | Manual | Import(path)
    pub note: Option<SharedString>,         // sent to the agent on Deny
    pub expires_at: Option<SystemTime>,
    pub stats: RuleStats,                   // hits, last_used
}

pub struct PermissionRules<'a> {
    pub agent: &'a AgentState,              // AwaitingApproval { request } → "Test" prefilled with the open request
    pub rules: &'a [Rule],
    pub conflicts: &'a [Conflict],          // computed in oxyn-core: Shadowed | Overlap | Duplicate | PolicyOverride | Stale
    pub audit: Option<&'a AuditLogRef>,     // for "would match N past calls"
    pub editable_scopes: ScopeSet,          // which scopes this surface may write
    pub variant: RulesVariant,              // Table | Editor | Pro(..)
    pub labels: RulesLabels,
    pub motion: MotionLevel,
    pub on_save: Callback<RuleChange>,      // Add | Update | Delete | Revoke, with undo token
    pub on_test: Callback<TestInput, TestResult>, // Matched(RuleId) | Level(LevelId) | Denied(RuleId)
}

pub struct SandboxBadge {
    pub status: SandboxStatus,              // Starting | Active { level, network, protected_paths } | Unavailable { reason } | Off
    pub violations: SmallVec<[Violation; 4]>,
    pub agent: AgentStateRef,               // ToolRunning → violations attach to the running call
    pub variant: SandboxVariant,            // Chip | Glyph | Pro(..)
    pub motion: MotionLevel,
    pub on_open_permissions: Callback<()>,
}

pub struct AuditLog<'a> {
    pub entries: &'a AuditLogRef,           // append-only, paged
    pub filter: AuditFilter,                // decision, by, risk, kind, agent, date range, text
    pub follow: bool,
    pub coalesce: Duration,                 // 50 ms batches
    pub variant: AuditVariant,              // Timeline | Pro(..)
    pub on_open_request: Callback<ApprovalId>,
    pub on_open_rule: Callback<RuleId>,
    pub on_export: Option<Callback<ExportFormat>>, // app-owned; the component never writes files
    pub motion: MotionLevel,
}

/// oxyn-core: rule evaluation used by the approval queue (approvals.md §3 AutoResolved).
pub fn evaluate(req: &ApprovalRequest, rules: &[Rule], level: LevelId) -> Evaluation {
    // 1 managed · 2 deny > ask > allow · 3 most specific for display · 4 level if no rule · Critical ignores Allow
    /* … */
}
```

## 11. Related components and references
**Related:** [approvals.md](approvals.md) (model, audit entry) · [approval-card.md](approval-card.md)
(pattern editor, `AutoResolved`, revoke effects) · [composer-permission-selector.md](composer-permission-selector.md)
(rule count, sandbox row, policy cap, `sandbox-cube` / `rules-drawer`) · [tool-call.md](tool-call.md)
(provenance `Rule(RuleId)`, "Change rule" on `Denied`) · [trace-view.md](trace-view.md) (waiting
segments) · [skeletons.md](skeletons.md) · family J settings / MCP list.

| Reference | What we take | Licence note |
|---|---|---|
| Claude Code permission rules and scopes (note 05) | `Tool(pattern)` rules, allow / ask / deny, user / project / managed sources | Proprietary — reference only |
| Codex sandbox and approvals (notes 03, 05) | three sandbox levels, network off by default, `.git` protected, command-prefix "always" | MIT per licensing.md (note 08 reads Apache-2.0) — portable with attribution; re-check |
| Claude desktop per-site grants (note 03) | revocable site grants incl. subdomains | Proprietary — reference only |
| Cursor allowlists / team MCP allowlisting (note 05) | allowlist idea | Proprietary — reference only |
| signoff-ui `useApprovalPolicy` [R107] | rules by tool and argument pattern, audit trail | MIT — port with attribution after checking the repo |
| Research note 05 RuleEditor / AllowlistRow / SandboxIndicator | field set; 3 levels × network | Our synthesis |
| affectUI Tool Access Map, Agent Ledger (note 11) | names only | Paid — study only |
