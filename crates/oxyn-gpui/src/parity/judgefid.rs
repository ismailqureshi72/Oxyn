//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the exact **Judge Bracket**
//! ([`JudgeBracketEl`]) over `oxyn_core::parity::judgefid`. The element is `RenderOnce`: the host
//! passes contenders and verdicts and gets [`Act`] ids back (`m-<round>-<match>` for a match,
//! up to 16 contenders). A static snapshot: no elbows, shimmer or answers panel.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::parity::judgefid::{
    JbContender, JbVerdict, jb_champion, jb_current, jb_demo, jb_demo_verdicts, jb_matches,
    jb_on_path, jb_progress, jb_round_name, jb_rounds, jb_slots, jb_verdict,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const IDS: [[&str; 8]; 4] = [
    [
        "m-0-0", "m-0-1", "m-0-2", "m-0-3", "m-0-4", "m-0-5", "m-0-6", "m-0-7",
    ],
    ["m-1-0", "m-1-1", "m-1-2", "m-1-3", "", "", "", ""],
    ["m-2-0", "m-2-1", "", "", "", "", "", ""],
    ["m-3-0", "", "", "", "", "", "", ""],
];

/// `judge-bracket` (affectUI Judge Bracket): the host passes the bracket.
#[derive(IntoElement)]
pub struct JudgeBracketEl {
    contenders: Vec<JbContender>,
    verdicts: Vec<JbVerdict>,
    task: String,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for JudgeBracketEl {
    fn default() -> Self {
        Self {
            contenders: jb_demo(),
            verdicts: jb_demo_verdicts(),
            task: "Fix: a retried payment charges the customer twice".into(),
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl JudgeBracketEl {
    /// Contenders (seed order), verdicts and the task.
    pub fn bracket(mut self, c: Vec<JbContender>, v: Vec<JbVerdict>, task: &str) -> Self {
        self.contenders = c;
        self.verdicts = v;
        self.task = task.into();
        self
    }
    /// Classic style.
    pub fn classic(mut self, on: bool) -> Self {
        self.classic = on;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// Match clicks.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

fn fire(a: &Option<Act>, id: &'static str) -> impl Fn(&mut Window, &mut App) + 'static {
    let a = a.clone();
    move |w, cx| {
        if let Some(f) = &a {
            f(id, w, cx)
        }
    }
}

impl RenderOnce for JudgeBracketEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, path, path_bg) = if dark {
            (
                h(0x17171A),
                h(0x2A2A2E),
                h(0xEDEDEF),
                h(0x8A8A93),
                h(0x4C8DFF),
                h(0x1A2740),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xE5E7EB),
                h(0x111827),
                h(0x6B7280),
                h(0x2563EB),
                h(0xEFF6FF),
            )
        };
        let n = self.contenders.len();
        let v = &self.verdicts;
        let cur = jb_current(n, v);
        let champ = jb_champion(n, v);
        let rounds = jb_rounds(n).min(4);
        let mut cols = div().flex().gap(px(28.));
        for (r, ids) in IDS.iter().enumerate().take(rounds) {
            let mut col = div()
                .flex_1()
                .flex()
                .flex_col()
                .justify_around()
                .gap(px(8.))
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(mute)
                        .child(jb_round_name(n, r).to_uppercase()),
                );
            for (m, id) in ids.iter().enumerate().take(jb_matches(n, r).min(8)) {
                let sl = jb_slots(n, v, r, m);
                let vd = jb_verdict(v, r, m);
                let on_path = jb_on_path(n, v, r, m);
                let judging = cur == Some((r, m));
                let f = fire(&self.act, id);
                let mut mt = div()
                    .id(SharedString::from(format!("oxjb-{r}-{m}")))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .on_click(move |_, w, cx| f(w, cx));
                for (k, s) in sl.iter().enumerate() {
                    let won = vd.is_some_and(|x| x.winner == k);
                    let lost = vd.is_some() && !won;
                    let score = vd.map(|x| if k == 0 { x.scores.0 } else { x.scores.1 });
                    mt = mt.child(
                        div()
                            .h(px(30.))
                            .px(px(9.))
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .border_1()
                            .border_color(if won && on_path || judging { path } else { bd })
                            .rounded(px(8.))
                            .bg(if won && on_path { path_bg } else { card })
                            .when(lost, |d| d.opacity(0.45))
                            .child(div().flex_1().text_size(px(12.)).child(
                                s.map_or(String::new(), |i| self.contenders[i].name.clone()),
                            ))
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(score.map_or(String::new(), |x| format!("{x:.1}"))),
                            ),
                    );
                }
                col = col.child(mt);
            }
            cols = cols.child(col);
        }
        div()
            .w(px(640.))
            .p(px(18.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .border_1()
            .border_color(bd)
            .rounded(px(if self.classic { 16. } else { 14. }))
            .bg(card)
            .text_color(fg)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Judge"),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(mute)
                            .child(self.task.clone()),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(path)
                            .child(match champ {
                                Some(i) => format!("{} wins", self.contenders[i].name),
                                None => "Judging".into(),
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(mute)
                            .child(jb_progress(n, v)),
                    ),
            )
            .child(cols)
    }
}
