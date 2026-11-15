//! Plan #30 demos: diff viewer, inline diff, change summary, diff table, diff timeline, code
//! suggestion, AI inline edit, image compare, comment thread, review center and review card.
//! Sample code and people are invented.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::diff::{
        Comment, CompareMode, FileDiff, Filter, Finding, Order, Person, ReviewFile, ReviewItem,
        ReviewMode, ReviewStep, Revision, Risk, Severity, Span, SugHunk, SugStatus, Suggestion,
        Thread, table_diff,
    },
};
use oxyn_dioxus::{
    AiInlineEdit, ChangeSummary, CodeSuggestion, CommentThread, DiffTable, DiffTimeline,
    DiffViewer, ImageCompare, InlineDiff, ReviewCard, ReviewCenter,
};

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
                on_decide: move |(h, d): (oxyn_core::parity::diff::HunkRef, oxyn_core::parity::diff::Decision)| msg.set(format!("onDecide(file {}, hunk {}, {})", h.0, h.1, d.id())),
                on_send: move |c: Vec<oxyn_core::parity::diff::LineComment>| msg.set(format!("onSend({} comments)", c.len())),
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
    rsx! { InlineDiff { before: b.to_string(), after: a.to_string(), form: v.string("form"), counts: v.b("counts") } }
}

fn inline_diff_code(v: &Vals) -> String {
    Code::new("InlineDiff")
        .raw("before", q("Our agent is very careful with your data…"))
        .raw("after", q("Our agent is careful with your data…"))
        .s("form", v.s("form"), "inline")
        .done()
}

// ---------- change summary ----------

fn change_summary(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let blocks = v.u("blocks");
    rsx! {
        div { style: "width:100%;max-width:560px;display:flex;flex-direction:column;gap:6px",
            for k in [format!("{}{}", v.b("compact"), blocks)] {
                ChangeSummary { key: "{k}", files: change_set(), compact: v.b("compact"), blocks, on_open: move |p: String| msg.set(format!("onOpen({p})")) }
            }
            {status(msg)}
        }
    }
}

fn change_summary_code(v: &Vals) -> String {
    Code::new("ChangeSummary")
        .raw("files", "change_set.files.clone()")
        .b("compact", v.b("compact"))
        .raw("on_open", "move |path| open_diff(path)")
        .done()
}

// ---------- diff table ----------

fn diff_table(v: &Vals, _: ClockMs) -> Element {
    let rows = table_diff(
        &[
            ("model", "sonnet-4"),
            ("max_tokens", "4096"),
            ("temperature", "0.7"),
            ("retries", "2"),
            ("region", "eu-west"),
            ("tools.web", "off"),
        ],
        &[
            ("model", "sonnet-5"),
            ("max_tokens", "4096"),
            ("temperature", "0.2"),
            ("retries", "5"),
            ("timeout_s", "30"),
            ("tools.web", "off"),
            ("tools.shell", "ask"),
        ],
    );
    rsx! {
        for k in [format!("{}{}", v.b("only"), v.b("cols"))] {
            DiffTable { key: "{k}", rows: rows.clone(), key_label: "Setting".to_string(), changed_only: v.b("only"), columns: v.b("cols") }
        }
    }
}

fn diff_table_code(v: &Vals) -> String {
    Code::new("DiffTable")
        .pre("let rows = table_diff(&before, &after);")
        .raw("rows", "rows")
        .raw("key_label", q("Setting"))
        .b("columns", v.b("cols"))
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

// ---------- AI inline edit ----------

fn doc() -> Vec<String> {
    vec![
        "The launch is basically ready and we will really ship it to everyone in order to get feedback early. Please note that users must opt in before teh beta starts.".into(),
        "Our agent is very careful with your data. It's going to ask before it sends anything and we'll keep alot of logs so you can see what it did.".into(),
        "The rollout starts with internal teams, then design partners, then the waitlist".into(),
    ]
}

fn inline_edit(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    rsx! {
        div { style: "width:100%;max-width:680px;display:flex;flex-direction:column;gap:6px",
            AiInlineEdit { paragraphs: doc(), now, speed: v.f("speed") as f32, ghost: v.b("ghost"), title: v.string("title"),
                on_change: move |d: Vec<String>| msg.set(format!("onChange({} paragraphs)", d.len())) }
            {status(msg)}
        }
    }
}

fn inline_edit_code(v: &Vals) -> String {
    Code::new("AiInlineEdit")
        .raw("paragraphs", "doc.paragraphs.clone()")
        .raw("now", "clock.now()")
        .s("title", v.s("title"), "Launch plan")
        .b("ghost", v.b("ghost"))
        .raw("on_change", "move |doc| save(doc)")
        .done()
}

// ---------- image compare ----------

fn scene(after: bool) -> Element {
    // An invented landscape: the "after" is the retouched version (warmer sky, lifted shadows,
    // the stray pole removed).
    let (sky, hill, front, sun) = if after {
        ("#cfe0f5", "#7aa874", "#4d7d52", "#ffd36a")
    } else {
        ("#9aa4ad", "#5f6f5c", "#323d33", "#d9cfa8")
    };
    rsx! {
        svg { view_box: "0 0 600 400", preserve_aspect_ratio: "xMidYMid slice", role: "img",
            "aria-label": if after { "Retouched photo of hills" } else { "Unedited photo of hills" },
            rect { width: "600", height: "400", fill: "{sky}" }
            circle { cx: "450", cy: "110", r: "46", fill: "{sun}" }
            path { d: "M0 260 Q150 170 300 230 T600 210 V400 H0Z", fill: "{hill}" }
            path { d: "M0 320 Q200 260 380 310 T600 300 V400 H0Z", fill: "{front}" }
            if !after { rect { x: "140", y: "150", width: "6", height: "170", fill: "#2a2a2a" } }
        }
    }
}

fn image_compare(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let labels = if v.b("labels") {
        Some(("Before".to_string(), "After".to_string()))
    } else {
        None
    };
    rsx! {
        div { style: "width:100%;max-width:720px;display:flex;flex-direction:column;gap:6px",
            for k in [format!("{}{}{}", v.s("mode"), v.s("orient"), v.s("pos"))] {
                ImageCompare { key: "{k}", before: scene(false), after: scene(true), orientation: v.string("orient"), labels: labels.clone(),
                    default_position: v.f("pos") as f32, aspect_ratio: v.string("ratio"), mode: CompareMode::from_id(v.s("mode")),
                    on_position_change: move |p: f32| msg.set(format!("onPositionChange({p:.0})")) }
            }
            {status(msg)}
        }
    }
}

fn image_compare_code(v: &Vals) -> String {
    let mut c = Code::new("ImageCompare")
        .raw(
            "before",
            "rsx! { img { src: \"/raw.jpg\", alt: \"Unedited photo\" } }",
        )
        .raw(
            "after",
            "rsx! { img { src: \"/edit.jpg\", alt: \"Retouched photo\" } }",
        )
        .s("orientation", v.s("orient"), "horizontal")
        .s("aspect_ratio", v.s("ratio"), "3 / 2");
    if v.s("mode") != "slider" {
        c = c.raw("mode", format!("CompareMode::from_id({})", q(v.s("mode"))));
    }
    c.done()
}

// ---------- comment thread ----------

fn people() -> (Person, Vec<Person>) {
    let you = Person::new("you", "You");
    (
        you.clone(),
        vec![
            Person::new("mira", "Mira Sol"),
            Person::new("theo", "Theo Park"),
            Person::agent("agent", "Docs agent"),
            Person::new("ines", "Ines Moreau"),
            you,
        ],
    )
}

fn thread() -> Thread {
    let (you, p) = people();
    let (mira, theo, agent) = (p[0].clone(), p[1].clone(), p[2].clone());
    Thread::new(vec![
        Comment::new("c1", mira.clone(), "Can we try a shorter headline here? @Theo Park had a version that fit on one line on phones.", "2h")
            .react("👍", &["theo", "you"])
            .replies(vec![
                Comment::new("c2", theo, "Yes: “Plan less. Ship more.” Dropping it into the file now.", "1h").react("🎉", &["mira"]),
                Comment::new("c3", agent, "I checked it at 390 px: it fits on one line and saves the second line on mobile.", "48m"),
            ]),
        Comment::new("c4", you, "Once this lands I'll update the social image to match. @Mira Sol can you check the click-through after?", "30m")
            .react("👀", &["mira"]),
    ])
}

fn comment_thread(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let (me, pp) = people();
    let mut t = thread();
    if v.s("set") == "empty" {
        t = Thread::new(vec![]);
    }
    rsx! {
        div { style: "width:100%;max-width:560px;display:flex;flex-direction:column;gap:6px",
            for k in [format!("{}{}{}", v.s("set"), v.b("resolved"), v.s("depth"))] {
                CommentThread { key: "{k}", thread: t.clone(), me: me.clone(), people: pp.clone(), title: Some(v.string("title")), default_resolved: v.b("resolved"), max_depth: v.u("depth"),
                    on_change: move |(_, e): (Thread, oxyn_core::parity::diff::ThreadEvent)| msg.set(format!("onChange({e:?})")),
                    on_resolved: move |r: bool| msg.set(format!("onResolvedChange({r})")) }
            }
            {status(msg)}
        }
    }
}

fn comment_thread_code(v: &Vals) -> String {
    Code::new("CommentThread")
        .raw("thread", "Thread::new(comments)")
        .raw("me", "Person::new(\"you\", \"You\")")
        .raw("title", format!("Some({}.into())", q(v.s("title"))))
        .b("default_resolved", v.b("resolved"))
        .raw("on_change", "move |(thread, event)| sync(event)")
        .done()
}

// ---------- review center ----------

fn review_items() -> Vec<ReviewItem> {
    let mut refund = ReviewItem::new(
        "refund-4821",
        "Refund order #4821",
        "Support agent",
        "Refund",
        Risk::High,
        0.62,
        at(26 * 60_000),
    );
    refund.model = Some("Sonnet".into());
    refund.reasoning =
        Some("The blender arrived cracked; damaged items are covered within 30 days.".into());
    refund.fields = vec![
        (
            "Amount".into(),
            vec![Span::flag(
                "$184.00",
                "Includes $20 express shipping, which the policy may not cover.",
                Some("Refund policy §4"),
            )],
        ),
        (
            "Order".into(),
            vec![Span::t("#4821 · delivered 6 days ago")],
        ),
    ];
    refund.body_label = Some("Reply to the customer".into());
    refund.body = vec![
        Span::t("Hi, I'm sorry your blender arrived cracked. I've refunded "),
        Span::flag("the full $184.00", "Full amount includes shipping.", None),
        Span::t(" to your card; it should show within 3–5 days."),
    ];

    let mut code = ReviewItem::new(
        "pr-311",
        "Rate-limit sign-in attempts",
        "Code agent",
        "Code change",
        Risk::Medium,
        0.81,
        at(41 * 60_000),
    );
    code.reasoning =
        Some("Brute-force attempts spiked last week; five tries a minute per account.".into());
    code.files = vec![ReviewFile {
        path: "src/auth.rs".into(),
        diff: FileDiff::with_context("src/auth.rs", AUTH_OLD, AUTH_NEW, 1),
    }];

    let mut plan = ReviewItem::new(
        "migrate-7",
        "Migrate billing tables",
        "Ops agent",
        "Plan",
        Risk::High,
        0.74,
        at(8 * 60_000),
    );
    plan.reasoning = Some("Moves invoices to the new schema with a dual-write window.".into());
    plan.steps = vec![
        ReviewStep {
            title: "Snapshot the billing database".into(),
            tool: Some("db.snapshot".into()),
            flagged: false,
        },
        ReviewStep {
            title: "Create the new tables".into(),
            tool: Some("db.migrate".into()),
            flagged: false,
        },
        ReviewStep {
            title: "Dual-write for 24 hours".into(),
            tool: None,
            flagged: true,
        },
        ReviewStep {
            title: "Drop the old tables".into(),
            tool: Some("db.drop".into()),
            flagged: true,
        },
    ];

    let mut mail = ReviewItem::new(
        "mail-55",
        "Send the November product update",
        "Growth agent",
        "Email",
        Risk::Low,
        0.93,
        at(3 * 60_000),
    );
    mail.body_label = Some("Email".into());
    mail.body = vec![Span::t(
        "This month: faster search, a new review center and dark mode for everyone.",
    )];

    let mut tag = ReviewItem::new(
        "tag-9",
        "Tag 120 tickets as billing",
        "Triage agent",
        "Bulk edit",
        Risk::Low,
        0.66,
        at(19 * 60_000),
    );
    tag.reasoning = Some("Matched on words like invoice, charge and refund.".into());
    tag.body = vec![
        Span::t("Tag 120 open tickets "),
        Span::flag(
            "as billing",
            "18 of them also mention login problems.",
            None,
        ),
        Span::t("."),
    ];
    vec![refund, code, plan, mail, tag]
}

fn review_center(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let sla = (v.f("sla") as u64).max(1) * 60_000;
    rsx! {
        div { style: "width:100%;max-width:980px;display:flex;flex-direction:column;gap:6px",
            for k in [format!("{}{}{}{}", v.s("order"), v.s("filter"), v.s("sla"), v.b("batch"))] {
                ReviewCenter { key: "{k}", items: review_items(), now: shifted(now), reviewed: 24, sla, order: Order::from_id(v.s("order")), default_filter: Filter::from_id(v.s("filter")), batch_low: v.b("batch"),
                    on_decide: move |(id, d): (String, oxyn_core::parity::diff::Verdict)| msg.set(format!("onDecide({id}, {d:?})")),
                    on_undo: move |id: String| msg.set(format!("onUndo({id})")),
                    on_skip: move |id: String| msg.set(format!("onSkip({id})")) }
            }
            {status(msg)}
        }
    }
}

fn review_center_code(v: &Vals) -> String {
    let mut c = Code::new("ReviewCenter")
        .raw("items", "queue.items()")
        .raw("now", "clock.now()")
        .raw("reviewed", "24");
    if v.s("order") != "risk" {
        c = c.raw("order", format!("Order::from_id({})", q(v.s("order"))));
    }
    c.b("batch_low", v.b("batch"))
        .raw("on_decide", "move |(id, verdict)| save(id, verdict)")
        .raw("on_undo", "move |id| revert(id)")
        .done()
}

// ---------- review card ----------

fn findings() -> Vec<Finding> {
    vec![
        Finding::new("f1", Severity::Blocker, "src/auth.rs", 8, "verify() errors are swallowed")
            .body("The ? on verify turns a store outage into a generic error, so locked-out users see “wrong password”.")
            .fix("let ok = verify(pw, &user.salt)?;", "let ok = verify(pw, &user.salt).map_err(AuthError::Store)?;"),
        Finding::new("f2", Severity::Major, "src/rate.rs", 9, "Limiter never resets its window")
            .body("count() reads hits since start; add the window so a user is not locked out forever."),
        Finding::new("f3", Severity::Minor, "src/session.rs", 14, "revoke_tokens takes &self but mutates")
            .fix("pub fn revoke_tokens(&self) {", "pub fn revoke_tokens(&mut self) {"),
        Finding::new("f4", Severity::Nit, "src/auth.rs", 2, "Unused import after the change"),
    ]
}

fn review_card(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    rsx! {
        div { style: "width:100%;max-width:600px;display:flex;flex-direction:column;gap:6px",
            for k in [format!("{}{}", v.b("nits"), v.b("first"))] {
                ReviewCard { key: "{k}", findings: findings(), subject: "PR #311 · 4 files".to_string(), hide_nits: !v.b("nits"), expand_first: v.b("first"),
                    on_change: move |(id, s): (String, oxyn_core::parity::diff::FindingState)| msg.set(format!("onChange({id}, {s:?})")) }
            }
            {status(msg)}
        }
    }
}

fn review_card_code(v: &Vals) -> String {
    Code::new("ReviewCard")
        .raw("findings", "review.findings()")
        .raw("subject", q("PR #311 · 4 files"))
        .b("hide_nits", !v.b("nits"))
        .raw("on_change", "move |(id, state)| sync(id, state)")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
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
                "Hunk buttons name their hunk (“Accept fn login”) and carry aria-keyshortcuts; decisions are announced in a polite live region with the count left.",
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
                "parity/diff.rs",
            )
        },
        Demo {
            controls: vec![
                sel("form", "Form", &["inline", "lines", "after-only"], "inline"),
                sel("text", "Text", &["privacy", "policy"], "privacy"),
                tog("counts", "Word counts", true),
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
            ],
            render: inline_diff,
            code: inline_diff_code,
            keyboard: vec![("Tab", "No stops of its own; it is read as text")],
            a11y: vec![
                "Deletions and insertions are real del and ins elements, so screen readers announce them; colour is paired with strike-through and underline.",
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
                "parity/diff.rs",
            )
        },
        Demo {
            controls: vec![
                tog("compact", "Start folded", false),
                rng("blocks", "Blocks", 3.0, 10.0, 1.0, "5"),
            ],
            variants: vec![
                var("expanded", "Headline, block bar and a row per file.", &[]),
                var(
                    "compact",
                    "Folded to the headline for a chat message.",
                    &[("compact", "true")],
                ),
            ],
            render: change_summary,
            code: change_summary_code,
            keyboard: vec![("Enter / Space", "Fold or unfold; on a row, open that file")],
            a11y: vec![
                "The headline is a button with aria-expanded; status letters have the word as a title and the block bar is aria-hidden.",
                "Counts are text (+58 −21), so the colour is never the only signal.",
            ],
            responsive: vec!["Paths ellipsize from the end; the bar keeps its size."],
            height: 300,
            align: Align::Start,
            ..mk(
                "change-summary",
                "Change summary",
                "tools",
                "What an agent changed at a glance: “4 files changed +58 −21”, a block bar, and a row per file with its status and counts that opens the file.",
                "ChangeSummary",
                "ChangeSummaryEl",
                "change-summary",
                "parity/diff.rs",
            )
        },
        Demo {
            controls: vec![
                tog("only", "Changed only", true),
                tog("cols", "Before column", true),
            ],
            variants: vec![
                var("columns", "Before and After columns.", &[]),
                var(
                    "inline",
                    "One value column with the old value struck in front.",
                    &[("cols", "false")],
                ),
                var(
                    "all-rows",
                    "Unchanged rows shown, muted.",
                    &[("only", "false")],
                ),
            ],
            render: diff_table,
            code: diff_table_code,
            keyboard: vec![
                ("Tab", "Focus the table to scroll it sideways"),
                ("Space", "Toggle Changed only when focused"),
            ],
            a11y: vec![
                "A real table with a row header per key; each row's change is spoken (added, removed, changed) in a hidden cell.",
                "Removed values are struck through as well as tinted.",
            ],
            responsive: vec!["The table scrolls sideways inside its card below its natural width."],
            align: Align::Start,
            height: 340,
            ..mk(
                "diff-table",
                "Diff table",
                "data-display",
                "Row-level changes in a keyed table (settings, records, environment variables): added, removed and changed rows with before → after and per-kind counts.",
                "DiffTable",
                "DiffTableEl",
                "diff-table",
                "parity/diff.rs",
            )
        },
        Demo {
            controls: vec![sel(
                "layout",
                "Diff layout",
                &["unified", "split", "stacked"],
                "unified",
            )],
            variants: vec![
                var("rail", "Revisions on a rail; the last two compared.", &[]),
                var(
                    "stacked",
                    "Before over after, for narrow panels.",
                    &[("layout", "stacked")],
                ),
            ],
            render: diff_timeline,
            code: diff_timeline_code,
            keyboard: vec![
                ("← / →", "Move the compare target"),
                ("Shift + ← / →", "Move the base"),
                ("Home / End", "First base / latest revision"),
            ],
            a11y: vec![
                "The rail is a listbox; each revision says who wrote it, when and its +/− counts.",
                "Your revisions draw a square dot and the agent's a round one, so author is not colour alone.",
            ],
            responsive: vec!["The rail scrolls sideways when there are more revisions than room."],
            height: 520,
            align: Align::Start,
            ..mk(
                "diff-timeline",
                "Diff timeline",
                "tools",
                "One file across the agent's and your revisions: pick any two points on the rail and read the diff between them.",
                "DiffTimeline",
                "DiffTimelineEl",
                "diff-timeline",
                "parity/diff.rs",
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
                "parity/diff.rs",
            )
        },
        Demo {
            controls: vec![
                rng("speed", "Stream speed", 0.25, 3.0, 0.25, "1"),
                tog("ghost", "Ghost text", true),
                sel(
                    "title",
                    "Title",
                    &["Launch plan", "Privacy note"],
                    "Launch plan",
                ),
            ],
            variants: vec![
                var(
                    "rewrite",
                    "Click a word, ⌘K, pick Shorten: the rewrite streams in as a diff.",
                    &[],
                ),
                var(
                    "slow-stream",
                    "Half speed, to watch unreached text wait dimmed.",
                    &[("speed", "0.5")],
                ),
                var(
                    "no-ghost",
                    "Without ghost-text completions.",
                    &[("ghost", "false")],
                ),
            ],
            render: inline_edit,
            code: inline_edit_code,
            keyboard: vec![
                (
                    "⌘K / Ctrl+K",
                    "Open the prompt on the selection (or the first sentence); again to close",
                ),
                (
                    "↑ / ↓, Enter",
                    "Move through quick actions and run one; typed text sends as a free instruction",
                ),
                (
                    "Esc",
                    "Close the prompt, stop the stream, reject all in review, or dismiss ghost text",
                ),
                ("Tab", "Accept ghost text; in review, move between changes"),
                ("Enter / Backspace", "Accept / reject the focused change"),
                ("⌘↵", "Accept all remaining changes"),
                ("⌘Z / ⇧⌘Z", "Undo / redo AI edits"),
            ],
            a11y: vec![
                "The editor is a labelled region; the prompt is a dialog with a combobox and a listbox of actions (aria-activedescendant).",
                "Each change is a focusable group named “Change “very careful” to “careful””; deletions and insertions pair colour with strike-through and highlight.",
                "Streaming, review and applied states are announced in a status region; ghost text is aria-hidden.",
            ],
            responsive: vec![
                "The prompt keeps to 320 px or the page width; below a 560 px card the title steps down and the key hint hides.",
            ],
            height: 520,
            align: Align::Start,
            live: true,
            ..mk(
                "ai-inline-edit",
                "AI inline edit",
                "chat",
                "Edit a document with AI in place: select a sentence, ⌘K, pick an action, and the rewrite streams in as a word diff you accept change by change, with undo and ghost-text completions.",
                "AiInlineEdit",
                "AiInlineEditEl",
                "ai-inline-edit",
                "parity/review.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "mode",
                    "Mode",
                    &["slider", "side", "onion", "difference"],
                    "slider",
                ),
                sel(
                    "orient",
                    "Orientation",
                    &["horizontal", "vertical"],
                    "horizontal",
                ),
                rng("pos", "Start position", 0.0, 100.0, 1.0, "50"),
                sel("ratio", "Aspect", &["3 / 2", "16 / 9", "1 / 1"], "3 / 2"),
                tog("labels", "Captions", true),
            ],
            variants: vec![
                var("slider", "Drag the divider anywhere on the frame.", &[]),
                var("vertical", "Top and bottom.", &[("orient", "vertical")]),
                var("side-by-side", "Both at once.", &[("mode", "side")]),
                var(
                    "onion",
                    "After over before with an opacity slider.",
                    &[("mode", "onion")],
                ),
                var(
                    "difference",
                    "Difference blend: what changed lights up.",
                    &[("mode", "difference")],
                ),
            ],
            render: image_compare,
            code: image_compare_code,
            keyboard: vec![
                ("Arrow keys", "Move the divider 1 % (10 % with Shift)"),
                ("Page Up / Page Down", "Move 10 %"),
                ("Home / End", "All before / all after"),
                ("Double-click", "Recentre the divider"),
            ],
            a11y: vec![
                "The divider is a focusable role=slider with aria-orientation and aria-valuetext like “40% after”.",
                "Captions are aria-hidden; give both images real alt text.",
                "Reduced motion moves the divider and handle without the stretch or bump.",
            ],
            responsive: vec![
                "Full width at a fixed aspect ratio; on touch a vertical swipe still scrolls the page.",
            ],
            align: Align::Start,
            height: 520,
            ..mk(
                "image-compare",
                "Image compare",
                "data-display",
                "Two images under one divider to see what changed: drag anywhere, arrow keys and Home / End, double-click to recentre; plus side by side, onion skin and a difference blend.",
                "ImageCompare",
                "ImageCompareEl",
                "image-compare",
                "parity/review.rs",
            )
        },
        Demo {
            controls: vec![
                sel("set", "Thread", &["review", "empty"], "review"),
                tog("resolved", "Resolved", false),
                rng("depth", "Max depth", 1.0, 4.0, 1.0, "2"),
                sel(
                    "title",
                    "Title",
                    &["Hero headline", "Pricing copy"],
                    "Hero headline",
                ),
            ],
            variants: vec![
                var(
                    "thread",
                    "Replies, reactions, mentions and an agent's reply.",
                    &[],
                ),
                var(
                    "resolved",
                    "Folded into the resolved chip.",
                    &[("resolved", "true")],
                ),
                var("empty", "No comments yet.", &[("set", "empty")]),
            ],
            render: comment_thread,
            code: comment_thread_code,
            keyboard: vec![
                ("⌘↵ / Ctrl+Enter", "Send the composer"),
                ("@", "Open mention suggestions"),
                ("↑ / ↓", "Move through suggestions"),
                ("Enter / Tab", "Insert the active mention"),
                (
                    "Esc",
                    "Close suggestions, then cancel an edit or reply; closes the reaction picker and delete confirm",
                ),
            ],
            a11y: vec![
                "The composer is a combobox with a listbox of people and aria-activedescendant.",
                "Each comment is an article labelled with author and time; a deleted one reads “Deleted comment”.",
                "Reaction chips are toggle buttons with aria-pressed and a label like “👍 2, including you”; reply toggles expose aria-expanded.",
            ],
            responsive: vec![
                "Below 420 px the composer hides its avatar and replies indent less (12 px); every action is a visible button on touch.",
            ],
            height: 560,
            align: Align::Start,
            ..mk(
                "comment-thread",
                "Comment thread",
                "chat",
                "Threaded comments with nested replies, reactions, @mentions, edit and delete for your own comments, agent replies, and Resolve, which folds the thread into a chip.",
                "CommentThread",
                "CommentThreadEl",
                "comment-thread",
                "parity/review.rs",
            )
        },
        Demo {
            controls: vec![
                sel("order", "Order", &["risk", "oldest", "given"], "risk"),
                sel("filter", "Filter", &["all", "high", "unsure"], "all"),
                rng("sla", "Service level (min)", 5.0, 60.0, 5.0, "30"),
                tog("batch", "Approve all low risk", false),
            ],
            variants: vec![
                var(
                    "queue",
                    "High risk first, then oldest; one item at a time.",
                    &[],
                ),
                var(
                    "unsure",
                    "Only items below the confidence bar.",
                    &[("filter", "unsure")],
                ),
                var(
                    "tight-sla",
                    "A 20-minute service level: waiting times turn amber, then red.",
                    &[("sla", "20")],
                ),
                var(
                    "batch",
                    "Approve every low-risk item in one go.",
                    &[("batch", "true"), ("order", "oldest")],
                ),
            ],
            render: review_center,
            code: review_center_code,
            keyboard: vec![
                ("A", "Approve the current item"),
                (
                    "E",
                    "Edit the proposal; ⌘↵ approves with the edits, Esc cancels",
                ),
                ("R", "Reject, then 1–4 picks the reason; Esc cancels"),
                ("S", "Skip without deciding"),
                ("J / K or ↓ / ↑", "Next / previous item"),
                ("F", "Show the agent's note on the next flagged span"),
                ("Z or ⌘Z", "Undo the last decision"),
            ],
            a11y: vec![
                "Shortcuts work anywhere inside; buttons carry aria-keyshortcuts; the reason picker is a labelled group.",
                "Queue rows are labelled with title, type, risk, confidence and waiting time; the current one has aria-current.",
                "Flagged spans are focusable and point to their note with aria-describedby (role tooltip); decisions are announced with the new count.",
            ],
            responsive: vec![
                "Container query: wide, the queue sits left of the item in a fixed-height workspace; under 640 px the queue goes on top (three rows) and buttons share the width.",
                "Under 420 px the flag counter hides and button icons step aside.",
            ],
            height: 640,
            align: Align::Start,
            ..mk(
                "review-center",
                "Review center",
                "approvals",
                "A focused workspace for what agents want to do: the queue sorted by risk, one item at a time with the shaky spans flagged, and Approve, Edit, Reject and Skip on single keys with undo.",
                "ReviewCenter",
                "ReviewCenterEl",
                "review-center",
                "parity/review.rs",
            )
        },
        Demo {
            controls: vec![
                tog("nits", "Show nits", true),
                tog("first", "First open", true),
            ],
            variants: vec![
                var("findings", "Verdict, counts and findings with fixes.", &[]),
                var("no-nits", "Nits hidden.", &[("nits", "false")]),
            ],
            render: review_card,
            code: review_card_code,
            keyboard: vec![(
                "Enter / Space",
                "Open a finding; press Apply fix, Dismiss or Reopen",
            )],
            a11y: vec![
                "The verdict is a status; severities are words, not colour alone.",
                "Each finding header is a button with aria-expanded; fixes read “Removed:” / “Added:”.",
            ],
            responsive: vec![
                "Finding headers wrap the file and line under the title on narrow cards.",
            ],
            height: 460,
            align: Align::Start,
            ..mk(
                "review-card",
                "Review card",
                "tools",
                "An agent's code review in one card: the verdict, open findings by severity, and each finding with its file and line, a suggested fix as a diff, and Apply fix / Dismiss / Reopen.",
                "ReviewCard",
                "ReviewCardEl",
                "review-card",
                "parity/review.rs",
            )
        },
    ]
}
