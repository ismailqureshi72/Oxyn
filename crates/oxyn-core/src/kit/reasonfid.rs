//! Thinking Selector family, variant "Reasoning panel": the model behind a model-details panel with
//! a notched reasoning slider (R steps it), a Fast mode switch (F) and a Favorite button. Shared
//! level logic is in [`super::thinkfid`]. Behaviour only.

/// What a single key does inside the panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelKey {
    /// R: the next reasoning level, wrapping after the top.
    Reasoning,
    /// F: Fast mode on or off.
    Fast,
}

/// The panel's single-key shortcuts (either case); `None` for any other key or a modified one.
pub fn panel_key(key: &str, modified: bool) -> Option<PanelKey> {
    if modified {
        return None;
    }
    match key {
        "r" | "R" => Some(PanelKey::Reasoning),
        "f" | "F" => Some(PanelKey::Fast),
        _ => None,
    }
}

/// A token count as the panel shows it: 1_100_000 → "1.1M", 512_000 → "512K", 200 → "200".
pub fn context_label(tokens: u64) -> String {
    fn trim(x: f64) -> String {
        let s = format!("{x:.1}");
        s.strip_suffix(".0").map(str::to_string).unwrap_or(s)
    }
    if tokens >= 1_000_000 {
        format!("{}M", trim(tokens as f64 / 1_000_000.0))
    } else if tokens >= 1_000 {
        format!("{}K", trim(tokens as f64 / 1_000.0))
    } else {
        tokens.to_string()
    }
}

/// The model the demo panel describes (fictional).
#[derive(Clone, Debug, PartialEq)]
pub struct PanelModel {
    /// Name.
    pub name: String,
    /// Context window in tokens.
    pub context: u64,
    /// Whether it has a Fast mode.
    pub has_fast: bool,
}

/// The demo model: "Atlas 6.1", 1.1M context, with Fast mode.
pub fn panel_demo_model() -> PanelModel {
    PanelModel {
        name: "Atlas 6.1".into(),
        context: 1_100_000,
        has_fast: true,
    }
}

/// The status line after Fast mode flips.
pub fn fast_said(on: bool) -> &'static str {
    if on {
        "Fast mode on: shorter waits, same reasoning level"
    } else {
        "Fast mode off"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_and_labels() {
        assert_eq!(panel_key("r", false), Some(PanelKey::Reasoning));
        assert_eq!(panel_key("F", false), Some(PanelKey::Fast));
        assert_eq!(panel_key("f", true), None);
        assert_eq!(panel_key("x", false), None);
        assert_eq!(context_label(1_100_000), "1.1M");
        assert_eq!(context_label(1_000_000), "1M");
        assert_eq!(context_label(512_000), "512K");
        assert_eq!(context_label(200), "200");
        assert_eq!(panel_demo_model().name, "Atlas 6.1");
        assert!(fast_said(true).starts_with("Fast mode on"));
    }
}
