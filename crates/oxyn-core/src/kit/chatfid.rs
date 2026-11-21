//! affectUI fidelity pass, chunk 2 (plan #32): the models behind the exact **Model Picker** (rows
//! with speed, quality and cost dashes, thinking effort and the tray's first-token time and price)
//! and **Run Controls** (a run that thinks, streams, takes a steer mid-sentence, stops at a word,
//! resumes, finishes and then runs what was queued). Behaviour only; time is passed in.

use super::compose::Effort;

/// A model in the Model Picker. Strengths go 1..=5 (cost 5 is the most expensive); leave all three
/// `None` for a router like Auto and the row says "Varies".
#[derive(Clone, Debug, PartialEq)]
pub struct PickerModel {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// One line under the name.
    pub description: String,
    /// Letter on the tile (`+` for a router).
    pub mark: String,
    /// Brand for the coloured-logo variant (`claude`, `openai`, … or empty).
    pub brand: String,
    /// Speed 1..=5.
    pub speed: Option<u8>,
    /// Quality 1..=5.
    pub quality: Option<u8>,
    /// Cost 1..=5.
    pub cost: Option<u8>,
    /// First-token time, ms.
    pub latency_ms: Option<u32>,
    /// Price per 1M tokens, dollars.
    pub price_per_m: Option<f64>,
}

impl PickerModel {
    /// A rated model.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        brand: &str,
        stats: (u8, u8, u8),
        latency_ms: u32,
        price_per_m: f64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            mark: name.chars().next().map(String::from).unwrap_or_default(),
            brand: brand.into(),
            speed: Some(stats.0),
            quality: Some(stats.1),
            cost: Some(stats.2),
            latency_ms: Some(latency_ms),
            price_per_m: Some(price_per_m),
        }
    }

    /// A router: no strengths ("Varies").
    pub fn router(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            mark: "+".into(),
            brand: String::new(),
            speed: None,
            quality: None,
            cost: None,
            latency_ms: None,
            price_per_m: None,
        }
    }

    /// True for a router (no strengths).
    pub fn varies(&self) -> bool {
        self.speed.is_none() && self.quality.is_none() && self.cost.is_none()
    }
}

/// How much thinking multiplies the first-token time: ×1.6 low, ×2.5 medium, ×4 high.
pub fn effort_factor(e: Effort) -> f64 {
    match e {
        Effort::Low => 1.6,
        Effort::Medium => 2.5,
        _ => 4.0,
    }
}

/// The tray line for the pointed model: (`~2.1s`, `$25`), or `None` for a router. Thinking
/// multiplies the time by the effort factor.
pub fn tray_values(m: &PickerModel, thinking: bool, effort: Effort) -> Option<(String, String)> {
    let ms = u64::from(m.latency_ms?);
    let ms = if thinking {
        ms * (effort_factor(effort) * 10.0) as u64 / 10
    } else {
        ms
    };
    let tenths = (ms + 50) / 100;
    let p = m.price_per_m?;
    let price = if p.fract() == 0.0 {
        format!("${p:.0}")
    } else {
        format!("${p:.2}")
    };
    Some((format!("~{}.{}s", tenths / 10, tenths % 10), price))
}

/// The demo models: three rated models and Auto.
pub fn demo_models() -> Vec<PickerModel> {
    vec![
        PickerModel::new(
            "opus",
            "Opus 5.5",
            "Hardest problems",
            "claude",
            (2, 5, 5),
            2_100,
            25.0,
        ),
        PickerModel::new(
            "sonnet",
            "Sonnet 5.5",
            "Everyday work",
            "claude",
            (4, 4, 3),
            900,
            15.0,
        ),
        PickerModel::new(
            "haiku",
            "Haiku 5.5",
            "Quick and light",
            "claude",
            (5, 3, 1),
            400,
            4.0,
        ),
        PickerModel::router("auto", "Auto", "Picks per message"),
    ]
}

// ───────────────────────────────────────────────────────────────── Run Controls ──

/// A piece of an answer: words, a paragraph break, or the pill of a steer that landed here.
#[derive(Clone, Debug, PartialEq)]
pub enum Piece {
    /// Words.
    Text(String),
    /// Paragraph break.
    Break,
    /// A steer message, shown as a blue pill where it landed.
    Steer(String),
}

/// One turn of a run: what was asked and the answer it streams (with any steer built in).
#[derive(Clone, Debug, PartialEq)]
pub struct RunTurn {
    /// The message that started it (shown as a bubble; empty for the first, implied prompt).
    pub prompt: String,
    /// The answer, in pieces.
    pub pieces: Vec<Piece>,
}

impl RunTurn {
    /// A turn from text: blank lines are breaks, `[[steer:…]]` marks a steer pill.
    pub fn parse(prompt: &str, text: &str) -> Self {
        let mut pieces = vec![];
        for (pi, para) in text.split("\n\n").enumerate() {
            if pi > 0 {
                pieces.push(Piece::Break);
            }
            let mut rest = para;
            while let Some(a) = rest.find("[[steer:") {
                if !rest[..a].trim().is_empty() {
                    pieces.push(Piece::Text(rest[..a].trim().to_string()));
                }
                let b = rest[a..].find("]]").map_or(rest.len(), |b| a + b);
                pieces.push(Piece::Steer(rest[a + 8..b].to_string()));
                rest = rest.get(b + 2..).unwrap_or("");
            }
            if !rest.trim().is_empty() {
                pieces.push(Piece::Text(rest.trim().to_string()));
            }
        }
        Self {
            prompt: prompt.into(),
            pieces,
        }
    }

    /// Total words.
    pub fn words(&self) -> usize {
        self.pieces
            .iter()
            .map(|p| match p {
                Piece::Text(t) => t.split_whitespace().count(),
                _ => 0,
            })
            .sum()
    }

    /// The pieces with only the first `n` words shown (pills and breaks appear once the words
    /// before them have).
    pub fn upto(&self, n: usize) -> Vec<Piece> {
        let mut left = n;
        let mut out = vec![];
        for p in &self.pieces {
            match p {
                Piece::Text(t) => {
                    if left == 0 {
                        break;
                    }
                    let w: Vec<&str> = t.split_whitespace().collect();
                    let k = w.len().min(left);
                    out.push(Piece::Text(w[..k].join(" ")));
                    left -= k;
                    if k < w.len() {
                        break;
                    }
                }
                other => out.push(other.clone()),
            }
        }
        out
    }

    /// Whether a steer pill shows within the first `n` words.
    pub fn steered_by(&self, n: usize) -> bool {
        self.upto(n).iter().any(|p| matches!(p, Piece::Steer(_)))
    }
}

/// Thinking time before the first word, ms.
pub const THINK_MS: u64 = 900;
/// Time per streamed word, ms.
pub const WORD_MS: u64 = 55;

/// Words shown `elapsed_ms` into a turn (none while thinking).
pub fn words_at(elapsed_ms: u64, total: usize) -> usize {
    (elapsed_ms.saturating_sub(THINK_MS) / WORD_MS).min(total as u64) as usize
}

/// Run phase of a turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Nothing asked yet.
    Idle,
    /// Before the first word.
    Thinking,
    /// Streaming.
    Writing,
    /// Stopped at a word.
    Stopped,
    /// All words in.
    Done,
}

impl Phase {
    /// The header status word.
    pub fn label(self, steered: bool) -> &'static str {
        match self {
            Phase::Idle => "",
            Phase::Thinking => "Thinking…",
            Phase::Writing if steered => "Writing · steered",
            Phase::Writing => "Writing",
            Phase::Stopped => "Stopped",
            Phase::Done => "Done",
        }
    }
    /// Still producing words.
    pub fn running(self) -> bool {
        matches!(self, Phase::Thinking | Phase::Writing)
    }
}

/// Phase `elapsed_ms` into a turn of `total` words, unless stopped.
pub fn phase_at(elapsed_ms: u64, total: usize, stopped: bool) -> Phase {
    if stopped {
        Phase::Stopped
    } else if elapsed_ms < THINK_MS {
        Phase::Thinking
    } else if words_at(elapsed_ms, total) >= total {
        Phase::Done
    } else {
        Phase::Writing
    }
}

/// How long a finished turn took, "0:04".
pub fn took(total: usize) -> String {
    let s = (THINK_MS + total as u64 * WORD_MS).div_ceil(1000);
    format!("{}:{:02}", s / 60, s % 60)
}

/// The run permission levels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunPermission {
    /// Asks before every change.
    AskFirst,
    /// Edits files, asks for the rest.
    AutoEdit,
    /// Runs commands without asking (shown in the danger colour).
    FullAccess,
}

impl RunPermission {
    /// All, in menu order.
    pub const ALL: [RunPermission; 3] = [
        RunPermission::AskFirst,
        RunPermission::AutoEdit,
        RunPermission::FullAccess,
    ];
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            RunPermission::AskFirst => "Ask first",
            RunPermission::AutoEdit => "Auto-edit",
            RunPermission::FullAccess => "Full access",
        }
    }
    /// Line under the label.
    pub fn desc(self) -> &'static str {
        match self {
            RunPermission::AskFirst => "Asks before every change",
            RunPermission::AutoEdit => "Edits files, asks for the rest",
            RunPermission::FullAccess => "Runs commands without asking",
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            RunPermission::AskFirst => "ask",
            RunPermission::AutoEdit => "auto",
            RunPermission::FullAccess => "full",
        }
    }
    /// From an id (`ask` when unknown).
    pub fn from_id(s: &str) -> Self {
        match s {
            "auto" => RunPermission::AutoEdit,
            "full" => RunPermission::FullAccess,
            _ => RunPermission::AskFirst,
        }
    }
}

/// Context in use, percent, after `words` streamed words on top of `base`.
pub fn context_pct(base: f32, words: usize) -> u32 {
    (base + words as f32 * 0.12).clamp(0.0, 99.0).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_multiplies_by_effort() {
        let m = &demo_models()[0];
        assert_eq!(
            tray_values(m, false, Effort::Medium),
            Some(("~2.1s".into(), "$25".into()))
        );
        assert_eq!(tray_values(m, true, Effort::Medium).unwrap().0, "~5.3s");
        assert!(tray_values(&demo_models()[3], false, Effort::Low).is_none());
        assert!(demo_models()[3].varies());
    }

    #[test]
    fn turn_streams_with_its_steer() {
        let t = RunTurn::parse(
            "",
            "One two three. [[steer:Keep it short]] Four five.\n\nSix.",
        );
        assert_eq!(t.words(), 6);
        assert!(!t.steered_by(2));
        assert!(t.steered_by(3));
        assert_eq!(t.upto(3).len(), 2);
        assert_eq!(t.upto(6).len(), 5);
        assert_eq!(phase_at(0, 6, false), Phase::Thinking);
        assert_eq!(phase_at(THINK_MS + WORD_MS * 2, 6, false), Phase::Writing);
        assert_eq!(phase_at(THINK_MS + WORD_MS * 6, 6, false), Phase::Done);
        assert_eq!(phase_at(10, 6, true), Phase::Stopped);
        assert_eq!(Phase::Writing.label(true), "Writing · steered");
        assert_eq!(took(60), "0:05");
    }

    #[test]
    fn permissions_round_trip() {
        for p in RunPermission::ALL {
            assert_eq!(RunPermission::from_id(p.id()), p);
        }
        assert_eq!(context_pct(40.0, 50), 46);
    }
}
