//! Screens S8: the onboarding wizard (Website, Brand, Tone, Ready). Four steps that set up a
//! support assistant, each a form on the left and the assistant's dashboard as a live preview on
//! the right. The rules live here so Dioxus and GPUI agree: domain clean-up and checks, the
//! simulated site fetch, brand colours and marks, the voice and its preview replies, what blocks
//! Continue, and the Ready summary. Every name, address and company is fictional.

pub use super::screensfid::{PERSON_EMAIL, ScreenState, WORKSPACE};

/// The product named in the footer and on the logo label.
pub const PRODUCT: &str = "Oxyn";
/// The company's own (fictional) domain, from the sign-up address.
pub const OWN_DOMAIN: &str = "kestrel.example";
/// The handover address in the sample.
pub const SUPPORT: &str = "support@kestrel.example";

/// The steps, in order: key and label.
pub const STEPS: [(&str, &str); 4] = [
    ("website", "Website"),
    ("brand", "Brand"),
    ("tone", "Tone"),
    ("ready", "Ready"),
];

/// A state list: key and label for the state bar.
pub type StateList = &'static [(&'static str, &'static str)];

const WEBSITE_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("fetching", "Reading site"),
];
const BRAND_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("custom", "Own colour"),
];
const TONE_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("testing", "Test reply"),
];
const READY_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("live", "Live"),
];

/// The screens' own states.
pub const OWN: [&str; 4] = ["fetching", "custom", "testing", "live"];

/// The states of an S8 screen by its slug.
pub fn states_for(slug: &str) -> StateList {
    match slug {
        "onboarding-brand" => BRAND_STATES,
        "onboarding-tone" => TONE_STATES,
        "onboarding-ready" => READY_STATES,
        _ => WEBSITE_STATES,
    }
}

/// Split a state key into the data state and the screen's own state ("" for none).
pub fn split(k: &str) -> (ScreenState, &'static str) {
    match OWN.iter().find(|o| **o == k) {
        Some(o) => (ScreenState::Sample, o),
        None => (ScreenState::from_key(k), ""),
    }
}

/// The step index a slug opens on.
pub fn step_of(slug: &str) -> usize {
    STEPS.iter().position(|s| slug.ends_with(s.0)).unwrap_or(0)
}

// ───────────────────────────────────────────────────────────── Website ──

/// How far reading a site has got.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Waiting its turn.
    Queued,
    /// Reading pages, 0–100.
    Reading(u8),
    /// Read: this many pages learned.
    Ready(u32),
    /// The site could not be reached (a private or unreachable address).
    Failed,
}

impl Phase {
    /// Short label for the Knowledge table.
    pub fn label(self) -> String {
        match self {
            Phase::Queued => "Queued".to_string(),
            Phase::Reading(p) => format!("Learning {p}%"),
            Phase::Ready(n) => format!("{n} pages"),
            Phase::Failed => "Can't reach".to_string(),
        }
    }
    /// Status tone: `run`, `ok`, `bad` or "" (neutral).
    pub fn tone(self) -> &'static str {
        match self {
            Phase::Queued => "",
            Phase::Reading(_) => "run",
            Phase::Ready(_) => "ok",
            Phase::Failed => "bad",
        }
    }
}

/// A domain the assistant learns from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Domain {
    /// The cleaned host (no scheme, www. or path).
    pub host: String,
    /// Where reading it has got.
    pub phase: Phase,
}

/// Clean what was typed into a bare host: drops the scheme, `www.`, a path, query or port, and
/// lowercases. Then checks it: a name, a dot and a real-looking ending.
pub fn clean_domain(raw: &str) -> Result<String, &'static str> {
    let mut s = raw.trim().to_lowercase();
    if s.is_empty() {
        return Err("Type a website or docs address");
    }
    for p in ["https://", "http://"] {
        if let Some(r) = s.strip_prefix(p) {
            s = r.to_string();
        }
    }
    if let Some(i) = s.find(['/', '?', '#']) {
        s.truncate(i);
    }
    if let Some(i) = s.find(':') {
        s.truncate(i);
    }
    if let Some(r) = s.strip_prefix("www.") {
        s = r.to_string();
    }
    let s = s.trim_end_matches('.').to_string();
    if s.contains(' ') || s.contains('@') {
        return Err("That looks like an email or a sentence, not an address");
    }
    if !s.contains('.') {
        return Err("Add the ending too, like .com");
    }
    let ok_chars = s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.');
    let parts: Vec<&str> = s.split('.').collect();
    let tld = parts.last().copied().unwrap_or("");
    if !ok_chars
        || parts
            .iter()
            .any(|p| p.is_empty() || p.starts_with('-') || p.ends_with('-'))
    {
        return Err("Only letters, numbers, dots and dashes");
    }
    if tld.len() < 2 || !tld.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("That ending isn't a real domain");
    }
    if s.len() > 253 {
        return Err("That address is too long");
    }
    Ok(s)
}

/// Add a typed address to the list, cleaned; an error says why it was not added.
pub fn add_domain(list: &mut Vec<Domain>, raw: &str) -> Result<String, &'static str> {
    let h = clean_domain(raw)?;
    if list.iter().any(|d| d.host == h) {
        return Err("Already added");
    }
    if list.len() >= MAX_DOMAINS {
        return Err("Up to 5 sites for now; add more from Knowledge later");
    }
    list.push(Domain {
        host: h.clone(),
        phase: Phase::Reading(0),
    });
    Ok(h)
}

/// The most sites the wizard takes.
pub const MAX_DOMAINS: usize = 5;

/// Whether a host is private (the assistant cannot read it from outside).
pub fn private_host(h: &str) -> bool {
    h.ends_with(".local")
        || h.ends_with(".internal")
        || h.ends_with(".lan")
        || h.starts_with("intranet.")
        || h.starts_with("localhost")
}

/// Pages a host yields when read (a stable, fictional count).
pub fn pages_of(h: &str) -> u32 {
    let sum: u32 = h.bytes().map(u32::from).sum();
    let base = if h.starts_with("docs.") || h.starts_with("help.") {
        60
    } else {
        12
    };
    base + sum % 48
}

/// Step one site's reading by one tick (25 %); private hosts fail on the first tick.
pub fn tick(d: &mut Domain) {
    d.phase = match d.phase {
        Phase::Queued => Phase::Reading(0),
        Phase::Reading(_) if private_host(&d.host) => Phase::Failed,
        Phase::Reading(p) if p >= 75 => Phase::Ready(pages_of(&d.host)),
        Phase::Reading(p) => Phase::Reading(p + 25),
        x => x,
    };
}

/// Retry a failed site.
pub fn retry(d: &mut Domain) {
    if d.phase == Phase::Failed {
        d.phase = Phase::Reading(0);
    }
}

/// "3 sources · 1 still reading" style line for the Knowledge tile.
pub fn sources_words(list: &[Domain]) -> String {
    let n = list.len();
    let reading = list
        .iter()
        .filter(|d| matches!(d.phase, Phase::Reading(_) | Phase::Queued))
        .count();
    let base = match n {
        0 => "No sources".to_string(),
        1 => "1 source".to_string(),
        _ => format!("{n} sources"),
    };
    if reading > 0 {
        format!("{base} · {reading} reading")
    } else {
        base
    }
}

/// Pages learned so far over every source.
pub fn pages_total(list: &[Domain]) -> u32 {
    list.iter()
        .map(|d| match d.phase {
            Phase::Ready(n) => n,
            _ => 0,
        })
        .sum()
}

// ─────────────────────────────────────────────────────────────── Brand ──

/// Brand colour swatches: name and hex (saturated; never a warm dark).
pub const SWATCHES: [(&str, &str); 8] = [
    ("Blue", "#2563EB"),
    ("Indigo", "#5B5BD6"),
    ("Violet", "#8E4EC6"),
    ("Pink", "#D6409F"),
    ("Red", "#E5484D"),
    ("Orange", "#F76B15"),
    ("Green", "#30A46C"),
    ("Teal", "#12A594"),
];

/// Logo marks: key and label. `upload` stands for a picked image (simulated).
pub const MARKS: [(&str, &str); 4] = [
    ("initials", "Initials"),
    ("square", "Square"),
    ("round", "Round"),
    ("upload", "Upload"),
];

/// Normalise a typed colour to `#RRGGBB` (accepts `#abc`, `abc`, `#aabbcc`).
pub fn hex_ok(raw: &str) -> Result<String, &'static str> {
    let s = raw.trim().trim_start_matches('#');
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Use a hex colour like #2563EB");
    }
    let full = match s.len() {
        3 => s.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 => s.to_string(),
        _ => return Err("Use 3 or 6 hex digits"),
    };
    Ok(format!("#{}", full.to_uppercase()))
}

fn rgb_of(hex: &str) -> (f32, f32, f32) {
    let h = hex.trim_start_matches('#');
    let v = |i: usize| {
        u8::from_str_radix(h.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0) as f32 / 255.0
    };
    (v(0), v(2), v(4))
}

fn lum(hex: &str) -> f32 {
    let f = |c: f32| {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = rgb_of(hex);
    0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b)
}

/// Contrast ratio of two colours (WCAG).
pub fn contrast(a: &str, b: &str) -> f32 {
    let (x, y) = (lum(a), lum(b));
    let (hi, lo) = if x > y { (x, y) } else { (y, x) };
    (hi + 0.05) / (lo + 0.05)
}

/// The ink on a brand colour: `#FFFFFF` or `#000000`, whichever reads better.
pub fn ink_on(hex: &str) -> &'static str {
    if contrast(hex, "#FFFFFF") >= contrast(hex, "#000000") {
        "#FFFFFF"
    } else {
        "#000000"
    }
}

/// A warning when a colour is too pale to read on white or too dark to read on the dark theme.
pub fn colour_warning(hex: &str) -> Option<&'static str> {
    if contrast(hex, "#FFFFFF") < 1.6 {
        Some("Very light: buttons on white will be hard to see")
    } else if contrast(hex, "#1C1C1E") < 1.6 {
        Some("Very dark: it will disappear in dark mode")
    } else {
        None
    }
}

/// Initials for a company mark (up to two letters).
pub fn initials(name: &str) -> String {
    let w: Vec<&str> = name.split_whitespace().collect();
    let mut s: String = w.iter().take(2).filter_map(|x| x.chars().next()).collect();
    if s.is_empty() {
        s = "?".to_string();
    }
    s.to_uppercase()
}

/// Add a company name or short form; an error says why not.
pub fn add_name(list: &mut Vec<String>, raw: &str) -> Result<String, &'static str> {
    let s = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.is_empty() {
        return Err("Type your company's name");
    }
    if s.chars().count() > NAME_CAP {
        return Err("Keep it under 40 characters");
    }
    if list.iter().any(|x| x.eq_ignore_ascii_case(&s)) {
        return Err("Already added");
    }
    if list.len() >= 4 {
        return Err("Up to 4 names");
    }
    list.push(s.clone());
    Ok(s)
}

/// Longest company or assistant name.
pub const NAME_CAP: usize = 40;

// ──────────────────────────────────────────────────────────────── Tone ──

/// Tones of voice.
pub const TONES: [&str; 3] = ["Friendly", "Neutral", "Formal"];
/// Reply lengths.
pub const LENGTHS: [&str; 3] = ["Short", "Balanced", "Detailed"];

/// The test question in the preview chat.
pub const TEST_Q: &str = "Can I change my plan mid-month?";

/// The assistant's greeting for a tone.
pub fn greeting(tone: usize, name: &str, company: &str) -> String {
    let n = if name.trim().is_empty() {
        "your assistant"
    } else {
        name.trim()
    };
    let c = if company.trim().is_empty() {
        "the team"
    } else {
        company.trim()
    };
    match tone {
        0 => format!("Hey! I'm {n}. Ask me anything about {c}: plans, products or your account."),
        2 => format!(
            "Good day. I am {n}, here to help with questions about {c}'s products, plans and your account."
        ),
        _ => format!("Hi, I'm {n}. Ask me anything about {c}'s products, plans or your account."),
    }
}

/// The preview's answer to [`TEST_Q`] for a tone and length.
pub fn answer(tone: usize, len: usize) -> String {
    let head = match tone {
        0 => "Yep, any time!",
        2 => "Certainly.",
        _ => "Yes.",
    };
    match len {
        0 => format!("{head} The difference is prorated."),
        2 => format!(
            "{head} You can switch plans whenever you like from Settings → Billing. The difference is prorated to the day, and the new limits apply straight away. Moving down takes effect at the next renewal."
        ),
        _ => format!("{head} You can change your plan at any time; the difference is prorated."),
    }
}

/// "Neutral, balanced replies".
pub fn voice_words(tone: usize, len: usize) -> String {
    let t = TONES.get(tone).copied().unwrap_or("Neutral");
    let l = LENGTHS
        .get(len)
        .copied()
        .unwrap_or("Balanced")
        .to_lowercase();
    format!("{t}, {l} replies")
}

/// A check of the assistant's name.
pub fn name_error(s: &str) -> Option<&'static str> {
    let t = s.trim();
    if t.is_empty() {
        Some("Give it a name")
    } else if t.chars().count() > NAME_CAP {
        Some("Keep it under 40 characters")
    } else {
        None
    }
}

/// A check of the handover address.
pub fn email_error(s: &str) -> Option<&'static str> {
    let t = s.trim();
    if t.is_empty() {
        return Some("Add an address to hand over to");
    }
    let mut it = t.splitn(2, '@');
    let (a, b) = (it.next().unwrap_or(""), it.next().unwrap_or(""));
    if a.is_empty() || b.is_empty() || t.contains(' ') || b.contains('@') {
        return Some("That isn't an email address");
    }
    if clean_domain(b).ok().as_deref() != Some(b) {
        return Some("Check the part after @");
    }
    None
}

// ────────────────────────────────────────────────────────────── Wizard ──

/// Everything the wizard collects.
#[derive(Clone, Debug, PartialEq)]
pub struct Wiz {
    /// Sites it learns from; the first is the company's own.
    pub domains: Vec<Domain>,
    /// Company name, then short forms it also answers to.
    pub names: Vec<String>,
    /// Brand colour, `#RRGGBB`.
    pub colour: String,
    /// Logo mark (index into [`MARKS`]).
    pub mark: usize,
    /// The assistant's name.
    pub name: String,
    /// Index into [`TONES`].
    pub tone: usize,
    /// Index into [`LENGTHS`].
    pub length: usize,
    /// Hand over to a person when unsure.
    pub handover: bool,
    /// Who it hands over to.
    pub handover_to: String,
    /// Published (Open dashboard pressed).
    pub live: bool,
}

/// The wizard's sample (`sample` true) or a blank start.
pub fn wiz(sample: bool) -> Wiz {
    if sample {
        Wiz {
            domains: vec![Domain {
                host: OWN_DOMAIN.to_string(),
                phase: Phase::Ready(pages_of(OWN_DOMAIN)),
            }],
            names: vec![WORKSPACE.to_string(), "Kestrel".to_string()],
            colour: SWATCHES[0].1.to_string(),
            mark: 0,
            name: format!("{WORKSPACE} Assistant"),
            tone: 1,
            length: 1,
            handover: true,
            handover_to: SUPPORT.to_string(),
            live: false,
        }
    } else {
        Wiz {
            domains: Vec::new(),
            names: Vec::new(),
            colour: SWATCHES[0].1.to_string(),
            mark: 0,
            name: String::new(),
            tone: 1,
            length: 1,
            handover: false,
            handover_to: String::new(),
            live: false,
        }
    }
}

/// The company name (the first name), or "".
pub fn company(w: &Wiz) -> &str {
    w.names.first().map(String::as_str).unwrap_or("")
}

/// What stops Continue on a step, if anything.
pub fn blocker(w: &Wiz, step: usize) -> Option<&'static str> {
    match step {
        0 if w.domains.is_empty() => Some("Add at least one site"),
        0 if w.domains.iter().all(|d| d.phase == Phase::Failed) => {
            Some("None of the sites could be read yet")
        }
        1 if w.names.is_empty() => Some("Add your company's name"),
        1 => colour_ok(&w.colour),
        2 => name_error(&w.name).or(if w.handover {
            email_error(&w.handover_to)
        } else {
            None
        }),
        3 => (0..3).find_map(|s| blocker(w, s)),
        _ => None,
    }
}

fn colour_ok(c: &str) -> Option<&'static str> {
    hex_ok(c).err()
}

/// The first step that still needs something, if any (Ready sends you there).
pub fn first_gap(w: &Wiz) -> Option<usize> {
    (0..3).find(|s| blocker(w, *s).is_some())
}

/// One row of the Ready summary: label, value, the step its Edit opens.
pub type SumRow = (&'static str, String, usize);

/// The Ready summary.
pub fn summary(w: &Wiz) -> Vec<SumRow> {
    let learns = if w.domains.is_empty() {
        "Nothing yet".to_string()
    } else {
        w.domains
            .iter()
            .map(|d| d.host.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let name = if w.name.trim().is_empty() {
        "Not named yet".to_string()
    } else {
        w.name.trim().to_string()
    };
    let brand = if w.names.is_empty() {
        "Not set".to_string()
    } else {
        let sw = SWATCHES
            .iter()
            .find(|s| s.1.eq_ignore_ascii_case(&w.colour))
            .map(|s| s.0.to_string())
            .unwrap_or_else(|| w.colour.clone());
        format!("{} · {}", company(w), sw)
    };
    let hand = if w.handover {
        if w.handover_to.trim().is_empty() {
            "No address yet".to_string()
        } else {
            w.handover_to.trim().to_string()
        }
    } else {
        "Answers everything itself".to_string()
    };
    vec![
        ("Learns from", learns, 0),
        ("Brand", brand, 1),
        ("Name", name, 2),
        ("Voice", voice_words(w.tone, w.length), 2),
        ("Hands over to", hand, 2),
    ]
}

/// The heading and lead of each step.
pub fn heading(step: usize) -> (&'static str, &'static str) {
    match step {
        1 => (
            "What's your company called?",
            "Your assistant introduces itself with it. Pick a colour and a mark, too.",
        ),
        2 => (
            "How should your assistant sound?",
            "Give it a name, a voice and a person to pass hard questions to.",
        ),
        3 => (
            "Your assistant is ready",
            "Here is what it knows. You can change any of it later.",
        ),
        _ => (
            "What's your company's website?",
            "Your assistant learns from it. Add your docs or help centre as well.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s8_onboarding() {
        assert_eq!(
            clean_domain("https://www.Kestrel.example/pricing?x=1").unwrap(),
            "kestrel.example"
        );
        assert_eq!(
            clean_domain("docs.kestrel.example:443/").unwrap(),
            "docs.kestrel.example"
        );
        assert!(clean_domain("kestrel").is_err());
        assert!(clean_domain("me@kestrel.example").is_err());
        assert!(clean_domain("kestrel.c0m").is_err());
        let mut l = Vec::new();
        assert!(add_domain(&mut l, "kestrel.example").is_ok());
        assert_eq!(
            add_domain(&mut l, "www.kestrel.example"),
            Err("Already added")
        );
        for _ in 0..5 {
            tick(&mut l[0]);
        }
        assert!(matches!(l[0].phase, Phase::Ready(_)));
        let mut p = Domain {
            host: "intranet.kestrel.example".into(),
            phase: Phase::Reading(0),
        };
        tick(&mut p);
        assert_eq!(p.phase, Phase::Failed);
        assert_eq!(hex_ok("2a6").unwrap(), "#22AA66");
        assert!(hex_ok("#12345").is_err());
        assert_eq!(ink_on("#2563EB"), "#FFFFFF");
        assert_eq!(ink_on("#FFE629"), "#000000");
        assert!(colour_warning("#FAFAFA").is_some());
        let w = wiz(true);
        assert!(first_gap(&w).is_none());
        assert_eq!(summary(&w)[3].1, "Neutral, balanced replies");
        let e = wiz(false);
        assert_eq!(first_gap(&e), Some(0));
        assert_eq!(blocker(&e, 2), Some("Give it a name"));
        assert_eq!(email_error("support@kestrel.example"), None);
        assert!(email_error("support@kestrel").is_some());
        assert!(answer(2, 2).len() > answer(0, 0).len());
        assert_eq!(split("live").1, "live");
        assert_eq!(step_of("onboarding-tone"), 2);
        assert_eq!(states_for("onboarding-ready")[3].0, "live");
        assert_eq!(initials("Kestrel Studio"), "KS");
    }
}
