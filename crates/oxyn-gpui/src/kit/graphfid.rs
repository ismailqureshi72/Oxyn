//! affectUI fidelity pass, overnight batch, family B part 3 (plan #32) for GPUI: the exact
//! **Tool network** ([`ToolNetworkEl`]) over `oxyn_core::kit::graphfid`. The element is
//! `RenderOnce`: the host keeps the clock and the open node and routes [`Act`] ids back
//! (`node-<n>` opens node n's card, `close`). A static snapshot: links are drawn as their straight
//! runs (one rounded turn becomes a corner), pulses are not animated.

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px, rgb,
};
use oxyn_core::kit::graphfid::{
    CallStatus, GraphScene, NodeKind, NodeStatus, call_duration, calls_per_min, graph_at,
    graph_cost, graph_layout, graph_links, link_turn,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, alpha, c};
use crate::brand_mark;

const W: f32 = 756.0;
const H: f32 = 454.0;
const NODE_ACTS: [&str; 16] = [
    "node-0", "node-1", "node-2", "node-3", "node-4", "node-5", "node-6", "node-7", "node-8",
    "node-9", "node-10", "node-11", "node-12", "node-13", "node-14", "node-15",
];

#[derive(Clone, Copy)]
struct Pal {
    bg: Hsla,
    line: Hsla,
    fg: Hsla,
    mute: Hsla,
    link: Hsla,
    tile: Hsla,
    you: Hsla,
    you_fg: Hsla,
    blue: Hsla,
    green: Hsla,
    amber: Hsla,
    red: Hsla,
    amber_bg: Hsla,
    red_bg: Hsla,
    band: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            bg: h(0x141416),
            line: h(0x29292E),
            fg: h(0xF4F4F5),
            mute: h(0x8E8E97),
            link: h(0x34343A),
            tile: h(0x1E1E21),
            you: h(0xF4F4F5),
            you_fg: h(0x111114),
            blue: h(0x5B8DEF),
            green: h(0x34C77B),
            amber: h(0xF5A524),
            red: h(0xF2555A),
            amber_bg: h(0x33270F),
            red_bg: h(0x361719),
            band: h(0x1C1C1F),
        }
    } else {
        Pal {
            bg: h(0xFFFFFF),
            line: h(0xE8E8EB),
            fg: h(0x18181B),
            mute: h(0x8A8A93),
            link: h(0xE2E2E6),
            tile: h(0xFFFFFF),
            you: h(0x18181B),
            you_fg: h(0xFFFFFF),
            blue: h(0x3B82F6),
            green: h(0x22A55B),
            amber: h(0xE39B1B),
            red: h(0xE5484D),
            amber_bg: h(0xFFF6E5),
            red_bg: h(0xFFF0F0),
            band: h(0xF6F6F7),
        }
    }
}

/// `agent-graph` (affectUI Agent Graph): the host passes the scene (the demo's at some ms, or
/// its own) and the open node; acts `node-<n>` and `close`.
#[derive(IntoElement)]
pub struct ToolNetworkEl {
    scene: GraphScene,
    title: String,
    elapsed: u64,
    open: Option<usize>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ToolNetworkEl {
    fn default() -> Self {
        Self {
            scene: graph_at(15_000),
            title: "Agent graph".into(),
            elapsed: 15_000,
            open: None,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ToolNetworkEl {
    /// The demo graph `ms` into its pass.
    pub fn demo(mut self, ms: u64) -> Self {
        self.scene = graph_at(ms);
        self.elapsed = ms;
        self
    }
    /// Your own graph, `ms` on its clock.
    pub fn scene(mut self, scene: GraphScene, ms: u64) -> Self {
        self.scene = scene;
        self.elapsed = ms;
        self
    }
    /// Heading of the bar.
    pub fn title(mut self, t: &str) -> Self {
        self.title = t.into();
        self
    }
    /// The node (index) whose card is open.
    pub fn open(mut self, i: Option<usize>) -> Self {
        self.open = i;
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
    /// Acts.
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

/// A straight run from (x0, y0) to (x1, y1) (horizontal or vertical) as a thin div.
fn run(x0: f32, y0: f32, x1: f32, y1: f32, w: f32, col: Hsla) -> AnyElement {
    let (l, t) = (x0.min(x1), y0.min(y1));
    let (dw, dh) = ((x1 - x0).abs().max(w), (y1 - y0).abs().max(w));
    div()
        .absolute()
        .left(px(l - w / 2.0))
        .top(px(t - w / 2.0))
        .w(px(dw))
        .h(px(dh))
        .bg(col)
        .into_any_element()
}

impl RenderOnce for ToolNetworkEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let sc = &self.scene;
        let el = self.elapsed;
        let links = graph_links(&sc.edges, &sc.calls);
        let pos = graph_layout(&sc.nodes, &links, W, H);
        let at = |id: &str| pos.iter().find(|x| x.0 == id).map(|x| (x.1, x.2));
        let centres: Vec<(f32, f32)> = pos.iter().map(|x| (x.1, x.2)).collect();
        let agents = sc
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Agent)
            .count();
        let tools = sc.nodes.iter().filter(|n| n.kind == NodeKind::Tool).count();
        let cpm = calls_per_min(&sc.calls, el, 20_000);
        let total: f64 = sc.nodes.iter().map(|n| n.cost).sum();

        let mut bar = div()
            .h(px(32.))
            .mb(px(if self.classic { 0. } else { 10. }))
            .px(px(if self.classic { 12. } else { 2. }))
            .flex()
            .items_center()
            .gap(px(8.))
            .when(self.classic, |d| d.bg(p.band).rounded_t(px(12.)))
            .child(div().text_color(p.blue).child("⁘"))
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title.clone()),
            );
        for n in sc.nodes.iter() {
            let (txt, fg, bg) = match n.status {
                NodeStatus::Waiting => (format!("{} needs you", n.name), p.amber, p.amber_bg),
                NodeStatus::Error => (format!("{} failed", n.name), p.red, p.red_bg),
                _ => continue,
            };
            bar = bar.child(
                div()
                    .h(px(24.))
                    .px(px(9.))
                    .rounded_full()
                    .border_1()
                    .border_color(alpha(fg, 0.45))
                    .bg(bg)
                    .text_color(fg)
                    .text_size(px(12.))
                    .flex()
                    .items_center()
                    .child(txt),
            );
        }
        let stat = |n: String, w: &str| {
            div()
                .flex()
                .gap(px(3.))
                .child(
                    div()
                        .text_color(p.fg)
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(n),
                )
                .child(div().text_color(p.mute).child(w.to_string()))
        };
        bar = bar.child(div().flex_1()).child(
            div()
                .flex()
                .gap(px(14.))
                .text_size(px(12.5))
                .child(stat(agents.to_string(), "agents"))
                .child(stat(tools.to_string(), "tools"))
                .child(stat(cpm.to_string(), "calls/min"))
                .child(stat(graph_cost(total), "")),
        );

        let mut panel = div()
            .relative()
            .w(px(W))
            .h(px(H))
            .border_1()
            .border_color(p.line)
            .rounded(px(12.))
            .when(self.classic, |d| d.rounded_t(px(0.)).shadow_md())
            .bg(p.bg)
            .overflow_hidden();
        // Links: straight runs with one corner.
        for (a, b) in links.iter() {
            let (Some(pa), Some(pb)) = (at(a), at(b)) else {
                continue;
            };
            let heat = sc
                .calls
                .iter()
                .filter(|c| el.saturating_sub(c.at) < 20_000)
                .filter(|c| (&c.from == a && &c.to == b) || (&c.from == b && &c.to == a))
                .count();
            let (w, col) = if self.classic {
                (
                    1.5 + (heat.min(6) as f32) * 0.35,
                    alpha(p.blue, 0.4 + heat.min(6) as f32 * 0.04),
                )
            } else {
                (1.5, p.link)
            };
            if link_turn(pa, pb, &centres) {
                panel = panel
                    .child(run(pa.0, pa.1, pa.0, pb.1, w, col))
                    .child(run(pa.0, pb.1, pb.0, pb.1, w, col));
            } else {
                panel = panel
                    .child(run(pa.0, pa.1, pb.0, pa.1, w, col))
                    .child(run(pb.0, pa.1, pb.0, pb.1, w, col));
            }
        }
        for (i, n) in sc.nodes.iter().enumerate().take(NODE_ACTS.len()) {
            let Some((x, y)) = at(&n.id) else { continue };
            let s = match (n.kind, n.lead) {
                (NodeKind::Agent, true) => 56.0,
                (NodeKind::Agent, false) => 48.0,
                _ => 32.0,
            };
            let (bg, border) = match (n.kind, n.status) {
                (NodeKind::User, _) => (p.you, p.you),
                (_, NodeStatus::Waiting) => (p.amber_bg, p.amber),
                (_, NodeStatus::Error) => (p.red_bg, p.red),
                _ => (p.tile, p.line),
            };
            let mut tile = div()
                .id(SharedString::from(format!("agr-n-{i}")))
                .absolute()
                .left(px(x - s / 2.0))
                .top(px(y - s / 2.0))
                .size(px(s))
                .rounded(px(s * 0.27))
                .border_1()
                .border_color(border)
                .bg(bg)
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .when(n.lead, |d| d.shadow_md())
                .on_click({
                    let f = fire(&self.act, NODE_ACTS[i]);
                    move |_, w, cx| f(w, cx)
                });
            tile = match n.kind {
                NodeKind::User => tile.child(div().text_color(p.you_fg).child("●")),
                _ => match brand_mark(&n.brand, s * 0.46, p.fg) {
                    Some(m) => tile.child(m),
                    None => tile.child(
                        div()
                            .text_color(p.blue)
                            .text_size(px(s * 0.4))
                            .child(if n.id == "web" { "⌕" } else { "•" }),
                    ),
                },
            };
            let badge = match n.status {
                NodeStatus::Done => Some((p.green, "✓")),
                NodeStatus::Waiting => Some((p.amber, "◷")),
                NodeStatus::Error => Some((p.red, "!")),
                _ => None,
            };
            if let Some((col, g)) = badge {
                tile = tile.child(
                    div()
                        .absolute()
                        .right(px(-5.))
                        .top(px(-5.))
                        .size(px(15.))
                        .rounded_full()
                        .bg(col)
                        .text_color(rgb(0xFFFFFF))
                        .text_size(px(9.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(g),
                );
            }
            panel = panel.child(tile);
            let sub = match n.status {
                NodeStatus::Waiting => "Needs you".to_string(),
                NodeStatus::Error => "Error".to_string(),
                _ => n.detail.clone(),
            };
            let label_y = if n.kind == NodeKind::User {
                y - s / 2.0 - 22.0
            } else {
                y + s / 2.0 + 5.0
            };
            let mut lbl = div()
                .absolute()
                .left(px(x - 70.0))
                .top(px(label_y))
                .w(px(140.))
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .text_size(px(12.5))
                        .when(n.kind == NodeKind::Agent, |d| {
                            d.font_weight(FontWeight::SEMIBOLD).text_color(p.fg)
                        })
                        .when(n.kind != NodeKind::Agent, |d| d.text_color(p.mute))
                        .child(n.name.clone()),
                );
            if n.kind == NodeKind::Agent {
                let col = match n.status {
                    NodeStatus::Waiting => p.amber,
                    NodeStatus::Error => p.red,
                    _ => p.mute,
                };
                lbl = lbl.child(div().text_size(px(11.5)).text_color(col).child(sub));
            }
            panel = panel.child(lbl);
        }
        // The open card beside its node.
        if let Some(i) = self.open.filter(|i| *i < sc.nodes.len())
            && let Some((x, y)) = at(&sc.nodes[i].id)
        {
            let n = &sc.nodes[i];
            let left = if x > W * 0.55 {
                x - 40.0 - 244.0
            } else {
                x + 40.0
            };
            let mut card = div()
                .absolute()
                .left(px(left))
                .top(px((y - 30.0).clamp(8.0, H - 220.0)))
                .w(px(244.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .bg(p.tile)
                .shadow_lg()
                .flex()
                .flex_col()
                .text_size(px(12.5))
                .child(
                    div()
                        .p(px(10.))
                        .flex()
                        .gap(px(8.))
                        .items_center()
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(n.name.clone()),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(p.mute)
                                        .child(n.detail.clone()),
                                ),
                        )
                        .child(
                            div()
                                .px(px(7.))
                                .rounded_full()
                                .bg(alpha(p.blue, 0.12))
                                .text_color(p.blue)
                                .text_size(px(11.))
                                .child(n.status.label()),
                        )
                        .child(
                            div()
                                .id("agr-close")
                                .text_color(p.mute)
                                .cursor_pointer()
                                .on_click({
                                    let f = fire(&self.act, "close");
                                    move |_, w, cx| f(w, cx)
                                })
                                .child("×"),
                        ),
                );
            if !n.doing.is_empty() {
                card = card.child(
                    div()
                        .px(px(10.))
                        .py(px(8.))
                        .border_t_1()
                        .border_color(p.line)
                        .child(n.doing.clone()),
                );
            }
            let mut rows = div()
                .px(px(10.))
                .py(px(6.))
                .border_t_1()
                .border_color(p.line)
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(p.mute)
                        .child("Latest calls"),
                );
            for c in sc
                .calls
                .iter()
                .rev()
                .filter(|c| c.from == n.id && c.at <= el)
                .take(4)
            {
                let to = sc
                    .nodes
                    .iter()
                    .find(|x| x.id == c.to)
                    .map(|x| x.name.clone())
                    .unwrap_or_else(|| c.to.clone());
                let pend = c.status == CallStatus::Pending || el < c.at + c.ms;
                rows = rows.child(
                    div()
                        .flex()
                        .gap(px(5.))
                        .child(div().text_color(p.mute).child("→"))
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(to))
                        .child(div().text_color(p.mute).child(format!("· {}", c.label)))
                        .child(div().flex_1())
                        .child(div().text_color(p.mute).child(if pend {
                            "◌".to_string()
                        } else {
                            call_duration(c.ms)
                        })),
                );
            }
            card = card.child(rows).child(
                div()
                    .px(px(10.))
                    .py(px(8.))
                    .border_t_1()
                    .border_color(p.line)
                    .flex()
                    .gap(px(8.))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(graph_cost(n.cost)),
                    )
                    .child(div().text_color(p.mute).child(format!(
                        "{} calls in the last minute",
                        sc.calls
                            .iter()
                            .filter(|c| c.from == n.id && el.saturating_sub(c.at) < 60_000)
                            .count()
                    ))),
            );
            panel = panel.child(card);
        }

        let legend: &[(&str, Hsla)] = if self.classic {
            &[
                ("Call", p.blue),
                ("Answer", p.green),
                ("Busier link", alpha(p.blue, 0.55)),
                ("Needs you", p.amber),
            ]
        } else {
            &[
                ("Call", p.blue),
                ("Answer", p.green),
                ("Needs you", p.amber),
                ("Failed", p.red),
            ]
        };
        let mut leg = div()
            .mt(px(10.))
            .flex()
            .justify_center()
            .gap(px(16.))
            .text_size(px(11.))
            .text_color(p.mute);
        for (l, col) in legend {
            leg = leg.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(div().w(px(12.)).h(px(2.)).rounded(px(2.)).bg(*col))
                    .child(l.to_string()),
            );
        }
        div()
            .w(px(W))
            .text_color(p.fg)
            .flex()
            .flex_col()
            .child(bar)
            .child(panel)
            .child(leg)
    }
}
