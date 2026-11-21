//! Screens S16 in GPUI (Agent apps IV, part b): Pilot Panel. Task history with cost; the task's
//! steps with a checkpoint after each change; the composer with Plan / Act and Model; and the
//! Editor pane with the diff. Own states: `plan` (the proposed plan and Switch to Act) and
//! `approval` (a command waiting on Approve / Reject). `.state()` sets Data / Loading / Empty and
//! `.extra()` the own state or `rail` / `hidden`. Presses fire `row-N`, `view:plan`, `act`,
//! `approve`, `reject`, `restore` and `send` through `on_act`. Rules from
//! `oxyn_core::kit::pilotpanel`.
//!
//! [`super::AgentAppView`] draws these screens as a live, interactive GPUI view.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled,
    Window, div, px,
};
use oxyn_core::kit::pilotpanel::{
    PK_APP, PK_CTX, PK_DIFF, PK_FILE, PK_PENDING, PK_PLAN, PK_PROMPT, PK_STEPS, ScreenState,
    checkpoints, kilo, pk_tasks, usd,
};
use oxyn_core::kit::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::apps::Live;
use super::chatscr::clip;
use super::cmdstudio::{brand, chip, composer, mono, press, tone_of};
use super::fleetwin::{faint, s11_col, s11_el, s11_frame, s11_row, s11_side, you};
use super::rulerfid::{ThPal, th_pal};
use super::screensfid::skel;
use super::threadbar::{diff_line, pane_col, pane_head};

s11_el!(
    /// Pilot Panel: task history, steps with checkpoints, the composer with Plan / Act and Model,
    /// and the Editor pane; Plan mode and a pending approval as own states.
    PilotPanelEl,
    pk_screen
);

fn editor(p: &ThPal, st: ScreenState) -> Div {
    let d = pane_col(p).child(pane_head(
        p,
        "Editor",
        format!("{} / {}", kilo(PK_CTX.0), kilo(PK_CTX.1)),
    ));
    if st == ScreenState::Loading {
        return d.child(skel(p, 8));
    }
    if st == ScreenState::Empty {
        return d.child(faint(p, "Edited files show here"));
    }
    let mut diff = div().py(px(6.)).bg(p.inset).flex().flex_col();
    for (s, l) in PK_DIFF {
        diff = diff.child(diff_line(p, s, l));
    }
    d.child(
        div()
            .h(px(36.))
            .px(px(12.))
            .flex()
            .items_center()
            .gap(px(8.))
            .child(clip(PK_FILE))
            .child(mono("+14").text_color(tone_of(p, "ok")))
            .child(mono("−3").text_color(tone_of(p, "bad"))),
    )
    .child(diff)
}

pub(super) fn pk_screen(
    p: &ThPal,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    lv: &Live,
) -> Div {
    let tasks = pk_tasks(st);
    let open = lv.sel.unwrap_or(0);
    let rows: Vec<Stateful<Div>> = tasks
        .iter()
        .enumerate()
        .map(|(i, c)| {
            s11_row(
                p,
                if c.running { "run" } else { "" },
                c.title.clone(),
                i == open,
                div()
                    .text_size(px(11.))
                    .text_color(p.mute)
                    .child(usd(c.cents)),
                i,
                act,
            )
        })
        .collect();
    let side = s11_side(
        p,
        PK_APP,
        "New task",
        "History",
        rows,
        st,
        extra == "rail",
        act,
        lv,
    );
    let plan = extra == "plan";
    let chips = vec![
        chip(p, None, if plan { "Plan" } else { "Act" }, true),
        chip(p, None, "Auto-approve: Read", false),
        chip(p, Some(brand(p, "claude")), "Sonnet 5.5", false),
    ];
    let comp = composer(
        p,
        chips,
        "Describe the change · @ to add files",
        plan,
        act,
        lv,
    );
    let cur = tasks.get(open);
    let inner = if st == ScreenState::Loading {
        skel(p, 6)
    } else if st == ScreenState::Empty {
        faint(p, "What can I do for you?")
    } else if open != 0 {
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, cur.map(|c| c.title.clone()).unwrap_or_default()))
            .child(
                div()
                    .text_color(p.fg2)
                    .child("Finished. Open a checkpoint to compare, or ask for a follow-up."),
            )
    } else {
        let shown = if plan { 1 } else { PK_STEPS.len() };
        let cps = checkpoints(&PK_STEPS[..shown]);
        let mut steps = div().flex().flex_col().gap(px(6.));
        for (i, s) in PK_STEPS.iter().take(shown).enumerate() {
            steps = if s.kind == "say" {
                steps.child(div().child(s.title))
            } else {
                steps.child(
                    div()
                        .h(px(36.))
                        .px(px(10.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(p.line)
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(match s.kind {
                            "read" => "Read",
                            "edit" => "Edited",
                            _ => "Ran",
                        }))
                        .child(clip(s.title))
                        .child(div().text_size(px(12.)).text_color(p.mute).child(s.detail)),
                )
            };
            if cps.contains(&i) {
                steps = steps.child(
                    div()
                        .h(px(26.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_size(px(12.))
                        .text_color(p.mute)
                        .child(div().flex_1().h(px(1.)).bg(p.line))
                        .child("Checkpoint")
                        .child(press(p, "Restore", false, act, "restore"))
                        .child(div().flex_1().h(px(1.)).bg(p.line)),
                );
            }
        }
        let mut d = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(you(p, PK_PROMPT))
            .child(steps);
        if plan {
            let mut pl = div()
                .p(px(12.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(div().font_weight(FontWeight::SEMIBOLD).child("Plan"));
            for (i, l) in PK_PLAN.iter().copied().enumerate() {
                pl = pl.child(div().child(format!("{}. {l}", i + 1)));
            }
            d = d.child(pl.child(div().pt(px(8.)).flex().justify_end().child(press(
                p,
                "Switch to Act",
                true,
                act,
                "act",
            ))));
        } else if extra == "approval" {
            d = d.child(
                div()
                    .p(px(12.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(tone_of(p, "wait"))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Pilot Panel wants to run a command"),
                    )
                    .child(
                        div()
                            .p(px(8.))
                            .rounded(px(8.))
                            .bg(p.inset)
                            .child(mono(format!("$ {PK_PENDING}"))),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(8.))
                            .child(press(p, "Reject", false, act, "reject"))
                            .child(press(p, "Approve", true, act, "approve")),
                    ),
            );
        } else {
            d = d.child(div().child(
                "Done. /orders now takes a cursor and a limit (at most 50) and returns next_cursor.",
            ));
        }
        d
    };
    let title: SharedString = cur
        .map(|c| c.title.clone().into())
        .unwrap_or_else(|| "New task".into());
    let pl = cur.map(|_| match extra {
        "approval" => ("Needs approval", "wait"),
        "plan" => ("Plan", ""),
        _ => ("Act", "ok"),
    });
    s11_frame(
        p,
        lay,
        extra,
        side,
        title,
        pl,
        s11_col(p, inner, comp),
        editor(p, st),
    )
}
