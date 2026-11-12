//! The WCAG contrast table of `docs/tokens.md` §8, computed from the resolved theme colours.

use std::fmt;

use serde_json::Value;

use crate::model::TokenSet;

/// An sRGB colour, 0–255 per channel.
#[derive(Clone, Copy, PartialEq)]
pub struct Rgb(pub [u8; 3]);

impl Rgb {
    pub fn from_hex(h: &str) -> Rgb {
        let h = h.trim_start_matches('#');
        let p = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).expect("hex colour");
        Rgb([p(0), p(2), p(4)])
    }

    pub fn hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.0[0], self.0[1], self.0[2])
    }

    fn luminance(self) -> f64 {
        let lin = |c: u8| {
            let c = c as f64 / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(self.0[0]) + 0.7152 * lin(self.0[1]) + 0.0722 * lin(self.0[2])
    }

    pub fn contrast(self, other: Rgb) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// `self` at `alpha` over `bg`, in sRGB, rounded to 8 bits.
    pub fn over(self, alpha: f64, bg: Rgb) -> Rgb {
        let m = |f: u8, b: u8| {
            let v = alpha * (f as f64 / 255.0) + (1.0 - alpha) * (b as f64 / 255.0);
            (v * 255.0).round().clamp(0.0, 255.0) as u8
        };
        Rgb([
            m(self.0[0], bg.0[0]),
            m(self.0[1], bg.0[1]),
            m(self.0[2], bg.0[2]),
        ])
    }
}

/// A resolved DTCG colour value: hex plus alpha.
pub fn hex_alpha(v: &Value) -> (Rgb, f64) {
    let hex = v
        .get("hex")
        .and_then(Value::as_str)
        .expect("colour has hex");
    let alpha = v.get("alpha").and_then(Value::as_f64).unwrap_or(1.0);
    (Rgb::from_hex(hex), alpha)
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Text,
    Ui,
    Exempt,
}

pub struct Row {
    pub theme: String,
    pub fg: String,
    pub bg: String,
    pub kind: Kind,
    pub ratio: f64,
    pub need: f64,
    pub fg_hex: String,
    pub bg_hex: String,
}

impl Row {
    pub fn fails(&self) -> bool {
        self.kind != Kind::Exempt && self.ratio < self.need
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} on {}: {:.2} < {} ({} / {})",
            self.theme, self.fg, self.bg, self.ratio, self.need, self.fg_hex, self.bg_hex
        )
    }
}

const SURF: &[&str] = &[
    "bg.base",
    "surface.sunken",
    "surface.raised",
    "surface.overlay",
];
const TEXT: &[&str] = &[
    "fg.default",
    "fg.muted",
    "fg.subtle",
    "accent.fg",
    "status.success.fg",
    "status.warning.fg",
    "status.danger.fg",
    "status.info.fg",
    "status.waiting.fg",
    "risk.low.fg",
    "risk.medium.fg",
    "risk.high.fg",
    "risk.critical.fg",
    "diff.add.fg",
    "diff.del.fg",
    "merge.current",
    "merge.incoming",
    "merge.suggested",
    "merge.resolved",
    "git.dirty",
    "git.worktree",
    "git.detached",
    "text-status.base",
    "term.prompt",
    "checkpoint.current",
];
const UI: &[&str] = &[
    "border.strong",
    "accent.solid",
    "accent.border",
    "focus",
    "icon.fg",
    "status.success.border",
    "status.warning.border",
    "status.danger.border",
    "status.info.border",
    "risk.low.border",
    "risk.medium.border",
    "risk.high.border",
    "risk.critical.border",
    "diff.add.stripe",
    "diff.del.stripe",
    "agent.red",
    "agent.orange",
    "agent.yellow",
    "agent.green",
    "agent.blue",
    "agent.violet",
    "agent.pink",
    "agent.cyan",
    "loader.spectral.0",
    "loader.spectral.1",
    "loader.spectral.2",
    "loader.spectral.3",
    "loader.spectral.4",
    "loader.spectral.5",
];

struct Ctx<'a> {
    set: &'a TokenSet,
    theme: String,
    rows: Vec<Row>,
}

impl Ctx<'_> {
    fn raw(&self, role: &str) -> (Rgb, f64) {
        let v = self
            .set
            .resolved(&format!("theme.{}.color.{role}", self.theme))
            .unwrap_or_else(|e| panic!("{e}"));
        hex_alpha(&v)
    }

    /// The colour as it lands on `on` (translucent roles are composited).
    fn col(&self, role: &str, on: Option<Rgb>) -> Rgb {
        let (c, a) = self.raw(role);
        if a < 1.0 {
            c.over(
                a,
                on.unwrap_or_else(|| panic!("{role} is translucent and needs a surface")),
            )
        } else {
            c
        }
    }

    fn add(&mut self, fg: &str, bg: &str, kind: Kind, fg_rgb: Rgb, bg_rgb: Rgb) {
        let hc = self.theme == "hc-dark";
        let need = match (hc, kind) {
            (true, Kind::Text) => 7.0,
            (true, _) => 4.5,
            (false, Kind::Text) => 4.5,
            (false, _) => 3.0,
        };
        self.rows.push(Row {
            theme: self.theme.clone(),
            fg: fg.to_owned(),
            bg: bg.to_owned(),
            kind,
            ratio: fg_rgb.contrast(bg_rgb),
            need,
            fg_hex: fg_rgb.hex(),
            bg_hex: bg_rgb.hex(),
        });
    }
}

/// Every contrast row for every theme (§8 of `docs/tokens.md`).
pub fn rows(set: &TokenSet) -> Vec<Row> {
    let mut all = Vec::new();
    for theme in set.themes() {
        let mut c = Ctx {
            set,
            theme: theme.clone(),
            rows: Vec::new(),
        };
        let mut seen = Vec::new();
        for s in SURF {
            let sh = c.col(s, None);
            if seen.contains(&sh) {
                continue;
            }
            seen.push(sh);
            for t in TEXT {
                let f = c.col(t, Some(sh));
                c.add(t, s, Kind::Text, f, sh);
            }
            for u in UI {
                let f = c.col(u, Some(sh));
                c.add(u, s, Kind::Ui, f, sh);
            }
            let dis = c.raw("fg.disabled").0;
            c.add("fg.disabled", s, Kind::Exempt, dis, sh);
        }
        let b = c.col("bg.base", None);
        for a in ["accent.solid", "accent.solid-hover", "accent.solid-pressed"] {
            let (f, bg) = (c.col("fg.on-accent", None), c.col(a, None));
            c.add("fg.on-accent", a, Kind::Text, f, bg);
        }
        for nm in ["success", "warning", "danger", "info", "waiting"] {
            let tint = c.col(&format!("status.{nm}.bg"), Some(b));
            let f = c.col(&format!("status.{nm}.fg"), None);
            c.add(
                &format!("status.{nm}.fg"),
                &format!("status.{nm}.bg/bg.base"),
                Kind::Text,
                f,
                tint,
            );
        }
        let (f, bg) = (c.col("accent.fg", None), c.col("accent.bg", Some(b)));
        c.add("accent.fg", "accent.bg/bg.base", Kind::Text, f, bg);
        let fg_default = c.col("fg.default", None);
        let fg_muted = c.col("fg.muted", None);
        for side in ["add", "del"] {
            let row = c.col(&format!("diff.{side}.bg"), Some(b));
            let (wc, wa) = c.raw(&format!("diff.{side}.word"));
            let word = wc.over(wa, row);
            let gut = c.col(&format!("diff.gutter.{side}"), Some(b));
            c.add(
                "fg.default",
                &format!("diff.{side}.bg/bg.base"),
                Kind::Text,
                fg_default,
                row,
            );
            c.add(
                "fg.default",
                &format!("diff.{side}.word/row"),
                Kind::Text,
                fg_default,
                word,
            );
            c.add(
                "fg.muted",
                &format!("diff.gutter.{side}/bg.base"),
                Kind::Text,
                fg_muted,
                gut,
            );
            let df = c.col(&format!("diff.{side}.fg"), None);
            c.add(
                &format!("diff.{side}.fg"),
                &format!("diff.{side}.bg/bg.base"),
                Kind::Text,
                df,
                row,
            );
        }
        let (f, bg) = (c.col("diff.hunk.header", None), c.col("diff.hunk.bg", None));
        c.add("diff.hunk.header", "diff.hunk.bg", Kind::Text, f, bg);
        let marker = c.col("merge.marker", Some(b));
        c.add(
            "fg.default",
            "merge.marker/bg.base",
            Kind::Text,
            fg_default,
            marker,
        );
        for m in ["merge.current", "merge.incoming"] {
            let tint = c.col(m, None).over(0.18, b);
            c.add(
                "fg.default",
                &format!("{m}@18%/bg.base"),
                Kind::Text,
                fg_default,
                tint,
            );
        }
        for (role, label) in [
            ("surface.selected", "surface.selected/bg.base"),
            ("selection", "selection/bg.base"),
        ] {
            let bg = c.col(role, Some(b));
            c.add("fg.default", label, Kind::Text, fg_default, bg);
        }
        let hl = c.col("highlight", None).over(0.24, b);
        c.add(
            "fg.default",
            "highlight@24%/bg.base",
            Kind::Text,
            fg_default,
            hl,
        );
        for role in ["code.inline.bg", "bubble.user"] {
            let bg = c.col(role, None);
            c.add("fg.default", role, Kind::Text, fg_default, bg);
        }
        let bubble = c.col("bubble.user", None);
        c.add("fg.muted", "bubble.user", Kind::Text, fg_muted, bubble);
        for i in 0..12 {
            let (f, bg) = (
                c.col(&format!("monogram.{i}.fg"), None),
                c.col(&format!("monogram.{i}.bg"), None),
            );
            c.add(
                &format!("monogram.{i}.fg"),
                &format!("monogram.{i}.bg"),
                Kind::Text,
                f,
                bg,
            );
        }
        for i in 0..16 {
            let exempt = (theme != "light" && i == 0) || (theme == "light" && (i == 7 || i == 15));
            let f = c.col(&format!("ansi.{i}"), None);
            let kind = if exempt { Kind::Exempt } else { Kind::Text };
            c.add(&format!("ansi.{i}"), "bg.base", kind, f, b);
        }
        all.extend(c.rows);
    }
    all
}
