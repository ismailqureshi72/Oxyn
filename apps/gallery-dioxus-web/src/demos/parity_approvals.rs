//! Plan #28 parity demos, approvals: hold to confirm, confirm morph, confirm popover, approval
//! gates, permission request and the agent approvals block.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::{
        confirm::{
            ConfirmMode, ConfirmMorph as MorphModel, ConfirmPopover as PopModel, ConfirmTone,
            HoldConfirm, MorphFace, PopoverAlign, PopoverPhase,
        },
        gates::{
            AgentApprovals as AgentModel, Arg, Gate, GateDecision, GateKind, GateStatus, Gates,
            PermRisk, PermissionRequest as PermModel, PreviewRow, Rule, RuleScope, Scope, ToolAsk,
        },
    },
};
use oxyn_dioxus::{
    AgentApprovals, ApprovalGates, ConfirmMorph, ConfirmPopover, HoldToConfirm, PermissionRequest,
};

use super::{Code, mk, q, sel, tog, var};
use crate::{Align, Demo, Vals};

// ---------- hold to confirm ----------

fn hold(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveHold { tone: v.string("tone"), size: v.string("size"), duration: v.string("duration"), hint: v.b("hint"), label: v.string("label"), now } }
}

#[component]
fn LiveHold(
    tone: String,
    size: String,
    duration: String,
    hint: bool,
    label: String,
    now: ClockMs,
) -> Element {
    let ms: u64 = duration.parse().unwrap_or(1200);
    let t = ConfirmTone::parse(&tone);
    let mut m = use_signal(move || {
        let mut h = HoldConfirm::new(ms);
        h.tone = t;
        h
    });
    if m.read().duration_ms != ms || m.read().tone != t {
        let mut h = HoldConfirm::new(ms);
        h.tone = t;
        m.set(h);
    }
    let mut shown = m.read().clone();
    shown.tick(now);
    let done = shown.confirmed;
    rsx! {
        div { class: "oxp-demo-col",
            HoldToConfirm {
                model: shown,
                now,
                size,
                hint,
                label,
                on_hold: move |down: bool| if down { m.write().press(now) } else { m.write().release(now) },
            }
            if done { button { class: "oxp-lnk", r#type: "button", onclick: move |_| m.write().reset(), "Reset" } }
        }
    }
}

fn hold_code(v: &Vals) -> String {
    Code::new("HoldToConfirm")
        .pre(format!(
            "let mut hold = use_signal(|| HoldConfirm::new({}));",
            v.s("duration")
        ))
        .raw("model", "hold()")
        .raw("now", "clock.now()")
        .s("label", v.s("label"), "Hold to delete")
        .s("size", v.s("size"), "md")
        .raw(
            "on_hold",
            "move |down| if down { hold.write().press(now) } else { hold.write().release(now) }",
        )
        .done()
}

// ---------- confirm morph ----------

fn settle_morph(m: &mut MorphModel, pend: Option<u64>, undo: Option<u64>, ok: bool, now: ClockMs) {
    if let Some(p) = pend
        && now.0 >= p + 900
    {
        m.resolve(ok, ClockMs(p + 900));
    }
    if let Some(u) = undo
        && now.0 >= u + 700
    {
        m.undone(ClockMs(u + 700));
    }
}

fn morph(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveMorph { tone: v.string("tone"), mode: v.string("mode"), result: v.string("result"), undo: v.b("undo"), start: v.string("start"), now } }
}

#[component]
fn LiveMorph(
    tone: String,
    mode: String,
    result: String,
    undo: bool,
    start: String,
    now: ClockMs,
) -> Element {
    let at = now;
    let key = format!("{tone}{mode}{undo}{start}");
    let build = move |key: &str| {
        let mut m = MorphModel::new("Delete 3 files");
        m.labels.prompt = Some("Delete 3 files?".into());
        let parts = key;
        m.tone = if parts.starts_with("neutral") {
            ConfirmTone::Neutral
        } else {
            ConfirmTone::Danger
        };
        m.mode = ConfirmMode::parse(if parts.contains("hold") {
            "hold"
        } else if parts.contains("countdown") {
            "countdown"
        } else {
            "press"
        });
        m.undoable = parts.contains("true");
        if parts.ends_with("confirming") {
            m = m.with_face(MorphFace::Confirming, ClockMs(at.0.saturating_sub(10)));
            m.confirm_timeout_ms = u64::MAX / 4;
        } else if parts.ends_with("done") {
            m = m.with_face(MorphFace::Done, at);
        } else if parts.ends_with("error") {
            m = m.with_face(MorphFace::Error, at);
        }
        m
    };
    let mut m = use_signal(|| build(&key));
    let mut k = use_signal(|| key.clone());
    let mut pend = use_signal(|| None::<u64>);
    let mut undo_at = use_signal(|| None::<u64>);
    if k() != key {
        k.set(key.clone());
        m.set(build(&key));
        pend.set(None);
        undo_at.set(None);
    }
    let ok = result != "error";
    let mut shown = m.read().clone();
    settle_morph(&mut shown, pend(), undo_at(), ok, now);
    rsx! {
        ConfirmMorph {
            model: shown,
            now,
            on_open: move |_| { let mut w = m.write(); settle_morph(&mut w, pend(), undo_at(), ok, now); pend.set(None); undo_at.set(None); w.open(now) },
            on_confirm: move |_| {
                let mut w = m.write();
                settle_morph(&mut w, pend(), undo_at(), ok, now);
                undo_at.set(None);
                if w.confirm(now) {
                    pend.set(Some(now.0));
                } else if let MorphFace::Counting(s) = w.face(now) {
                    pend.set(Some(now.0 + s * 1000));
                }
            },
            on_cancel: move |_| { m.write().cancel(now); pend.set(None) },
            on_undo: move |_| { let mut w = m.write(); settle_morph(&mut w, pend(), undo_at(), ok, now); w.undo(now); pend.set(None); undo_at.set(Some(now.0)) },
            on_hold: move |down: bool| {
                let mut w = m.write();
                if down { w.hold.press(now) } else {
                    w.hold.release(now);
                    if w.tick(now) { pend.set(Some(now.0)) }
                }
            },
        }
    }
}

fn morph_code(v: &Vals) -> String {
    Code::new("ConfirmMorph")
        .pre("let mut m = use_signal(|| ConfirmMorph::new(\"Delete 3 files\"));")
        .pre(format!(
            "m.write().mode = ConfirmMode::{};",
            match v.s("mode") {
                "hold" => "Hold",
                "countdown" => "Countdown",
                _ => "Press",
            }
        ))
        .raw("model", "m()")
        .raw("now", "clock.now()")
        .raw(
            "on_confirm",
            "move |_| if m.write().confirm(now) { spawn(delete_files(m)) }",
        )
        .raw("on_cancel", "move |_| m.write().cancel(now)")
        .raw("on_undo", "move |_| m.write().undo(now)")
        .done()
}

// ---------- confirm popover ----------

fn pop(v: &Vals, now: ClockMs) -> Element {
    rsx! { LivePop { tone: v.string("tone"), phrase: v.b("phrase"), align: v.string("align"), result: v.string("result"), progress: v.b("progress"), open: v.b("open"), now } }
}

fn settle_pop(m: &mut PopModel, work: Option<u64>, ok: bool, progress: bool, now: ClockMs) {
    if let Some(w) = work {
        if progress {
            m.progress((now.0.saturating_sub(w)) as f32 / 1600.0);
        }
        if now.0 >= w + 1600 {
            m.finish(
                if ok {
                    Ok(())
                } else {
                    Err("You don't have permission to delete acme-web.".into())
                },
                ClockMs(w + 1600),
            );
        }
    }
}

#[component]
fn LivePop(
    tone: String,
    phrase: bool,
    align: String,
    result: String,
    progress: bool,
    open: bool,
    now: ClockMs,
) -> Element {
    let at = now;
    let key = format!("{tone}{phrase}{align}{open}");
    let align_cls = align.clone();
    let build = move || {
        let mut p = PopModel::new();
        p.tone = ConfirmTone::parse(&tone);
        p.align = PopoverAlign::parse(&align);
        if phrase {
            p = p.require("acme-web");
        }
        if open {
            p = p.with_phase(PopoverPhase::Confirming, at);
        }
        p
    };
    let mut m = use_signal(build.clone());
    let mut k = use_signal(|| key.clone());
    let mut work = use_signal(|| None::<u64>);
    if k() != key {
        k.set(key.clone());
        m.set(build());
        work.set(None);
    }
    let ok = result != "error";
    let mut shown = m.read().clone();
    settle_pop(&mut shown, work(), ok, progress, now);
    rsx! {
        div { class: "oxp-demo-pop {align_cls}",
            ConfirmPopover {
                model: shown,
                now,
                description: "This removes the acme-web project, its 14 deployments and its domains. It can't be undone.".to_string(),
                on_show: move |_| { let mut w = m.write(); settle_pop(&mut w, work(), ok, progress, now); work.set(None); w.show(now) },
                on_dismiss: move |_| m.write().dismiss(now),
                on_type: move |s: String| m.write().type_text(&s),
                on_submit: move |_| {
                    let mut w = m.write();
                    settle_pop(&mut w, work(), ok, progress, now);
                    if w.submit(now) { work.set(Some(now.0)) }
                },
            }
        }
    }
}

fn pop_code(v: &Vals) -> String {
    let mut c = Code::new("ConfirmPopover").pre("let mut p = use_signal(ConfirmPopover::new);");
    if v.b("phrase") {
        c = c.pre("p.write().confirm_text = Some(\"acme-web\".into()); // type to confirm");
    }
    c.raw("model", "p()")
        .raw("now", "clock.now()")
        .raw("description", q("This removes the acme-web project…"))
        .raw(
            "on_submit",
            "move |_| if p.write().submit(now) { spawn(delete(p)) }",
        )
        .raw("on_type", "move |s| p.write().type_text(&s)")
        .done()
}

// ---------- approval gates ----------

fn sample_gates(scenario: &str, live: bool) -> Gates {
    let mut g = Gates {
        task: "Launch the spring newsletter".into(),
        live,
        gates: vec![
            Gate::new(
                "send",
                "Writer",
                GateKind::Message,
                "Send “Spring is here” to 4,210 subscribers",
                "The draft passed review and the segment is ready.",
            )
            .input("to: list:spring-2026\nsubject: Spring is here 🌱\nsend_at: now")
            .holds(2),
            Gate::new(
                "pricing",
                "Coder",
                GateKind::File,
                "Edit pricing.tsx",
                "The newsletter links to the new annual plan.",
            )
            .diff(&[
                "  const plans = [",
                "-   { id: \"pro\", price: 12 },",
                "+   { id: \"pro\", price: 12, annual: 120 },",
                "+   { id: \"team\", price: 30, annual: 300 },",
                "  ];",
            ])
            .holds(1),
            Gate::new(
                "handoff",
                "Planner",
                GateKind::Agent,
                "Hand analytics to the Analyst agent",
                "Tracking links need UTM tags before the send.",
            )
            .input("Add utm_source=newsletter to every link in the draft"),
            Gate::new(
                "search",
                "Researcher",
                GateKind::Tool,
                "web.search “newsletter send times 2026”",
                "Pick the best send hour.",
            )
            .status(GateStatus::Approved),
        ],
    };
    match scenario {
        "files" => g.gates.rotate_left(1),
        "decided" => {
            for x in g.gates.iter_mut() {
                if x.status == GateStatus::Open {
                    x.status = GateStatus::Approved;
                }
            }
            g.gates[0].status = GateStatus::Edited;
            g.gates[0].note = Some("send_at: 9:00".into());
            g.gates[2].status = GateStatus::Rejected;
        }
        _ => {}
    }
    g
}

fn gates(v: &Vals, _: ClockMs) -> Element {
    rsx! { LiveGates { scenario: v.string("scenario"), live: v.b("live"), decided: v.b("decided") } }
}

#[component]
fn LiveGates(scenario: String, live: bool, decided: bool) -> Element {
    let key = format!("{scenario}{live}");
    let mut g = use_signal(|| sample_gates(&scenario, live));
    let mut k = use_signal(|| key.clone());
    if k() != key {
        k.set(key.clone());
        g.set(sample_gates(&scenario, live));
    }
    rsx! {
        div { class: "oxd-w520",
            ApprovalGates {
                gates: g(),
                decided,
                on_decide: move |(id, d): (String, GateDecision)| { g.write().decide(&id, d); },
            }
        }
    }
}

fn gates_code(v: &Vals) -> String {
    Code::new("ApprovalGates")
        .pre("let mut gates = use_signal(|| Gates { task, live: true, gates: run_gates() });")
        .raw("gates", "gates()")
        .b("decided", !v.b("decided"))
        .raw(
            "on_decide",
            "move |(id, d)| { gates.write().decide(&id, d); agent.resume(&id) }",
        )
        .done()
        .replace("decided: true", "decided: false")
}

// ---------- permission request ----------

fn sample_perm(risk: &str, scopes: &str, rows: usize, model: bool) -> PermModel {
    let mut p = PermModel::new("Release bot", "push to main", "acme/web");
    p.risk = PermRisk::parse(risk);
    if risk == "destructive" {
        p.action = "force-push to main".into();
    }
    if model {
        p.model = Some("Claude Sonnet 5.5".into());
    }
    p.reason = "CI is green on release/2.4 and the changelog is merged; tagging needs main.".into();
    let all = [
        ("Merge release/2.4 into main", Some("+412 −96"), "branch"),
        ("Tag v2.4.0", None, "check"),
        ("Trigger the production deploy", Some("Vercel"), "bolt"),
        ("Post to #releases", Some("Slack"), "comment"),
        ("Close milestone 2.4", Some("12 issues"), "check-circle"),
    ];
    p.preview = all
        .iter()
        .take(rows)
        .map(|(l, m, i)| PreviewRow {
            label: (*l).into(),
            meta: m.map(Into::into),
            icon: i,
        })
        .collect();
    p.scopes = match scopes {
        "once" => vec![Scope::Once],
        "once-timed" => vec![Scope::Once, Scope::Timed],
        _ => vec![Scope::Once, Scope::Timed, Scope::Always],
    };
    p
}

fn perm(v: &Vals, now: ClockMs) -> Element {
    rsx! { LivePerm { risk: v.string("risk"), scopes: v.string("scopes"), rows: v.u("rows"), state: v.string("state"), model: v.b("model"), now } }
}

#[component]
fn LivePerm(
    risk: String,
    scopes: String,
    rows: usize,
    state: String,
    model: bool,
    now: ClockMs,
) -> Element {
    let at = now;
    let key = format!("{risk}{scopes}{rows}{state}{model}");
    let build = move || {
        let mut p = sample_perm(&risk, &scopes, rows, model);
        match state.as_str() {
            "timed" => {
                p.scope = Scope::Timed;
                p.allow(ClockMs(at.0.saturating_sub(120_000)));
            }
            "always" => {
                p.scope = Scope::Always;
                p.allow(at);
            }
            "denied" => p.deny(at),
            _ => {}
        }
        p
    };
    let mut p = use_signal(build.clone());
    let mut k = use_signal(|| key.clone());
    if k() != key {
        k.set(key.clone());
        p.set(build());
    }
    rsx! {
        div { class: "oxd-w520",
            PermissionRequest {
                req: p(),
                now,
                on_scope: move |s| p.write().scope = s,
                on_allow: move |_| p.write().allow(now),
                on_deny: move |_| p.write().deny(now),
                on_revoke: move |_| p.write().revoke(),
                on_more: move |_| p.write().expanded = true,
            }
        }
    }
}

fn perm_code(v: &Vals) -> String {
    Code::new("PermissionRequest")
        .pre("let mut p = use_signal(|| PermissionRequest::new(\"Release bot\", \"push to main\", \"acme/web\"));")
        .pre(format!(
            "p.write().risk = PermRisk::{};",
            match v.s("risk") {
                "safe" => "Safe",
                "destructive" => "Destructive",
                _ => "Risky",
            }
        ))
        .raw("req", "p()")
        .raw("now", "clock.now()")
        .raw("on_scope", "move |s| p.write().scope = s")
        .raw("on_allow", "move |_| p.write().allow(now)")
        .raw("on_deny", "move |_| p.write().deny(now)")
        .raw("on_revoke", "move |_| p.write().revoke()")
        .done()
}

// ---------- agent approvals ----------

#[allow(clippy::too_many_arguments)]
fn ask(
    id: &str,
    agent: &str,
    tool: &str,
    perm: &str,
    title: &str,
    reason: &str,
    risk: &'static str,
    args: &[(&str, &str, bool)],
) -> ToolAsk {
    ToolAsk {
        id: id.into(),
        agent: agent.into(),
        model: if agent == "Deployer" {
            "GPT-5.5".into()
        } else {
            "Claude Sonnet 5.5".into()
        },
        tool: tool.into(),
        permission: perm.into(),
        title: title.into(),
        summary: title.replace('`', ""),
        reason: reason.into(),
        args: args
            .iter()
            .map(|(k, v, e)| Arg {
                key: (*k).into(),
                value: (*v).into(),
                editable: *e,
            })
            .collect(),
        risk,
    }
}

fn sample_agent(rules: bool) -> AgentModel {
    let mut a = AgentModel {
        queue: vec![
            ask(
                "1",
                "Deployer",
                "vercel.deploy",
                "Deploy to production",
                "Run `vercel deploy --prod` for northwind-web",
                "Checks passed on main; the release notes are approved.",
                "high",
                &[
                    ("project", "northwind-web", false),
                    ("target", "production", true),
                    ("commit", "a91f3c2", false),
                ],
            ),
            ask(
                "2",
                "Analyst",
                "postgres.update",
                "Write to PostgreSQL",
                "`UPDATE` 1,284 rows in PostgreSQL",
                "Backfill the plan column for annual customers.",
                "medium",
                &[
                    ("table", "customers", false),
                    ("where", "plan IS NULL AND billing = 'annual'", true),
                ],
            ),
            ask(
                "3",
                "Deployer",
                "vercel.deploy",
                "Deploy to production",
                "Run `vercel deploy --prod` for northwind-docs",
                "Docs for the same release.",
                "high",
                &[
                    ("project", "northwind-docs", false),
                    ("target", "production", true),
                ],
            ),
            ask(
                "4",
                "Support",
                "gmail.send",
                "Send email",
                "Email 3 customers about the outage",
                "They opened tickets during the incident.",
                "low",
                &[
                    ("to", "3 recipients", false),
                    ("subject", "We're back up", true),
                ],
            ),
        ],
        reviewer: "Emma Collins".into(),
        workspace: "Northwind".into(),
        ..Default::default()
    };
    if rules {
        a.rules.push(Rule {
            tool: "github.comment".into(),
            permission: "Comment on pull requests".into(),
            scope: RuleScope::Workspace,
        });
        a.rules.push(Rule {
            tool: "linear.create".into(),
            permission: "Create Linear issues".into(),
            scope: RuleScope::Agent("Support".into()),
        });
    }
    a
}

fn agents(v: &Vals, _: ClockMs) -> Element {
    rsx! { LiveAgents { tab: v.string("tab"), rules: v.b("rules") } }
}

#[component]
fn LiveAgents(tab: String, rules: bool) -> Element {
    let key = format!("{tab}{rules}");
    let mut a = use_signal(|| sample_agent(rules));
    let mut k = use_signal(|| key.clone());
    if k() != key {
        k.set(key.clone());
        a.set(sample_agent(rules));
    }
    rsx! {
        div { class: "oxd-w560", key: "{key}",
            AgentApprovals {
                model: a(),
                tab,
                on_decide: move |(ok, edited)| a.write().decide(ok, edited),
                on_always: move |s| { a.write().always(s); },
                on_arg: move |(k, v): (String, String)| { a.write().edit_arg(&k, &v); },
                on_remove_rule: move |i| a.write().remove_rule(i),
            }
        }
    }
}

fn agents_code(_: &Vals) -> String {
    Code::new("AgentApprovals")
        .pre("let mut a = use_signal(|| AgentApprovals { queue, rules, reviewer: \"Emma\".into(), workspace: \"Northwind\".into(), ..Default::default() });")
        .raw("model", "a()")
        .raw("on_decide", "move |(ok, edited)| a.write().decide(ok, edited)")
        .raw("on_always", "move |scope| { a.write().always(scope); }")
        .raw("on_arg", "move |(k, v)| { a.write().edit_arg(&k, &v); }")
        .done()
}

/// The approval demos added for plan #28.
pub fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel("tone", "Tone", &["danger", "neutral"], "danger"),
                sel("duration", "Hold (ms)", &["800", "1200", "2000"], "1200"),
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                sel(
                    "label",
                    "Label",
                    &["Hold to delete", "Hold to deploy", "Hold to wipe"],
                    "Hold to delete",
                ),
                tog("hint", "Hint below", true),
            ],
            variants: vec![
                var("danger", "Red fill, 1.2 s.", &[]),
                var(
                    "neutral",
                    "Foreground fill, for reversible actions.",
                    &[("tone", "neutral"), ("label", "Hold to deploy")],
                ),
                var(
                    "long",
                    "2 s hold, large.",
                    &[
                        ("duration", "2000"),
                        ("size", "lg"),
                        ("label", "Hold to wipe"),
                    ],
                ),
                var(
                    "bare",
                    "No hint, small.",
                    &[("hint", "false"), ("size", "sm")],
                ),
            ],
            render: hold,
            code: hold_code,
            keyboard: vec![
                (
                    "Space / Enter (hold)",
                    "Fills the button; releasing early rewinds it",
                ),
                ("Tab away", "Cancels the hold"),
            ],
            a11y: vec![
                "A real button named “… hold to confirm”; the hint below is its description and a polite live region (“Keep holding…”, “Done”).",
                "The check is drawn in after completion; the label changes to the confirmed label.",
            ],
            responsive: vec!["Inline size; touch-action none so a hold never scrolls the page."],
            height: 220,
            live: true,
            ..mk(
                "hold-to-confirm",
                "Hold to confirm",
                "approvals",
                "Confirm a destructive action by holding, not tapping: the fill runs while held, rewinds smoothly when you let go early and resumes from where it was on a quick re-press; a check draws in when done.",
                "HoldToConfirm",
                "HoldConfirmEl",
                "hold-to-confirm",
                "parity/approvals.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "mode",
                    "Confirm by",
                    &["press", "hold", "countdown"],
                    "press",
                ),
                sel("tone", "Tone", &["danger", "neutral"], "danger"),
                sel("result", "Action result", &["ok", "error"], "ok"),
                sel(
                    "start",
                    "Start at",
                    &["idle", "confirming", "done", "error"],
                    "idle",
                ),
                tog("undo", "Undo", true),
            ],
            variants: vec![
                var("press", "Press Delete to confirm.", &[]),
                var(
                    "hold",
                    "Hold Delete while it fills.",
                    &[("mode", "hold"), ("start", "confirming")],
                ),
                var(
                    "countdown",
                    "Confirm, then 3 s you can still cancel.",
                    &[("mode", "countdown")],
                ),
                var(
                    "error",
                    "The action fails: Retry.",
                    &[("result", "error"), ("start", "error")],
                ),
                var("done", "Result with Undo.", &[("start", "done")]),
            ],
            render: morph,
            code: morph_code,
            keyboard: vec![
                ("Enter", "Confirm"),
                ("Escape", "Cancel the question or stop the countdown"),
            ],
            a11y: vec![
                "One group; every face keeps a real button so focus never drops to the page.",
                "Each change (asking, deleting, deleted, error) is announced in a polite live region.",
            ],
            responsive: vec!["Grows inline from the resting button; the question never wraps."],
            height: 200,
            live: true,
            ..mk(
                "confirm-morph",
                "Confirm morph",
                "approvals",
                "A destructive button that morphs into an inline question, a spinner and a result with Undo, or an error with Retry. Confirm by press, by hold, or with a countdown Escape can still stop.",
                "ConfirmMorph",
                "ConfirmMorphEl",
                "confirm-morph",
                "parity/approvals.rs",
            )
        },
        Demo {
            controls: vec![
                tog("phrase", "Type to confirm", true),
                sel("tone", "Tone", &["danger", "neutral"], "danger"),
                sel("align", "Align", &["start", "center", "end"], "start"),
                sel("result", "Action result", &["ok", "error"], "ok"),
                tog("progress", "Reports progress", true),
                tog("open", "Start open", true),
            ],
            variants: vec![
                var("phrase", "Type the project name to unlock Delete.", &[]),
                var(
                    "plain",
                    "Consequence and two buttons.",
                    &[("phrase", "false")],
                ),
                var(
                    "neutral",
                    "Reversible: foreground fill.",
                    &[("tone", "neutral"), ("phrase", "false")],
                ),
                var(
                    "error",
                    "Fails: the message and Try again.",
                    &[("result", "error"), ("phrase", "false")],
                ),
                var(
                    "sweep",
                    "No progress reported: an indeterminate sweep.",
                    &[("progress", "false"), ("phrase", "false")],
                ),
            ],
            render: pop,
            code: pop_code,
            keyboard: vec![
                ("Enter", "Confirm (shakes when the phrase does not match)"),
                ("Escape", "Dismiss"),
                ("Tab", "Phrase field, Cancel, Delete"),
            ],
            a11y: vec![
                "The card is an alertdialog named by its heading and described by the consequence.",
                "The trigger has aria-expanded; Delete is aria-disabled until the phrase matches (case and outer spaces ignored).",
            ],
            responsive: vec![
                "300 px card, max 100 %; `end` keeps the right edge for buttons at the end of a row.",
            ],
            height: 320,
            align: Align::Start,
            live: true,
            ..mk(
                "confirm-popover",
                "Confirm popover",
                "approvals",
                "A destructive button that grows into a small card: the consequence, an optional type-to-confirm phrase that fills Delete as you type, progress on the button, then a check in the button's shape or the error with Try again.",
                "ConfirmPopover",
                "ConfirmPopoverEl",
                "confirm-popover",
                "parity/approvals.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "scenario",
                    "Run",
                    &["newsletter", "files", "decided"],
                    "newsletter",
                ),
                tog("live", "Run still going", true),
                tog("decided", "Decided list", true),
            ],
            variants: vec![
                var("stack", "Three open gates; the message send on top.", &[]),
                var(
                    "file",
                    "A file change with its diff on top.",
                    &[("scenario", "files")],
                ),
                var(
                    "decided",
                    "Everything decided: Approved, Edited, Rejected.",
                    &[("scenario", "decided")],
                ),
                var(
                    "idle",
                    "Nothing open and the run is over.",
                    &[("scenario", "decided"), ("live", "false")],
                ),
            ],
            render: gates,
            code: gates_code,
            keyboard: vec![
                ("⌘/Ctrl + Enter", "Approve the top gate"),
                ("Tab", "Reject, Change, Approve"),
            ],
            a11y: vec![
                "A section named Approvals; the waiting count and held steps are text in the header.",
                "The peeking cards behind the stack are aria-hidden; only the top gate is interactive.",
            ],
            responsive: vec![
                "Max 520 px; the decided rows truncate the action and keep the outcome.",
            ],
            height: 520,
            align: Align::Start,
            ..mk(
                "approval-gates",
                "Approval gates",
                "approvals",
                "Every approval request of a run as a stack: the one to decide now on top with who asks, what exactly and why, its input or diff; approve, change it first or reject, and it drops into the Decided list while the next rises. Open gates say how many steps they hold.",
                "ApprovalGates",
                "ApprovalGatesEl",
                "approval-gates",
                "parity/approvals.rs",
            )
        },
        Demo {
            controls: vec![
                sel("risk", "Risk", &["safe", "risky", "destructive"], "risky"),
                sel("scopes", "Scopes", &["all", "once-timed", "once"], "all"),
                sel(
                    "state",
                    "State",
                    &["waiting", "timed", "always", "denied"],
                    "waiting",
                ),
                super::rng("rows", "Preview rows", 0.0, 5.0, 1.0, "5"),
                tog("model", "Model name", true),
            ],
            variants: vec![
                var("risky", "Once, for 1 h or always.", &[]),
                var(
                    "destructive",
                    "Red Allow, no Always.",
                    &[("risk", "destructive")],
                ),
                var(
                    "timed",
                    "Folded: allowed with the time left and Revoke.",
                    &[("state", "timed")],
                ),
                var(
                    "once",
                    "One scope hides the switch.",
                    &[("scopes", "once"), ("risk", "safe"), ("rows", "2")],
                ),
            ],
            render: perm,
            code: perm_code,
            keyboard: vec![("Tab", "Scope radios, Deny, Allow"), ("Escape", "Deny")],
            a11y: vec![
                "Named “<agent> wants to <action>”; the scope switch is a radiogroup.",
                "The folded line is a status with the time left as text.",
            ],
            responsive: vec![
                "Max 440 px; the footer wraps the scope switch above the buttons when narrow.",
            ],
            height: 420,
            align: Align::Start,
            live: true,
            ..mk(
                "permission-request",
                "Permission request",
                "approvals",
                "An agent asks for one specific right: who, why and exactly what will happen. Allow it once, for an hour or always; it folds into one line with the time left and Revoke. Destructive requests turn Allow red and drop Always.",
                "PermissionRequest",
                "PermissionRequestEl",
                "permission-request",
                "parity/approvals.rs",
            )
        },
        Demo {
            controls: vec![
                sel("tab", "Tab", &["queue", "rules", "log"], "queue"),
                tog("rules", "Existing rules", true),
            ],
            variants: vec![
                var("queue", "The waiting deploy with editable arguments.", &[]),
                var(
                    "rules",
                    "Always-allow rules with Remove.",
                    &[("tab", "rules")],
                ),
                var("fresh", "No rules yet.", &[("rules", "false")]),
            ],
            render: agents,
            code: agents_code,
            keyboard: vec![("Tab", "Arguments, Deny, Approve, Always allow menu")],
            a11y: vec![
                "Tabs are a real tablist; the always-allow menu is a menu of two menuitems.",
                "Edited arguments change the Approve label to “Approve edited” and are logged as edited.",
            ],
            responsive: vec!["Max 520 px; arguments keep a 110 px key column."],
            height: 460,
            align: Align::Start,
            ..mk(
                "agent-approvals",
                "Agent approvals",
                "approvals",
                "One card for agents' tool calls: the call in plain words with code set in mono, the agent's reason, editable arguments; approve, deny or always allow for this agent or the whole workspace, which also clears every queued call the new rule covers. Rules and a decision log sit behind tabs.",
                "AgentApprovals",
                "AgentApprovalsEl",
                "agent-approvals",
                "parity/approvals.rs",
            )
        },
    ]
}
