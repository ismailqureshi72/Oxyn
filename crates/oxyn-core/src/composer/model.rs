//! Model picker model (`docs/specs/composer-model-picker.md`): catalogue, selection with the
//! mid-run policy, incompatibility checks, fuzzy search with match ranges, the popover's row list
//! and keyboard handling. Oxyn never ships model names or prices; hosts supply the catalogue.

use std::time::Duration;

use super::{effort::ReasoningSupport, mid_run};
use crate::{AgentState, ClockMs};

/// Model picker variants (`composer-model-picker.md` §6, free set).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ModelPickerVariant {
    /// Pill + floating popover with search, Auto, Recent, ★ and provider groups.
    #[default]
    PillPopover,
    /// The pill surface grows into the list.
    MorphList,
    /// Full-width command sheet over the composer.
    Command,
    /// Numbered terminal list with a `›` cursor.
    Glyph,
    /// Provider mark only; name in the tooltip.
    Compact,
    /// Two to four favourites as a segmented control plus "More…".
    FavoritesSegmented,
}

impl ModelPickerVariant {
    /// Every free variant.
    pub const ALL: [ModelPickerVariant; 6] = [
        Self::PillPopover,
        Self::MorphList,
        Self::Command,
        Self::Glyph,
        Self::Compact,
        Self::FavoritesSegmented,
    ];

    /// Short id; registry id is `model-picker/<id>`.
    pub fn id(self) -> &'static str {
        match self {
            Self::PillPopover => "pill-popover",
            Self::MorphList => "morph-list",
            Self::Command => "command",
            Self::Glyph => "glyph",
            Self::Compact => "compact",
            Self::FavoritesSegmented => "favorites-segmented",
        }
    }

    /// Parses `glyph` or `model-picker/glyph`.
    pub fn from_id(id: &str) -> Option<Self> {
        let id = id.strip_prefix("model-picker/").unwrap_or(id);
        Self::ALL.into_iter().find(|v| v.id() == id)
    }
}

/// What a model can take.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// Reads images.
    pub vision: bool,
    /// Calls tools.
    pub tools: bool,
    /// Reasoning control.
    pub reasoning: ReasoningSupport,
}

/// Can the model run right now.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ModelStatus {
    /// Ready.
    #[default]
    Available,
    /// No API key for the provider.
    NeedsKey,
    /// Rate limited until the given time.
    RateLimited {
        /// When it resets (host clock).
        until: ClockMs,
    },
    /// Not usable ("Provider outage", "Not on your plan").
    Unavailable {
        /// Reason.
        reason: String,
    },
    /// Retiring; selectable with a badge.
    Deprecated {
        /// Badge text, `Retires Dec 1`.
        sunset: String,
    },
}

/// Relative speed, supplied by the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier3 {
    /// Slow.
    Slow,
    /// Medium.
    Medium,
    /// Fast.
    Fast,
}

/// A model the host offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelInfo {
    /// Stable id (`provider/model`).
    pub id: String,
    /// Provider id; the provider's display name is in the catalogue.
    pub provider: String,
    /// Full name.
    pub name: String,
    /// Pill name, if shorter.
    pub short_name: Option<String>,
    /// What it can do.
    pub caps: Capabilities,
    /// Context window in tokens.
    pub context_tokens: u32,
    /// Relative speed.
    pub speed: Option<Tier3>,
    /// Host-supplied price text (`$3 / $15 per Mtok`). Oxyn never invents prices.
    pub cost: Option<String>,
    /// Availability.
    pub status: ModelStatus,
}

impl ModelInfo {
    /// The pill label.
    pub fn short(&self) -> &str {
        self.short_name.as_deref().unwrap_or(&self.name)
    }
}

/// `200K`, `1M`, `128K`, `1.5M`.
pub fn format_context(tokens: u32) -> String {
    if tokens >= 1_000_000 {
        let m = tokens as f64 / 1e6;
        if (m - m.round()).abs() < 0.05 {
            format!("{}M", m.round() as u32)
        } else {
            format!("{m:.1}M")
        }
    } else {
        format!("{}K", (tokens as f64 / 1e3).round() as u32)
    }
}

/// The host's catalogue.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelCatalog {
    /// Providers in display order: `(id, name)`.
    pub providers: Vec<(String, String)>,
    /// Models in display order within each provider.
    pub models: Vec<ModelInfo>,
    /// Offer an "Auto" row.
    pub auto: bool,
}

impl ModelCatalog {
    /// A model by id.
    pub fn get(&self, id: &str) -> Option<&ModelInfo> {
        self.models.iter().find(|m| m.id == id)
    }

    /// A provider's display name.
    pub fn provider_name<'a>(&'a self, id: &'a str) -> &'a str {
        self.providers
            .iter()
            .find(|(p, _)| p == id)
            .map(|(_, n)| n.as_str())
            .unwrap_or(id)
    }
}

/// The selected model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelSelection {
    /// A specific model.
    Model(String),
    /// The router chooses; `resolved` once the first event names it.
    Auto {
        /// The model the router picked for this turn.
        resolved: Option<String>,
    },
}

impl Default for ModelSelection {
    fn default() -> Self {
        Self::Auto { resolved: None }
    }
}

impl ModelSelection {
    /// The model id, if one is fixed or resolved.
    pub fn model_id(&self) -> Option<&str> {
        match self {
            Self::Model(id) => Some(id),
            Self::Auto { resolved } => resolved.as_deref(),
        }
    }

    /// Pill label: `Sonnet 5`, `Auto`, `Auto · Sonnet 5`.
    pub fn label(&self, catalog: &ModelCatalog) -> String {
        let name = |id: &str| {
            catalog
                .get(id)
                .map(|m| m.short().to_string())
                .unwrap_or_else(|| id.to_string())
        };
        match self {
            Self::Model(id) => name(id),
            Self::Auto { resolved: None } => "Auto".into(),
            Self::Auto { resolved: Some(id) } => format!("Auto · {}", name(id)),
        }
    }

    /// Auto is still choosing during a run (`Routing`).
    pub fn routing(&self, agent: &AgentState) -> bool {
        matches!(self, Self::Auto { resolved: None }) && mid_run(agent)
    }
}

/// What the draft needs from a model.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DraftNeeds {
    /// Images are attached.
    pub images: usize,
    /// The turn needs tools.
    pub tools: bool,
    /// Conversation size in tokens.
    pub context_tokens: u32,
}

/// Why `model` can't take this draft, if it can't (`Can't read the 2 attached images`).
pub fn incompatibility(model: &ModelInfo, needs: &DraftNeeds) -> Option<String> {
    if needs.images > 0 && !model.caps.vision {
        return Some(match needs.images {
            1 => "Can't read the attached image".into(),
            n => format!("Can't read the {n} attached images"),
        });
    }
    if needs.tools && !model.caps.tools {
        return Some("Can't use tools".into());
    }
    if needs.context_tokens > model.context_tokens {
        return Some(format!(
            "Conversation is {}, model holds {}",
            format_context(needs.context_tokens),
            format_context(model.context_tokens)
        ));
    }
    None
}

/// What a running turn does when the user picks another model.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MidRunPolicy {
    /// Applies from the next turn (default; the running turn is never re-routed silently).
    #[default]
    NextTurn,
    /// Restart the turn with the new model.
    ApplyNow,
    /// Ask with "Next turn" · "Restart turn".
    Ask,
}

/// The result of choosing a row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectOutcome {
    /// Selected now.
    Applied(ModelSelection),
    /// Selected for the next turn ("→ Opus" badge).
    PendingNext(ModelSelection),
    /// Selected and the running turn restarts with it.
    Restart(ModelSelection),
    /// Show the two-button choice in the row.
    Ask(ModelSelection),
    /// Start the host's key flow for this provider; the popover stays open.
    NeedsKey(String),
    /// Ask before selecting (incompatible with the draft).
    Confirm {
        /// The model.
        selection: ModelSelection,
        /// Why.
        reason: String,
    },
    /// Not selectable; the reason is announced.
    Disabled(String),
}

/// Applies the picker rules to a choice (`composer-model-picker.md` §2).
pub fn select(
    catalog: &ModelCatalog,
    choice: ModelSelection,
    agent: &AgentState,
    policy: MidRunPolicy,
    needs: &DraftNeeds,
    now: ClockMs,
) -> SelectOutcome {
    if let ModelSelection::Model(id) = &choice {
        let Some(m) = catalog.get(id) else {
            return SelectOutcome::Disabled("Unknown model".into());
        };
        match &m.status {
            ModelStatus::NeedsKey => return SelectOutcome::NeedsKey(m.provider.clone()),
            ModelStatus::Unavailable { reason } => return SelectOutcome::Disabled(reason.clone()),
            ModelStatus::RateLimited { until } if *until > now => {
                return SelectOutcome::Disabled(format!(
                    "Resets in {}",
                    countdown(until.since(now))
                ));
            }
            _ => {}
        }
        if let Some(reason) = incompatibility(m, needs) {
            return SelectOutcome::Confirm {
                selection: choice,
                reason,
            };
        }
    }
    if !mid_run(agent) {
        return SelectOutcome::Applied(choice);
    }
    match policy {
        MidRunPolicy::NextTurn => SelectOutcome::PendingNext(choice),
        MidRunPolicy::ApplyNow => SelectOutcome::Restart(choice),
        MidRunPolicy::Ask => SelectOutcome::Ask(choice),
    }
}

/// `4:12`.
pub fn countdown(d: Duration) -> String {
    let s = d.as_secs_f64().ceil() as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

/// The row's status note in the popover.
pub fn status_note(m: &ModelInfo, needs: &DraftNeeds, now: ClockMs) -> Option<String> {
    match &m.status {
        ModelStatus::NeedsKey => Some("Add API key".into()),
        ModelStatus::RateLimited { until } if *until > now => {
            Some(format!("Resets in {}", countdown(until.since(now))))
        }
        ModelStatus::Unavailable { reason } => Some(reason.clone()),
        ModelStatus::Deprecated { sunset } => Some(sunset.clone()),
        _ => incompatibility(m, needs),
    }
}

/// Cycles favourites (`Alt+M`), skipping unavailable and incompatible models.
pub fn cycle_favorite(
    catalog: &ModelCatalog,
    favorites: &[String],
    current: Option<&str>,
    needs: &DraftNeeds,
    forward: bool,
    now: ClockMs,
) -> Option<String> {
    let usable = |id: &String| {
        catalog.get(id).is_some_and(|m| {
            incompatibility(m, needs).is_none()
                && match &m.status {
                    ModelStatus::Available | ModelStatus::Deprecated { .. } => true,
                    ModelStatus::RateLimited { until } => *until <= now,
                    _ => false,
                }
        })
    };
    let n = favorites.len();
    if n == 0 {
        return None;
    }
    let start = current
        .and_then(|c| favorites.iter().position(|f| f == c))
        .unwrap_or(if forward { n - 1 } else { 0 });
    (1..=n)
        .map(|k| {
            if forward {
                (start + k) % n
            } else {
                (start + n - k % n) % n
            }
        })
        .map(|i| &favorites[i])
        .find(|id| usable(id) && Some(id.as_str()) != current)
        .cloned()
}

/// Case-insensitive subsequence match. Returns the matched char indices in `name`, preferring
/// matches at word starts, or `None`.
pub fn fuzzy(query: &str, name: &str) -> Option<(i32, Vec<usize>)> {
    let q: Vec<char> = query
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect();
    if q.is_empty() {
        return Some((0, Vec::new()));
    }
    let chars: Vec<char> = name.chars().collect();
    let lower: Vec<char> = chars
        .iter()
        .map(|c| c.to_lowercase().next().unwrap_or(*c))
        .collect();
    let mut hits = Vec::with_capacity(q.len());
    let mut score = 0i32;
    let mut qi = 0;
    let mut prev: Option<usize> = None;
    for (i, c) in lower.iter().enumerate() {
        if qi < q.len() && *c == q[qi] {
            let word_start = i == 0 || !chars[i - 1].is_alphanumeric();
            score += if word_start { 8 } else { 1 };
            if prev == Some(i.wrapping_sub(1)) {
                score += 4;
            }
            hits.push(i);
            prev = Some(i);
            qi += 1;
        }
    }
    (qi == q.len()).then(|| (score - hits[0] as i32, hits))
}

/// A row in the popover.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PickerRow {
    /// Section header (`Recent`, `Favourites`, provider name).
    Header(String),
    /// The Auto row.
    Auto,
    /// A model, with matched char indices of its name.
    Model {
        /// Model id.
        id: String,
        /// Matched character indices in the name (bold).
        matched: Vec<usize>,
    },
    /// `No model matches 'gpt-9'`.
    NoResults(String),
}

impl PickerRow {
    /// The row can be highlighted.
    pub fn selectable(&self) -> bool {
        matches!(self, Self::Auto | Self::Model { .. })
    }
}

/// A key the popover handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerKey {
    /// ↑.
    Up,
    /// ↓.
    Down,
    /// Enter.
    Enter,
    /// Esc.
    Escape,
    /// Backspace in the search field.
    Backspace,
    /// A typed character.
    Char(char),
}

/// What the popover asks the host to do after a key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PickerEvent {
    /// Nothing beyond a re-render.
    None,
    /// Close the popover.
    Close,
    /// The user chose a row; run [`select`].
    Choose(ModelSelection),
}

/// The popover's own state: open, search query, highlighted row.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Picker {
    /// Popover is open.
    pub open: bool,
    /// Search query.
    pub query: String,
    /// Index into the selectable rows.
    pub highlighted: usize,
}

impl Picker {
    /// Opens with the current model highlighted.
    pub fn open(&mut self, rows: &[PickerRow], current: &ModelSelection) {
        self.open = true;
        self.query.clear();
        let sel: Vec<_> = rows.iter().filter(|r| r.selectable()).collect();
        self.highlighted = sel
            .iter()
            .position(|r| match (r, current) {
                (PickerRow::Auto, ModelSelection::Auto { .. }) => true,
                (PickerRow::Model { id, .. }, ModelSelection::Model(c)) => id == c,
                _ => false,
            })
            .unwrap_or(0);
    }

    /// Rows for the current query. With no query: Auto, Recent (≤ 3, excluding the current
    /// model), Favourites, then provider groups. With a query: matches by score, no headers.
    pub fn rows(
        &self,
        catalog: &ModelCatalog,
        favorites: &[String],
        recent: &[String],
        current: &ModelSelection,
    ) -> Vec<PickerRow> {
        let row = |id: &str| PickerRow::Model {
            id: id.to_string(),
            matched: Vec::new(),
        };
        let q = self.query.trim();
        if !q.is_empty() {
            let mut hits: Vec<(i32, usize, PickerRow)> = catalog
                .models
                .iter()
                .enumerate()
                .filter_map(|(i, m)| {
                    let (s, matched) = fuzzy(q, &m.name)?;
                    Some((
                        s,
                        i,
                        PickerRow::Model {
                            id: m.id.clone(),
                            matched,
                        },
                    ))
                })
                .collect();
            if catalog.auto && fuzzy(q, "Auto").is_some() {
                hits.push((100, 0, PickerRow::Auto));
            }
            hits.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
            if hits.is_empty() {
                return vec![PickerRow::NoResults(format!("No model matches '{q}'"))];
            }
            return hits.into_iter().map(|h| h.2).collect();
        }
        let mut rows = Vec::new();
        if catalog.auto {
            rows.push(PickerRow::Auto);
        }
        let cur = current.model_id();
        let recent: Vec<_> = recent
            .iter()
            .filter(|id| Some(id.as_str()) != cur && catalog.get(id).is_some())
            .take(3)
            .collect();
        if !recent.is_empty() {
            rows.push(PickerRow::Header("Recent".into()));
            rows.extend(recent.iter().map(|id| row(id)));
        }
        let favs: Vec<_> = favorites
            .iter()
            .filter(|id| catalog.get(id).is_some())
            .collect();
        if !favs.is_empty() {
            rows.push(PickerRow::Header("Favourites".into()));
            rows.extend(favs.iter().map(|id| row(id)));
        }
        for (pid, pname) in &catalog.providers {
            let ms: Vec<_> = catalog
                .models
                .iter()
                .filter(|m| &m.provider == pid)
                .collect();
            if ms.is_empty() {
                continue;
            }
            rows.push(PickerRow::Header(pname.clone()));
            rows.extend(ms.iter().map(|m| row(&m.id)));
        }
        rows
    }

    /// The highlighted row among `rows`.
    pub fn highlighted_row<'a>(&self, rows: &'a [PickerRow]) -> Option<&'a PickerRow> {
        rows.iter().filter(|r| r.selectable()).nth(self.highlighted)
    }

    /// Handles a key against the rows it was rendered with. Esc clears the query first, then
    /// closes; typing resets the highlight to the best match.
    pub fn key(&mut self, key: PickerKey, rows: &[PickerRow]) -> PickerEvent {
        let n = rows.iter().filter(|r| r.selectable()).count();
        match key {
            PickerKey::Up if n > 0 => {
                self.highlighted = (self.highlighted + n - 1) % n;
                PickerEvent::None
            }
            PickerKey::Down if n > 0 => {
                self.highlighted = (self.highlighted + 1) % n;
                PickerEvent::None
            }
            PickerKey::Enter => match self.highlighted_row(rows) {
                Some(PickerRow::Auto) => {
                    PickerEvent::Choose(ModelSelection::Auto { resolved: None })
                }
                Some(PickerRow::Model { id, .. }) => {
                    PickerEvent::Choose(ModelSelection::Model(id.clone()))
                }
                _ => PickerEvent::None,
            },
            PickerKey::Escape if !self.query.is_empty() => {
                self.query.clear();
                self.highlighted = 0;
                PickerEvent::None
            }
            PickerKey::Escape => {
                self.open = false;
                PickerEvent::Close
            }
            PickerKey::Backspace => {
                self.query.pop();
                self.highlighted = 0;
                PickerEvent::None
            }
            PickerKey::Char(c) => {
                self.query.push(c);
                self.highlighted = 0;
                PickerEvent::None
            }
            _ => PickerEvent::None,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::composer::EffortLevel;

    pub(crate) fn catalog() -> ModelCatalog {
        let m = |id: &str, p: &str, name: &str, vision: bool, ctx: u32, status| ModelInfo {
            id: id.into(),
            provider: p.into(),
            name: name.into(),
            short_name: None,
            caps: Capabilities {
                vision,
                tools: true,
                reasoning: ReasoningSupport::Levels {
                    allowed: vec![EffortLevel::Low, EffortLevel::Medium, EffortLevel::High],
                    default: EffortLevel::Medium,
                },
            },
            context_tokens: ctx,
            speed: Some(Tier3::Medium),
            cost: None,
            status,
        };
        ModelCatalog {
            providers: vec![
                ("larch".into(), "Larch".into()),
                ("tallow".into(), "Tallow".into()),
            ],
            models: vec![
                m(
                    "larch/swift",
                    "larch",
                    "Swift 4",
                    true,
                    200_000,
                    ModelStatus::Available,
                ),
                m(
                    "larch/deep",
                    "larch",
                    "Deep 4",
                    true,
                    1_000_000,
                    ModelStatus::Available,
                ),
                m(
                    "tallow/text",
                    "tallow",
                    "Text One",
                    false,
                    128_000,
                    ModelStatus::NeedsKey,
                ),
            ],
            auto: true,
        }
    }

    #[test]
    fn mid_run_goes_next_turn() {
        let c = catalog();
        let pick = ModelSelection::Model("larch/deep".into());
        let needs = DraftNeeds::default();
        assert_eq!(
            select(
                &c,
                pick.clone(),
                &AgentState::Idle,
                MidRunPolicy::NextTurn,
                &needs,
                ClockMs(0)
            ),
            SelectOutcome::Applied(pick.clone())
        );
        assert_eq!(
            select(
                &c,
                pick.clone(),
                &AgentState::Streaming,
                MidRunPolicy::NextTurn,
                &needs,
                ClockMs(0)
            ),
            SelectOutcome::PendingNext(pick)
        );
        assert_eq!(
            select(
                &c,
                ModelSelection::Model("tallow/text".into()),
                &AgentState::Idle,
                MidRunPolicy::NextTurn,
                &needs,
                ClockMs(0)
            ),
            SelectOutcome::NeedsKey("tallow".into())
        );
    }

    #[test]
    fn incompatible_needs_confirm() {
        let c = catalog();
        let needs = DraftNeeds {
            images: 2,
            ..Default::default()
        };
        let m = c.get("tallow/text").unwrap();
        assert_eq!(
            incompatibility(m, &needs).as_deref(),
            Some("Can't read the 2 attached images")
        );
        let big = DraftNeeds {
            context_tokens: 240_000,
            ..Default::default()
        };
        assert_eq!(
            incompatibility(c.get("larch/swift").unwrap(), &big).as_deref(),
            Some("Conversation is 240K, model holds 200K")
        );
    }

    #[test]
    fn search_and_keys() {
        let c = catalog();
        let mut p = Picker::default();
        let cur = ModelSelection::Model("larch/swift".into());
        let rows = p.rows(&c, &["larch/deep".into()], &["larch/swift".into()], &cur);
        assert_eq!(rows[0], PickerRow::Auto);
        assert!(rows.contains(&PickerRow::Header("Favourites".into())));
        assert!(
            !rows.contains(&PickerRow::Header("Recent".into())),
            "current is excluded"
        );
        p.open(&rows, &cur);
        assert_eq!(
            p.highlighted_row(&rows),
            Some(&PickerRow::Model {
                id: "larch/swift".into(),
                matched: vec![]
            })
        );
        for ch in "dp".chars() {
            p.key(PickerKey::Char(ch), &rows);
        }
        let rows = p.rows(&c, &[], &[], &cur);
        assert!(
            matches!(&rows[0], PickerRow::Model { id, matched } if id == "larch/deep" && matched == &vec![0, 3])
        );
        assert_eq!(
            p.key(PickerKey::Enter, &rows),
            PickerEvent::Choose(ModelSelection::Model("larch/deep".into()))
        );
        assert_eq!(p.key(PickerKey::Escape, &rows), PickerEvent::None);
        assert!(p.query.is_empty() && p.open);
        assert_eq!(p.key(PickerKey::Escape, &rows), PickerEvent::Close);
        p.query = "zzz".into();
        assert_eq!(
            p.rows(&c, &[], &[], &cur),
            vec![PickerRow::NoResults("No model matches 'zzz'".into())]
        );
    }

    #[test]
    fn cycling_skips_unusable() {
        let c = catalog();
        let favs = vec![
            "larch/swift".to_string(),
            "tallow/text".into(),
            "larch/deep".into(),
        ];
        let next = cycle_favorite(
            &c,
            &favs,
            Some("larch/swift"),
            &DraftNeeds::default(),
            true,
            ClockMs(0),
        );
        assert_eq!(next.as_deref(), Some("larch/deep"));
        assert_eq!(format_context(1_000_000), "1M");
        assert_eq!(format_context(128_000), "128K");
        assert_eq!(
            ModelSelection::Auto {
                resolved: Some("larch/swift".into())
            }
            .label(&c),
            "Auto · Swift 4"
        );
    }
}
