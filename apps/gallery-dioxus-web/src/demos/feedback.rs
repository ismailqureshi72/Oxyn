//! Feedback (plan #26): toaster, notification stack, alert, skeleton (+ shimmer loader),
//! skeleton reveal.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{ClockMs, primitives::toast::Notice};
use oxyn_dioxus::{ShimmerLabel, Skeleton};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{Alert, MorphBlock, NotificationStack, SeedToast, SkeletonReveal, Toaster, use_toasts};

use super::{Code, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals};

fn skeleton(v: &Vals, _: ClockMs) -> Element {
    let loading = v.b("loading");
    rsx! {
        div { class: "oxd-w360", style: "display:flex;flex-direction:column;gap:18px",
            Skeleton {
                lines: v.u("lines"),
                avatar: v.b("avatar"),
                shape: v.string("shape"),
                sweep: v.string("sweep"),
                speed: v.f("speed") as f32,
                angle: v.f("angle") as f32,
                loading,
                delay: v.u("delay") as u32,
                width: 360.0,
                div { class: "oxd-loaded",
                    div { class: "oxd-loaded-h", span { class: "oxd-av", "MC" } b { "Maya Chen" } span { class: "oxd-muted", "· 2 min ago" } }
                    p { "Both tabs call refresh() with the same token; the second one gets 401. A single-flight lock fixes it." }
                }
            }
            if v.b("shimmer_text") && loading {
                ShimmerLabel { text: "Thinking… reading 14 files", duration: v.f("speed") as f32 + 0.4 }
            }
        }
    }
}

fn skeleton_code(v: &Vals) -> String {
    Code::new("Skeleton")
        .raw("lines", v.s("lines"))
        .b("avatar", v.b("avatar"))
        .s("shape", v.s("shape"), "text")
        .s("sweep", v.s("sweep"), "shimmer")
        .raw("loading", "loading()")
        .raw("Message", "{ message }")
        .done()
        .replace("Message: { message },", "Message { message }")
}

pub(crate) fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![        Demo {
            slug: "skeleton",
            name: "Skeleton",
            category: "feedback",
            summary: "Grey shapes the size of what they stand in for, one sheen sweeping across all of them in step (or a staggered pulse), shown only after a short delay, then a crossfade into the content.",
            dioxus: "Skeleton, ShimmerLabel",
            gpui: "SkeletonEl",
            spec: "skeletons",
            source: "primitives/surfaces.rs",
            controls: vec![
                tog("loading", "Loading", true),
                sel(
                    "shape",
                    "Shape",
                    &[
                        "text", "message", "tool-row", "diff", "card", "table", "form",
                    ],
                    "message",
                ),
                rng("lines", "Lines", 1.0, 6.0, 1.0, "3"),
                tog("avatar", "Avatar", true),
                sel(
                    "sweep",
                    "Sweep",
                    &["shimmer", "pulse", "wave", "none"],
                    "shimmer",
                ),
                rng("speed", "Period s", 0.8, 3.0, 0.2, "1.6"),
                rng("angle", "Sweep angle", 60.0, 140.0, 10.0, "100"),
                rng("delay", "Reveal after ms", 0.0, 600.0, 50.0, "150"),
                tog("shimmer_text", "Shimmer text", true),
            ],
            variants: vec![
                var("message", "A chat message.", &[]),
                var(
                    "tool-rows",
                    "Tool calls in a run.",
                    &[
                        ("shape", "tool-row"),
                        ("lines", "4"),
                        ("shimmer_text", "false"),
                    ],
                ),
                var(
                    "wave",
                    "A staggered pulse.",
                    &[("shape", "text"), ("sweep", "wave"), ("lines", "5")],
                ),
                var(
                    "loaded",
                    "Crossfaded into the content.",
                    &[("loading", "false")],
                ),
            ],
            render: skeleton,
            code: skeleton_code,
            keyboard: vec![],
            a11y: vec![
                "role=status with aria-busy and a label; the shapes are aria-hidden.",
                "Reduced motion stops the sheen and pulse.",
            ],
            responsive: vec![
                "Bars are percentages of the slot width, so it scales with its container.",
            ],
            height: 260,
            align: Align::Center,
            live: false,
        },
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 5] = ["toaster", "notification-stack", "alert", "skeleton", "skeleton-reveal"];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
