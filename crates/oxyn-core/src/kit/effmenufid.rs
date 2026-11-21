//! Thinking Selector family, menu variants (Effort list, Context + Effort dropdown, Model list with
//! effort submenu): roving highlight, grouped options and the fictional model list. Shared level
//! logic is in [`super::thinkfid`]. Behaviour only.

use super::thinkfid::{EffortLevel, effort_scale};

/// The highlighted row after a key: `delta` ±1 wraps around; `i64::MIN`/`MAX` jump to the ends.
pub fn menu_move(i: usize, delta: i64, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    match delta {
        i64::MIN => 0,
        i64::MAX => n - 1,
        d => (i as i64 + d).rem_euclid(n as i64) as usize,
    }
}

/// One context-window choice.
#[derive(Clone, Debug, PartialEq)]
pub struct ContextOption {
    /// Tokens.
    pub tokens: u64,
    /// A small tag after the size ("Higher usage"), empty for none.
    pub tag: String,
}

/// The demo's two windows: 512K, and 1M that costs more.
pub fn context_options() -> Vec<ContextOption> {
    vec![
        ContextOption {
            tokens: 512_000,
            tag: String::new(),
        },
        ContextOption {
            tokens: 1_000_000,
            tag: "Higher usage".into(),
        },
    ]
}

/// A model in the effort-suffixed list.
#[derive(Clone, Debug, PartialEq)]
pub struct EffortModel {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// The scale this model offers.
    pub levels: Vec<EffortLevel>,
    /// Its current level.
    pub level: usize,
}

/// Seven fictional models, each at High on the five-step scale (one at Max).
pub fn effort_models() -> Vec<EffortModel> {
    [
        ("vela", "Vela 5.3"),
        ("kite", "Kite K3"),
        ("ridge", "Ridge 5 Preview"),
        ("atlas", "Atlas 6"),
        ("atlas-s", "Atlas 6 Swift"),
        ("halo", "Halo 6.1"),
        ("nimbus", "Nimbus 3"),
    ]
    .iter()
    .enumerate()
    .map(|(i, (id, name))| EffortModel {
        id: (*id).into(),
        name: (*name).into(),
        levels: effort_scale(),
        level: if i == 6 { 4 } else { 2 },
    })
    .collect()
}

/// A model row's label with its effort suffix: "Atlas 6 · High" for screen readers.
pub fn model_effort_label(m: &EffortModel) -> String {
    let l = m.levels.get(m.level).map_or("", |l| l.label.as_str());
    format!("{} · {l}", m.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_and_data() {
        assert_eq!(menu_move(0, -1, 4), 3);
        assert_eq!(menu_move(3, 1, 4), 0);
        assert_eq!(menu_move(1, 1, 4), 2);
        assert_eq!(menu_move(2, i64::MIN, 4), 0);
        assert_eq!(menu_move(0, i64::MAX, 4), 3);
        assert_eq!(menu_move(0, 1, 0), 0);
        assert_eq!(context_options()[1].tag, "Higher usage");
        let m = effort_models();
        assert_eq!(m.len(), 7);
        assert_eq!(model_effort_label(&m[3]), "Atlas 6 · High");
        assert_eq!(model_effort_label(&m[6]), "Nimbus 3 · Max");
    }
}
