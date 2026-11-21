//! Screens S15 (Arc screens IV, the admin demo app) in GPUI: [`AdminOverviewEl`],
//! [`AuditLogEl`], [`StatusBoardEl`], [`ApiKeysWebhooksEl`], [`MembersRolesEl`],
//! [`BillingUsageEl`] and [`AgentToolCallsEl`] on the Arc frame of S12, over
//! `oxyn_core::kit::arcscr4`.
//!
//! Each element draws one state (`.state()`, `.extra()`); with `.arc4(Arc4Ui)` it draws that
//! live state instead of the sample, and its presses (`ov-*`, `au-*`, `st-*`, `ak-*`, `wh-*`,
//! `mb-*`, `bl-*`, `tc-*`) go back through `on_act` for the host to apply with
//! [`oxyn_core::kit::arcscr4::Arc4Ui::press`].

use gpui::{
    AnyElement, App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px, relative,
};
use oxyn_core::kit::arcscr4::{
    AU_TYPES, Arc4Ui, Audit, Billing, COMPONENTS, CallSt, HOOK_EVENTS, Keys, Members, Mode,
    OV_KPIS, OV_RANGES, Overview, PAST, PEOPLE, PLANS, ROLES, SCOPES, STATUS_DAYS, ScreenState,
    Sel, Status, TC_AGENT, TC_RANGES, TC_TOOLS, TODAY, ToolCalls, day_health, dur, hm, hms, ids,
    iso, member_email, money, money0, open_incident, ov_delta, ov_label, ov_series, ov_value, pct,
    pct2, short_n, thousands, today_health, uptime,
};
use oxyn_core::kit::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::apps::hit;
use super::arcscr::{
    Frame, av, btn, card, chip, frame, glyph, group, hue, icon_for, nrow, q, seg, skel, tint,
};
use super::chatscr::clip;
use super::rulerfid::{ThPal, th_pal};
use crate::brand::brand_mark;

macro_rules! arc4_el {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(IntoElement)]
        pub struct $name {
            state: ScreenState,
            extra: &'static str,
            layout: ShellLayout,
            act: Option<Act>,
            theme: &'static Theme,
            arc: Option<Arc4Ui>,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { state: ScreenState::Sample, extra: "", layout: ShellLayout::Inset, act: None, theme: &Theme::DARK, arc: None }
            }
        }
        impl $name {
            /// Sample, Loading or Empty.
            pub fn state(mut self, s: ScreenState) -> Self {
                self.state = s;
                self
            }
            /// The screen's own state (see `oxyn_core::kit::arcscr4::s15_states`).
            pub fn extra(mut self, k: &'static str) -> Self {
                self.extra = k;
                self
            }
            /// Inset or Flat.
            pub fn layout(mut self, l: ShellLayout) -> Self {
                self.layout = l;
                self
            }
            /// Light or dark chrome.
            pub fn theme(mut self, t: &'static Theme) -> Self {
                self.theme = t;
                self
            }
            /// Press ids (`nav-<key>`, `acct`, and the screen's own ids).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
            /// Draw this live state instead of the sample.
            pub fn arc4(mut self, a: Arc4Ui) -> Self {
                self.arc = Some(a);
                self
            }
        }
    };
}

arc4_el!(
    /// Overview: KPI tabs over a range, the chart with a readout, components and activity.
    AdminOverviewEl
);
arc4_el!(
    /// Audit log: events by type with failures, an event's before and after, show more.
    AuditLogEl
);
arc4_el!(
    /// Status: the overall state, 90 days per component, the open incident and past ones.
    StatusBoardEl
);
arc4_el!(
    /// API keys and webhooks: keys with roll and revoke, the secret once; endpoints and deliveries.
    ApiKeysWebhooksEl
);
arc4_el!(
    /// Members: roles, invites, seats, remove pressed twice.
    MembersRolesEl
);
arc4_el!(
    /// Billing: plans with the prorated change, usage, card and invoices.
    BillingUsageEl
);
arc4_el!(
    /// Agent / Tool calls: the agent and its tools as an org chart, the calls, the panel.
    AgentToolCallsEl
);

const NAV: [(&str, &str, &str); 6] = [
    ("admin-overview", "nav-overview", "Overview"),
    ("audit-log", "nav-audit", "Audit log"),
    ("status-board", "nav-status", "Status"),
    ("api-keys-webhooks", "nav-keys", "API keys"),
    ("members-roles", "nav-members", "Members"),
    ("billing-usage", "nav-billing", "Billing"),
];

fn logo(p: &ThPal, brand: &str, size: f32) -> AnyElement {
    brand_mark(brand, size, p.fg).unwrap_or_else(|| glyph(p, "•").into_any_element())
}

fn side(p: &ThPal, act: &Option<Act>, cur: &str) -> Div {
    let icons = ["home", "list", "chart", "plug", "people", "card"];
    let mut d = div().flex().flex_col().gap(px(1.));
    for (i, (slug, id, l)) in NAV.iter().copied().enumerate() {
        d = d.child(nrow(
            p,
            act,
            id,
            glyph(p, icon_for(icons[i])),
            l,
            "",
            slug == cur,
        ));
    }
    d.child(group(p, "Agents")).child(
        hit(div(), act, "nav-tools")
            .flex_none()
            .h(px(32.))
            .px(px(8.))
            .flex()
            .items_center()
            .gap(px(9.))
            .rounded(px(8.))
            .when(cur == "agent-tool-calls", |d| d.bg(p.sel))
            .child(logo(p, "claude", 14.))
            .child(clip(TC_AGENT.0)),
    )
}

fn plan_ctx(p: &ThPal) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(q(p, "Team plan · 9 of 10 seats"))
        .child(bar(p, 0.9, false))
        .child(q(p, format!("Renews {}", iso(TODAY + 18))))
}

fn bar(p: &ThPal, f: f32, warn: bool) -> Div {
    div().h(px(6.)).w_full().rounded(px(3.)).bg(p.sel).child(
        div()
            .h_full()
            .w(relative(f.clamp(0., 1.)))
            .rounded(px(3.))
            .bg(if warn { hue(p, "amber") } else { p.fg }),
    )
}

fn pill(p: &ThPal, h: &str, t: impl Into<SharedString>) -> Div {
    div()
        .flex_none()
        .h(px(20.))
        .px(px(7.))
        .flex()
        .items_center()
        .gap(px(5.))
        .rounded(px(10.))
        .border_1()
        .border_color(p.line)
        .text_size(px(11.5))
        .text_color(p.fg2)
        .child(div().size(px(6.)).rounded_full().bg(hue(p, h)))
        .child(t.into())
}

fn h2(t: impl Into<SharedString>) -> Div {
    div()
        .font_weight(FontWeight::SEMIBOLD)
        .text_size(px(13.5))
        .child(t.into())
}

fn bx(p: &ThPal) -> Div {
    card(p).p(px(14.)).flex().flex_col().gap(px(8.)).min_w_0()
}

fn line(p: &ThPal) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(9.))
        .min_h(px(36.))
        .min_w_0()
        .border_b_1()
        .border_color(p.line)
}

fn body(_: &ThPal) -> Div {
    div().p(px(18.)).flex().flex_col().gap(px(14.)).min_w_0()
}

fn none(p: &ThPal, t: &'static str, l: &'static str) -> Div {
    super::arcscr::none(p, t, l)
}

fn meter(p: &ThPal, l: &'static str, used: String, cap: String, f: f32) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(
            div()
                .flex()
                .child(l)
                .child(div().flex_1())
                .child(q(p, format!("{used} of {cap}"))),
        )
        .child(bar(p, f, f >= 0.9))
}

#[allow(clippy::too_many_arguments)]
fn fr(
    p: &ThPal,
    act: &Option<Act>,
    lay: ShellLayout,
    slug: &'static str,
    crumb: &str,
    meta: String,
    actions: Div,
    bar: Option<Div>,
    body: Div,
    right: Option<Div>,
) -> Div {
    frame(
        p,
        lay,
        act,
        Frame {
            slug,
            side: side(p, act, slug),
            ctx: Some(plan_ctx(p)),
            crumbs: vec!["Corvel Labs".into(), crumb.into()],
            meta,
            actions,
            bar,
            body,
            foot: None,
            right,
        },
    )
}

fn live(a: &Option<Arc4Ui>) -> Arc4Ui {
    a.clone().unwrap_or_default()
}

// ───────────────────────────────────────────────────────────── overview ──

impl RenderOnce for AdminOverviewEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.ov = Overview::new(st != ScreenState::Empty);
            u.ov.range = 2;
            match self.extra {
                "week" => u.ov.range = 1,
                "incident" => u.ov.incident = true,
                "errors" => u.ov.kpi = 1,
                _ => {}
            }
        }
        let o = &u.ov;
        let ranges = OV_RANGES
            .iter()
            .enumerate()
            .map(|(i, r)| chip(&p, &act, ids::at(ids::OV_RANGE, i), r.0, i == o.range))
            .collect();
        let actions =
            div()
                .child(seg(&p, ranges))
                .child(btn(&p, &act, "ov-export", "Export", false));
        let b = if st == ScreenState::Loading {
            body(&p).child(skel(&p, 10))
        } else if st == ScreenState::Empty || !o.sample {
            body(&p).child(none(
                &p,
                "No traffic yet",
                "Send your first API request and the numbers land here.",
            ))
        } else {
            let ser = ov_series(o.kpi, o.range);
            let hi = ser.iter().copied().max().unwrap_or(1).max(1) as f32;
            let mut kpis = div().flex().gap(px(12.));
            for (k, (_, l)) in OV_KPIS.iter().copied().enumerate() {
                let (bp, good) = ov_delta(k, o.range);
                kpis = kpis.child(
                    hit(card(&p), &act, ids::at(ids::OV_KPI, k))
                        .flex_1()
                        .min_w_0()
                        .p(px(12.))
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .when(k == o.kpi, |d| d.border_color(p.fg))
                        .child(q(&p, l))
                        .child(
                            div()
                                .text_size(px(20.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(ov_value(k, o.range)),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(hue(&p, if good { "green" } else { "rose" }))
                                .child(pct(bp)),
                        ),
                );
            }
            let mut bars = div().h(px(170.)).flex().items_end().gap(px(3.));
            for (i, v) in ser.iter().copied().enumerate() {
                let on = o.point == Some(i);
                bars = bars.child(
                    div()
                        .flex_1()
                        .h(relative((v as f32 / hi).max(0.03)))
                        .rounded(px(2.))
                        .bg(if on { p.fg } else { p.sel }),
                );
            }
            let mut comps = bx(&p).child(h2("Components"));
            for (c, (_, n, _)) in COMPONENTS.iter().copied().enumerate() {
                let h = today_health(c, if o.incident { "degraded" } else { "" });
                comps = comps.child(
                    line(&p)
                        .child(clip(n))
                        .child(q(&p, pct2(uptime(c, ""))))
                        .child(pill(&p, h.hue(), h.word())),
                );
            }
            let mut acts = bx(&p).child(h2("Recent activity"));
            for e in oxyn_core::kit::arcscr4::events(true).into_iter().take(6) {
                acts = acts.child(
                    line(&p)
                        .child(av(&p, e.actor.unwrap_or(9), 22.))
                        .child(clip(format!("{} · {}", e.action, e.target)))
                        .child(q(&p, hm(e.at))),
                );
            }
            body(&p)
                .when(o.incident, |d| {
                    d.child(
                        card(&p)
                            .p(px(10.))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .border_color(hue(&p, "amber"))
                            .child(clip("Webhook deliveries are delayed (since 09:14 UTC). Events are queued, not lost."))
                            .child(btn(&p, &act, "ov-incident", "Dismiss", false)),
                    )
                })
                .child(kpis)
                .child(
                    bx(&p)
                        .child(div().flex().child(h2(format!("{} · last {}", OV_KPIS[o.kpi].1, OV_RANGES[o.range].0))))
                        .child(bars)
                        .child(div().flex().child(q(&p, ov_label(o.range, 0))).child(div().flex_1()).child(q(&p, ov_label(o.range, ser.len() - 1)))),
                )
                .child(div().flex().gap(px(14.)).child(comps.flex_1()).child(acts.flex_1()))
        };
        fr(
            &p,
            &act,
            self.layout,
            "admin-overview",
            "Overview",
            format!("Updated {} 09:41 UTC", iso(TODAY)),
            actions,
            None,
            b,
            None,
        )
    }
}

// ───────────────────────────────────────────────────────────── audit ──

impl RenderOnce for AuditLogEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.au = Audit::new(st != ScreenState::Empty);
            match self.extra {
                "filtered" => u.au.kind = 2,
                "detail" => u.au.open = Some(1),
                "nomatch" => u.au.q = "staging-cluster".into(),
                _ => {}
            }
        }
        let a = &u.au;
        let mut fbar = div().flex().items_center().gap(px(6.));
        for (k, (_, l)) in AU_TYPES.iter().copied().enumerate() {
            fbar = fbar.child(chip(&p, &act, ids::at(ids::AU_KIND, k), l, a.kind == k));
        }
        fbar = fbar.child(chip(&p, &act, "au-failed", "Failures only", a.failed));
        let list = a.list();
        let b = if st == ScreenState::Loading {
            body(&p).child(skel(&p, 10))
        } else if a.all.is_empty() {
            body(&p).child(none(
                &p,
                "Nothing logged yet",
                "Sign-ins, keys, members and billing land here.",
            ))
        } else if list.is_empty() {
            body(&p)
                .child(none(&p, "No events match", "Try another type or search."))
                .child(btn(&p, &act, "au-clear", "Clear filters", false))
        } else {
            let mut t = div().flex().flex_col().child(
                line(&p)
                    .text_color(p.mute)
                    .text_size(px(12.))
                    .child(div().w(px(170.)).child("Time (UTC)"))
                    .child(div().w(px(150.)).child("Actor"))
                    .child(div().flex_1().child("Action"))
                    .child(div().w(px(70.)).child("Result")),
            );
            for e in list.iter() {
                t = t.child(
                    hit(line(&p), &act, ids::at(ids::AU_OPEN, e.id as usize))
                        .when(a.open == Some(e.id), |d| d.bg(p.sel))
                        .child(div().w(px(170.)).text_color(p.fg2).child(format!(
                            "{} {}",
                            iso(e.day),
                            hms(e.at)
                        )))
                        .child(
                            div()
                                .w(px(150.))
                                .flex()
                                .gap(px(6.))
                                .child(av(&p, e.actor.unwrap_or(9), 20.))
                                .child(clip(e.who())),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(clip(format!("{} · {}", e.action, e.target))),
                        )
                        .child(div().w(px(70.)).child(pill(
                            &p,
                            if e.ok { "green" } else { "rose" },
                            if e.ok { "OK" } else { "Blocked" },
                        ))),
                );
            }
            body(&p)
                .child(q(&p, format!("{} events", a.matching().len())))
                .child(t)
                .when(list.len() < a.matching().len(), |d| {
                    d.child(btn(&p, &act, "au-more", "Show more", false))
                })
        };
        let right = a.current().map(|e| {
            div()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .child(h2(e.action))
                        .child(div().flex_1())
                        .child(btn(&p, &act, "au-close", "Close", false)),
                )
                .child(pill(
                    &p,
                    if e.ok { "green" } else { "rose" },
                    if e.ok { "Succeeded" } else { "Blocked" },
                ))
                .child(q(&p, e.when()))
                .child(q(&p, format!("{} · {}", e.who(), e.ip)))
                .child(q(&p, e.target))
                .child(
                    card(&p)
                        .p(px(10.))
                        .bg(p.inset)
                        .child(
                            div()
                                .text_color(hue(&p, "rose"))
                                .child(format!("- {}", e.before)),
                        )
                        .child(
                            div()
                                .text_color(hue(&p, "green"))
                                .child(format!("+ {}", e.after)),
                        ),
                )
        });
        fr(
            &p,
            &act,
            self.layout,
            "audit-log",
            "Audit log",
            "Kept for 365 days · times in UTC".into(),
            div().child(btn(&p, &act, "au-export", "Export CSV", false)),
            Some(fbar),
            b,
            right,
        )
    }
}

// ───────────────────────────────────────────────────────────── status ──

impl RenderOnce for StatusBoardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.st = Status::new(st != ScreenState::Empty, self.extra);
        }
        let s = &u.st;
        let (word, h) = s.overall();
        let b = if st == ScreenState::Loading {
            body(&p).child(skel(&p, 10))
        } else if !s.sample {
            body(&p).child(none(
                &p,
                "No components yet",
                "Add the parts people depend on.",
            ))
        } else {
            let mut comps = bx(&p);
            for (c, (_, n, w)) in COMPONENTS.iter().copied().enumerate() {
                let now = today_health(c, s.mode);
                let mut days = div().h(px(24.)).flex().gap(px(2.));
                for d in 0..STATUS_DAYS {
                    let x = day_health(c, d, s.mode);
                    let col = match x.key() {
                        "up" => tint(&p, "green", 0.8),
                        k => hue(
                            &p,
                            match k {
                                "slow" => "amber",
                                "down" => "rose",
                                _ => "blue",
                            },
                        ),
                    };
                    let cell = div().flex_1().h_full().rounded(px(2.)).bg(col);
                    days = if d + 1 == STATUS_DAYS {
                        days.child(hit(cell, &act, ids::at(ids::ST_TODAY, c)))
                    } else {
                        days.child(cell)
                    };
                }
                comps = comps.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .pb(px(8.))
                        .border_b_1()
                        .border_color(p.line)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(div().font_weight(FontWeight::MEDIUM).child(n))
                                        .child(q(&p, w)),
                                )
                                .child(div().flex_1())
                                .child(q(&p, pct2(uptime(c, s.mode))))
                                .child(pill(&p, now.hue(), now.word())),
                        )
                        .child(days)
                        .child(
                            div()
                                .flex()
                                .child(q(&p, iso(TODAY - 89)))
                                .child(div().flex_1())
                                .child(q(&p, "Today")),
                        ),
                );
            }
            let mut past = bx(&p).child(h2("Past incidents"));
            for (i, (_, ago, x, t, _)) in PAST.iter().copied().enumerate() {
                past = past.child(
                    hit(line(&p), &act, ids::at(ids::ST_PAST, i))
                        .child(pill(&p, x.hue(), x.word()))
                        .child(clip(t))
                        .child(q(&p, iso(TODAY - ago as i32))),
                );
            }
            body(&p)
                .child(
                    card(&p)
                        .p(px(14.))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .border_color(hue(&p, h))
                        .bg(tint(&p, h, 0.07))
                        .child(div().size(px(10.)).rounded_full().bg(hue(&p, h)))
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(word)),
                )
                .children(open_incident(s.mode).map(|(t, c, x, ups)| {
                    let mut d = bx(&p).border_color(hue(&p, x.hue())).child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(pill(&p, x.hue(), x.word()))
                            .child(h2(t))
                            .child(div().flex_1())
                            .child(q(&p, COMPONENTS[c].1)),
                    );
                    for (tm, stg, tx) in ups.iter().copied() {
                        d = d.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.))
                                        .child(div().font_weight(FontWeight::MEDIUM).child(stg))
                                        .child(q(&p, format!("{} {tm} UTC", iso(TODAY)))),
                                )
                                .child(div().text_color(p.fg2).child(tx)),
                        );
                    }
                    d
                }))
                .child(comps)
                .child(past)
        };
        let actions = div().child(btn(
            &p,
            &act,
            "st-sub",
            if s.subscribed {
                "Subscribed"
            } else {
                "Subscribe"
            },
            !s.subscribed,
        ));
        fr(
            &p,
            &act,
            self.layout,
            "status-board",
            "Status",
            format!("checked {} 09:41 UTC", iso(TODAY)),
            actions,
            None,
            b,
            None,
        )
    }
}

// ───────────────────────────────────────────────────────────── keys ──

impl RenderOnce for ApiKeysWebhooksEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.ak = Keys::new(st != ScreenState::Empty);
            match self.extra {
                "created" => {
                    u.ak.draft = Some(("support-bot".into(), 1));
                    let _ = u.ak.create();
                }
                "hooks" => u.ak.tab = 1,
                "failing" => {
                    u.ak.tab = 1;
                    if let Some(h) = u.ak.hooks.get_mut(0) {
                        for d in h.deliveries.iter_mut().take(3) {
                            d.code = 500;
                        }
                    }
                }
                _ => {}
            }
        }
        let k = &u.ak;
        let tabs = vec![
            chip(
                &p,
                &act,
                ids::AK_TAB[0],
                format!("API keys ({})", k.keys.len()),
                k.tab == 0,
            ),
            chip(
                &p,
                &act,
                ids::AK_TAB[1],
                format!("Webhooks ({})", k.hooks.len()),
                k.tab == 1,
            ),
        ];
        let actions = div().child(seg(&p, tabs)).child(if k.tab == 0 {
            btn(&p, &act, "ak-new", "New key", true)
        } else {
            btn(&p, &act, "wh-add", "Add endpoint", true)
        });
        let b = if st == ScreenState::Loading {
            body(&p).child(skel(&p, 8))
        } else if k.tab == 0 {
            let mut d = body(&p);
            if let Some(sec) = &k.secret {
                d = d.child(
                    bx(&p)
                        .border_color(p.fg)
                        .child(h2("Copy your secret now"))
                        .child(q(&p, "It will not show again."))
                        .child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .child(
                                    card(&p)
                                        .bg(p.inset)
                                        .px(px(10.))
                                        .py(px(6.))
                                        .flex_1()
                                        .min_w_0()
                                        .child(clip(sec.clone())),
                                )
                                .child(btn(
                                    &p,
                                    &act,
                                    "ak-copy",
                                    if k.copied { "Copied" } else { "Copy" },
                                    false,
                                ))
                                .child(btn(&p, &act, "ak-done", "Done", false)),
                        ),
                );
            }
            if let Some((n, sc)) = &k.draft {
                let scopes = SCOPES
                    .iter()
                    .enumerate()
                    .map(|(i, s)| chip(&p, &act, ids::at(ids::AK_SCOPE, i), s.1, *sc == i))
                    .collect();
                d = d.child(
                    bx(&p)
                        .child(h2("New API key"))
                        .child(hit(card(&p).px(px(10.)).py(px(7.)), &act, "ak-name").child(
                            if n.is_empty() {
                                q(&p, "e.g. support-bot")
                            } else {
                                div().child(n.clone())
                            },
                        ))
                        .child(seg(&p, scopes))
                        .children(
                            k.check()
                                .filter(|_| !n.is_empty())
                                .map(|e| div().text_color(hue(&p, "rose")).child(e)),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(6.))
                                .child(div().flex_1())
                                .child(btn(&p, &act, "ak-cancel", "Cancel", false))
                                .child(btn(&p, &act, "ak-create", "Create key", true)),
                        ),
                );
            }
            if k.keys.is_empty() {
                d = d.child(none(
                    &p,
                    "No API keys",
                    "Create a key to call the API from your servers.",
                ));
            }
            for x in k.keys.iter() {
                let ask = k.confirm == Some(x.id);
                d = d.child(
                    line(&p)
                        .child(av(&p, x.by, 20.))
                        .child(
                            div()
                                .w(px(150.))
                                .font_weight(FontWeight::MEDIUM)
                                .child(clip(x.name.clone())),
                        )
                        .child(
                            div()
                                .w(px(130.))
                                .text_color(p.fg2)
                                .child(format!("ox_live_…{}", x.tail)),
                        )
                        .child(pill(
                            &p,
                            if x.scope == 2 { "amber" } else { "" },
                            SCOPES[x.scope].1,
                        ))
                        .child(div().flex_1())
                        .child(q(&p, x.used.map(iso).unwrap_or_else(|| "Never".into())))
                        .child(btn(
                            &p,
                            &act,
                            ids::at(ids::AK_ROLL, x.id as usize),
                            "Roll",
                            false,
                        ))
                        .child(
                            btn(
                                &p,
                                &act,
                                ids::at(ids::AK_REVOKE, x.id as usize),
                                if ask { "Revoke?" } else { "Revoke" },
                                false,
                            )
                            .when(ask, |d| {
                                d.border_color(hue(&p, "rose")).text_color(hue(&p, "rose"))
                            }),
                        ),
                );
            }
            d
        } else if k.hooks.is_empty() {
            body(&p).child(none(&p, "No endpoints", "Add a URL and pick events."))
        } else {
            let mut l = div().w(px(280.)).flex_none().flex().flex_col().gap(px(6.));
            for (i, h) in k.hooks.iter().enumerate() {
                let (w, hh) = if !h.on {
                    ("Off", "")
                } else if h.failing() {
                    ("Failing", "rose")
                } else {
                    ("On", "green")
                };
                l = l.child(
                    hit(card(&p), &act, ids::at(ids::WH_OPEN, i))
                        .p(px(10.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .when(i == k.hook, |d| d.border_color(p.fg))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(clip(h.url))
                                .child(q(
                                    &p,
                                    format!(
                                        "{} events · {}% delivered",
                                        h.events.len(),
                                        h.success()
                                    ),
                                )),
                        )
                        .child(pill(&p, hh, w)),
                );
            }
            let h = &k.hooks[k.hook.min(k.hooks.len() - 1)];
            let mut r = bx(&p).flex_1().child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(clip(h.url))
                    .child(btn(&p, &act, "wh-test", "Send test", false))
                    .child(btn(
                        &p,
                        &act,
                        "wh-toggle",
                        if h.on { "Turn off" } else { "Turn on" },
                        false,
                    )),
            );
            if h.failing() {
                r = r.child(
                    div()
                        .text_color(hue(&p, "rose"))
                        .child("The last deliveries failed with 500. We retry for 3 days."),
                );
            }
            r = r.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(6.))
                    .children(h.events.iter().map(|e| {
                        div()
                            .px(px(7.))
                            .rounded(px(6.))
                            .bg(p.sel)
                            .text_size(px(11.5))
                            .child(HOOK_EVENTS[*e])
                    })),
            );
            for d in h.deliveries.iter() {
                r = r.child(
                    line(&p)
                        .child(pill(
                            &p,
                            if d.ok() { "green" } else { "rose" },
                            d.code_word(),
                        ))
                        .child(clip(HOOK_EVENTS[d.event]))
                        .child(q(
                            &p,
                            format!("{} · {} {}", dur(d.ms), iso(TODAY), hms(d.at)),
                        ))
                        .when(!d.ok(), |x| {
                            x.child(btn(
                                &p,
                                &act,
                                ids::at(ids::WH_RETRY, d.id as usize),
                                "Retry",
                                false,
                            ))
                        }),
                );
            }
            body(&p).child(div().flex().gap(px(14.)).items_start().child(l).child(r))
        };
        fr(
            &p,
            &act,
            self.layout,
            "api-keys-webhooks",
            if k.tab == 0 { "API keys" } else { "Webhooks" },
            "Developers".into(),
            actions,
            None,
            b,
            None,
        )
    }
}

// ───────────────────────────────────────────────────────────── members ──

impl RenderOnce for MembersRolesEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.mb = Members::new(st != ScreenState::Empty, self.extra == "full");
            match self.extra {
                "invite" => {
                    u.mb.form = true;
                    u.mb.email = "sana@corvel".into();
                }
                "full" => u.mb.form = true,
                "admins" => u.mb.filter = Some(1),
                _ => {}
            }
        }
        let m = &u.mb;
        let filters = vec![
            chip(&p, &act, "mb-all", "Everyone", m.filter.is_none()),
            chip(&p, &act, ids::MB_FILTER[1], "Admins", m.filter == Some(1)),
            chip(&p, &act, ids::MB_FILTER[2], "Members", m.filter == Some(2)),
            chip(&p, &act, ids::MB_FILTER[3], "Viewers", m.filter == Some(3)),
        ];
        let actions =
            div()
                .child(seg(&p, filters))
                .child(btn(&p, &act, "mb-invite", "Invite", true));
        let b = if st == ScreenState::Loading {
            body(&p).child(skel(&p, 9))
        } else {
            let mut d = body(&p).child(meter(
                &p,
                "Seats",
                m.used().to_string(),
                m.seats.to_string(),
                m.used() as f32 / m.seats.max(1) as f32,
            ));
            if m.form {
                let err = if m.email.is_empty() && m.used() < m.seats {
                    None
                } else {
                    oxyn_core::kit::arcscr4::invite_error(&m.email, m)
                };
                let roles = (1..4)
                    .map(|r| {
                        chip(
                            &p,
                            &act,
                            ids::at(ids::MB_IROLE, r - 1),
                            ROLES[r],
                            m.irole == r,
                        )
                    })
                    .collect();
                d = d.child(
                    bx(&p)
                        .child(h2("Invite a member"))
                        .child(
                            div()
                                .flex()
                                .gap(px(10.))
                                .child(
                                    hit(card(&p).flex_1().px(px(10.)).py(px(7.)), &act, "mb-email")
                                        .child(if m.email.is_empty() {
                                            q(&p, "name@company.com")
                                        } else {
                                            div().child(m.email.clone())
                                        }),
                                )
                                .child(seg(&p, roles)),
                        )
                        .children(err.map(|e| div().text_color(hue(&p, "rose")).child(e)))
                        .child(
                            div()
                                .flex()
                                .child(q(&p, "The link works for 7 days."))
                                .child(div().flex_1())
                                .child(btn(&p, &act, "mb-send", "Send invite", true)),
                        ),
                );
            }
            for (i, x) in m.shown() {
                let ask = m.confirm == Some(i);
                let name = PEOPLE[x.who % PEOPLE.len()].0;
                let rh = match x.role {
                    0 => "violet",
                    1 => "blue",
                    _ => "",
                };
                d = d.child(
                    line(&p)
                        .min_h(px(46.))
                        .child(av(&p, x.who, 28.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(clip(name))
                                .child(q(&p, member_email(x.who))),
                        )
                        .child(
                            hit(div(), &act, ids::at(ids::MB_ROLE, i))
                                .px(px(8.))
                                .h(px(24.))
                                .flex()
                                .items_center()
                                .rounded(px(7.))
                                .border_1()
                                .border_color(p.line)
                                .text_color(if rh.is_empty() { p.fg } else { hue(&p, rh) })
                                .child(format!("{} ⇅", ROLES[x.role])),
                        )
                        .child(pill(
                            &p,
                            if x.mfa { "green" } else { "amber" },
                            if x.mfa { "2FA on" } else { "2FA off" },
                        ))
                        .child(q(&p, iso(x.active)))
                        .when(x.role != 0, |r| {
                            r.child(btn(
                                &p,
                                &act,
                                ids::at(ids::MB_RM, i),
                                if ask { "Remove?" } else { "Remove" },
                                false,
                            ))
                        }),
                );
            }
            if !m.invites.is_empty() {
                d = d.child(h2(format!("Pending invites ({})", m.invites.len())));
                for v in m.invites.iter() {
                    d = d.child(
                        line(&p)
                            .child(glyph(&p, "✉"))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .child(clip(v.email.clone()))
                                    .child(q(
                                        &p,
                                        format!("{} · sent {}", ROLES[v.role], iso(v.sent)),
                                    )),
                            )
                            .child(btn(
                                &p,
                                &act,
                                ids::at(ids::MB_RESEND, v.id as usize),
                                "Resend",
                                false,
                            ))
                            .child(btn(
                                &p,
                                &act,
                                ids::at(ids::MB_REVOKE, v.id as usize),
                                "Revoke",
                                false,
                            )),
                    );
                }
            }
            d
        };
        fr(
            &p,
            &act,
            self.layout,
            "members-roles",
            "Members",
            format!("{} of {} seats", m.used(), m.seats),
            actions,
            None,
            b,
            None,
        )
    }
}

// ───────────────────────────────────────────────────────────── billing ──

impl RenderOnce for BillingUsageEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.bl = Billing::new(st != ScreenState::Empty);
            match self.extra {
                "yearly" => u.bl.yearly = true,
                "change" => {
                    let _ = u.bl.pick(2);
                }
                "pastdue" => {
                    if let Some(i) = u.bl.invoices.first_mut() {
                        i.status = 2;
                    }
                }
                _ => {}
            }
        }
        let b = &u.bl;
        let actions = div().child(chip(
            &p,
            &act,
            "bl-cycle",
            if b.yearly {
                "Yearly · save 17%"
            } else {
                "Monthly"
            },
            true,
        ));
        let body_d = if st == ScreenState::Loading {
            body(&p).child(skel(&p, 10))
        } else {
            let cur = PLANS[b.plan];
            let mut plans = div().flex().gap(px(12.));
            for (i, (n, _, _, seats, calls)) in PLANS.iter().copied().enumerate() {
                let on = i == b.plan;
                let price = b.price(i);
                plans = plans.child(
                    bx(&p)
                        .flex_1()
                        .when(on || b.pick == Some(i), |d| d.border_color(p.fg))
                        .child(
                            div()
                                .flex()
                                .child(div().font_weight(FontWeight::SEMIBOLD).child(n))
                                .child(div().flex_1())
                                .when(on, |d| d.child(pill(&p, "green", "Current"))),
                        )
                        .child(
                            div()
                                .text_size(px(22.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!("{} / month", money0(price))),
                        )
                        .child(q(
                            &p,
                            format!("{seats} seats · {} API calls", short_n(calls)),
                        ))
                        .child(btn(
                            &p,
                            &act,
                            ids::at(ids::BL_PICK, i),
                            if on {
                                "Your plan"
                            } else if i > b.plan {
                                "Upgrade"
                            } else {
                                "Downgrade"
                            },
                            b.pick == Some(i),
                        )),
                );
            }
            let mut d = body(&p);
            if b.past_due() {
                d = d.child(
                    card(&p)
                        .p(px(10.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .border_color(hue(&p, "rose"))
                        .child(clip(format!(
                            "Your last payment failed on {}. Retry before {}.",
                            iso(TODAY - 1),
                            iso(TODAY + 6)
                        )))
                        .child(btn(&p, &act, "bl-retry", "Retry payment", true)),
                );
            }
            d = d.child(plans);
            if let Some(pk) = b.pick {
                let c = b.prorate(pk);
                d = d.child(
                    card(&p)
                        .p(px(12.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .border_color(p.fg)
                        .child(clip(if c >= 0 {
                            format!(
                                "Change to {}: {} today, then {} a month",
                                PLANS[pk].0,
                                money(c),
                                money0(b.price(pk))
                            )
                        } else {
                            format!("Change to {}: {} credit", PLANS[pk].0, money(-c))
                        }))
                        .child(btn(&p, &act, "bl-cancel", "Cancel", false))
                        .child(btn(&p, &act, "bl-confirm", "Confirm change", true)),
                );
            }
            let usage = bx(&p)
                .flex_1()
                .child(h2("Usage this period"))
                .child(meter(
                    &p,
                    "Seats",
                    b.seats.to_string(),
                    cur.3.to_string(),
                    b.seats as f32 / cur.3 as f32,
                ))
                .child(meter(
                    &p,
                    "API calls",
                    short_n(b.calls),
                    short_n(cur.4),
                    b.calls as f32 / cur.4 as f32,
                ))
                .child(meter(
                    &p,
                    "Storage",
                    format!("{}.{} GB", b.storage / 1000, (b.storage % 1000) / 100),
                    "50 GB".into(),
                    b.storage as f32 / 50_000.,
                ));
            let pay = bx(&p).flex_1().child(h2("Payment method")).child(
                if b.plan == 0 && b.invoices.is_empty() {
                    q(&p, "No card on file.")
                } else {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(logo(&p, "visa", 28.))
                        .child(div().flex_1().child("Visa ending 4242"))
                        .child(btn(&p, &act, "bl-card", "Update", false))
                },
            );
            let mut inv = bx(&p).child(h2("Invoices"));
            for (i, v) in b.invoices.iter().enumerate() {
                let (w, h) = match v.status {
                    0 => ("Paid", "green"),
                    1 => ("Open", "amber"),
                    _ => ("Failed", "rose"),
                };
                inv = inv.child(
                    line(&p)
                        .child(div().w(px(130.)).child(v.no.clone()))
                        .child(q(&p, iso(v.day)))
                        .child(div().flex_1())
                        .child(money(v.cents))
                        .child(pill(&p, h, w))
                        .child(btn(&p, &act, ids::at(ids::BL_INV, i), "PDF", false)),
                );
            }
            d.child(div().flex().gap(px(14.)).child(usage).child(pay))
                .child(inv)
        };
        fr(
            &p,
            &act,
            self.layout,
            "billing-usage",
            "Billing",
            format!("Period {} – {}", iso(TODAY - 12), iso(TODAY + 18)),
            actions,
            None,
            body_d,
            None,
        )
    }
}

// ───────────────────────────────────────────────────────────── tool calls ──

fn tool_card(p: &ThPal, act: &Option<Act>, t: &ToolCalls, ti: usize, m: Mode) -> Div {
    let (name, f, brand, src, _, _, _) = TC_TOOLS[ti];
    let (c, fl, avg) = t.stats(ti);
    let lit = t.live() == Some(ti);
    let on = t.sel == Some(Sel::Tool(ti));
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .items_center()
        .child(
            div()
                .w(px(if lit { 2. } else { 1. }))
                .h(px(18.))
                .bg(if lit { hue(p, "blue") } else { p.line }),
        )
        .child(
            hit(card(p), act, ids::at(ids::TC_TOOL, ti))
                .w_full()
                .p(px(10.))
                .flex()
                .flex_col()
                .gap(px(5.))
                .when(on, |d| d.border_color(p.fg))
                .when(lit, |d| d.border_color(hue(p, "blue")))
                .when(m == Mode::Off, |d| d.opacity(0.62))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(logo(p, brand, 16.))
                        .child(clip(name))
                        .when(lit, |d| {
                            d.child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(hue(p, "blue"))
                                    .child("Calling"),
                            )
                        }),
                )
                .child(div().text_size(px(11.5)).text_color(p.fg2).child(clip(f)))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .px(px(6.))
                                .rounded(px(5.))
                                .bg(p.sel)
                                .text_size(px(11.))
                                .child(src),
                        )
                        .child(div().flex_1())
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(match m {
                                    Mode::Ask => hue(p, "amber"),
                                    Mode::Off => p.mute,
                                    Mode::Auto => p.fg2,
                                })
                                .child(m.word()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .text_size(px(11.5))
                        .text_color(p.mute)
                        .child(format!("{c} calls"))
                        .when(fl > 0, |d| {
                            d.child(
                                div()
                                    .ml(px(6.))
                                    .text_color(hue(p, "rose"))
                                    .child(format!("{fl} failed")),
                            )
                        })
                        .child(div().flex_1())
                        .when(avg > 0, |d| d.child(dur(avg))),
                ),
        )
}

impl RenderOnce for AgentToolCallsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.tc = ToolCalls::new(st != ScreenState::Empty, self.extra);
        }
        let t = &u.tc;
        let (agent, task, model, maker) = TC_AGENT;
        let ranges = TC_RANGES
            .iter()
            .enumerate()
            .map(|(i, r)| chip(&p, &act, ids::at(ids::TC_RANGE, i), r.0, i == t.range))
            .collect();
        let actions = div()
            .child(seg(&p, ranges))
            .child(btn(
                &p,
                &act,
                "tc-pause",
                if t.paused {
                    "Resume agent"
                } else {
                    "Pause agent"
                },
                false,
            ))
            .child(btn(&p, &act, "tc-add", "Connect tool", true));
        let (n, nf, nw, tok) = t.totals();
        let b = if st == ScreenState::Loading {
            body(&p).child(skel(&p, 10))
        } else {
            let mut d = body(&p);
            if t.picker {
                let mut pk = bx(&p).child(h2("Connect a tool"));
                for ti in t.unconnected() {
                    pk = pk.child(
                        line(&p)
                            .child(logo(&p, TC_TOOLS[ti].2, 16.))
                            .child(clip(TC_TOOLS[ti].0))
                            .child(btn(
                                &p,
                                &act,
                                ids::at(ids::TC_CONNECT, ti),
                                "Connect",
                                false,
                            )),
                    );
                }
                if t.unconnected().is_empty() {
                    pk = pk.child(q(&p, "Every tool in the catalogue is connected."));
                }
                d = d.child(pk);
            }
            let agent_card = hit(card(&p), &act, "tc-agent")
                .w(px(340.))
                .p(px(12.))
                .flex()
                .items_center()
                .gap(px(10.))
                .when(t.sel == Some(Sel::Agent), |d| d.border_color(p.fg))
                .child(
                    div()
                        .size(px(34.))
                        .rounded(px(9.))
                        .bg(p.inset)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(logo(&p, maker, 20.)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(agent))
                        .child(q(&p, model)),
                )
                .child(if t.paused {
                    pill(&p, "amber", "Paused")
                } else if t.live().is_some() {
                    pill(&p, "blue", "Working")
                } else {
                    pill(&p, "green", "Idle")
                });
            let half = t.tools.len().div_ceil(2);
            let row = |v: &[(usize, Mode)]| {
                div()
                    .w_full()
                    .flex()
                    .gap(px(12.))
                    .children(v.iter().map(|(ti, m)| tool_card(&p, &act, t, *ti, *m)))
            };
            let mut org = div()
                .flex()
                .flex_col()
                .items_center()
                .child(agent_card)
                .child(div().w(px(1.)).h(px(18.)).bg(p.line));
            if t.tools.is_empty() {
                org = org.child(
                    hit(div(), &act, "tc-add")
                        .px(px(18.))
                        .py(px(12.))
                        .rounded(px(12.))
                        .border_1()
                        .border_dashed()
                        .border_color(p.line)
                        .text_color(p.fg2)
                        .child("+ Connect the first tool"),
                );
            } else {
                org = org
                    .child(div().w(relative(0.75)).h(px(1.)).bg(p.line))
                    .child(row(&t.tools[..half]));
                if t.tools.len() > half {
                    org = org.child(div().h(px(14.))).child(row(&t.tools[half..]));
                }
            }
            d = d.child(org).child(q(
                &p,
                format!(
                    "{n} calls · {nf} failed · {nw} waiting · {} tokens",
                    thousands(tok as u64)
                ),
            ));
            let shown = t.shown();
            if shown.is_empty() {
                d = d.child(none(
                    &p,
                    "No calls yet",
                    "Connect a tool and its calls show here.",
                ));
            } else {
                let mut tb = div().flex().flex_col().child(
                    line(&p)
                        .text_color(p.mute)
                        .text_size(px(12.))
                        .child(div().w(px(80.)).child("Time"))
                        .child(div().w(px(200.)).child("Tool"))
                        .child(div().w(px(100.)).child("Status"))
                        .child(div().w(px(70.)).child("Duration"))
                        .child(div().w(px(60.)).child("Tokens"))
                        .child(div().flex_1().child("Input")),
                );
                for c in shown.iter() {
                    tb = tb.child(
                        hit(line(&p), &act, ids::at(ids::TC_CALL, c.id as usize))
                            .when(t.sel == Some(Sel::Call(c.id)), |d| d.bg(p.sel))
                            .child(div().w(px(80.)).text_color(p.fg2).child(if c.ago == 0 {
                                hms(c.at)
                            } else {
                                iso(TODAY - c.ago as i32)
                            }))
                            .child(
                                div()
                                    .w(px(200.))
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .child(logo(&p, TC_TOOLS[c.tool].2, 14.))
                                    .child(clip(TC_TOOLS[c.tool].1)),
                            )
                            .child(div().w(px(100.)).child(pill(&p, c.st.hue(), c.st.word())))
                            .child(div().w(px(70.)).child(if c.ms > 0 {
                                dur(c.ms)
                            } else {
                                "—".into()
                            }))
                            .child(div().w(px(60.)).child(if c.tokens > 0 {
                                thousands(c.tokens as u64)
                            } else {
                                "—".into()
                            }))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .text_color(p.fg2)
                                    .child(clip(c.input.clone())),
                            ),
                    );
                }
                d = d.child(tb);
            }
            d
        };
        let right = match t.sel {
            Some(Sel::Agent) => Some(
                div()
                    .p(px(16.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(logo(&p, maker, 18.))
                            .child(h2(agent))
                            .child(div().flex_1())
                            .child(btn(&p, &act, "tc-close", "Close", false)),
                    )
                    .child(q(&p, task))
                    .child(q(&p, format!("{model} · {} tools", t.tools.len())))
                    .child(
                        div()
                            .flex()
                            .gap(px(6.))
                            .child(btn(
                                &p,
                                &act,
                                "tc-pause",
                                if t.paused { "Resume" } else { "Pause agent" },
                                false,
                            ))
                            .child(btn(&p, &act, "tc-add", "Connect tool", true)),
                    ),
            ),
            Some(Sel::Tool(ti)) => {
                let (name, f, brand, src, desc, _, params) = TC_TOOLS[ti];
                let m = t.mode(ti).unwrap_or(Mode::Off);
                let modes = vec![
                    chip(&p, &act, "tc-mode-auto", "Allowed", m == Mode::Auto),
                    chip(&p, &act, "tc-mode-ask", "Ask first", m == Mode::Ask),
                    chip(&p, &act, "tc-mode-off", "Paused", m == Mode::Off),
                ];
                let mut d = div()
                    .p(px(16.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(logo(&p, brand, 18.))
                            .child(h2(name))
                            .child(div().flex_1())
                            .child(btn(&p, &act, "tc-close", "Close", false)),
                    )
                    .child(q(&p, format!("{desc} · {src}")))
                    .child(div().text_color(p.fg2).child(f))
                    .child(q(&p, "May run"))
                    .child(seg(&p, modes))
                    .child(q(&p, "Parameters"));
                for (pn, pt, req) in params.iter().copied() {
                    d = d.child(
                        line(&p)
                            .child(div().flex_1().child(pn))
                            .child(q(&p, pt))
                            .when(req, |x| x.child(pill(&p, "", "required"))),
                    );
                }
                Some(d.child(btn(
                    &p,
                    &act,
                    "tc-disc",
                    if t.disc == Some(ti) {
                        "Press again to disconnect"
                    } else {
                        "Disconnect"
                    },
                    false,
                )))
            }
            Some(Sel::Call(id)) => t.calls.iter().find(|c| c.id == id).map(|c| {
                let mut d = div()
                    .p(px(16.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(logo(&p, TC_TOOLS[c.tool].2, 18.))
                            .child(h2(TC_TOOLS[c.tool].1))
                            .child(div().flex_1())
                            .child(btn(&p, &act, "tc-close", "Close", false)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(pill(&p, c.st.hue(), c.st.word()))
                            .child(q(&p, format!("{} · {}", c.when(), c.run))),
                    );
                if c.st == CallSt::Waiting {
                    d = d.child(
                        card(&p)
                            .p(px(10.))
                            .border_color(hue(&p, "amber"))
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(format!(
                                "{agent} wants to run {}. {} asks first.",
                                TC_TOOLS[c.tool].1, TC_TOOLS[c.tool].0
                            ))
                            .child(
                                div()
                                    .flex()
                                    .gap(px(6.))
                                    .child(btn(&p, &act, "tc-approve", "Approve", true))
                                    .child(btn(&p, &act, "tc-reject", "Reject", false)),
                            ),
                    );
                }
                d = d.child(q(&p, "Input")).child(
                    card(&p)
                        .bg(p.inset)
                        .p(px(10.))
                        .text_size(px(11.5))
                        .child(c.input.clone()),
                );
                if let Some(o) = &c.output {
                    d = d
                        .child(q(&p, format!("Output · {} tokens", c.tokens)))
                        .child(
                            card(&p)
                                .bg(p.inset)
                                .p(px(10.))
                                .text_size(px(11.5))
                                .child(o.clone()),
                        );
                }
                if let Some(e) = &c.error {
                    d = d.child(q(&p, "Error")).child(
                        card(&p)
                            .bg(p.inset)
                            .p(px(10.))
                            .text_size(px(11.5))
                            .text_color(hue(&p, "rose"))
                            .child(e.clone()),
                    );
                }
                if c.st == CallSt::Failed {
                    d = d.child(btn(&p, &act, "tc-solve", "Run again", true));
                }
                d
            }),
            None => None,
        };
        let crumb = format!("{agent} · Tool calls");
        fr(
            &p,
            &act,
            self.layout,
            "agent-tool-calls",
            &crumb,
            if t.paused {
                "Paused".into()
            } else {
                format!("Live · {}", iso(TODAY))
            },
            actions,
            None,
            b,
            right,
        )
    }
}
