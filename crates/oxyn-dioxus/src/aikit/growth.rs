//! Growth components (Dioxus): Upgrade prompt, Referral card, Trial banner, Live visitors and
//! Social proof toast.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::proof::Proof;
use oxyn_core::aikit::referral::{LINK, MILESTONE, REWARD, Referral};
use oxyn_core::aikit::thousands;
use oxyn_core::aikit::trial::{
    FEATURES as TRIAL_FEATURES, PLAN as TRIAL_PLAN, SNOOZES, Tone as TrialTone, Trial,
};
use oxyn_core::aikit::upgrade::{FEATURES, LIMITS, PLAN, Phase as UpPhase, Upgrade};
use oxyn_core::aikit::visitors::{CITIES, MAP, Visitors, flag, land, project};

use super::{cls, dots, ico, key_of, live, mark, seg};

/// Upgrade prompt: the nudge for the moment someone hits a limit. A usage meter (role meter,
/// `50 of 50 AI credits used`), the price with a monthly and yearly switch (arrow keys), three
/// unlocks with the marks of the models they name, and Upgrade or Maybe later. Upgrading lands
/// on the new limits with Continue; Maybe later leaves a reminder with See Pro again; a declined
/// card says nothing was charged. Every state shares one footprint.
#[component]
pub fn UpgradePrompt(
    /// Host clock.
    now: ClockMs,
    /// Used.
    #[props(default = 50)]
    used: u64,
    /// Current limit.
    #[props(default = 50)]
    limit: u64,
    /// What is counted, plural.
    #[props(default = "AI credits")]
    unit: &'static str,
    /// Start on yearly billing.
    #[props(default = true)]
    yearly: bool,
    /// The card is declined (error scenario).
    #[props(default)]
    fail: bool,
) -> Element {
    let mut up = use_signal(move || Upgrade::new(used, limit, unit, yearly, fail));
    let mut said = use_signal(String::new);
    let n = now.0;
    let u = up.read().clone();
    let mut act = move |id: String| {
        let x = up.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let ph = u.phase(n);
    let (val, max, words) = u.meter(n);
    let frac = val as f64 / max.max(1) as f64;
    let hue = if ph == UpPhase::Upgraded {
        "green"
    } else if u.at_limit() {
        "red"
    } else {
        "amber"
    };
    rsx! {
        section { class: "oxaik oxaik-up", "aria-label": "Upgrade to {PLAN.0}", "data-ph": "{ph:?}",
            header { class: "oxaik-hl",
                span { class: "oxaik-up-ic", "data-hue": "{hue}", {ico(match ph { UpPhase::Upgraded => "check-circle", UpPhase::Later => "clock", UpPhase::Error => "x-circle", _ => "bolt" }, 18.0)} }
                h3 { class: "oxaik-h", key: "{u.title(n)}", "{u.title(n)}" }
                p { class: "oxaik-sub oxaik-up-desc", "{u.description(n)}" }
            }
            div { class: "oxaik-up-meter",
                div { class: "oxaik-row oxaik-between",
                    span { class: "oxaik-row", "data-hue": "{hue}", {ico(if ph == UpPhase::Upgraded { "check" } else if u.at_limit() { "alert" } else { "info" }, 12.0)} b { class: "oxaik-up-w", "{words}" } }
                    span { class: "oxaik-mute oxaik-num", "{thousands(val)} / {thousands(max)}" }
                }
                div { class: "oxaik-bar", role: "meter", "aria-valuemin": "0", "aria-valuemax": "{max}", "aria-valuenow": "{val}", "aria-valuetext": "{thousands(val)} of {thousands(max)} {u.unit} used", "data-hue": "{hue}",
                    span { style: "width:{frac * 100.0}%" }
                }
            }
            div { class: "oxaik-up-stack",
                if matches!(ph, UpPhase::Offer | UpPhase::Upgrading | UpPhase::Error) {
                    div { class: "oxaik-up-offer",
                        div { class: "oxaik-row oxaik-between oxaik-up-price",
                            div { class: "oxaik-up-amt",
                                span { class: "oxaik-num oxaik-up-big", key: "{u.price()}", "${u.price()}" }
                                span { class: "oxaik-mute", " / month" }
                                p { class: "oxaik-mute oxaik-up-bill", "{u.billing()}" }
                            }
                            {seg("Billing", vec!["Monthly".into(), "Yearly".into()], usize::from(u.yearly), EventHandler::new(move |i| act(format!("up-int-{i}"))), EventHandler::new(move |k: String| { let x = up.write().key(&k, n); if let Some(x) = x { said.set(x) } }))}
                        }
                        ul { class: "oxaik-up-feat",
                            for (i, (f, marks)) in FEATURES.iter().enumerate() {
                                li { key: "{i}", {ico("check", 12.0)} span { "{f}" }
                                    span { class: "oxaik-row oxaik-up-mk", for m in marks.iter() { {mark(m, 14.0)} } }
                                }
                            }
                        }
                    }
                } else if ph == UpPhase::Upgraded {
                    dl { class: "oxaik-up-lims",
                        for (k, v) in LIMITS { div { key: "{k}", dt { "{k}" } dd { "{v}" } } }
                    }
                } else {
                    div { class: "oxaik-up-later",
                        p { class: "oxaik-row", {ico("bell", 13.0)} "We'll remind you a day before your {u.unit} reset." }
                    }
                }
            }
            div { class: "oxaik-row oxaik-up-acts",
                {match ph {
                    UpPhase::Offer | UpPhase::Upgrading => rsx! {
                        button { class: "oxaik-btn", r#type: "button", disabled: ph == UpPhase::Upgrading, onclick: move |_| act("up-later".into()), "Maybe later" }
                        button { class: "oxaik-btn pri oxaik-up-go", r#type: "button", disabled: ph == UpPhase::Upgrading, "aria-busy": "{ph == UpPhase::Upgrading}", onclick: move |_| act("up-upgrade".into()),
                            if ph == UpPhase::Upgrading { {dots()} "Upgrading" } else { "Upgrade to {PLAN.0} · ${u.due()}" } }
                    },
                    UpPhase::Error => rsx! {
                        button { class: "oxaik-btn", r#type: "button", onclick: move |_| act("up-later".into()), "Maybe later" }
                        button { class: "oxaik-btn pri oxaik-up-go", r#type: "button", onclick: move |_| act("up-retry".into()), {ico("retry", 12.0)} "Try again" }
                    },
                    UpPhase::Upgraded => rsx! { button { class: "oxaik-btn pri oxaik-up-go", r#type: "button", onclick: move |_| act("up-continue".into()), "Continue" } },
                    UpPhase::Later => rsx! { button { class: "oxaik-btn oxaik-up-go", r#type: "button", onclick: move |_| act("up-again".into()), "See {PLAN.0} again" } },
                }}
            }
            {live(&said())}
        }
    }
}

/// Referral card: a two-sided referral program in one card. Your personal link in a muted box (site
/// greyed, code in full) with Copy; posting buttons for X, Slack and email; an address field where
/// Enter, comma, space or Tab makes a chip (Backspace on an empty field drops the last one, Enter
/// on an empty field sends). Under it: invited, joined and credit totals, a segmented bonus bar,
/// and every invite with where it stands as a glyph and a label.
#[component]
pub fn ReferralCard(
    /// Host clock.
    now: ClockMs,
    /// Play the sample program (friends sign up and subscribe).
    #[props(default = true)]
    simulate: bool,
    /// The first send fails (error scenario).
    #[props(default)]
    fail: bool,
    /// Show the invite list.
    #[props(default = true)]
    list: bool,
) -> Element {
    let t0 = now.0;
    let mut rf = use_signal(move || Referral::new(simulate, fail, t0));
    let mut said = use_signal(String::new);
    let mut draft = use_signal(String::new);
    let n = now.0;
    let r = rf.read().clone();
    let mut act = move |id: String| {
        let x = rf.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let (inv, su, earned) = r.funnel(n);
    let paid = r.paid(n);
    let copied = r.copied.is_some_and(|c| n < c + 1_500);
    let busy = r.busy(n);
    let rows = r.rows(n);
    rsx! {
        section { class: "oxaik oxaik-rf", "aria-label": "Referrals",
            header { class: "oxaik-hl",
                h3 { class: "oxaik-h", "Give ${REWARD}, get ${REWARD}" }
                p { class: "oxaik-sub", "Friends get ${REWARD} off their first month, and you get ${REWARD} in credit when they subscribe." }
            }
            div { class: "oxaik-row oxaik-rf-link",
                div { class: "oxaik-rf-url", span { class: "oxaik-mute", "{LINK.0}" } b { "{LINK.1}" } }
                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("rf-copy".into()), {ico(if copied { "check" } else { "copy" }, 12.0)} if copied { "Copied" } else { "Copy" } }
                div { class: "oxaik-row oxaik-rf-share",
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Post on X", title: "Post on X", onclick: move |_| act("rf-share-x".into()), {mark("x", 13.0)} }
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Share in Slack", title: "Share in Slack", onclick: move |_| act("rf-share-slack".into()), {mark("slack", 14.0)} }
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Share by email", title: "Share by email", onclick: move |_| act("rf-share-email".into()), {ico("mail", 14.0)} }
                }
            }
            div { class: "oxaik-rf-inv",
                div { class: "oxaik-rf-field", "data-err": "{r.hint.1}",
                    div { class: "oxaik-rf-chips oxaik-scroll",
                        for (i, c) in r.chips.iter().enumerate() {
                            span { key: "{c}", class: "oxaik-rf-chip", "{c}"
                                button { r#type: "button", "aria-label": "Remove {c}", onclick: move |_| act(format!("rf-chip-{i}")), {ico("x", 10.0)} }
                            }
                        }
                        input { class: "oxaik-rf-in", r#type: "email", "aria-label": "Invite by email", placeholder: if r.chips.is_empty() { "name@company.com" } else { "" },
                            "aria-describedby": "oxaik-rf-hint", "aria-invalid": "{r.hint.1}", value: "{draft}",
                            oninput: move |e| {
                                let v = e.value();
                                if v.ends_with(',') || v.ends_with(' ') {
                                    let (x, clear) = rf.write().key(",", v.trim_end_matches([',', ' ']), n);
                                    if let Some(x) = x { said.set(x) }
                                    if clear { draft.set(String::new()) } else { draft.set(v) }
                                } else {
                                    draft.set(v)
                                }
                            },
                            onkeydown: move |e: KeyboardEvent| {
                                let k = key_of(&e);
                                if matches!(k.as_str(), "Enter" | "Tab" | "Backspace") {
                                    let d = draft();
                                    if k == "Backspace" && !d.is_empty() { return; }
                                    if k == "Tab" && d.trim().is_empty() { return; }
                                    e.prevent_default();
                                    let (x, clear) = rf.write().key(&k, &d, n);
                                    if let Some(x) = x { said.set(x) }
                                    if clear { draft.set(String::new()) }
                                }
                            },
                        }
                    }
                    button { class: "oxaik-btn sm pri", r#type: "button", disabled: busy, "aria-busy": "{busy}", onclick: move |_| {
                            let d = draft();
                            if !d.trim().is_empty() { let (_, c) = rf.write().key("Enter", &d, n); if c { draft.set(String::new()) } }
                            act("rf-send".into())
                        },
                        if busy { {dots()} "Sending" } else { "Send" } }
                }
                p { id: "oxaik-rf-hint", class: "oxaik-row oxaik-rf-hint", role: "status", "data-err": "{r.hint.1}",
                    if r.hint.1 { {ico("alert", 12.0)} } "{r.hint.0}" }
            }
            dl { class: "oxaik-rf-fun",
                div { dt { "Invited" } dd { class: "oxaik-num", "{inv}" } }
                div { dt { "Signed up" } dd { class: "oxaik-num", "{su}" } }
                div { dt { "Earned" } dd { class: "oxaik-num", "${earned}" } }
            }
            div { class: "oxaik-rf-ms",
                div { class: "oxaik-rf-segs", role: "img", "aria-label": "{paid} of {MILESTONE.0} paid referrals",
                    for i in 0..MILESTONE.0 { i { key: "{i}", class: cls("", i < paid), "data-done": "{paid >= MILESTONE.0}" } }
                }
                p { class: "oxaik-row oxaik-rf-msl", "data-done": "{paid >= MILESTONE.0}",
                    {ico(if paid >= MILESTONE.0 { "check-circle" } else { "star" }, 13.0)} span { key: "{r.milestone(n)}", class: "oxaik-fade", "{r.milestone(n)}" } }
            }
            if list {
                ul { class: "oxaik-rf-list",
                    for row in rows.into_iter().take(5) {
                        li { key: "{row.email}", "data-stage": "{row.stage:?}",
                            span { class: "oxaik-rf-who", b { if row.name.is_empty() { "{row.email}" } else { "{row.name}" } }
                                if !row.name.is_empty() { span { class: "oxaik-mute", "{row.email}" } } }
                            span { class: "oxaik-rf-stage", {ico(row.stage.icon(), 12.0)} span { key: "{row.stage.word()}", class: "oxaik-fade", "{row.stage.word()}" } }
                            span { class: "oxaik-mute oxaik-num oxaik-rf-when", "{row.when}" }
                        }
                    }
                }
            }
            {live(&said())}
        }
    }
}

fn ring(frac: f64, size: f32) -> Element {
    let r = 7.0;
    let dash = 2.0 * std::f64::consts::PI * r;
    rsx! {
        svg { class: "oxaik-tb-ring", width: "{size}", height: "{size}", view_box: "0 0 18 18", "aria-hidden": "true",
            circle { cx: "9", cy: "9", r: "{r}", class: "bg" }
            circle { cx: "9", cy: "9", r: "{r}", class: "fg", "stroke-dasharray": "{dash:.2}", "stroke-dashoffset": "{dash * (1.0 - frac):.2}" }
        }
    }
}

/// Trial banner: how much of a trial is left, what the workspace loses and the way to keep it,
/// as a slim `banner`, a sidebar `card` with the feature list, or a header `pill`. A ring counts
/// the days down (the days are always in the text); the last days switch to a warning tone that
/// names the end day; expired and upgraded have their own copy. Snooze (a menu with arrow keys)
/// and Dismiss fold it into the pill, which opens it again.
#[component]
pub fn TrialBanner(
    /// Host clock.
    now: ClockMs,
    /// `banner`, `card` or `pill`.
    #[props(default = "banner".to_string())]
    variant: String,
    /// Days remaining; zero or below means the trial is over.
    #[props(default = 9)]
    days_left: i64,
    /// The warning starts at this many days.
    #[props(default = 3)]
    warn_at: i64,
    /// Adding a card fails once (error scenario).
    #[props(default)]
    fail: bool,
) -> Element {
    let pill0 = variant == "pill";
    let mut tb = use_signal(move || Trial::new(days_left, warn_at, pill0, fail));
    let mut said = use_signal(String::new);
    let n = now.0;
    let t = tb.read().clone();
    let mut act = move |id: String| {
        let x = tb.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let tone = t.tone(n);
    let tone_s = match tone {
        TrialTone::Active => "active",
        TrialTone::Warning => "warning",
        TrialTone::Expired => "expired",
        TrialTone::Upgraded => "upgraded",
    };
    let (title, sub) = t.copy(n);
    let busy = t.busy(n);
    let failed = t.failed(n);
    let note = t.note_at(n);
    let card = variant == "card" || (t.folded && t.popover);
    let menu_open = t.menu.is_some();
    let actions = rsx! {
        div { class: "oxaik-row oxaik-tb-acts",
            if tone == TrialTone::Upgraded {
                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("tb-manage".into()), "Manage billing" }
            } else {
                button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("tb-compare".into()), "Compare plans" }
                button { class: "oxaik-btn sm pri", r#type: "button", disabled: busy, "aria-busy": "{busy}", onclick: move |_| act("tb-pay".into()),
                    if busy { {dots()} "Opening checkout" } else if failed { {ico("retry", 12.0)} "Try again" } else { "Add payment method" } }
                if tone != TrialTone::Expired {
                    div { class: "oxaik-menu-wrap",
                        button { class: "oxaik-ib", r#type: "button", "aria-label": "Snooze", "aria-haspopup": "menu", "aria-expanded": "{t.menu.is_some()}", onclick: move |_| act("tb-snooze".into()), {ico("clock", 13.0)} }
                        if let Some(sel) = t.menu {
                            ul { class: "oxaik-menu", role: "menu", "aria-label": "Snooze until",
                                for (i, s) in SNOOZES.iter().enumerate() {
                                    li { key: "{i}", role: "menuitem", tabindex: if i == sel { "0" } else { "-1" }, class: cls("oxaik-tb-mi", i == sel),
                                        onclick: move |_| act(format!("tb-snooze-{i}")), "{s}" }
                                }
                            }
                        }
                    }
                }
            }
            if !(t.folded && t.popover) {
                button { class: "oxaik-ib", r#type: "button", "aria-label": "Dismiss", onclick: move |_| act("tb-dismiss".into()), {ico("x", 13.0)} }
            }
        }
    };
    let body = if card {
        rsx! {
            section { class: "oxaik oxaik-tb oxaik-tb-card", "data-tone": "{tone_s}", "aria-label": "{TRIAL_PLAN} trial",
                div { class: "oxaik-row oxaik-tb-top", {ring(t.ring(n), 34.0)}
                    div { class: "oxaik-hl", b { class: "oxaik-tb-t", key: "{title}", "{title}" } span { class: "oxaik-mute", "{sub}" } } }
                if tone != TrialTone::Upgraded {
                    ul { class: "oxaik-tb-feat",
                        for (l, d, ic) in TRIAL_FEATURES {
                            li { key: "{l}", if ic == "lock" || ic == "users" { {ico(ic, 13.0)} } else { {mark(ic, 13.0)} } span { "{l}" } span { class: "oxaik-mute", "{d}" } }
                        }
                    }
                }
                {actions}
                if let Some(nt) = note { p { class: "oxaik-mute oxaik-tb-note", "{nt}" } }
            }
        }
    } else {
        rsx! {
            section { class: "oxaik oxaik-tb oxaik-tb-bar", "data-tone": "{tone_s}", "aria-label": "{TRIAL_PLAN} trial",
                div { class: "oxaik-row oxaik-tb-copy", {ring(t.ring(n), 24.0)}
                    div { class: "oxaik-hl", b { class: "oxaik-tb-t", key: "{title}", "{title}" } span { class: "oxaik-mute oxaik-tb-sub", "{sub}" } } }
                {actions}
                if let Some(nt) = note { p { class: "oxaik-mute oxaik-tb-note", "{nt}" } }
            }
        }
    };
    rsx! {
        div { class: "oxaik-tbw",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                if menu_open || k == "Escape" {
                    if matches!(k.as_str(), "ArrowUp" | "ArrowDown" | "Home" | "End" | "Enter" | "Escape") { e.prevent_default(); }
                    let x = tb.write().key(&k, n);
                    if let Some(x) = x { said.set(x) }
                }
            },
            if t.folded {
                div { class: "oxaik-tb-hdr",
                    span { class: "oxaik-mute", "Header" }
                    button { class: "oxaik-tb-pill", "data-tone": "{tone_s}", r#type: "button", "aria-expanded": "{t.popover}", onclick: move |_| act("tb-pill".into()),
                        {ring(t.ring(n), 16.0)} span { "{t.pill(n)}" }
                    }
                }
                if t.popover { div { class: "oxaik-tb-pop", {body} } }
                if !t.popover { if let Some(nt) = t.note_at(n) { p { class: "oxaik-mute oxaik-tb-note", "{nt}" } } }
            } else {
                {body}
            }
            {live(&said())}
        }
    }
}

fn flag_el(code: &str) -> Element {
    let (k, c) = flag(code);
    let bg = match k {
        "h" => format!(
            "linear-gradient(#{:06x} 0 33.4%,#{:06x} 0 66.7%,#{:06x} 0)",
            c[0], c[1], c[2]
        ),
        "v" => format!(
            "linear-gradient(90deg,#{:06x} 0 33.4%,#{:06x} 0 66.7%,#{:06x} 0)",
            c[0], c[1], c[2]
        ),
        "dot" => format!(
            "radial-gradient(circle at 50% 50%,#{:06x} 0 28%,#{:06x} 30%)",
            c[1], c[0]
        ),
        "cross" => format!(
            "linear-gradient(90deg,transparent 0 30%,#{:06x} 0 44%,transparent 0),linear-gradient(transparent 0 40%,#{:06x} 0 60%,#{:06x} 0)",
            c[1], c[1], c[0]
        ),
        _ => format!(
            "radial-gradient(circle at 62% 50%,#{:06x} 0 18%,transparent 20%),linear-gradient(90deg,#{:06x} 0 25%,#{:06x} 0)",
            c[1], c[1], c[0]
        ),
    };
    rsx! { i { class: "oxaik-lv-flag", title: "{code}", style: "background:{bg}", "aria-hidden": "true" } }
}

/// Live visitors: a realtime card for a marketing site. A live status (dot plus a word), a count
/// that rolls digit by digit in tabular numerals (read once as plain text), a dotted world map
/// where arrivals ping in and conversions ring stronger, the top pages with bars, and a ticker
/// of signups and upgrades with country flags. Pause holds the last snapshot; connecting and
/// reconnecting keep the same space.
#[component]
pub fn LiveVisitors(
    /// Host clock.
    now: ClockMs,
    /// `live`, `connecting` or `reconnecting`.
    #[props(default = "live")]
    connection: &'static str,
    /// Start paused.
    #[props(default)]
    paused: bool,
    /// Pages listed.
    #[props(default = 4)]
    max_pages: usize,
    /// Show the map.
    #[props(default = true)]
    map: bool,
) -> Element {
    let t0 = now.0;
    let mut lv = use_signal(move || Visitors::new(connection, paused, t0));
    let dots_land = use_hook(land);
    let mut said = use_signal(String::new);
    let n = now.0;
    let v = lv.read().clone();
    let mut act = move |id: String| {
        let x = lv.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let connecting = v.connection == "connecting";
    let count = v.count(n);
    let digits: Vec<char> = count.to_string().chars().collect();
    let pages = v.pages(n, max_pages);
    let top = pages.first().map(|p| p.1).unwrap_or(1).max(1);
    let evs = v.events(n, 4);
    let pings = v.pings(n);
    let k = v.tick(n);
    let st = v.status();
    let hue = match st {
        "Live" => "green",
        "Paused" => "neutral",
        _ => "amber",
    };
    let is_paused = v.paused.is_some();
    rsx! {
        section { class: "oxaik oxaik-lv", "aria-label": "Live visitors",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    span { class: "oxaik-row oxaik-lv-st", "data-hue": "{hue}", i { class: "oxaik-dot", "data-live": "{st == \"Live\"}" } b { "{st}" } }
                    if connecting {
                        p { class: "oxaik-lv-count", span { class: "oxaik-lv-ph" } span { class: "oxaik-mute", "Connecting to the stream" } }
                    } else {
                        p { class: "oxaik-lv-count",
                            span { class: "oxaik-sr", "{count} people on the site now" }
                            span { class: "oxaik-lv-digits oxaik-num", "aria-hidden": "true",
                                for (i, d) in digits.iter().enumerate() {
                                    span { key: "{digits.len() - i}", class: "oxaik-lv-dg", span { key: "{d}", class: "oxaik-lv-roll", "{d}" } }
                                }
                            }
                            span { class: "oxaik-mute", "aria-hidden": "true", "people on the site now" }
                        }
                    }
                }
                button { class: "oxaik-btn sm", r#type: "button", "aria-pressed": "{is_paused}", "aria-label": if is_paused { "Resume live updates" } else { "Pause live updates" }, onclick: move |_| act("lv-pause".into()),
                    {ico(if is_paused { "play" } else { "pause" }, 11.0)} if is_paused { "Resume" } else { "Pause" } }
            }
            if map {
                div { class: "oxaik-lv-map", "aria-hidden": "true", style: "aspect-ratio:{MAP.0} / {MAP.1}",
                    svg { view_box: "0 0 {MAP.0} {MAP.1}", preserve_aspect_ratio: "none",
                        for (c, r) in dots_land.iter() { circle { key: "{c}-{r}", cx: "{*c as f32 + 0.5}", cy: "{*r as f32 + 0.5}", r: "0.3" } }
                    }
                    if !connecting {
                        for (ci, t, conv) in pings {
                            {
                                let (x, y) = project(CITIES[ci].2, CITIES[ci].3);
                                let fresh = t == k;
                                rsx! { i { key: "{t}", class: if conv { "oxaik-lv-ping ev" } else { "oxaik-lv-ping" }, "data-fresh": "{fresh}",
                                    style: "left:{x * 100.0}%;top:{y * 100.0}%;opacity:{1.0 - (k - t) as f32 * 0.11}" } }
                            }
                        }
                    }
                }
            }
            div { class: "oxaik-lv-cols",
                div { class: "oxaik-lv-col",
                    h4 { class: "oxaik-h4", "Top pages" }
                    ol { class: "oxaik-lv-pages",
                        for (p, c) in pages {
                            li { key: "{p}",
                                i { class: "oxaik-lv-pbar", style: "width:{c as f64 / top as f64 * 100.0}%" }
                                span { class: "oxaik-lv-path", "{p}" }
                                span { class: "oxaik-num", "{c}" }
                            }
                        }
                    }
                }
                div { class: "oxaik-lv-col",
                    h4 { class: "oxaik-h4", "Conversions" }
                    ul { class: "oxaik-lv-evs",
                        for e in evs {
                            li { key: "{e.tick}",
                                {flag_el(CITIES[e.city].1)}
                                span { class: "oxaik-lv-el", b { "{e.label}" } span { class: "oxaik-mute", "{CITIES[e.city].0}, {CITIES[e.city].1}" } }
                                span { class: "oxaik-mute oxaik-num", "{v.ago(e.tick, n)}" }
                            }
                        }
                    }
                }
            }
            {live(&said())}
        }
    }
}

/// Social proof toast: small "someone just joined" cards for marketing pages. A card slides in at
/// the front on a timer while older ones tuck behind and fade; pointing at the pile or tabbing into
/// it freezes the timer (keeping its place) and spreads the cards. Every card has a close button
/// named after its content, and Escape closes the focused or front card. Arrivals are not spoken,
/// so nobody using a screen reader gets cut off.
#[component]
pub fn SocialProofToast(
    /// Host clock.
    now: ClockMs,
    /// Ms between toasts.
    #[props(default = 4_000)]
    interval: u64,
    /// Toasts stacked at once.
    #[props(default = 3)]
    max_visible: usize,
    /// Toasts already shown on mount.
    #[props(default = 1)]
    prefill: usize,
    /// Start over after the last event.
    #[props(default = true)]
    looping: bool,
    /// Fan the stack out on hover or focus.
    #[props(default = true)]
    expand_on_hover: bool,
    /// Show the header line (Oxyn extra).
    #[props(default = true)]
    header: bool,
) -> Element {
    let t0 = now.0;
    let mut sp = use_signal(move || {
        let mut p = Proof::new(interval, max_visible, prefill, looping, None, t0);
        p.fan = expand_on_hover;
        p
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let p = sp.read().clone();
    let mut act = move |id: String| {
        let x = sp.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let vis = p.visible(n);
    let fanned = p.paused.is_some() && p.fan;
    let age = p.age(n);
    rsx! {
        div { class: "oxaik-spw",
            if header { p { class: "oxaik-row oxaik-mute oxaik-pf-hd", {ico("users", 13.0)} "{p.week()}" } }
            section { class: "oxaik-pf", "aria-label": "Recent activity", "data-fan": "{fanned}", style: "--n:{vis.len()}",
                onmouseenter: move |_| act("sp-pause".into()),
                onmouseleave: move |_| act("sp-resume".into()),
                onfocusin: move |_| act("sp-pause".into()),
                onfocusout: move |_| act("sp-resume".into()),
                onkeydown: move |e: KeyboardEvent| { if key_of(&e) == "Escape" { act("sp-dismiss-front".into()) } },
                for (pos, seq) in vis.iter().copied().enumerate() {
                    {
                        let e = Proof::event(seq);
                        let fresh = pos == 0 && age < 400;
                        rsx! {
                            div { key: "{seq}", class: "oxaik-pf-t", "data-pos": "{pos}", "data-fresh": "{fresh}", style: "--i:{pos}",
                                span { class: "oxaik-pf-av", "aria-hidden": "true",
                                    if e.4.is_empty() { "{e.0.chars().next().unwrap_or('?')}" } else { {mark(e.4, 16.0)} }
                                }
                                div { class: "oxaik-pf-tx",
                                    p { class: "oxaik-pf-l1", b { "{e.0}" } " {e.1}" }
                                    p { class: "oxaik-pf-l2 oxaik-mute",
                                        if !e.2.is_empty() { span { "{e.2}\u{a0}·\u{a0}" } }
                                        span { class: "oxaik-num", "{p.when(seq, n)}" }
                                        if !e.5.is_empty() {
                                            span { class: "oxaik-pf-ver", {ico("check-circle", 11.0)} "{e.5}" if e.5.ends_with("Stripe") { {mark("stripe", 11.0)} } }
                                        }
                                    }
                                }
                                button { class: "oxaik-pf-x", r#type: "button", "aria-label": "Dismiss: {Proof::line(seq)}", onclick: move |_| act(format!("sp-dismiss-{seq}")), {ico("x", 11.0)} }
                            }
                        }
                    }
                }
                if vis.is_empty() { p { class: "oxaik-mute oxaik-pf-empty", "Waiting for activity" } }
            }
            {live(&said())}
        }
    }
}
