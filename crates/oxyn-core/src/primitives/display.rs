//! Display primitives: badge, chip, value pill, monogram + entity chip, avatar with agent ring,
//! kbd, separator, spinner and empty state. All framework-free: renderers read the derived values.

use std::time::Duration;

use oxyn_tokens::ColorRole;

use crate::{AgentState, ClockMs};

// ---------------------------------------------------------------------------------------------
// Tones

/// Badge / pill tone. The six affectUI tones plus Oxyn-plus agent states. Neutral is the only
/// chrome tone; every other tone is semantic (status, agent identity).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Tone {
    /// Grey.
    #[default]
    Neutral,
    /// Info.
    Info,
    /// Success.
    Success,
    /// Warning.
    Warning,
    /// Danger.
    Danger,
    /// affectUI `purple`: an agent's identity colour (agent violet).
    Purple,
    /// Oxyn plus: agent working.
    Working,
    /// Oxyn plus: waiting on the user.
    Waiting,
    /// Oxyn plus: queued.
    Queued,
    /// Oxyn plus: stalled.
    Stalled,
}

impl Tone {
    /// The six source tones.
    pub const SOURCE: [Self; 6] = [
        Self::Neutral,
        Self::Info,
        Self::Success,
        Self::Warning,
        Self::Danger,
        Self::Purple,
    ];
    /// All ten.
    pub const ALL: [Self; 10] = [
        Self::Neutral,
        Self::Info,
        Self::Success,
        Self::Warning,
        Self::Danger,
        Self::Purple,
        Self::Working,
        Self::Waiting,
        Self::Queued,
        Self::Stalled,
    ];
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Purple => "purple",
            Self::Working => "working",
            Self::Waiting => "waiting",
            Self::Queued => "queued",
            Self::Stalled => "stalled",
        }
    }
    /// Parses an id; Beautiful UI value-pill names (`green`, `orange`, `red`, `accent`) map on.
    pub fn from_id(s: &str) -> Option<Self> {
        Some(match s {
            "neutral" | "accent" => Self::Neutral,
            "info" => Self::Info,
            "success" | "green" => Self::Success,
            "warning" | "orange" => Self::Warning,
            "danger" | "red" => Self::Danger,
            "purple" => Self::Purple,
            "working" => Self::Working,
            "waiting" => Self::Waiting,
            "queued" => Self::Queued,
            "stalled" => Self::Stalled,
            _ => return None,
        })
    }
    /// `(fg, bg)` roles.
    pub fn roles(self) -> (ColorRole, ColorRole) {
        match self {
            Self::Neutral => (ColorRole::FgMuted, ColorRole::SurfaceHover),
            Self::Info => (ColorRole::StatusInfoFg, ColorRole::StatusInfoBg),
            Self::Success => (ColorRole::StatusSuccessFg, ColorRole::StatusSuccessBg),
            Self::Warning => (ColorRole::StatusWarningFg, ColorRole::StatusWarningBg),
            Self::Danger => (ColorRole::StatusDangerFg, ColorRole::StatusDangerBg),
            Self::Purple => (ColorRole::AgentViolet, ColorRole::SurfaceHover),
            Self::Working => (ColorRole::StateThinking, ColorRole::SurfaceHover),
            Self::Waiting => (ColorRole::StatusWaitingFg, ColorRole::StatusWaitingBg),
            Self::Queued => (ColorRole::StateQueued, ColorRole::SurfaceHover),
            Self::Stalled => (ColorRole::StateStalled, ColorRole::StatusWarningBg),
        }
    }
    /// The tone an agent state reads as.
    pub fn of(agent: &AgentState) -> Self {
        match agent {
            AgentState::Idle | AgentState::Composing => Self::Neutral,
            AgentState::Submitted
            | AgentState::Thinking { .. }
            | AgentState::Streaming
            | AgentState::ToolRunning { .. }
            | AgentState::Steering => Self::Working,
            AgentState::AwaitingApproval { .. } => Self::Waiting,
            AgentState::Queued { .. } => Self::Queued,
            AgentState::Done { .. } => Self::Success,
            AgentState::Error { .. } => Self::Danger,
            AgentState::Cancelled { .. } => Self::Neutral,
            AgentState::Stalled { .. } => Self::Stalled,
        }
    }
}

/// Fill style (Oxyn plus: the source has soft only; Arc adds `outline`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Fill {
    /// Tinted background (default).
    #[default]
    Soft,
    /// Border only.
    Outline,
    /// Strong fill.
    Solid,
    /// Oxyn plus: monospace tag, no fill.
    Mono,
}

impl Fill {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Soft => "soft",
            Self::Outline => "outline",
            Self::Solid => "solid",
            Self::Mono => "mono",
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Badge

/// A count as a badge shows it: up to `max`, then `99+`.
pub fn badge_count(n: u64, max: u64) -> String {
    if n > max {
        format!("{max}+")
    } else {
        n.to_string()
    }
}

/// The live dot's halo: grows out of the dot and fades every 1.8 s. Returns `(scale, opacity)`.
pub fn live_halo(now: ClockMs, start: ClockMs, reduced: bool) -> (f32, f32) {
    if reduced {
        return (1.0, 0.0);
    }
    let p = (now.since(start).as_millis() % LIVE_PERIOD.as_millis()) as f32
        / LIVE_PERIOD.as_millis() as f32;
    (1.0 + 1.6 * p, 0.45 * (1.0 - p))
}

/// The live halo's period (affectUI 1.8 s).
pub const LIVE_PERIOD: Duration = Duration::from_millis(1800);

/// Badge.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Badge {
    /// Text.
    pub label: String,
    /// Tone.
    pub tone: Tone,
    /// Fill.
    pub fill: Fill,
    /// Leading dot.
    pub dot: bool,
    /// The dot pulses (something runs).
    pub live: bool,
    /// Round count instead of a label.
    pub count: Option<u64>,
    /// Count cap (99).
    pub max: u64,
    /// Arc `pill`: fully round; otherwise 6 px corners.
    pub pill: bool,
}

impl Badge {
    /// A badge.
    pub fn new(label: &str, tone: Tone) -> Self {
        Self {
            label: label.into(),
            tone,
            max: 99,
            pill: true,
            ..Self::default()
        }
    }
    /// Text shown.
    pub fn text(&self) -> String {
        match self.count {
            Some(n) => badge_count(n, self.max),
            None => self.label.clone(),
        }
    }
    /// Accessible text: a count says what it counts; tone is spoken because colour is not enough.
    pub fn aria(&self) -> String {
        match self.count {
            Some(n) if n > self.max => format!("More than {} {}", self.max, self.label)
                .trim()
                .into(),
            Some(n) => format!("{n} {}", self.label).trim().into(),
            None if self.live => format!("{} (in progress)", self.label),
            None => self.label.clone(),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Monogram + entity chip

/// Number of monogram colour pairs in the tokens.
pub const MONOGRAM_COLOURS: usize = 12;

/// Stable colour index for a name (FNV-1a over the lowercased name).
pub fn seed_index(seed: &str, n: usize) -> usize {
    let mut h: u32 = 0x811c_9dc5;
    for b in seed.trim().to_lowercase().bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    (h as usize) % n.max(1)
}

/// `(bg, fg)` roles of monogram colour `i`.
pub fn monogram_roles(i: usize) -> (ColorRole, ColorRole) {
    use ColorRole as C;
    const BG: [C; 12] = [
        C::Monogram0Bg,
        C::Monogram1Bg,
        C::Monogram2Bg,
        C::Monogram3Bg,
        C::Monogram4Bg,
        C::Monogram5Bg,
        C::Monogram6Bg,
        C::Monogram7Bg,
        C::Monogram8Bg,
        C::Monogram9Bg,
        C::Monogram10Bg,
        C::Monogram11Bg,
    ];
    const FG: [C; 12] = [
        C::Monogram0Fg,
        C::Monogram1Fg,
        C::Monogram2Fg,
        C::Monogram3Fg,
        C::Monogram4Fg,
        C::Monogram5Fg,
        C::Monogram6Fg,
        C::Monogram7Fg,
        C::Monogram8Fg,
        C::Monogram9Fg,
        C::Monogram10Fg,
        C::Monogram11Fg,
    ];
    (BG[i % 12], FG[i % 12])
}

/// Initials: first letters of the first and last words ("Cone King" → "CK", "Ada" → "A").
pub fn initials(name: &str) -> String {
    let words: Vec<&str> = name
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_' || c == '.')
        .filter(|w| w.chars().next().is_some_and(char::is_alphanumeric))
        .collect();
    let first = |w: &str| {
        w.chars()
            .next()
            .map(|c| c.to_uppercase().collect::<String>())
    };
    match words.as_slice() {
        [] => "?".into(),
        [one] => first(one).unwrap_or_default(),
        [a, .., b] => format!(
            "{}{}",
            first(a).unwrap_or_default(),
            first(b).unwrap_or_default()
        ),
    }
}

/// Monogram: letters on a seeded colour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Monogram {
    /// Letters (1–2).
    pub letters: String,
    /// Colour index (0..12).
    pub colour: usize,
}

impl Monogram {
    /// From a name; `colour` overrides the seeded colour.
    pub fn of(name: &str, colour: Option<usize>, letters: Option<&str>) -> Self {
        Self {
            letters: letters
                .map(str::to_string)
                .unwrap_or_else(|| initials(name)),
            colour: colour.unwrap_or_else(|| seed_index(name, MONOGRAM_COLOURS)) % MONOGRAM_COLOURS,
        }
    }
}

/// Entity chip (Beautiful UI `entity-chip`): inline company / person with a monogram. Oxyn plus:
/// a hover card, an avatar image, a status dot.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityChip {
    /// Name ("Cone King").
    pub name: String,
    /// Monogram.
    pub monogram: Monogram,
    /// Oxyn plus: image instead of the monogram.
    pub image: Option<String>,
    /// Oxyn plus: status dot.
    pub status: Option<Presence>,
    /// Oxyn plus: hover card lines (title, then detail rows).
    pub card: Vec<(String, String)>,
}

impl EntityChip {
    /// An entity.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            monogram: Monogram::of(name, None, None),
            image: None,
            status: None,
            card: vec![],
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Value pill

/// Direction of a value change (Oxyn plus: delta arrows).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delta {
    /// Up ▲.
    Up,
    /// Down ▼.
    Down,
    /// Unchanged.
    Flat,
}

impl Delta {
    /// From two values.
    pub fn of(prev: f64, now: f64) -> Self {
        if (now - prev).abs() < f64::EPSILON {
            Self::Flat
        } else if now > prev {
            Self::Up
        } else {
            Self::Down
        }
    }
    /// Glyph.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Up => "▲",
            Self::Down => "▼",
            Self::Flat => "–",
        }
    }
    /// Spoken.
    pub fn word(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Flat => "unchanged",
        }
    }
    /// Tone when up is good (`good_up`) or bad (cost, latency).
    pub fn tone(self, good_up: bool) -> Tone {
        match (self, good_up) {
            (Self::Flat, _) => Tone::Neutral,
            (Self::Up, true) | (Self::Down, false) => Tone::Success,
            _ => Tone::Danger,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Avatar

/// Presence dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Presence {
    /// Green.
    Online,
    /// Amber.
    Away,
    /// Red.
    Busy,
    /// Hollow grey (Arc `offline`).
    Offline,
}

impl Presence {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Away => "away",
            Self::Busy => "busy",
            Self::Offline => "offline",
        }
    }
    /// Colour role.
    pub fn role(self) -> ColorRole {
        match self {
            Self::Online => ColorRole::StatusSuccessFg,
            Self::Away => ColorRole::StatusWarningFg,
            Self::Busy => ColorRole::StatusDangerFg,
            Self::Offline => ColorRole::FgSubtle,
        }
    }
}

/// Avatar shape: circles for people, rounded squares for workspaces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AvatarShape {
    /// Circle.
    #[default]
    Circle,
    /// Rounded square.
    Square,
}

/// Oxyn plus: the seven avatar sizes (affectUI takes any number; default 32).
pub const AVATAR_SIZES: [f32; 7] = [12.0, 16.0, 24.0, 32.0, 40.0, 56.0, 96.0];

/// Picture fade-in once loaded (affectUI: over the initials, never an empty circle).
pub const AVATAR_IMAGE_FADE: Duration = oxyn_tokens::motion::dur::BASE;

/// Agent mark twinkle period while working (affectUI 1.2 s).
pub const AGENT_WAVE: Duration = Duration::from_millis(1200);

/// Oxyn plus: the agent's 5×5 pixel mark, mirrored left-right, from a seed. `true` = lit.
pub fn agent_mark(seed: &str) -> [[bool; 5]; 5] {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in seed.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    let mut g = [[false; 5]; 5];
    for (r, row) in g.iter_mut().enumerate() {
        for c in 0..3 {
            let bit = (h >> (r * 3 + c)) & 1 == 1;
            row[c] = bit;
            row[4 - c] = bit;
        }
    }
    // Never empty, never a full square: light the centre, clear one corner pair.
    g[2][2] = true;
    if g.iter().flatten().all(|&x| x) {
        g[0][0] = false;
        g[0][4] = false;
    }
    g
}

/// Opacity of cell `(r, c)` in the working wave (diagonal, 1.2 s). 1.0 when not working.
pub fn agent_cell_opacity(r: usize, c: usize, phase: f32, working: bool, reduced: bool) -> f32 {
    if !working || reduced {
        return 1.0;
    }
    let off = (r + c) as f32 / 8.0;
    let x = (phase - off).rem_euclid(1.0);
    // Dip to 0.35 and back once per period.
    let dip = (x * std::f32::consts::TAU).cos() * 0.5 + 0.5;
    0.35 + 0.65 * dip
}

/// The ring an agent avatar wears for its state (scope: "avatar with agent ring").
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AgentRing {
    /// No ring.
    None,
    /// A short arc going round (thinking, submitted).
    Orbit,
    /// A full ring breathing (streaming).
    Breathe,
    /// A dashed ring turning (tool running).
    Dashed,
    /// A steady ring in the waiting colour (awaiting approval).
    Waiting,
    /// A solid ring, one flash (done).
    Done,
    /// A solid danger ring (error).
    Error,
    /// A warning ring that ramps (stalled).
    Stalled,
    /// Oxyn plus: progress `0..1` drawn as an arc from 12 o'clock.
    Progress(f32),
}

impl AgentRing {
    /// The ring for an agent state.
    pub fn of(agent: &AgentState) -> Self {
        match agent {
            AgentState::Idle | AgentState::Composing | AgentState::Cancelled { .. } => Self::None,
            AgentState::Submitted | AgentState::Thinking { .. } | AgentState::Steering => {
                Self::Orbit
            }
            AgentState::Streaming => Self::Breathe,
            AgentState::ToolRunning { .. } => Self::Dashed,
            AgentState::AwaitingApproval { .. } | AgentState::Queued { .. } => Self::Waiting,
            AgentState::Done { .. } => Self::Done,
            AgentState::Error { .. } => Self::Error,
            AgentState::Stalled { .. } => Self::Stalled,
        }
    }
    /// Id.
    pub fn id(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Orbit => "orbit",
            Self::Breathe => "breathe",
            Self::Dashed => "dashed",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Error => "error",
            Self::Stalled => "stalled",
            Self::Progress(_) => "progress",
        }
    }
    /// Colour role.
    pub fn role(&self) -> ColorRole {
        match self {
            Self::None => ColorRole::BorderSubtle,
            Self::Orbit | Self::Breathe | Self::Dashed => ColorRole::StateThinking,
            Self::Waiting => ColorRole::StatusWaitingFg,
            Self::Done => ColorRole::StatusSuccessFg,
            Self::Error => ColorRole::StatusDangerFg,
            Self::Stalled => ColorRole::StateStalled,
            Self::Progress(_) => ColorRole::FgDefault,
        }
    }
    /// The ring moves (and so needs frames).
    pub fn animated(&self) -> bool {
        matches!(self, Self::Orbit | Self::Breathe | Self::Dashed)
    }
    /// Spoken suffix ("working").
    pub fn word(&self) -> Option<&'static str> {
        Some(match self {
            Self::None => return None,
            Self::Orbit | Self::Breathe | Self::Dashed => "working",
            Self::Waiting => "waiting for you",
            Self::Done => "done",
            Self::Error => "failed",
            Self::Stalled => "not responding",
            Self::Progress(_) => "in progress",
        })
    }
}

/// Avatar.
#[derive(Clone, Debug, PartialEq)]
pub struct Avatar {
    /// Name (alt text and initials).
    pub name: String,
    /// Picture.
    pub src: Option<String>,
    /// Size px (default 32).
    pub size: f32,
    /// Shape.
    pub shape: AvatarShape,
    /// Presence dot.
    pub status: Option<Presence>,
    /// Agent (pixel mark) instead of a person.
    pub agent: bool,
    /// Colour seed (defaults to the name).
    pub seed: Option<String>,
    /// Explicit monogram colour.
    pub colour: Option<usize>,
    /// Agent mark twinkles.
    pub working: bool,
    /// Agent ring.
    pub ring: AgentRing,
    /// Oxyn plus: model badge in the corner ("4o", "S").
    pub model: Option<String>,
}

impl Avatar {
    /// Arc UI size presets: `sm` 28, `md` 40, `lg` 56, `xl` 88 px (plan #26 gap).
    pub fn preset_size(id: &str) -> Option<f32> {
        match id {
            "sm" => Some(28.0),
            "md" => Some(40.0),
            "lg" => Some(56.0),
            "xl" => Some(88.0),
            _ => None,
        }
    }
    /// A person.
    pub fn person(name: &str) -> Self {
        Self {
            name: name.into(),
            src: None,
            size: 32.0,
            shape: AvatarShape::Circle,
            status: None,
            agent: false,
            seed: None,
            colour: None,
            working: false,
            ring: AgentRing::None,
            model: None,
        }
    }
    /// An agent, ringed by its state.
    pub fn agent(name: &str, state: &AgentState) -> Self {
        let ring = AgentRing::of(state);
        Self {
            agent: true,
            shape: AvatarShape::Square,
            working: ring.animated(),
            ring,
            ..Self::person(name)
        }
    }
    /// Monogram.
    pub fn monogram(&self) -> Monogram {
        let seed = self.seed.as_deref().unwrap_or(&self.name);
        Monogram {
            letters: initials(&self.name),
            colour: self
                .colour
                .unwrap_or_else(|| seed_index(seed, MONOGRAM_COLOURS))
                % MONOGRAM_COLOURS,
        }
    }
    /// Status dot diameter (≈ 28 % of the avatar, at least 6 px).
    pub fn dot(&self) -> f32 {
        (self.size * 0.28).max(6.0).round()
    }
    /// Initials font size.
    pub fn font(&self) -> f32 {
        (self.size * 0.4).max(6.0)
    }
    /// Corner radius (square: 25 %).
    pub fn radius(&self) -> f32 {
        match self.shape {
            AvatarShape::Circle => self.size / 2.0,
            AvatarShape::Square => (self.size * 0.25).round(),
        }
    }
    /// Accessible name ("Ada Lovelace, online", "Coder, working").
    pub fn aria(&self) -> String {
        let mut s = self.name.clone();
        if let Some(p) = self.status {
            s.push_str(", ");
            s.push_str(p.id());
        }
        if let Some(w) = self.ring.word() {
            s.push_str(", ");
            s.push_str(w);
        }
        s
    }
}

/// Avatar group: the first `max` avatars and a `+N` tile (`total` overrides the count).
pub fn avatar_overflow(len: usize, max: usize, total: Option<usize>) -> (usize, Option<usize>) {
    let total = total.unwrap_or(len).max(len);
    let shown = len.min(max);
    let rest = total - shown;
    (shown, (rest > 0).then_some(rest))
}

// ---------------------------------------------------------------------------------------------
// Kbd

/// Platform for shortcut glyphs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Platform {
    /// ⌘ ⇧ ⌥ ⌃.
    #[default]
    Mac,
    /// Ctrl Shift Alt.
    Other,
}

/// One key of a shortcut, as shown and as spoken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    /// Shown ("⌘", "K").
    pub glyph: String,
    /// Spoken ("Command").
    pub name: String,
}

/// Parses `"Mod+Shift+K"` (also `"⌘K"`-free ASCII forms: `Cmd`, `Ctrl`, `Alt`, `Opt`, `Enter`,
/// `Esc`, arrows). `Mod` is ⌘ on Mac and Ctrl elsewhere (Oxyn plus: platform-aware glyphs).
/// A space separates a sequence ("G I" = G then I).
pub fn parse_shortcut(s: &str, platform: Platform) -> Vec<Vec<Key>> {
    let mac = platform == Platform::Mac;
    s.split_whitespace()
        .map(|chord| {
            chord
                .split('+')
                .filter(|k| !k.is_empty())
                .map(|k| {
                    let (g, n): (&str, &str) = match k.to_lowercase().as_str() {
                        "mod" if mac => ("⌘", "Command"),
                        "mod" => ("Ctrl", "Control"),
                        "cmd" | "command" | "meta" => ("⌘", "Command"),
                        "ctrl" | "control" if mac => ("⌃", "Control"),
                        "ctrl" | "control" => ("Ctrl", "Control"),
                        "shift" if mac => ("⇧", "Shift"),
                        "shift" => ("Shift", "Shift"),
                        "alt" | "opt" | "option" if mac => ("⌥", "Option"),
                        "alt" | "opt" | "option" => ("Alt", "Alt"),
                        "enter" | "return" => ("↵", "Enter"),
                        "esc" | "escape" => ("Esc", "Escape"),
                        "tab" => ("⇥", "Tab"),
                        "backspace" => ("⌫", "Backspace"),
                        "delete" | "del" => ("⌦", "Delete"),
                        "space" => ("Space", "Space"),
                        "up" => ("↑", "Up"),
                        "down" => ("↓", "Down"),
                        "left" => ("←", "Left"),
                        "right" => ("→", "Right"),
                        _ => {
                            let up = k.to_uppercase();
                            return Key {
                                glyph: up.clone(),
                                name: up,
                            };
                        }
                    };
                    Key {
                        glyph: g.into(),
                        name: n.into(),
                    }
                })
                .collect()
        })
        .collect()
}

/// `aria-keyshortcuts` value ("Meta+Shift+K").
pub fn aria_keyshortcuts(s: &str, platform: Platform) -> String {
    let mac = platform == Platform::Mac;
    s.split_whitespace()
        .map(|chord| {
            chord
                .split('+')
                .map(|k| match k.to_lowercase().as_str() {
                    "mod" if mac => "Meta".to_string(),
                    "mod" | "ctrl" | "control" => "Control".to_string(),
                    "cmd" | "command" => "Meta".to_string(),
                    "opt" | "option" => "Alt".to_string(),
                    "esc" => "Escape".to_string(),
                    _ => {
                        let mut c = k.chars();
                        c.next()
                            .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                            .unwrap_or_default()
                    }
                })
                .collect::<Vec<_>>()
                .join("+")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------------------------
// Spinner

/// Spinner style (Oxyn plus: affectUI has the arc only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpinnerStyle {
    /// A 2 px arc on a faint ring.
    #[default]
    Arc,
    /// Eight dots fading round.
    Dots,
    /// Twelve rays (system style).
    Ray,
    /// One dot orbiting.
    Orbit,
}

impl SpinnerStyle {
    /// Every style.
    pub const ALL: [Self; 4] = [Self::Arc, Self::Dots, Self::Ray, Self::Orbit];
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Arc => "arc",
            Self::Dots => "dots",
            Self::Ray => "ray",
            Self::Orbit => "orbit",
        }
    }
}

/// Spinner.
#[derive(Clone, Debug, PartialEq)]
pub struct Spinner {
    /// Size (affectUI default 14).
    pub size: f32,
    /// Announced only when set.
    pub label: Option<String>,
    /// One turn (affectUI 750 ms).
    pub period: Duration,
    /// Arc length as a fraction of the circle (Oxyn plus; default 0.25).
    pub arc: f32,
    /// Style.
    pub style: SpinnerStyle,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            size: 14.0,
            label: None,
            period: SPINNER_PERIOD,
            arc: 0.25,
            style: SpinnerStyle::Arc,
        }
    }
}

/// One turn every 750 ms, linear.
pub const SPINNER_PERIOD: Duration = Duration::from_millis(750);
/// Reduced motion: a slow 2.4 s turn instead of a freeze, so "working" still reads.
pub const SPINNER_REDUCED_PERIOD: Duration = Duration::from_millis(2400);
/// Track opacity (ring at 20 % of the colour).
pub const SPINNER_TRACK: f32 = 0.2;

impl Spinner {
    /// Stroke width: 2 px up to 24 px, then 1/12 of the size.
    pub fn stroke(&self) -> f32 {
        if self.size <= 24.0 {
            2.0
        } else {
            (self.size / 12.0).round()
        }
    }
    /// Rotation in degrees at `now`.
    pub fn angle(&self, now: ClockMs, reduced: bool) -> f32 {
        let p = if reduced {
            SPINNER_REDUCED_PERIOD
        } else {
            self.period
        };
        (now.0 % p.as_millis() as u64) as f32 / p.as_millis() as f32 * 360.0
    }
    /// `stroke-dasharray` for the arc: `(dash, gap)` on circumference `c`.
    pub fn dash(&self) -> (f32, f32) {
        let r = (self.size - self.stroke()) / 2.0;
        let c = std::f32::consts::TAU * r;
        let a = self.arc.clamp(0.05, 0.95);
        (c * a, c * (1.0 - a))
    }
    /// Per-segment opacity for dots / rays at `now` (`n` segments; the head is brightest).
    pub fn segments(&self, n: usize, now: ClockMs, reduced: bool) -> Vec<f32> {
        let head = (self.angle(now, reduced) / 360.0 * n as f32).floor() as usize % n.max(1);
        (0..n)
            .map(|i| {
                let behind = (head + n - i) % n;
                (1.0 - behind as f32 / n as f32).max(0.15)
            })
            .collect()
    }
    /// `role=status` + name only when there is a label.
    pub fn announced(&self) -> bool {
        self.label.is_some()
    }
}

// ---------------------------------------------------------------------------------------------
// Separator

/// Separator.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Separator {
    /// Orientation.
    pub vertical: bool,
    /// Purely visual (`role=none`); otherwise `role=separator` with `aria-orientation`.
    pub decorative: bool,
    /// Oxyn plus: centred label ("Today", "or").
    pub label: Option<String>,
    /// Oxyn plus: dashed.
    pub dashed: bool,
    /// Oxyn plus: ends fade out.
    pub fade: bool,
    /// Inset from the start (px), for list dividers under an icon column.
    pub inset: f32,
}

// ---------------------------------------------------------------------------------------------
// Chip

/// Chip behaviour (Oxyn plus: toggle vs removable on one component).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ChipKind {
    /// Static label.
    #[default]
    Static,
    /// Toggles `selected` (`aria-pressed`).
    Toggle,
    /// Has an × that removes it (Backspace / Delete when focused).
    Removable,
}

/// Chip.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chip {
    /// Label.
    pub label: String,
    /// Kind.
    pub kind: ChipKind,
    /// Pressed / picked.
    pub selected: bool,
    /// Oxyn plus: count badge.
    pub count: Option<u64>,
    /// Tone of the leading dot (status chips).
    pub dot: Option<Tone>,
    /// Disabled.
    pub disabled: bool,
}

impl Chip {
    /// A chip.
    pub fn new(label: &str, kind: ChipKind) -> Self {
        Self {
            label: label.into(),
            kind,
            ..Self::default()
        }
    }
    /// Activate (click / Enter / Space). Returns the new `selected` for toggles.
    pub fn activate(&mut self) -> Option<bool> {
        if self.disabled || self.kind != ChipKind::Toggle {
            return None;
        }
        self.selected = !self.selected;
        Some(self.selected)
    }
    /// The remove button's name ("Remove Rust").
    pub fn remove_label(&self) -> String {
        format!("Remove {}", self.label)
    }
}

// ---------------------------------------------------------------------------------------------
// Empty state

/// Empty-state layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EmptyLayout {
    /// Dashed frame (default).
    #[default]
    Framed,
    /// No frame (`bare`).
    Bare,
    /// Oxyn plus: one line inside a table row or list.
    Inline,
}

impl EmptyLayout {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Framed => "framed",
            Self::Bare => "bare",
            Self::Inline => "inline",
        }
    }
}

/// Empty state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EmptyState {
    /// "No runs yet".
    pub title: String,
    /// One line.
    pub text: String,
    /// Primary action label.
    pub action: Option<String>,
    /// Oxyn plus: secondary action / docs link.
    pub secondary: Option<String>,
    /// Icon name (default `file`).
    pub icon: String,
    /// Layout.
    pub layout: EmptyLayout,
    /// Taller block for page-level empties.
    pub tall: bool,
    /// Oxyn plus: starter prompts (agent apps: "Summarise this repo").
    pub prompts: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_counts_and_aria() {
        assert_eq!(badge_count(7, 99), "7");
        assert_eq!(badge_count(140, 99), "99+");
        let mut b = Badge::new("unread", Tone::Info);
        b.count = Some(120);
        assert_eq!(b.text(), "99+");
        assert_eq!(b.aria(), "More than 99 unread");
        let mut l = Badge::new("Running", Tone::Working);
        l.live = true;
        assert_eq!(l.aria(), "Running (in progress)");
        assert_eq!(Tone::from_id("orange"), Some(Tone::Warning));
        let (s0, o0) = live_halo(ClockMs(0), ClockMs(0), false);
        let (s1, o1) = live_halo(ClockMs(900), ClockMs(0), false);
        assert!(s1 > s0 && o1 < o0);
        assert_eq!(live_halo(ClockMs(900), ClockMs(0), true).1, 0.0);
    }

    #[test]
    fn initials_and_monograms() {
        assert_eq!(initials("Cone King"), "CK");
        assert_eq!(initials("ada"), "A");
        assert_eq!(initials("Jean-Luc Picard"), "JP");
        assert_eq!(initials("  "), "?");
        assert_eq!(seed_index("Cone King", 12), seed_index("cone king ", 12));
        let m = Monogram::of("Northwind", Some(14), None);
        assert_eq!((m.letters.as_str(), m.colour), ("N", 2));
    }

    #[test]
    fn avatar_and_ring() {
        let a = Avatar::agent("Coder", &AgentState::Streaming);
        assert_eq!(a.ring, AgentRing::Breathe);
        assert!(a.working);
        assert_eq!(a.aria(), "Coder, working");
        let mut p = Avatar::person("Ada Lovelace");
        p.status = Some(Presence::Online);
        assert_eq!(p.aria(), "Ada Lovelace, online");
        assert_eq!(p.radius(), 16.0);
        assert_eq!(avatar_overflow(6, 4, None), (4, Some(2)));
        assert_eq!(avatar_overflow(3, 4, Some(12)), (3, Some(9)));
        assert_eq!(avatar_overflow(2, 4, None), (2, None));
        let m = agent_mark("Coder");
        assert!(m[2][2]);
        for row in m {
            assert_eq!(row[0], row[4]);
            assert_eq!(row[1], row[3]);
        }
        assert_ne!(agent_mark("Coder"), agent_mark("Reviewer"));
        assert_eq!(agent_cell_opacity(0, 0, 0.3, false, false), 1.0);
        assert!(agent_cell_opacity(0, 0, 0.5, true, false) < 0.5);
    }

    #[test]
    fn shortcuts() {
        let k = parse_shortcut("Mod+Shift+K", Platform::Mac);
        assert_eq!(
            k[0].iter().map(|k| k.glyph.as_str()).collect::<String>(),
            "⌘⇧K"
        );
        let k = parse_shortcut("Mod+Enter", Platform::Other);
        assert_eq!(k[0][0].glyph, "Ctrl");
        assert_eq!(k[0][1].name, "Enter");
        assert_eq!(parse_shortcut("G I", Platform::Mac).len(), 2);
        assert_eq!(
            aria_keyshortcuts("Mod+Shift+k", Platform::Mac),
            "Meta+Shift+K"
        );
        assert_eq!(aria_keyshortcuts("Mod+k", Platform::Other), "Control+K");
    }

    #[test]
    fn spinner_geometry() {
        let s = Spinner::default();
        assert_eq!(s.stroke(), 2.0);
        assert_eq!(s.angle(ClockMs(375), false), 180.0);
        assert_eq!(s.angle(ClockMs(1200), true), 180.0);
        let (d, g) = s.dash();
        assert!((d / (d + g) - 0.25).abs() < 1e-4);
        let seg = s.segments(8, ClockMs(0), false);
        assert_eq!(seg[0], 1.0);
        assert!(!s.announced());
        assert_eq!(Spinner { size: 48.0, ..s }.stroke(), 4.0);
    }

    #[test]
    fn chips_and_deltas() {
        let mut c = Chip::new("Rust", ChipKind::Toggle);
        assert_eq!(c.activate(), Some(true));
        assert_eq!(c.activate(), Some(false));
        c.disabled = true;
        assert_eq!(c.activate(), None);
        assert_eq!(
            Chip::new("Rust", ChipKind::Removable).remove_label(),
            "Remove Rust"
        );
        assert_eq!(Delta::of(1.0, 2.0), Delta::Up);
        assert_eq!(Delta::Up.tone(false), Tone::Danger);
        assert_eq!(Tone::of(&AgentState::Idle), Tone::Neutral);
    }
}
