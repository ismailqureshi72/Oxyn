//! Screens S7 in GPUI: the Settings II pages (Members, Models, Security, Billing, Get started) drawn
//! inside the S6 settings shell (`setscr.rs`). Static snapshots of one state each, like the other
//! screen sets; presses fire `invite`, `add-key`, `rotate-key`, `setup-two-step`, `add-passkey`,
//! `sign-out-everyone`, `change-plan`, `update-card`, `cancel-plan` and `open-<page>` through
//! `on_act`. Rules come from `oxyn_core::parity::setscr2`.

use gpui::{Div, FontWeight, Hsla, ParentElement, SharedString, Styled, div, prelude::*, px};
use oxyn_core::icon_tone::IconTone;
use oxyn_core::parity::setscr::Prefs;
use oxyn_core::parity::setscr2::{
    ACCESS, AUDIT_KINDS, CAPS, INTERVALS, INVOICES, MODELS, PLANS, PROVIDERS, ROLES, SEATS,
    SESSION_LEN, STEPS, TEAMS, TOTP_SECRET, audit, dollars, mask, meter_tone, meter_words, meters,
    models_in, next_bill, progress, progress_words, provider_on, recovery_codes, rotated,
    seat_price, seats_used, seats_words, team_size,
};

use super::Act;
use super::chatscr::clip;
use super::rulerfid::{ThPal, fire};
use super::setscr::{Rows, av, hue, ic, menu, pill, sec, sw};
use super::workscr::{btn, quiet, tone};
use crate::brand::brand_mark;

fn tone_of(p: &ThPal, t: &str) -> Hsla {
    match t {
        "warn" => tone(p, IconTone::Approval),
        "ok" => tone(p, IconTone::Done),
        "bad" => tone(p, IconTone::Failed),
        "run" => tone(p, IconTone::Running),
        _ => p.mute,
    }
}

/// A meter bar `w` px wide.
fn bar(p: &ThPal, pct: f32, t: &str, w: f32) -> Div {
    let f = (pct.clamp(0.0, 1.0) * w).max(2.0);
    div()
        .flex_none()
        .w(px(w))
        .h(px(5.))
        .rounded(px(3.))
        .bg(p.line)
        .child(div().w(px(f)).h_full().rounded(px(3.)).bg(tone_of(p, t)))
}

fn press(p: &ThPal, l: &'static str, primary: bool, act: &Option<Act>, id: &'static str) -> Div {
    let f = fire(act, id);
    div().child(
        btn(p, l, primary)
            .id(SharedString::from(id))
            .on_click(move |_, w, cx| f(w, cx)),
    )
}

fn two(p: &ThPal, a: impl Into<SharedString>, b: impl Into<SharedString>) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .child(clip(a.into()))
        .child(div().text_size(px(12.)).text_color(p.mute).child(b.into()))
}

fn line_row(p: &ThPal) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .px(px(14.))
        .py(px(10.))
        .border_t_1()
        .border_color(p.line)
}

fn mark(name: &str, p: &ThPal, size: f32) -> Div {
    div()
        .w(px(30.))
        .h(px(30.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(8.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .children(brand_mark(name, size, p.fg))
}

fn tile(p: &ThPal, k: &'static str, v: String, sub: Div) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(3.))
        .px(px(14.))
        .py(px(12.))
        .child(quiet(p, k))
        .child(
            div()
                .text_size(px(18.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(v),
        )
        .child(sub)
}

fn boxed(p: &ThPal, kids: Vec<Div>) -> Div {
    let mut d = div()
        .mt(px(10.))
        .flex()
        .rounded(px(12.))
        .border_1()
        .border_color(p.line)
        .bg(p.card);
    for (i, k) in kids.into_iter().enumerate() {
        d = d.child(if i > 0 {
            k.border_l_1().border_color(p.line)
        } else {
            k
        });
    }
    d
}

pub(super) fn members(
    p: &ThPal,
    _r: &Rows,
    d: &Prefs,
    inviting: bool,
    act: &Option<Act>,
) -> Vec<Div> {
    let w = &d.ws;
    let joined = w.members.iter().filter(|m| !m.invited).count();
    let used = seats_used(&w.members);
    let admins = w
        .members
        .iter()
        .filter(|m| m.role <= 1 && !m.invited)
        .count();
    let mut out = vec![
        div()
            .mt(px(10.))
            .flex()
            .gap(px(14.))
            .text_size(px(12.5))
            .child(div().font_weight(FontWeight::MEDIUM).child("Members"))
            .child(div().text_color(p.mute).child("Teams")),
        boxed(
            p,
            vec![
                tile(
                    p,
                    "Members",
                    joined.to_string(),
                    quiet(p, format!("{joined} joined · {} invited", used - joined)),
                ),
                tile(
                    p,
                    "Seats",
                    format!("{used} of {SEATS}"),
                    bar(
                        p,
                        used as f32 / SEATS as f32,
                        meter_tone(used as f32, SEATS as f32),
                        150.,
                    ),
                ),
                tile(
                    p,
                    "Admins",
                    admins.to_string(),
                    quiet(p, "Can invite, remove and bill"),
                ),
            ],
        ),
        div()
            .mt(px(12.))
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .w(px(240.))
                    .h(px(30.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card)
                    .text_color(p.mute)
                    .child("Search name or email"),
            )
            .child(menu(p, "All roles"))
            .child(div().flex_1())
            .child(btn(p, "Export", false))
            .child(press(p, "Invite members", true, act, "invite")),
    ];
    if inviting {
        out.push(
            div()
                .mt(px(8.))
                .p(px(12.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .rounded(px(10.))
                .bg(p.inset)
                .child(quiet(p, "Email addresses"))
                .child(
                    div()
                        .p(px(10.))
                        .rounded(px(9.))
                        .border_1()
                        .border_color(p.line)
                        .bg(p.card)
                        .child("lena@kestrel.example, arlo@kestrel.example"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(quiet(p, "Role"))
                        .child(menu(p, "Member"))
                        .child(quiet(p, ROLES[2].1)),
                )
                .child(quiet(p, seats_words(&w.members)))
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .gap(px(8.))
                        .child(btn(p, "Cancel", false))
                        .child(btn(p, "Send invites", true)),
                ),
        );
    }
    let mut table = div()
        .mt(px(8.))
        .flex()
        .flex_col()
        .rounded(px(12.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .child(
            div()
                .flex()
                .px(px(14.))
                .py(px(7.))
                .text_size(px(12.))
                .text_color(p.mute)
                .child(div().flex_1().child("Name"))
                .child(div().w(px(120.)).child("Role"))
                .child(div().w(px(100.)).child("Team"))
                .child(div().w(px(130.)).child("Usage")),
        );
    for m in &w.members {
        let who = if m.invited {
            av(p, "@", 30.)
        } else {
            av(p, oxyn_core::parity::setscr::initials(m.name), 30.)
                .text_color(hue(p, m.id as usize % 6))
        };
        let label = m.label().to_string();
        let sub = if m.invited {
            m.seen.to_string()
        } else {
            m.email.clone()
        };
        let name = if m.is_me() {
            format!("{label} (you)")
        } else {
            label
        };
        let mut row = line_row(p).child(who).child(two(p, name, sub));
        if m.invited {
            row = row.child(pill(p, "Invited", IconTone::Approval));
        }
        let usage = if m.invited {
            quiet(p, "—")
        } else {
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(bar(
                    p,
                    f32::from(m.usage) / 100.,
                    meter_tone(f32::from(m.usage), 100.),
                    80.,
                ))
                .child(quiet(p, format!("{}%", m.usage)))
        };
        row = row
            .child(div().w(px(120.)).child(menu(p, ROLES[m.role].0)))
            .child(
                div()
                    .w(px(100.))
                    .text_color(p.mute)
                    .child(m.team.map(|t| TEAMS[t].0).unwrap_or("—")),
            )
            .child(div().w(px(130.)).child(usage))
            .child(quiet(p, "⋯"));
        table = table.child(row);
    }
    out.push(table);
    let mut teams = div().mt(px(10.)).flex().gap(px(10.));
    for (i, (n, _)) in TEAMS.iter().enumerate() {
        teams = teams.child(
            div()
                .flex_1()
                .p(px(12.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .flex()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .w(px(8.))
                        .h(px(8.))
                        .rounded(px(3.))
                        .bg(hue(p, [0, 1, 2][i] + 1)),
                )
                .child(div().font_weight(FontWeight::SEMIBOLD).child(*n))
                .child(quiet(p, format!("{} people", team_size(&w.members, i)))),
        );
    }
    out.push(teams);
    out
}

pub(super) fn models(p: &ThPal, r: &Rows, d: &Prefs, newkey: bool, act: &Option<Act>) -> Vec<Div> {
    let w = &d.ws;
    let chat = w.chat_default.map(|i| MODELS[i].0).unwrap_or("Auto");
    let image = w.image_default.map(|i| MODELS[i].0).unwrap_or("Off");
    let mut out = vec![sec(
        p,
        "Defaults",
        None,
        vec![
            r.row(
                "Chat model",
                "Every new chat starts with it; people can pick their own.",
                menu(p, chat),
            ),
            r.row(
                "Image model",
                "Used when someone asks for a picture.",
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .children(
                        w.image_default
                            .and_then(|i| brand_mark(MODELS[i].3, 14., p.fg)),
                    )
                    .child(menu(p, image)),
            ),
        ],
    )];
    for (pi, (pb, pn, ph)) in PROVIDERS.iter().enumerate() {
        let (on, all) = provider_on(&w.models_on, pi);
        let mut rows = vec![
            line_row(p)
                .bg(p.card2)
                .child(mark(pb, p, 16.))
                .child(two(p, *pn, *ph))
                .child(quiet(p, format!("{on}/{all} on"))),
        ];
        for i in models_in(pi, "all", "") {
            let (mn, _, _, mk, ml) = MODELS[i];
            rows.push(
                line_row(p)
                    .pl(px(18.))
                    .child(
                        div()
                            .w(px(22.))
                            .flex_none()
                            .children(brand_mark(mk, 15., p.fg)),
                    )
                    .child(two(p, mn, ml))
                    .child(menu(p, ACCESS[w.access[i]]))
                    .child(sw(p, w.models_on[i])),
            );
        }
        let mut card = div()
            .mt(px(10.))
            .flex()
            .flex_col()
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .overflow_hidden();
        for row in rows {
            card = card.child(row);
        }
        out.push(card);
    }
    let mut keys: Vec<Div> = Vec::new();
    if w.keys.is_empty() {
        keys.push(
            div()
                .p(px(18.))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(4.))
                .child(div().font_weight(FontWeight::SEMIBOLD).child("No API keys"))
                .child(quiet(
                    p,
                    "Usage runs on your plan until you add a provider's key.",
                ))
                .child(press(p, "Add key", false, act, "add-key")),
        );
    }
    for (i, k) in w.keys.iter().enumerate() {
        let fresh = newkey && i == 0;
        let secret = if fresh {
            rotated(&k.secret, 1)
        } else {
            mask(&k.secret)
        };
        keys.push(
            line_row(p)
                .child(mark(PROVIDERS[k.prov].0, p, 16.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(clip(k.name.clone()))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(if fresh { p.fg } else { p.fg2 })
                                .child(secret),
                        )
                        .child(quiet(
                            p,
                            format!(
                                "{} · added {} · used {}",
                                PROVIDERS[k.prov].1, k.added, k.used
                            ),
                        )),
                )
                .child(quiet(p, if fresh { "Hide  Copy" } else { "Reveal  Copy" }))
                .child(press(p, "Rotate", false, act, "rotate-key")),
        );
        if fresh {
            keys.push(
                div()
                    .mx(px(14.))
                    .mb(px(10.))
                    .p(px(8.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(tone(p, IconTone::Approval).opacity(0.5))
                    .text_size(px(12.5))
                    .child("New key: copy it now. Once you leave this page it shows masked."),
            );
        }
    }
    out.push(sec(p, "API keys", Some("+ Add key".to_string()), keys));
    out
}

pub(super) fn security(p: &ThPal, r: &Rows, d: &Prefs, setup: bool, act: &Option<Act>) -> Vec<Div> {
    let w = &d.ws;
    let mut mine = vec![
        line_row(p)
            .child(ic(p, "🔒", p.fg2))
            .child(two(
                p,
                "Two-step sign-in",
                if w.two_step_me {
                    "On: a code from your authenticator app after your password."
                } else {
                    "Off: add a code from an authenticator app."
                },
            ))
            .child(if w.two_step_me {
                pill(p, "On", IconTone::Done)
            } else {
                press(p, "Set up", true, act, "setup-two-step")
            }),
    ];
    if setup {
        let mut qr = div()
            .flex_none()
            .w(px(132.))
            .h(px(132.))
            .p(px(6.))
            .rounded(px(10.))
            .bg(gpui::rgb(0xFFFFFF))
            .flex()
            .flex_col();
        let cells = oxyn_core::parity::setscr2::qr_cells(3);
        for y in 0..21 {
            let mut rw = div().flex().h(px(120. / 21.));
            for x in 0..21 {
                rw = rw.child(
                    div()
                        .w(px(120. / 21.))
                        .h_full()
                        .when(cells[y * 21 + x], |d| d.bg(gpui::rgb(0x111111))),
                );
            }
            qr = qr.child(rw);
        }
        mine.push(
            div()
                .mx(px(14.))
                .mb(px(12.))
                .p(px(12.))
                .rounded(px(10.))
                .bg(p.inset)
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(quiet(p, "1 Scan   ·   2 Enter code   ·   3 Save codes"))
                .child(
                    div().flex().gap(px(16.)).child(qr).child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Scan with your authenticator app"),
                            )
                            .child(quiet(p, "Can't scan? Type this key:"))
                            .child(div().child(TOTP_SECRET))
                            .child(
                                div()
                                    .flex()
                                    .gap(px(8.))
                                    .child(btn(p, "Copy key", false))
                                    .child(btn(p, "Next", true)),
                            ),
                    ),
                ),
        );
    }
    if w.two_step_me {
        let codes = recovery_codes(7);
        mine.push(r.row(
            "Recovery codes",
            &format!("{} left. New codes replace the old ones.", codes.len()),
            btn(p, "New codes", false),
        ));
    }
    mine.push(r.row(
        "Passkeys",
        "Sign in with your fingerprint, face or screen lock.",
        press(p, "Add passkey", false, act, "add-passkey"),
    ));
    for k in &w.passkeys {
        mine.push(
            line_row(p)
                .child(ic(p, "◉", p.fg2))
                .child(two(
                    p,
                    k.name.clone(),
                    format!("Passkey · added {} · used {}", k.added, k.used),
                ))
                .child(btn(p, "Remove", false)),
        );
    }
    let mut domains: Vec<Div> = Vec::new();
    for dm in &w.domains {
        domains.push(
            line_row(p)
                .child(ic(p, "◍", p.fg2))
                .child(two(
                    p,
                    dm.name.clone(),
                    if dm.verified {
                        "Members join with this address".to_string()
                    } else {
                        format!(
                            "Add a TXT record {} to its DNS, then Verify.",
                            oxyn_core::parity::setscr2::txt_record(&dm.name)
                        )
                    },
                ))
                .child(if dm.verified {
                    pill(p, "Verified", IconTone::Done)
                } else {
                    pill(p, "Pending", IconTone::Approval)
                }),
        );
    }
    if domains.is_empty() {
        domains.push(r.row(
            "No domains yet",
            "Verify the domain your team signs in with.",
            btn(p, "Add domain", false),
        ));
    }
    let mut devs: Vec<Div> = Vec::new();
    for dv in &w.devices {
        devs.push(
            line_row(p)
                .child(ic(p, if dv.glyph == "phone" { "▯" } else { "▭" }, p.fg2))
                .child(two(
                    p,
                    dv.who,
                    format!("{} · {} · {}", dv.device, dv.place, dv.seen),
                ))
                .child(if dv.id == 1 {
                    pill(p, "This device", IconTone::Done)
                } else {
                    btn(p, "Sign out", false)
                }),
        );
    }
    let mut chips = div().flex().gap(px(6.)).px(px(12.)).py(px(10.));
    for (i, (_, l)) in AUDIT_KINDS.iter().enumerate() {
        let on = i == 0;
        chips = chips.child(
            div()
                .h(px(24.))
                .px(px(10.))
                .flex()
                .items_center()
                .rounded(px(12.))
                .border_1()
                .border_color(if on { p.fg } else { p.line })
                .when(on, |d| d.bg(p.fg).text_color(p.card))
                .text_size(px(12.))
                .child(*l),
        );
    }
    let mut log = vec![chips];
    for (_, who, what, when, place, t) in audit("all", !w.domains.is_empty()) {
        log.push(
            line_row(p)
                .child(div().w(px(7.)).h(px(7.)).rounded(px(4.)).bg(tone_of(p, t)))
                .child(two(p, what, format!("{who} · {place}")))
                .child(quiet(p, when)),
        );
    }
    vec![
        sec(p, "Your sign-in", None, mine),
        sec(
            p,
            "Workspace sign-in",
            None,
            vec![
                r.row(
                    "Require two-step for everyone",
                    "People without it set it up at their next sign-in",
                    sw(p, w.two_step_all),
                ),
                r.row(
                    "Require single sign-on",
                    "People with a verified domain sign in through your identity provider",
                    sw(p, w.sso),
                ),
                r.row(
                    "Session length",
                    "How long people stay signed in without activity",
                    menu(p, SESSION_LEN[w.session_len]),
                ),
                r.row(
                    "Join by domain",
                    "Anyone with a verified address joins as a Member",
                    sw(p, w.join_domain),
                ),
            ],
        ),
        sec(
            p,
            "Email domains",
            Some("+ Add domain".to_string()),
            domains,
        ),
        sec(
            p,
            "Signed-in devices",
            Some("Sign out everyone else".to_string()),
            devs,
        ),
        sec(p, "Audit log", Some("Export".to_string()), log),
    ]
}

pub(super) fn billing(
    p: &ThPal,
    r: &Rows,
    d: &Prefs,
    cancelling: bool,
    act: &Option<Act>,
) -> Vec<Div> {
    let w = &d.ws;
    let seats = seats_used(&w.members);
    let plan = PLANS[w.plan];
    let price = seat_price(w.plan, w.yearly);
    let head = boxed(
        p,
        vec![
            div()
                .flex_1()
                .flex()
                .items_center()
                .gap(px(12.))
                .p(px(14.))
                .child(ic(p, "✦", tone(p, IconTone::Skills)))
                .child(two(
                    p,
                    format!("{} plan", plan.1),
                    if w.plan == 0 {
                        "Free for one seat".to_string()
                    } else {
                        format!(
                            "{} per seat / month, billed {}",
                            dollars(price),
                            INTERVALS[usize::from(w.yearly)].to_lowercase()
                        )
                    },
                )),
            div()
                .flex_1()
                .flex()
                .items_center()
                .gap(px(12.))
                .p(px(14.))
                .child(two(
                    p,
                    "Upcoming bill",
                    format!("Renews on 1 Dec 2026 · {seats} seats"),
                ))
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(dollars(next_bill(w.plan, w.yearly, seats))),
                ),
        ],
    );
    let mut ms: Vec<Div> = Vec::new();
    for (l, u, lim, unit, _) in meters(w.card.is_some(), seats) {
        ms.push(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(6.))
                .p(px(12.))
                .child(quiet(p, l))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(meter_words(u, lim, unit)),
                )
                .child(bar(p, u / lim, meter_tone(u, lim), 140.)),
        );
    }
    let mut plans = div().flex().gap(px(10.)).p(px(10.));
    for (i, (_, n, _, line, feats)) in PLANS.iter().enumerate() {
        let cur = i == w.plan;
        let mut c = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(8.))
            .p(px(14.))
            .rounded(px(12.))
            .border_1()
            .border_color(if cur { p.fg } else { p.line })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(*n))
                    .when(cur, |d| d.child(pill(p, "Current plan", IconTone::Doc))),
            )
            .child(quiet(p, *line))
            .child(
                div()
                    .text_size(px(22.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(dollars(seat_price(i, w.yearly))),
            );
        for f in feats {
            c = c.child(
                div()
                    .flex()
                    .gap(px(6.))
                    .text_size(px(12.5))
                    .text_color(p.fg2)
                    .child(div().text_color(tone(p, IconTone::Done)).child("✓"))
                    .child(*f),
            );
        }
        if !cur {
            c = c.child(press(
                p,
                if i > w.plan { "Upgrade" } else { "Switch" },
                i > w.plan,
                act,
                "change-plan",
            ));
        }
        plans = plans.child(c);
    }
    let mut inv: Vec<Div> = Vec::new();
    if w.card.is_none() {
        inv.push(r.row(
            "No invoices yet",
            "They appear here after your first paid month.",
            div(),
        ));
    } else {
        for (no, date, amt, st) in INVOICES {
            inv.push(
                line_row(p)
                    .child(div().w(px(130.)).child(no))
                    .child(div().flex_1().text_color(p.mute).child(date))
                    .child(div().w(px(70.)).child(dollars(amt)))
                    .child(div().w(px(80.)).child(pill(
                        p,
                        st,
                        match st {
                            "Paid" => IconTone::Done,
                            "Due" => IconTone::Approval,
                            _ => IconTone::Doc,
                        },
                    )))
                    .child(quiet(p, "↓")),
            );
        }
    }
    let card = match &w.card {
        Some(l) => format!("Card ending {l}"),
        None => "No card yet".to_string(),
    };
    let mut danger = vec![r.row(
        "Cancel plan",
        "You keep the plan until this billing period ends, then move to Free.",
        press(p, "Cancel plan", false, act, "cancel-plan"),
    )];
    if cancelling {
        let mut flow = div()
            .mx(px(14.))
            .mb(px(12.))
            .p(px(12.))
            .rounded(px(10.))
            .bg(p.inset)
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(quiet(p, "1 Why   ·   2 What changes"));
        for (i, reason) in oxyn_core::parity::setscr2::CANCEL_REASONS
            .iter()
            .enumerate()
        {
            flow = flow.child(
                div()
                    .h(px(30.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .rounded(px(9.))
                    .border_1()
                    .border_color(if i == 0 { p.fg } else { p.line })
                    .bg(p.card)
                    .child(if i == 0 { "◉" } else { "○" })
                    .child(*reason),
            );
        }
        flow = flow.child(
            div()
                .flex()
                .justify_end()
                .gap(px(8.))
                .child(btn(p, "Keep my plan", false))
                .child(btn(p, "Continue", true)),
        );
        danger.push(flow);
    }
    let mut out = vec![
        head,
        boxed(p, ms),
        sec(
            p,
            "Plans",
            Some(format!(
                "{}  ·  Yearly −20%",
                INTERVALS[usize::from(w.yearly)]
            )),
            vec![plans],
        ),
        sec(
            p,
            "Extra usage",
            None,
            vec![
                r.row(
                    "Monthly cap",
                    "When credits run out, agents keep running up to this amount",
                    menu(p, CAPS[w.cap]),
                ),
                r.row("Email invoices", "To the owners", sw(p, w.invoice_mail)),
            ],
        ),
        sec(
            p,
            "Payment method",
            None,
            vec![r.row(
                card,
                "Expires 08/28 · charged on the 1st",
                press(p, "Update", false, act, "update-card"),
            )],
        ),
        sec(p, "Invoices", None, inv),
    ];
    if w.plan > 0 {
        out.push(sec(p, "Danger zone", None, danger));
    }
    out
}

pub(super) fn start(p: &ThPal, _r: &Rows, d: &Prefs, act: &Option<Act>) -> Vec<Div> {
    let w = &d.ws;
    let (_, _, pct) = progress(&w.steps);
    let all = pct >= 100;
    let ring = if all {
        tone(p, IconTone::Done)
    } else {
        tone(p, IconTone::Running)
    };
    let head = div()
        .mt(px(10.))
        .flex()
        .items_center()
        .gap(px(14.))
        .p(px(14.))
        .rounded(px(12.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .child(
            div()
                .w(px(56.))
                .h(px(56.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(28.))
                .border_4()
                .border_color(ring)
                .font_weight(FontWeight::SEMIBOLD)
                .child(format!("{pct}%")),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(if all {
                    "You are all set"
                } else {
                    "Set up Kestrel Studio"
                }))
                .child(quiet(p, progress_words(&w.steps)))
                .child(bar(
                    p,
                    pct as f32 / 100.,
                    if all { "ok" } else { "run" },
                    320.,
                )),
        );
    let mut list = div()
        .mt(px(12.))
        .flex()
        .flex_col()
        .rounded(px(12.))
        .border_1()
        .border_color(p.line)
        .bg(p.card);
    let next = oxyn_core::parity::setscr2::next_step(&w.steps);
    for (i, (_, title, line, _, page)) in STEPS.iter().enumerate() {
        let done = w.steps[i];
        let id: &'static str = match *page {
            "profile" => "open-profile",
            "general" => "open-general",
            "members" => "open-members",
            "models" => "open-models",
            "security" => "open-security",
            _ => "open-billing",
        };
        let mut row = line_row(p)
            .when(next == Some(i), |d| d.bg(p.card2))
            .child(
                div()
                    .w(px(24.))
                    .h(px(24.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(12.))
                    .text_size(px(11.5))
                    .when(done, |d| {
                        d.bg(tone(p, IconTone::Done)).text_color(gpui::white())
                    })
                    .when(!done, |d| {
                        d.border_1().border_color(p.line).text_color(p.mute)
                    })
                    .child(if done {
                        "✓".to_string()
                    } else {
                        (i + 1).to_string()
                    }),
            )
            .child(two(p, *title, *line).when(done, |d| d.text_color(p.mute)));
        if !done {
            row = row.child(press(p, "Open", next == Some(i), act, id));
        }
        row = row.child(btn(p, if done { "Undo" } else { "Mark done" }, false));
        list = list.child(row);
    }
    vec![head, list]
}
