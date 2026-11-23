//! Plan #30 demos: diff viewer, inline diff, change summary, diff table, diff timeline, code
//! suggestion, AI inline edit, image compare, comment thread, review center and review card.
//! Sample code and people are invented.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::diff::{
        Comment, CompareMode, FileDiff, Filter, Finding, Order, Person, ReviewFile, ReviewItem,
        ReviewMode, ReviewStep, Revision, Risk, Severity, Span, SugHunk, SugStatus, Suggestion,
        Thread, table_diff,
    },
    kit::difftlfid::{dt_demo, dt_end},
};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{
    AiInlineEdit, ChangeSummary, CommentThread, DiffTable, DiffTimelineRun, ImageCompare,
    ReviewCard, ReviewCenter,
};
use oxyn_dioxus::{CodeSuggestion, DiffTimeline, DiffViewer, InlineDiff};

use super::{Code, mk, q, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

/// The sample clock runs four hours ahead of the demo base, so items can be an hour old.
const AHEAD: u64 = 4 * 3_600_000;

/// `ms` before the shifted clock.
fn at(ms: u64) -> ClockMs {
    ClockMs(crate::BASE.0 + AHEAD - ms)
}

/// The demo clock shifted by [`AHEAD`].
fn shifted(now: ClockMs) -> ClockMs {
    ClockMs(now.0 + AHEAD)
}

fn status(msg: Signal<String>) -> Element {
    rsx! { span { style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

// ---------- sample change set ----------

const AUTH_OLD: &str = "use crate::store::{find, Session};

/// Signs a user in.
pub fn login(id: u32, pw: &str) -> Result<Session, Denied> {
    let user = find(id)?;
    let ok = check(pw);
    if !ok {
        return Err(Denied);
    }
    Ok(Session::new(user))
}

fn check(pw: &str) -> bool {
    !pw.is_empty()
}

pub fn logout(s: Session) {
    s.end();
}
";

const AUTH_NEW: &str = "use crate::store::{find, Session};
use crate::rate::Limiter;

/// Signs a user in.
pub fn login(id: u32, pw: &str, rl: &Limiter) -> Result<Session, Denied> {
    rl.hit(id)?;
    let user = find(id)?;
    let ok = verify(pw, &user.salt)?;
    if !ok {
        return Err(Denied);
    }
    Ok(Session::new(user))
}

fn check(pw: &str) -> bool {
    !pw.is_empty()
}

pub fn logout(s: Session) {
    s.revoke_tokens();
    s.end();
}
";

const SESSION_OLD: &str = "pub struct Session {
    pub user: User,
    pub started: u64,
}

impl Session {
    pub fn new(user: User) -> Self {
        Self { user, started: now() }
    }

    pub fn end(self) {}
}
";

const SESSION_NEW: &str = "pub struct Session {
    pub user: User,
    pub started: u64,
    pub tokens: Vec<Token>,
}

impl Session {
    pub fn new(user: User) -> Self {
        Self { user, started: now(), tokens: Vec::new() }
    }

    pub fn revoke_tokens(&self) {
        for t in &self.tokens {
            t.revoke();
        }
    }

    pub fn end(self) {}
}
";

const RATE_NEW: &str = "/// Five tries a minute per account.
pub struct Limiter {
    window_ms: u64,
    max: u32,
}

impl Limiter {
    pub fn hit(&self, id: u32) -> Result<(), Denied> {
        if self.count(id) >= self.max {
            return Err(Denied);
        }
        Ok(())
    }
}
";

const LEGACY_OLD: &str = "// Old cookie check, replaced by Session.
pub fn cookie_ok(c: &str) -> bool { c.len() > 8 }
";

fn change_set() -> Vec<FileDiff> {
    vec![
        FileDiff::new("src/auth.rs", AUTH_OLD, AUTH_NEW),
        FileDiff::new("src/session.rs", SESSION_OLD, SESSION_NEW),
        FileDiff::new("src/rate.rs", "", RATE_NEW),
        FileDiff::new("src/legacy.rs", LEGACY_OLD, ""),
    ]
}

// ---------- diff viewer ----------

fn diff_viewer(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let files = if v.s("files") == "one" {
        change_set()[..1].to_vec()
    } else {
        change_set()
    };
    rsx! {
        div { style: "width:100%;max-width:1060px;display:flex;flex-direction:column;gap:6px;--oxyn-diff-max:none",
            DiffViewer { files, mode: ReviewMode::from_id(v.s("mode")), layout: v.string("layout"), change_style: v.string("style"),
                file_list: v.b("list"), word_diff: v.b("words"), wrap: v.b("wrap"), line_numbers: v.b("numbers"), minimap: v.b("minimap"),
                comments: v.b("comments"), focus_dim: v.b("dim"),
                on_decide: move |(h, d): (oxyn_core::kit::diff::HunkRef, oxyn_core::kit::diff::Decision)| msg.set(format!("onDecide(file {}, hunk {}, {})", h.0, h.1, d.id())),
                on_send: move |c: Vec<oxyn_core::kit::diff::LineComment>| msg.set(format!("onSend({} comments)", c.len())),
            }
            {status(msg)}
        }
    }
}

fn diff_viewer_code(v: &Vals) -> String {
    let mut c = Code::new("DiffViewer")
        .pre("let files = vec![")
        .pre("    FileDiff::new(\"src/auth.rs\", old_auth, new_auth),")
        .pre("    FileDiff::new(\"src/rate.rs\", \"\", new_rate),")
        .pre("];")
        .raw("files", "files");
    if v.s("mode") == "applied" {
        c = c.raw("mode", "ReviewMode::Applied");
    }
    c.s("layout", v.s("layout"), "unified")
        .s("change_style", v.s("style"), "background")
        .b("wrap", v.b("wrap"))
        .b("focus_dim", v.b("dim"))
        .raw("on_decide", "move |(hunk, d)| save(hunk, d)")
        .raw("on_send", "move |comments| agent.reply_to(comments)")
        .done()
}

// ---------- inline diff ----------

fn inline_diff(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let (b, a) = match v.s("text") {
        "policy" => (
            "Refunds are available within 14 days of purchase, if the item is unused and you contact support by email.",
            "Refunds are available within 30 days of delivery, if the item is unused. Contact support in the app or by email.",
        ),
        _ => (
            "Our agent is very careful with your data and it never shares it with anyone without asking you first.",
            "Our agent is careful with your data and never shares it without asking you first.",
        ),
    };
    if v.s("form") == "review" {
        let stage = v.s("stage");
        let decided = match stage {
            "partly" => vec![Some(true), Some(false)],
            "done" => vec![Some(true); 8],
            _ => vec![],
        };
        let style = if v.s("style") == "classic" {
            "classic"
        } else {
            ""
        }
        .to_string();
        let key = format!("{}{}{}", v.s("text"), stage, v.s("style"));
        return rsx! {
            div { key: "{key}", "data-k": "{key}", style: "display:flex;flex-direction:column;gap:8px;width:440px;max-width:100%;padding-top:34px",
                InlineDiff { before: b.to_string(), after: a.to_string(), review: true, decided, style,
                    on_change: move |t: String| msg.set(format!("onChange({} chars)", t.len())),
                    on_done: move |t: String| msg.set(format!("onDone({} chars)", t.len())) }
                span { style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" }
            }
        };
    }
    rsx! { InlineDiff { before: b.to_string(), after: a.to_string(), form: v.string("form"), counts: v.b("counts") } }
}

fn inline_diff_code(v: &Vals) -> String {
    Code::new("InlineDiff")
        .raw("before", q("Our agent is very careful with your data…"))
        .raw("after", q("Our agent is careful with your data…"))
        .s("form", v.s("form"), "inline")
        .done()
}

// ---------- diff timeline ----------

const R1: &str = "export async function fetchInvoice(id: string) {\n  const res = await fetch(`/api/invoices/${id}`);\n  return res.json();\n}\n";
const R2: &str = "export async function fetchInvoice(id: string) {\n  const res = await fetch(`/api/invoices/${id}`);\n  if (!res.ok) throw new Error(`Invoice ${id} failed`);\n  return res.json();\n}\n";
const R3: &str = "export async function fetchInvoice(id: string, signal?: AbortSignal) {\n  const res = await fetch(`/api/invoices/${id}`, { signal });\n  if (!res.ok) throw new Error(`Invoice ${id} failed: ${res.status}`);\n  return res.json();\n}\n";
const R4: &str = "export async function fetchInvoice(id: string, signal?: AbortSignal) {\n  for (let attempt = 0; attempt < 3; attempt++) {\n    const res = await fetch(`/api/invoices/${id}`, { signal });\n    if (res.ok) return res.json();\n    if (res.status < 500) throw new Error(`Invoice ${id} failed: ${res.status}`);\n    await sleep(2 ** attempt * 250);\n  }\n  throw new Error(`Invoice ${id} kept failing`);\n}\n";

fn revisions() -> Vec<Revision> {
    let r = |id: &str, label: &str, who: &str, ago: u64, text: &str| Revision {
        id: id.into(),
        label: label.into(),
        who: who.into(),
        at: at(ago),
        text: text.into(),
    };
    vec![
        r("r1", "First draft", "Agent", 52 * 60_000, R1),
        r("r2", "Throw on errors", "Agent", 31 * 60_000, R2),
        r("r3", "Abort signal", "You", 12 * 60_000, R3),
        r("r4", "Retry with backoff", "Agent", 2 * 60_000, R4),
    ]
}

fn diff_timeline(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    #[cfg(oxyn_pro)]
    if v.s("view") == "run" {
        let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
        let state = v.s("state");
        let at = match state {
            "live" => None,
            "early" => Some(47_000),
            "mid" | "open" => Some(126_000),
            _ => Some(dt_end(&dt_demo())),
        };
        let open = (state == "open").then_some(7);
        let style = if v.s("style") == "classic" {
            "classic".to_string()
        } else {
            String::new()
        };
        let w = if v.s("width") == "narrow" {
            "360px"
        } else {
            "460px"
        };
        return rsx! {
            div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:8px",
                DiffTimelineRun { now, at, open, style,
                    on_open: move |i: Option<usize>| msg.set(format!("onOpen({i:?})")) }
                {status(msg)}
            }
        };
    }
    rsx! {
        div { style: "width:100%;max-width:960px;display:flex;flex-direction:column;gap:6px;--oxyn-diff-max:none",
            for k in [v.string("layout")] {
                DiffTimeline { key: "{k}", path: "src/billing/fetch-invoice.ts".to_string(), revisions: revisions(), now: shifted(now), layout: v.string("layout"),
                    on_range: move |(a, b): (usize, usize)| msg.set(format!("onRange({a}, {b})")) }
            }
            {status(msg)}
        }
    }
}

fn diff_timeline_code(v: &Vals) -> String {
    if v.s("view") == "run" {
        return Code::new("DiffTimelineRun")
            .raw("edits", "Some(run.edits())")
            .raw("at", "Some(run.elapsed_ms())")
            .raw("agent", q("UI agent"))
            .raw("context", q("PR #57"))
            .raw("now", "clock.now()")
            .done();
    }
    Code::new("DiffTimeline")
        .raw("path", q("src/billing/fetch-invoice.ts"))
        .raw("revisions", "history.revisions()")
        .raw("now", "clock.now()")
        .s("layout", v.s("layout"), "unified")
        .done()
}

// ---------- code suggestion ----------

const INVOICE: &str = "import type { Invoice } from \"./types\";

export async function fetchInvoice(id: string): Promise<Invoice> {
  const res = await fetch(`/api/invoices/${id}`);
  if (!res.ok) throw new Error(`Invoice ${id} failed: ${res.status}`);
  return res.json();
}";

fn suggestion() -> Suggestion {
    let h = |id: &str, start, remove, insert: &[&str], summary: &str| SugHunk {
        id: id.into(),
        start,
        remove,
        insert: insert.iter().map(|s| s.to_string()).collect(),
        summary: summary.into(),
    };
    Suggestion::new(
        INVOICE,
        vec![
            h(
                "import",
                1,
                0,
                &["import { sleep } from \"./time\";"],
                "Import sleep",
            ),
            h(
                "retry",
                3,
                3,
                &[
                    "  for (let attempt = 0; attempt < 3; attempt++) {",
                    "    const res = await fetch(`/api/invoices/${id}`);",
                    "    if (res.ok) return res.json();",
                    "    if (res.status < 500 && res.status !== 429) break;",
                    "    await sleep(2 ** attempt * 250);",
                    "  }",
                ],
                "Retry 429 and 5xx with backoff",
            ),
            h(
                "throw",
                6,
                0,
                &["  throw new Error(`Invoice ${id} kept failing`);"],
                "Fail after the last try",
            ),
        ],
    )
}

fn code_suggestion(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let mut start = use_signal(|| Some(crate::BASE));
    let forced = match v.s("status") {
        "error" => Some(SugStatus::Error),
        "thinking" => Some(SugStatus::Thinking),
        _ => None,
    };
    let play = v.b("play");
    rsx! {
        div { style: "width:100%;max-width:720px;display:flex;flex-direction:column;gap:6px",
            for k in [format!("{:?}", start())] {
                CodeSuggestion { key: "{k}", suggestion: suggestion(), now, started: if play { start() } else { None }, status: forced,
                    file_path: if v.b("path") { Some("src/billing".to_string()) } else { None },
                    model: if v.b("model") { "Claude Sonnet".to_string() } else { String::new() },
                    summaries: v.b("summaries"), max_code_height: v.u("cap") as u32,
                    on_decision: move |(id, yes, _): (String, bool, String)| msg.set(format!("onDecision({id}, {})", if yes { "accepted" } else { "rejected" })),
                    on_resolve: move |_| msg.set("onResolve(code)".into()),
                    on_stop: move |_| msg.set("onStop()".into()),
                    on_regenerate: move |_| { msg.set("onRegenerate()".into()); start.set(Some(now)); },
                }
            }
            {status(msg)}
        }
    }
}

fn code_suggestion_code(v: &Vals) -> String {
    let mut c = Code::new("CodeSuggestion")
        .pre("let s = Suggestion::new(&file_text, vec![")
        .pre("    SugHunk { id: \"retry\".into(), start: 3, remove: 3, insert: retry_lines, summary: \"Retry 429 and 5xx\".into() },")
        .pre("]);")
        .raw("suggestion", "s")
        .raw("now", "clock.now()");
    if v.b("play") {
        c = c.raw("started", "Some(requested_at)");
    }
    c.raw("instruction", q("Add retry with backoff"))
        .raw("on_resolve", "move |code| write_file(code)")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![
        Demo {
            controls: vec![
                sel(
                    "layout",
                    "Layout",
                    &["unified", "split", "stacked"],
                    "unified",
                ),
                sel("mode", "Mode", &["proposed", "applied"], "proposed"),
                sel(
                    "style",
                    "Change style",
                    &["background", "markers", "bars"],
                    "background",
                ),
                sel("files", "Files", &["four", "one"], "four"),
                tog("list", "File list", true),
                tog("words", "Word diff", true),
                tog("numbers", "Line numbers", true),
                tog("minimap", "Minimap", true),
                tog("comments", "Line comments", true),
                tog("wrap", "Wrap", true),
                tog("dim", "Focus dim", false),
            ],
            variants: vec![
                var(
                    "unified",
                    "One column with two gutters, file list, minimap and the review bar.",
                    &[],
                ),
                var(
                    "split",
                    "Old on the left, new on the right; hatched fillers for unpaired rows.",
                    &[("layout", "split"), ("minimap", "false")],
                ),
                var(
                    "stacked",
                    "A Before block over an After block per hunk, for narrow panels.",
                    &[("layout", "stacked"), ("list", "false")],
                ),
                var(
                    "applied",
                    "Edits already on disk: Keep / Undo instead of Accept / Reject.",
                    &[("mode", "applied")],
                ),
                var(
                    "bar-style",
                    "Stripe-only change marks for dense IDE panels.",
                    &[("style", "bars"), ("minimap", "false")],
                ),
                var(
                    "focus-dim",
                    "Only the current hunk is at full strength.",
                    &[("dim", "true"), ("files", "one"), ("list", "false")],
                ),
            ],
            render: diff_viewer,
            code: diff_viewer_code,
            keyboard: vec![
                ("j / k", "Next / previous hunk"),
                ("[ / ]", "Previous / next file"),
                ("y / x", "Accept (Keep) / Reject (Undo) the current hunk"),
                ("c", "Comment on the current hunk; ⌘↵ adds it, Esc closes"),
                ("z", "Undo the last decision"),
                ("⌘↵", "Send every draft comment to the agent"),
            ],
            a11y: vec![
                "Every changed line starts with a hidden “Added:” or “Removed:”, so the diff never relies on the tint; the + and − signs carry it on screen too.",
                "Hunk buttons name their hunk (“Accept fn login”) and carry aria-keyshortcuts; each decision updates a polite status line that says how many hunks remain.",
                "The layout switch is a radio group; Viewed is a real checkbox; the file list marks the current file with aria-current.",
                "Reduced motion drops the chip pop, row fades and the smooth scroll to the next hunk.",
            ],
            responsive: vec![
                "Container query: under 640 px the file list becomes a sideways strip, the minimap hides and comments lose their indent.",
                "Long lines scroll inside the diff (or wrap with Wrap); the file header stays pinned while you scroll a file.",
            ],
            height: 620,
            align: Align::Start,
            ..mk(
                "diff-viewer",
                "Diff viewer",
                "tools",
                "Review an agent's change set: unified, split or stacked, Accept / Reject (or Keep / Undo) per hunk, per file and for all with undo, a file list with Viewed marks, hidden context on demand, line comments sent to the agent together, and a minimap.",
                "DiffViewer",
                "DiffViewerEl",
                "diff-viewer",
                "kit/diff.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "form",
                    "Form",
                    &["inline", "lines", "after-only", "review"],
                    "inline",
                ),
                sel("text", "Text", &["privacy", "policy"], "privacy"),
                tog("counts", "Word counts", true),
                sel("stage", "Review", &["pending", "partly", "done"], "pending"),
                sel("style", "Style", &["clean", "classic"], "clean"),
            ],
            variants: vec![
                var(
                    "redline",
                    "Struck deletions and underlined insertions in one paragraph.",
                    &[],
                ),
                var(
                    "lines",
                    "A Before line over an After line.",
                    &[("form", "lines"), ("text", "policy")],
                ),
                var(
                    "after-only",
                    "Only the new words, highlighted where they changed.",
                    &[("form", "after-only")],
                ),
                var(
                    "review",
                    "Review mode: each change is a suggestion; the highlighted one carries Accept (Y) / Reject (N) above it, with prev / next, Reject all and Accept all below.",
                    &[("form", "review"), ("text", "policy")],
                ),
                var(
                    "review-partly",
                    "Two decided: the accepted wording reads as plain text, the rejected one is back to the original, the count drops and Undo appears.",
                    &[("form", "review"), ("text", "policy"), ("stage", "partly")],
                ),
                var(
                    "review-done",
                    "Everything decided: a green check, All changes reviewed and Undo.",
                    &[("form", "review"), ("stage", "done")],
                ),
                var(
                    "review-classic",
                    "Classic review: the buttons sit above the text and changes are colour plus strike or underline, no tint.",
                    &[("form", "review"), ("text", "policy"), ("style", "classic")],
                ),
            ],
            render: inline_diff,
            code: inline_diff_code,
            keyboard: vec![
                (
                    "Tab",
                    "Read-only forms: no stops. Review: each suggestion, then the footer buttons",
                ),
                (
                    "Y / N",
                    "Review: accept / reject the highlighted suggestion",
                ),
                ("Enter", "Review: highlight the focused suggestion"),
            ],
            a11y: vec![
                "Deletions and insertions are real del and ins elements, so screen readers announce them; colour is paired with strike-through and underline.",
                "In review every change is its own button whose name gives its place, the words it swaps and the Y / N keys; each decision is read out by a polite status line.",
                "Neighbouring single-word changes merge into phrases, so the paragraph reads as a few edits, not confetti.",
            ],
            responsive: vec!["Wraps like any paragraph; fills its column up to 640 px."],
            align: Align::Start,
            height: 260,
            ..mk(
                "inline-diff",
                "Inline diff",
                "data-display",
                "A word-level diff for prose: removed words struck through, added words underlined, as one redline paragraph, a before line over an after line, or the new text alone.",
                "InlineDiff",
                "InlineDiffEl",
                "inline-diff",
                "kit/diff.rs",
            )
        },
        Demo {
            controls: vec![
                sel("view", "View", &["run", "revisions"], "run"),
                sel("style", "Style", &["clean", "classic"], "clean"),
                sel(
                    "state",
                    "Run state",
                    &["live", "early", "mid", "done", "open"],
                    "live",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
                sel(
                    "layout",
                    "Diff layout",
                    &["unified", "split", "stacked"],
                    "unified",
                ),
            ],
            variants: vec![
                var(
                    "run",
                    "The run replays and loops: files join the list as the agent first touches them, bars grow piece by piece, the green and red lines climb with breathing ends, then the status reads Done.",
                    &[],
                ),
                var(
                    "early",
                    "Under a minute in: four edits to three files, the row being edited washed grey.",
                    &[("state", "early")],
                ),
                var(
                    "done",
                    "Finished: fifteen edits in 3:22 across seven files, the image listed as binary, the net in the legend.",
                    &[("state", "done")],
                ),
                var(
                    "open",
                    "An edit picked on the chart: its file and time, running totals with this edit's share, the changed lines; other rows fade.",
                    &[("state", "open")],
                ),
                var(
                    "narrow",
                    "Under about 410 px the totals drop under the name, folders hide and big counts shorten.",
                    &[("state", "done"), ("width", "narrow")],
                ),
                var(
                    "classic",
                    "Classic style: a grey tray holding the chart and the file list as two panels.",
                    &[("style", "classic"), ("state", "mid")],
                ),
                var(
                    "rail",
                    "Revisions on a rail; the last two compared.",
                    &[("view", "revisions")],
                ),
                var(
                    "stacked",
                    "Before over after, for narrow panels.",
                    &[("view", "revisions"), ("layout", "stacked")],
                ),
            ],
            render: diff_timeline,
            code: diff_timeline_code,
            keyboard: vec![
                ("Tab", "Run view: the chart is one stop"),
                (
                    "← / → (run)",
                    "Step to the previous / next edit and open it",
                ),
                ("Home / End (run)", "First / last edit"),
                ("Escape (run)", "Closes the open edit"),
                ("← / →", "Move the compare target"),
                ("Shift + ← / →", "Move the base"),
                ("Home / End", "First base / latest revision"),
            ],
            a11y: vec![
                "Run view: the chart takes a single tab stop and is named with its edit count; an opened edit is announced politely, and the line under the name is a status region.",
                "Each file row reads its path, lines added and removed and how many edits, so the bar colours are never the only signal; reduced motion stops the breathing, the rise and the growing bars.",
                "The rail is a listbox; each revision says who wrote it, when and its +/− counts.",
                "Your revisions draw a square dot and the agent's a round one, so author is not colour alone.",
            ],
            responsive: vec![
                "Run view: the card fills its container up to 440 px; under about 410 px the totals move under the name, folders hide and counts shorten (4.8k); more than seven files scroll inside the list.",
                "The rail scrolls sideways when there are more revisions than room.",
            ],
            height: 560,
            live: true,
            align: Align::Start,
            ..mk(
                "diff-timeline",
                "Diff timeline",
                "tools",
                "An agent's edits over a run as lines added and removed against time, with a row per file and any edit opened to its changed lines; or one file across revisions on a rail, any two compared.",
                "DiffTimeline",
                "DiffTimelineEl",
                "diff-timeline",
                "kit/diff.rs",
            )
        },
        Demo {
            controls: vec![
                tog("play", "Stream in", true),
                sel("status", "Status", &["live", "thinking", "error"], "live"),
                tog("model", "Model line", true),
                tog("path", "Folder", true),
                tog("summaries", "Hunk summaries", true),
                rng("cap", "Max code height", 0.0, 400.0, 20.0, "0"),
            ],
            variants: vec![
                var(
                    "stream",
                    "Reading shimmer, then hunks stream in and get Accept / Reject.",
                    &[],
                ),
                var(
                    "ready",
                    "The finished suggestion, ready to review.",
                    &[("play", "false")],
                ),
                var(
                    "error",
                    "The model stopped; Retry.",
                    &[("status", "error"), ("play", "false")],
                ),
                var(
                    "capped",
                    "Code area capped at 220 px.",
                    &[("cap", "220"), ("play", "false")],
                ),
            ],
            render: code_suggestion,
            code: code_suggestion_code,
            keyboard: vec![
                (
                    "Tab",
                    "Code area, then each hunk's Reject and Accept, then the footer",
                ),
                ("Enter / Space", "Press the focused button"),
                ("⌘↵ / Ctrl+Enter", "Accept every pending hunk"),
                ("⌘⌫ / Ctrl+Backspace", "Reject every pending hunk"),
            ],
            a11y: vec![
                "The card is a labelled group; the code is a focusable region named after the file.",
                "Removed and added lines start with hidden “Removed:” / “Added:”; Accept and Reject name their hunk.",
                "A polite status speaks the state (“3 changes to review”), each decision and the result; streaming is announced once.",
            ],
            responsive: vec![
                "Fills its container up to 720 px; long lines scroll sideways while line numbers and review bars stay pinned.",
                "The folder hides below a 520 px card; below 440 px summaries hide and the footer stacks.",
            ],
            align: Align::Start,
            height: 520,
            live: true,
            ..mk(
                "code-suggestion",
                "Code suggestion",
                "tools",
                "An AI edit for one file: the instruction and model, a file tab with a running +/− count, hunks that stream in after a short Reading shimmer, Accept / Reject under each, and Accept all with ⌘↵.",
                "CodeSuggestion",
                "CodeSuggestionEl",
                "code-suggestion",
                "kit/diff.rs",
            )
        },
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 11] = [
    "diff-viewer",
    "inline-diff",
    "change-summary",
    "diff-table",
    "diff-timeline",
    "code-suggestion",
    "ai-inline-edit",
    "image-compare",
    "threaded-comments",
    "review-center",
    "review-card",
];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
