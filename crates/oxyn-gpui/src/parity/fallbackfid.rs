//! affectUI fidelity pass, family B part 8 (plan #32) for GPUI: the exact **Fallback Chain**
//! ([`FallbackChainEl`]) over `oxyn_core::parity::fallbackfid`. The element is `RenderOnce`: the
//! host passes the attempts and their states and gets [`Act`] ids back (`replay`, `m-<n>` for the
//! n-th model). A static snapshot: no curved links or travelling light.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::parity::fallbackfid::{
    FbAttempt, FbState, fb_demo, fb_final, fb_headline, fb_ms, fb_total_ms,
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
    response: String,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for FallbackChainEl {
    fn default() -> Self {
        Self {
            attempts: fb_demo(),
            states: None,
            request: "Summarise this week's support tickets".into(),
            response: "Three themes stand out: slow exports, SSO sign-in loops and…".into(),
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
    /// Request and response lines.
    pub fn text(mut self, request: &str, response: &str) -> Self {
        self.request = request.into();
        self.response = response.into();
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
    /// Replay and the models.
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
                h(0x18181B),
                h(0x2C2C31),
                h(0xEDEDEF),
                h(0x8B8B94),
                h(0x1B1B1F),
                h(0x6C9BFF),
                h(0xF97066),
                h(0x4ADE80),
                h(0x1D1D21),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xE6E6E9),
                h(0x18181B),
                h(0x8A8A93),
                h(0xFAFAFA),
                h(0x4C7DF0),
                h(0xD92D20),
                h(0x16A34A),
                h(0xF4F4F5),
            )
        };
        let states = self
            .states
            .clone()
            .unwrap_or_else(|| fb_final(&self.attempts));
        let headline = fb_headline(&self.attempts, &states);
        let answered = states.iter().position(|s| *s == FbState::Answered);
        let rf = fire(&self.act, "replay");
        let bar = div()
            .h(px(48.))
            .px(px(16.))
            .flex()
            .items_center()
            .gap(px(8.))
            .border_b_1()
            .border_color(bd)
            .when(self.classic, |d| d.bg(band))
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Fallback chain"),
            )
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(mute)
                    .child(format!("{} models in order", self.attempts.len())),
            )
            .child(div().flex_1())
            .child(div().text_size(px(12.)).text_color(blue).child(headline))
            .child(
                div()
                    .id("oxfb-replay")
                    .px(px(6.))
                    .text_color(mute)
                    .on_click(move |_, w, cx| rf(w, cx))
                    .child("↻"),
            );
        let node = |cap: &str, text: String| {
            div()
                .w(px(160.))
                .p(px(10.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .border_1()
                .border_color(bd)
                .rounded(px(10.))
                .bg(card)
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(mute)
                        .child(cap.to_uppercase()),
                )
                .child(div().text_size(px(12.5)).child(text))
        };
        let mut models = div().flex_1().flex().flex_col().gap(px(10.));
        for (i, (x, s)) in self.attempts.iter().zip(states.iter()).enumerate().take(6) {
            let f = fire(&self.act, M[i]);
            let (edge, note, tone) = match s {
                FbState::Failed => (red, x.outcome.reason(), red),
                FbState::Answered => (blue, format!("Answered in {}", fb_ms(x.ms)), ok),
                FbState::Trying => (blue, "Trying…".to_string(), mute),
                FbState::Skipped => (bd, "Not needed".to_string(), mute),
                FbState::Waiting => (bd, "Next in line".to_string(), mute),
            };
            models = models.child(
                div()
                    .id(SharedString::from(format!("oxfb-m{i}")))
                    .h(px(52.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .border_1()
                    .border_color(edge)
                    .rounded(px(10.))
                    .bg(card)
                    .when(*s == FbState::Skipped, |d| d.opacity(0.5))
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
                                    .text_size(px(13.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(x.model.clone()),
                            )
                            .child(div().text_size(px(11.5)).text_color(tone).child(note)),
                    ),
            );
        }
        let res = match answered {
            Some(i) => node(
                "Response",
                format!(
                    "{} – {} · {}",
                    self.response,
                    self.attempts[i].model,
                    fb_ms(fb_total_ms(&self.attempts, 0))
                ),
            ),
            None => node("No response", "Every model in the chain failed.".into()),
        };
        div()
            .w(px(760.))
            .flex()
            .flex_col()
            .border_1()
            .border_color(bd)
            .rounded(px(12.))
            .bg(card)
            .text_color(fg)
            .child(bar)
            .child(
                div()
                    .p(px(24.))
                    .flex()
                    .items_center()
                    .gap(px(40.))
                    .bg(box_bg)
                    .child(node("Request", self.request.clone()))
                    .child(models)
                    .child(res),
            )
    }
}
