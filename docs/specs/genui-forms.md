**Status:** draft v0.1 (2026-11-12).

# Spec: generative UI forms and question cards (`genui-form`, `genui-question`)

Part of family I ([genui.md](genui.md)); follows the 10-point standard in [README.md](README.md) at the
depth of [composer-send-control.md](composer-send-control.md). Target **6 free + 16 Pro = 22 variants**
(`genui-form` 3 + 10, `genui-question` 3 + 6). Field primitives are `TextField`, `Toggle`, `Choice`,
`Slider`, `DateTime` from [genui.md](genui.md) §2.3; this spec adds the `Form` and `QuestionCard` blocks,
the extra field kinds, validation, submission and the approvals hand-off. Question cards **are**
family-G `Question` requests rendered with gen-UI fields ([approvals.md](approvals.md) §2.2,
[approval-card.md](approval-card.md)). Values marked *proposed* are Oxyn's own.

## 1. Purpose
Mid-run the agent needs something only the user knows: which of three plans, a date range, a budget cap,
the recipient for an outreach email, shipping details, five answers before it can start. Instead of a
wall of prose questions it emits a **form** or a **question card**. The user wants to:
- answer fast (good defaults, one key per option, ⌘↵ to continue — numbered question card with
  "Continue ⌘↵", [R050]; options with one marked "(recommended)", [R022]);
- see which values **the agent filled in** versus which they chose;
- send what they have when some fields are optional, and **fix an answer after sending**;
- know exactly what **submitting does** (just tells the agent, or books / sends / pays);
- never be asked to type a **password, API key or card number** into something an agent generated
  ("Secrets stay out of the chat" card, [R049]).

Peers (note 05 §4, note 16): LangGraph Agent Inbox responses accept / edit / respond / ignore; CopilotKit
`useHumanInTheLoop` / `renderAndWaitForResponse` with `respond()`; Beautiful UI's multi-question approval
"1 / 3, Skip, Continue" ([R106]); Thesys C1 forms include inputs, dropdowns, checkboxes and multi-step
(note 05 §1); a CRM side-panel form with ⌘↵ submit ([R026]); a "Schedule a call" popover with a 15 / 30 /
45 / 60 segmented control, date strip and slots that resolves into a "Call scheduled" result card
([R047]).

## 2. Model and state machine

### 2.1 Field kinds (composed from the primitives, proposed)
| Kind | Primitive | Validation available | Notes |
|---|---|---|---|
| `text`, `multiline` | `TextField` | required, min / max length, pattern (linear-time subset ≤ 256 chars) | multiline grows 1 → 8 lines, then scrolls |
| `number`, `currency`, `percent` | `TextField{number}` + format | min, max, step, integer | locale-aware parsing; currency code shown as a prefix |
| `email`, `url`, `phone` | `TextField` | built-in syntax checks | phone keeps what the user typed, normalises on submit (E.164 if the region is known) |
| `select`, `radio`, `chips`, `segmented` | `Choice` | required, min / max selections | `allow_other` adds "Other…" with a text field |
| `multi_select`, `tags` | `Choice{multiple}` | min / max | tags allow free entries when `allow_other` |
| `checkbox`, `switch` | `Toggle` | required-true ("I agree") | |
| `slider`, `range` | `Slider` | min, max, step | value label always visible (not only on drag) |
| `date`, `time`, `datetime`, `date_range` | `DateTime` | min, max, disallowed days | ISO 8601 in and out; shown in the user's zone |
| `file` | hand-off | max size, types | opens the composer attachment flow ([composer-attachments.md](composer-attachments.md)); the form holds a reference, not bytes |
| `secure` | hand-off | — | **never a text box**: a "Provide securely" button that opens the host's secret / vault flow (§2.4) |
| `rating` | `Choice{segmented}` 1–5 / 1–10 | required | |
| `repeat` | group | min / max rows | repeatable row group ("Add another traveller") |

Every field may carry `default` (agent's value), `suggested: true` (show the suggestion marker),
`help`, `depends_on` (show when another field has a value; declarative only), and `section`.

### 2.2 Form state machine
```
  Skeleton ─ fields stream in ─▶ Filling (agent defaults land, marked "suggested")
                                   │ commit / node.done
                                   ▼
                                Editable ◀────────────────────────────────────────────┐
                                   │ user edits ─▶ Dirty ─ blur ─▶ Validating(field) ─▶ Valid | Invalid(msg)
                                   │                                 └ async check (agent / Query intent): "Checking…"
                                   │ Submit (⌘↵)                                         │
                                   ▼                                                    │
                             PreSubmitCheck ── any invalid ─▶ Editable (focus first error, summary at top)
                                   │ secret-shaped value typed? ─▶ Blocked("Looks like a secret") ─▶ Editable
                                   │ ok
                                   ▼
                       intent kind = Answer/Query ──▶ Submitting ──ack──▶ Submitted(receipt, rev n)
                       intent kind = Mutate/Spend/Send/Credential/Custom ──▶ AwaitingApproval (family G card)
                                                       │ approved ─▶ Submitting ─▶ Submitted
                                                       │ denied / expired / stale ─▶ Editable ("Not sent: denied")
                             Submitting ─ fail / offline ─▶ SubmitFailed ("Couldn't send · Retry"), values kept
                             Submitting ─ server field errors ─▶ Editable with field messages

  Submitted ── Edit (if editable_until not passed and agent accepts revisions) ─▶ Revising(rev n+1) ─▶ … ─▶ Submitted(rev n+1)
            └── agent already acted (receipt.acted = true) ─▶ Edit opens "Send a change" = new request (append-only)
  Partial: SaveDraft (local, no agent) · SubmitAnswered (sends answered optional fields; required must be valid)
  Stop mid-stream with required fields missing ─▶ Incomplete ("Ask the agent to finish this form"), submit disabled
  Run cancelled while Editable ─▶ Stale (values kept, submit disabled: "The agent stopped")
```

### 2.3 Question card state machine (paged)
```
  Requested(questions n) ─ input guard 400 ms ─▶ Page(k of n) ──answer──▶ Answered(k) ── auto-advance 220 ms ─▶ Page(k+1)
        │                                          │ Skip (if allowed) ─▶ Skipped(k)          │ ← back keeps answers
        │                                          │ Other… ─▶ TypingOther ─▶ Answered(k)     │
        ▼                                          ▼                                         ▼
     AutoResolved (rule / policy, family G)     Review (all pages: summary list, edit any) ─ Continue ⌘↵ ─▶ Submitting ─▶ Resolved
   Overlays/exits as family G: Expired · Stale · Superseded · SubmitFailed · answered on another device
```

### 2.4 Secrets and sensitive data (decision, proposed)
1. The schema has **no password or secret text kind**. A `TextField` whose `name`, `label` or
   `placeholder` matches the secret lexicon (password, passcode, API key, token, secret, private key,
   seed phrase, card number, CVV / CVC, SSN, IBAN — list owned by the app, localisable) is converted at
   validation to `secure`.
2. `secure` renders a button "Provide <label> securely". Pressing it raises a family-G `Credential`
   request naming the secret and destination ([approvals.md](approvals.md) §2.2) and hands off to the
   host's vault / secret UI. The form receives only a **reference** (`SecretRef`), never the value; the
   agent sees "provided" or "not provided".
3. **Typed-secret guard.** Before submit, every text value is checked against secret shapes (key-like
   prefixes and high-entropy strings ≥ 24 chars, 13–19 digit numbers passing Luhn). A match blocks
   submit for that field with "This looks like a secret. Use secure input instead" and a "Provide
   securely" button; "It's not a secret" overrides for that value only (logged).
4. Values marked `sensitive` by the app (addresses, phone numbers) are masked in receipts and audit
   snapshots (`•••• 4821`), shown in full only while editing.

### 2.5 Defaults from the agent
- Agent defaults render as normal values with a **suggested marker** (✦ 12 px + "Suggested" tooltip,
  muted tone). The marker disappears when the user edits or explicitly confirms the field (Enter / tick).
- "Reset to suggestion" appears in the field menu after an edit.
- A form with every field suggested still requires the user to press Submit; there is no "accept all
  and submit" without a press (genui.md §2.6 rule 4). A "Confirm all suggestions" action marks them
  confirmed without submitting (proposed).
- Agent defaults never pre-tick consent or payment fields (`required-true` toggles start unticked
  whatever the default says).

## 3. States table

### 3.1 Form
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Skeleton` / `Filling` | field skeletons (label bar + field box) swapping to fields as they arrive; suggested values fade in | — | — | Tab reaches arrived fields |
| `Editable` | fields, section headers, footer: Submit (label from the action, e.g. "Book call"), effect line ("Books a 30 min call via Calendar") from the intent registry | Submit | Save draft, Submit answered (if partial allowed), Cancel | Tab / ⇧Tab fields, ⌘/Ctrl+↵ submit, Esc blurs field |
| `Dirty` | edited field loses ✦; unsaved dot on the form header | Submit | Reset to suggestion | — |
| `Validating` | field trailing loader 12 px "Checking…" | — | — | — |
| `Invalid` | field outline danger 2 px, message under field (icon + text), error summary at top on submit attempt ("2 fields need attention", links) | fix | — | Enter on summary link focuses field |
| `Blocked(secret)` | field message "This looks like a secret…" + "Provide securely" | Provide securely | It's not a secret | — |
| `AwaitingApproval` | footer replaced by "Waiting for approval" + amber hand; family-G card attached below with the registry's effect description | answer card | Edit (withdraws request → `Superseded`) | family G keys |
| `Submitting` | Submit label morph "Sending…", fields read-only | — | Cancel (if intent cancellable) | Esc cancel |
| `Submitted` | collapsed **receipt**: title + 1–3 key answers + "Submitted 14:02 · rev 1" + Edit | Edit (if allowed) | View all answers | Enter on Edit |
| `Revising` | form reopens with previous answers, changed fields marked "changed"; footer "Send changes" | Send changes | Discard changes | ⌘↵ |
| `SubmitFailed` | danger line "Couldn't send · Retry", values kept | Retry | Copy answers | Enter |
| `Incomplete` | banner "This form is incomplete" | Ask agent to finish (Answer intent) | — | — |
| `Stale` | dimmed, submit disabled with reason | — | Copy answers | — |

### 3.2 Question card
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Page(k/n)` | header "Question 1 of 3" + pager ‹ ›; question text; numbered options (1–9) with descriptions, "(recommended)" tag on the agent's pick; optional "Other…" | choose (auto-advance) | Skip (if allowed), Back | `1`–`9` choose, `O` other, `K` skip, ←/→ page, ⌘↵ continue |
| `TypingOther` | inline text field under options | Enter to save | Esc back to options | — |
| `Answered(k)` | chosen option filled + ✓, others dim to 60 % | (auto-advance 220 ms after choice; disabled for multi-select) | change | number keys change |
| `Review` | list of questions with answers, each with Edit | Continue / Send | Back | ⌘↵ send |
| `Submitting` | Continue → "Sending…" | — | — | — |
| `Resolved` | folds to one line "Answered 3 questions · View" | View | — | Enter |
| `Expired` / `Stale` / `Superseded` | as family G with reason line | as family G | — | — |

## 4. Micro-interactions
- **Fields arrive** top to bottom with the shared swap (120 ms crossfade per field part, 30 ms stagger,
  cap 6). Suggested values fade in 120 ms **after** their field (so the user sees the field first, then
  the agent's value) with the ✦ marker.
- **Focus ring** 2 px accent outside the field, 120 ms in; label stays above (no floating-label motion
  in `plain`; floating labels only in `sheet-mobile`).
- **Validation timing.** Validate on blur, then on every change once a field has shown an error
  ("reward early, punish late"); messages slide down 4 px + fade (120 ms); fixing the error morphs the
  icon to ✓ (160 ms) then clears after 800 ms.
- **Submit with errors.** No shake of the form; focus moves to the error summary; the first invalid
  field gets a single 2 px outline pulse (300 ms).
- **Submit.** Button press 0.97; label width springs to "Sending…"; on success the form **collapses into
  its receipt** (height eases to the receipt height 240 ms `spring.snap`, fields crossfade to the summary
  lines) — the result card idea in [R047].
- **Approval hand-off.** Footer crossfades (160 ms) to "Waiting for approval"; the family-G card slides in
  below (200 ms `ease.out`, as tool-call approval footers do, [approvals.md](approvals.md) hand-offs).
  Approved: card resolves (220 ms morph) and the form continues to `Submitting`.
- **Edit after submit.** Receipt expands back into the form (240 ms); changed fields show a "changed"
  chip; "Send changes" sends rev n+1; the receipt keeps a "rev 2" marker and "View previous" link.
- **Question paging.** Choosing an option fills it (accent fill 120 ms), others dim (160 ms), then the page
  slides 12 px and crossfades to the next after 220 ms (the approval pager motion in
  [approval-card.md](approval-card.md)); the pager numerator rolls.
- **Recommended.** The recommended option has a "(recommended)" text tag and is the default keyboard
  highlight — never pre-selected.
- **Secure hand-off.** "Provide securely" opens the host flow; on return the button morphs into a chip
  "API key provided ✓ · Replace" (160 ms); the value is never shown.
- **Partial submit.** "Submit answered" shows a count ("Send 4 of 6 answers"); unanswered optional
  fields get a muted "Skipped" tag in the receipt.

## 5. Timing

| Transition | Duration | Delay / stagger | Easing / spring | Notes |
|---|---|---|---|---|
| Field arrive | 120 ms | 30 ms stagger, cap 6 | `ease.out` | skeletons.md partial fill |
| Suggested value fade | 120 ms | +120 ms after its field | `ease.out` | proposed |
| Focus ring | 120 ms | — | `ease.out` | |
| Error message in | 120 ms (4 px slide + fade) | on blur | `ease.out` | |
| Error → ✓ morph / clear | 160 ms / 800 ms hold | — | `spring.snap` | |
| First-invalid outline pulse | 300 ms | — | `ease.in-out` | no shake |
| Submit label morph | 160 ms | — | `spring.snap` | |
| Collapse to receipt / expand to edit | 240 ms | — | `spring.snap` | top edge fixed |
| Approval footer swap / card in | 160 ms / 200 ms | — | `ease.out` | family G |
| Question choose fill / others dim | 120 ms / 160 ms | — | `ease.out` | |
| Question auto-advance | 180 ms (12 px slide + crossfade) | 220 ms after choice | `ease.out` | |
| Pager numerator roll | 280 ms | — | `spring.snap` | `dur.count.roll` (approvals.md §4) |
| Input guard | — | 400 ms after mount | — | family G rule |
| Secure chip morph | 160 ms | — | `spring.snap` | |
| `countdown-default` (Pro, Question only) | visible countdown 30 s default, pauses on hover / focus | — | 1 Hz digit swap | never for side-effect intents |
| **Reduced motion** | fields appear without stagger; no slides; error messages appear instantly; collapse / expand and paging are 80 ms crossfades; no outline pulse (static 2 px outline) | — | — | |

## 6. Variants

### 6.1 `genui-form` — Free (3)
| Name | Idea |
|---|---|
| `genui-form/stacked` | Labels above fields, one column, footer with Submit and effect line (default) |
| `genui-form/inline` | One-row form inside a message (e.g. date range + Apply), collapses to a receipt chip |
| `genui-form/side-panel` | Form in the side-panel container with sticky footer and ⌘↵ submit ([R026]) |

### 6.2 `genui-form` — Pro (10)
| Name | Idea |
|---|---|
| `genui-form/stepper` | Sections as steps with a progress header, per-step validation, Back / Next |
| `genui-form/conversational` | One field at a time in the thread, each answer becomes a user bubble, Edit jumps back |
| `genui-form/review-diff` | Edit-after-submit shows a field-level diff (old → new) before "Send changes" |
| `genui-form/prefill-sparkle` | Agent-suggested values type in with a soft ✦ trail and a "Confirm all suggestions" bar |
| `genui-form/two-column` | Wide layout pairing related fields, collapses to one column under 560 px |
| `genui-form/scheduler` | Duration segmented control, date strip and time slots resolving into a "scheduled" card ([R047]) |
| `genui-form/sheet-mobile` | Bottom sheet with floating labels, 44 px targets, keyboard-aware sticky Submit |
| `genui-form/box-drawn` | Terminal form: `[ ]`, `(•)`, underlined inputs, Tab / Enter navigation |
| `genui-form/repeat-rows` | Table-like repeatable row group (travellers, line items) with add / remove and row validation |
| `genui-form/draft-review` | Agent-drafted artifact (email, quote) as editable fields with "Review" and "Approve and send" ([R052]) |

### 6.3 `genui-question` — Free (3)
| Name | Idea |
|---|---|
| `genui-question/numbered` | Single question, numbered options with "(recommended)", ⌘↵ continue ([R022], [R050]) |
| `genui-question/paged` | Several questions, "Question 1 of 3", auto-advance, Review page ([R106]) |
| `genui-question/chips` | Options as chips inline in the message, for quick one-tap answers |

### 6.4 `genui-question` — Pro (6)
| Name | Idea |
|---|---|
| `genui-question/inbox` | Questions from several agents in one attention inbox with unread dots per agent ([R022]) |
| `genui-question/scale` | Answer on a labelled scale (1–5, 0–10) with anchors at both ends |
| `genui-question/rank` | Drag or number-key to rank options; order sent as the answer |
| `genui-question/image-choice` | Options are image tiles (pairs with generated candidates) |
| `genui-question/voice` | Options read aloud; answer by saying the number or option name (family K) |
| `genui-question/countdown-default` | Question-only: a visible, pausable countdown after which the agent proceeds with its recommended option; never offered for intents with side effects |

## 7. Edge cases
- **Agent streams fields slowly.** User may start filling arrived fields; later fields append below;
  a field the user already edited is never overwritten by a late default (default stored, ✦ hidden).
- **Agent revises the form while the user types** (`nodes.upsert` on a dirty field): held, badge "Agent
  update waiting · Apply" (renderer §7); removed fields with user input move to a "Removed by agent"
  disclosure so input is not lost.
- **Long labels / options.** Labels wrap to 2 lines, then ellipsis with full text in tooltip; > 9
  options: number keys cover 1–9, a filter field appears above options when > 12.
- **50 options.** `select` display auto-switches from radio / chips to a searchable select (proposed
  threshold 8 for chips, 12 for radio).
- **Conditional fields** (`depends_on`). Revealed fields slide in (120 ms); hidden fields keep their
  values but are not submitted.
- **Required field the agent forgot to send** (stop mid-stream). `Incomplete` state.
- **Edit window passed.** Edit replaced by "Send a change", which posts a new message / request; the
  receipt notes "The agent already acted on this".
- **Offline.** Submit disabled with "Offline — your answers are kept"; drafts persist locally (app
  storage) if the app enables drafts.
- **Approval denied** (family G). Form returns to `Editable` with "Not sent: <reason>"; values kept.
- **Same question answered on another device.** Card resolves in place with origin ("Answered on
  phone").
- **RTL.** Labels and fields mirror; number keys still map 1–9 in visual reading order; phone and
  numbers stay LTR.
- **Tiny widths (< 320 px).** All fields one column, segmented controls become selects, date range
  becomes two date fields.
- **Touch.** 44 px targets; date pickers use the platform picker where available.
- **Dense IDE.** `inline` and `numbered` preferred; error messages as trailing icons with tooltips.
- **Very fast agents.** Whole form in one patch: one fade, no stagger.
- **Pasted secret into a non-secret field.** Typed-secret guard (§2.4.3) on paste as well as submit;
  pasted value is held in the field but submit stays blocked.
- **File fields.** Upload progress is the composer attachment tile's; the form shows its chip and
  blocks submit until uploads are `ready`.

## 8. Accessibility
- **Labels.** Every field has a programmatic label (required by the schema); help text linked via
  `aria-describedby`; required fields marked in text ("required"), not just an asterisk.
- **Suggested values.** Announced with the field: "Budget, €2,000, suggested by agent".
- **Errors.** On submit attempt, focus moves to the error summary (`role="alert"`, assertive): "2 fields
  need attention: Email, Date"; each field's message is linked to the field and announced on focus.
- **Effect line.** Submit's accessible description includes the effect: "Book call. Books a 30 minute
  call via Calendar. Requires approval."
- **Question cards.** `radiogroup` (single) / `group` of checkboxes (multi) labelled by the question;
  "Question 1 of 3" announced politely on page change; "(recommended)" read as part of the option name.
- **Receipts.** `region` "Submitted answers"; Edit labelled "Edit answers (revision 2)".
- **Secure hand-off.** Button "Provide API key securely, opens secure entry"; after return "API key
  provided".
- **Focus order.** Fields in tree order, then Save draft, Submit; the agent never moves focus into a
  form; after submit focus stays on the receipt's Edit button.
- **Contrast.** Field borders 3:1, text 4.5:1, error colour paired with icon and text, ✦ marker is
  supplementary (the tooltip and announcement carry the meaning).
- **Reduced motion:** §5 last row.

## 9. Props sketch
```rust
pub struct FormBlock {
    pub agent: AgentState,                    // Streaming → Filling; Cancelled → Stale; AwaitingApproval(gated: this form) → AwaitingApproval
    pub fields: Vec<FieldNode>,               // NodeIds of TextField / Toggle / Choice / Slider / DateTime + hand-off kinds
    pub sections: Vec<Section>,               // { id, title, fields } ; stepper uses these as steps
    pub submit: ActionId,                     // intent from the registry (genui.md §2.6)
    pub partial: PartialPolicy,               // None | SubmitAnswered | SaveDraft | Both
    pub revisions: RevisionPolicy,            // { allowed: bool, editable_until: Option<SystemTime> }
    pub secret_lexicon: Arc<SecretLexicon>,   // app-owned, localisable
    pub typed_secret_guard: bool,             // true (proposed)
    pub variant: FormVariant,                 // Stacked | Inline | SidePanel | Pro(..)
    pub on_secure: Callback<SecureRequest>,   // → family G Credential request + host vault; returns SecretRef
    pub on_file: Callback<FileRequest>,       // → composer attachments
}

pub struct FieldNode {
    pub id: NodeId,
    pub kind: FieldKind,                      // Text | Multiline | Number | Currency{code} | Percent | Email | Url | Phone | Select | Radio
                                              // | Chips | Segmented | MultiSelect | Tags | Checkbox | Switch | Slider | Range | Date | Time
                                              // | DateTime | DateRange | File | Secure | Rating | Repeat
    pub default: Option<Value>,
    pub suggested: bool,
    pub rules: Vec<Rule>,                     // Required | MinLen | MaxLen | Pattern(SafeRegex) | Min | Max | Step | MinSel | MaxSel | AsyncCheck(ActionId)
    pub depends_on: Option<Condition>,
    pub sensitive: bool,
}

pub enum FormPhase {
    Skeleton, Filling, Editable, AwaitingApproval(ApprovalId), Submitting, Submitted { rev: u32, acted: bool },
    Revising { rev: u32 }, SubmitFailed, Incomplete, Stale,
}

pub struct QuestionCard {
    pub request: ApprovalId,                  // family G ApprovalKind::Question; one request per card
    pub questions: Vec<Question>,             // { id, prompt, options: Vec<QuestionOption{ value, label, description, recommended }>, multiple, allow_other, skippable }
    pub page: usize,
    pub auto_advance: bool,                   // true for single-select
    pub review_page: bool,                    // true when questions > 1
    pub countdown_default: Option<Duration>,  // Pro only; rejected unless every question is side-effect free
    pub variant: QuestionVariant,             // Numbered | Paged | Chips | Pro(..)
}
```

## 10. Related components and references
Related: [genui.md](genui.md) (primitives, security rules 4–5) · [genui-renderer.md](genui-renderer.md)
(submitting, held patches) · [approvals.md](approvals.md) and [approval-card.md](approval-card.md) (Question
kind, pager, Credential kind, input guard) · [composer-attachments.md](composer-attachments.md) (file
fields) · [composer-send-control.md](composer-send-control.md) (↑ answers the open card) ·
[permission-rules.md](permission-rules.md) (rules that auto-resolve questions) ·
[genui-skeletons.md](genui-skeletons.md) (form and question skeleton shapes).

| Reference | IDs / note | What we take | Licence note |
|---|---|---|---|
| Beautiful UI paged question approval "1 / 3, Skip, Continue" | [R106], [R133] | `paged` | MIT: port with attribution |
| Agent questions with one option marked "(recommended)"; attention inbox | [R022] | recommended tag, `inbox` | Study only |
| Numbered agent question card "Continue ⌘↵" | [R050] | ⌘↵ continue | Study only |
| "Schedule a call" popover → "Call scheduled" result card | [R047] | `scheduler`, collapse to receipt | Study only |
| Twenty CRM side-panel form, ⌘↵ submit | [R026] | `side-panel` | Study only |
| Draft → Review / Approve and send | [R052] | `draft-review` | Study only |
| "Secrets stay out of the chat" STRIPE_KEY card | [R049] | secure hand-off | Study only |
| LangGraph Agent Inbox accept / edit / respond / ignore; CopilotKit `useHumanInTheLoop` | research note 05 §4 | response model | not recorded: concepts only |
| Thesys C1 forms (inputs, dropdowns, checkboxes, multi-step) | research note 05 §1 | field coverage | not recorded: concepts only |
| affectUI Stepper Form, Date Range Picker (Pro), OTP Input (Free; `autocomplete=one-time-code`) | research note 11 | coverage check | Paid / unclear: study only |
| coss ui form primitives (Base UI) | [R126] | field behaviour on web | MIT: may port with attribution (Dioxus) |

**Gaps.** The secret lexicon and typed-secret heuristics need a false-positive study (e.g. order IDs
that pass Luhn). Whether each adapter supports revisions (rev n+1) to an already-answered interrupt was
not researched; without support, Edit always becomes "Send a change".
