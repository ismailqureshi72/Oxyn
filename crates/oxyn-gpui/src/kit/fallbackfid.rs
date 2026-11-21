//! affectUI fidelity pass, family B part 8 (plan #32) for GPUI: the exact **Fallback Chain**
//! ([`FallbackChainEl`]) over `oxyn_core::kit::fallbackfid`. The element is `RenderOnce`: the
//! host passes the attempts and their states and gets [`Act`] ids back (`retry`, `m-<n>` for the
//! n-th model). A static snapshot: no curved links, travelling light or details card.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::kit::fallbackfid::{
    FbAttempt, FbState, fb_demo, fb_final, fb_meter, fb_s, fb_status, fb_usd,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const M: [&str; 6] = ["m-0", "m-1", "m-2", "m-3", "m-4", "m-5"];

/// `fallback-chain` (affectUI Fallback Chain): the host passes the chain.
#[derive(IntoElement)]
pub struct FallbackChainEl {
    attempts: Vec<FbAttempt>,
    states: Option<Vec<FbState>>,
    request: String,
    route: String,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for FallbackChainEl {
    fn default() -> Self {
        Self {
            attempts: fb_demo(),
            states: None,
            request: "Summarise the support ticket".into(),
            route: "router/auto".into(),
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl FallbackChainEl {
    /// The models in order.
    pub fn attempts(mut self, a: Vec<FbAttempt>) -> Self {
        self.attempts = a;
        self
    }
    /// Live states (settled when not set).
    pub fn states(mut self, s: Vec<FbState>) -> Self {
        self.states = Some(s);
        self
    }
    /// The request (header title) and the router it went through.
    pub fn text(mut self, request: &str, route: &str) -> Self {
        self.request = request.into();
        self.route = route.into();
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
    /// Retry and the models.
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

impl RenderOnce for FallbackChainEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, box_bg, blue, red, ok, band) = if dark {
            (
                h(0x17171A),
                h(0x2A2A2E),
                h(0xEDEDEF),
                h(0x8A8A93),
                h(0x111113),
                h(0x4C8DFF),
                h(0xF87171),
                h(0x34D399),
                h(0x111113),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xE6E6E6),
                h(0x111827),
                h(0x6B7280),
                h(0xFAFAFA),
                h(0x2563EB),
                h(0xDC2626),
                h(0x16A34A),
                h(0xFAFAFA),
            )
        };
        let states = self
            .states
            .clone()
            .unwrap_or_else(|| fb_final(&self.attempts));
        let (pill, word) = fb_status(&states);
        let (clock, cost) = fb_meter(&self.attempts, u64::MAX / 4, 0);
        let answered = states.iter().position(|s| *s == FbState::Answered);
        let rf = fire(&self.act, "retry");
        let tone = match pill {
            "done" => ok,
            "failed" => red,
            _ => blue,
        };
        let bar = div()
            .h(px(44.))
            .px(px(16.))
            .flex()
            .items_center()
            .gap(px(12.))
            .when(self.classic, |d| d.bg(band).border_b_1().border_color(bd))
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.request.clone()),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(tone)
                    .child(format!("● {word}")),
            )
            .child(div().text_size(px(11.)).text_color(mute).child(fb_s(clock)))
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(mute)
                    .child(fb_usd(cost)),
            )
            .when(pill == "failed", |d| {
                d.child(
                    div()
                        .id("oxfb-retry")
                        .px(px(8.))
                        .border_1()
                        .border_color(bd)
                        .rounded(px(7.))
                        .text_size(px(11.5))
                        .on_click(move |_, w, cx| rf(w, cx))
                        .child("Retry"),
                )
            });
        let node = |cap: &str, text: String, edge: Hsla| {
            div()
                .w(px(96.))
                .h(px(44.))
                .px(px(9.))
                .flex()
                .flex_col()
                .justify_center()
                .border_1()
                .border_color(edge)
                .rounded(px(10.))
                .bg(card)
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(cap.to_string()),
                )
                .child(div().text_size(px(10.)).text_color(mute).child(text))
        };
        let mut models = div().flex_1().flex().flex_col().gap(px(10.));
        for (i, (x, s)) in self.attempts.iter().zip(states.iter()).enumerate().take(6) {
            let f = fire(&self.act, M[i]);
            let (edge, note, tone) = match s {
                FbState::Failed => (bd, x.outcome.reason(), red),
                FbState::Answered => (ok, format!("Answered · {} tokens", x.tokens), ok),
                FbState::Trying => (blue, "Trying…".to_string(), blue),
                FbState::Skipped => (bd, "Not needed".to_string(), mute),
                FbState::Waiting => (bd, "Waiting".to_string(), mute),
            };
            models = models.child(
                div()
                    .id(SharedString::from(format!("oxfb-m{i}")))
                    .h(px(46.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .border_1()
                    .border_color(edge)
                    .rounded(px(10.))
                    .bg(card)
                    .when(matches!(s, FbState::Skipped | FbState::Waiting), |d| {
                        d.opacity(0.55)
                    })
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(mute)
                            .child(format!("{}", i + 1)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .flex()
                                    .gap(px(6.))
                                    .child(
                                        div()
                                            .text_size(px(12.))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(x.model.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(mute)
                                            .child(x.provider.clone()),
                                    ),
                            )
                            .child(div().text_size(px(11.)).text_color(tone).child(note)),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_end()
                            .text_size(px(11.))
                            .text_color(mute)
                            .child(match s {
                                FbState::Failed | FbState::Answered => fb_s(x.ms),
                                _ => "–".into(),
                            })
                            .child(match s {
                                FbState::Failed | FbState::Answered => fb_usd(x.cost),
                                _ => String::new(),
                            }),
                    ),
            );
        }
        let res = match answered {
            Some(i) => node(
                "Response",
                format!("{} tokens", self.attempts[i].tokens),
                ok,
            ),
            None => node("Response", "None".into(), red),
        };
        div()
            .w(px(640.))
            .flex()
            .flex_col()
            .border_1()
            .border_color(bd)
            .rounded(px(14.))
            .bg(card)
            .text_color(fg)
            .child(bar)
            .child(
                div()
                    .p(px(20.))
                    .flex()
                    .items_center()
                    .gap(px(40.))
                    .when(self.classic, |d| d.bg(box_bg))
                    .child(node("Request", self.route.clone(), bd))
                    .child(models)
                    .child(res),
            )
    }
}
