//! AI special components (Arc "Special components · AI", Oxyn Pro): the UI-free state machines the
//! Dioxus components and the GPUI view share, so a press or a key does the same thing in both
//! renderers and every rule is testable without a window.
//!
//! Every model takes the current time in milliseconds (`now`) and stores timestamps instead of
//! running timers: what is on screen (a streamed word count, a progress, a playhead) is derived
//! from `now`, so a host only has to keep its clock moving. Presses are string ids
//! (`press(id, now)`) and keys are key names (`key(name, shift, now)`); both return the line the
//! polite live region should announce, when there is one.
//!
//! - [`timeline`]: Agent timeline: an orchestrated run on a shared time axis with a playhead.
//! - [`model_card`]: Model card: specs, prices, a monthly cost estimate, benchmarks, compare.
//! - [`workflow`]: AI workflow: trigger, AI step, branch and actions on one rail, with a test run.
//! - [`imagegen`]: Image generation: four variations forming on a dot matrix, upscale and vary.
//! - [`speech`]: Speech player: read aloud with a word highlight, speed and voice.
//! - [`meeting`]: Meeting summary: notes that stream in, quotes per line, action items to send.
//! - [`reply`]: Reply draft: quick replies, tone and length rewrites, edit and insert.
//! - [`handoff`]: Human handoff: an assistant hands a support chat to a person.
//! - [`pii`]: PII redaction: scan, placeholders, a safe prompt and a restored reply.
//! - [`cost`]: LLM cost calculator: models re-ranked by monthly cost as usage changes.
//! - [`keys`]: Provider keys: bring-your-own-key rows with reveal, copy, rotate and remove.
//! - [`genstats`]: Generation stats: TTFT, speed, tokens and cost of a streaming reply.
//! - [`translate`]: AI translate: a message translated sentence by sentence in place.
//! - [`tldr`]: TL;DR toggle: full, summary and key points of an article.
//! - [`connector`]: Connector setup: connect apps inline with a first sync and scopes.
//! - [`enrich`]: Enrich column: one prompt runs down a table column.
//! - [`router`]: Ticket router: incoming tickets classified into team lanes.
//! - [`ask`]: Ask your data: a question becomes SQL, a query and a chart.
//! - [`doccursor`]: Agent doc cursor: an agent edits a document beside a teammate.
//! - [`provstatus`]: Provider status: health, latency and 30-day uptime of AI providers.
//! - [`training`]: Training run: a fine-tuning job with a live loss curve.
//! - [`tone`]: Tone tuner: a reply rewritten by formality, detail and voice.
//! - [`mascot`]: AI, ghost and cube mascots (shared status model and drawing).
//! - [`reveal`]: Image reveal: an image denoising into view.
//! - [`spark`]: AI spark: an animated status mark.
//! - [`pixel`]: Pixel mascot: a pixel robot sprite.
//! - [`upgrade`]: Upgrade prompt: a limit-reached nudge with a billing toggle.
//! - [`referral`]: Referral card: invite link, email chips and a referral funnel.
//! - [`trial`]: Trial banner: days left in a trial as a banner, card or pill.
//! - [`visitors`]: Live visitors: a realtime count, map, pages and conversions.
//! - [`proof`]: Social proof toast: a stack of recent-activity toasts.

pub mod ask;
pub mod connector;
pub mod cost;
pub mod doccursor;
pub mod enrich;
pub mod genstats;
pub mod handoff;
pub mod imagegen;
pub mod keys;
pub mod mascot;
pub mod meeting;
pub mod model_card;
pub mod pii;
pub mod pixel;
pub mod proof;
pub mod provstatus;
pub mod referral;
pub mod reply;
pub mod reveal;
pub mod router;
pub mod spark;
pub mod speech;
pub mod timeline;
pub mod tldr;
pub mod tone;
pub mod training;
pub mod translate;
pub mod trial;
pub mod upgrade;
pub mod visitors;
pub mod workflow;

/// Page slugs of the AI special components, in gallery order.
pub const AI_SLUGS: [&str; 33] = [
    "agent-timeline",
    "model-card",
    "ai-workflow",
    "image-generation",
    "speech-player",
    "meeting-summary",
    "reply-draft",
    "human-handoff",
    "pii-redaction",
    "llm-cost-calculator",
    "provider-keys",
    "generation-stats",
    "ai-translate",
    "tldr-toggle",
    "connector-setup",
    "enrich-column",
    "ticket-router",
    "ask-your-data",
    "agent-doc-cursor",
    "provider-status",
    "training-run",
    "tone-tuner",
    "ai-mascot",
    "image-reveal",
    "ai-spark",
    "pixel-mascot",
    "ghost-mascot",
    "cube-mascot",
    "upgrade-prompt",
    "referral-card",
    "trial-banner",
    "live-visitors",
    "social-proof-toast",
];

/// Streaming pace shared by every component: one chunk of words every 34 ms.
pub const STREAM_TICK_MS: u64 = 34;

/// Words of `text` visible `elapsed` ms after streaming started, `per` words per tick.
pub fn words_at(text: &str, elapsed: u64, per: usize) -> usize {
    let n = text.split_whitespace().count();
    (((elapsed / STREAM_TICK_MS) as usize + 1) * per).min(n)
}

/// The first `n` whitespace-separated words of `text`, joined by single spaces.
pub fn first_words(text: &str, n: usize) -> String {
    text.split_whitespace()
        .take(n)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Milliseconds a whole text takes to stream at `per` words a tick.
pub fn stream_ms(text: &str, per: usize) -> u64 {
    let n = text.split_whitespace().count().max(1);
    (n.div_ceil(per.max(1)) as u64) * STREAM_TICK_MS
}

/// `1,234,567`.
pub fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// US dollars with cents under $100 and whole dollars above (`$0.16`, `$1,240`).
pub fn usd(v: f64) -> String {
    if v.abs() >= 100.0 {
        format!("${}", thousands(v.round().max(0.0) as u64))
    } else if v.abs() >= 0.01 || v == 0.0 {
        format!("${v:.2}")
    } else {
        format!("${v:.4}")
    }
}

/// Seconds with one decimal (`26.4s`).
pub fn secs(ms: u64) -> String {
    format!("{:.1}s", ms as f64 / 1000.0)
}

/// `m:ss`.
pub fn mmss(ms: u64) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

/// Token counts as `1M`, `128K`, `950`.
pub fn tokens_short(n: u64) -> String {
    if n >= 1_000_000 && n.is_multiple_of(1_000_000) {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1e6)
    } else if n >= 1_000 && n.is_multiple_of(1_000) {
        format!("{}K", n / 1_000)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1e3)
    } else {
        n.to_string()
    }
}

/// The number after `prefix` in a press id (`"mc-model-2"` with `"mc-model-"` is 2).
pub fn nth(id: &str, prefix: &str) -> Option<usize> {
    id.strip_prefix(prefix)?.parse().ok()
}

/// Moves `i` by a key in a list of `n` (Left/Up back, Right/Down on, Home, End), clamped.
pub fn rove(i: usize, n: usize, key: &str) -> usize {
    if n == 0 {
        return 0;
    }
    match key {
        "ArrowLeft" | "ArrowUp" | "left" | "up" => i.saturating_sub(1),
        "ArrowRight" | "ArrowDown" | "right" | "down" => (i + 1).min(n - 1),
        "Home" | "home" => 0,
        "End" | "end" => n - 1,
        _ => i.min(n - 1),
    }
}

/// Normalises key names from the DOM (`ArrowLeft`, ` `) and GPUI (`left`, `space`).
pub fn norm_key(k: &str) -> &str {
    match k {
        "left" => "ArrowLeft",
        "right" => "ArrowRight",
        "up" => "ArrowUp",
        "down" => "ArrowDown",
        "home" => "Home",
        "end" => "End",
        "pageup" => "PageUp",
        "pagedown" => "PageDown",
        "enter" => "Enter",
        "escape" => "Escape",
        " " | "space" => "Space",
        "tab" => "Tab",
        _ => k,
    }
}

/// A small deterministic hash (FNV-1a) for sample variety without randomness.
pub fn hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers() {
        assert_eq!(thousands(1_234_567), "1,234,567");
        assert_eq!(usd(0.16), "$0.16");
        assert_eq!(usd(1240.4), "$1,240");
        assert_eq!(tokens_short(1_000_000), "1M");
        assert_eq!(tokens_short(128_000), "128K");
        assert_eq!(mmss(62_000), "1:02");
        assert_eq!(words_at("a b c d e f g h", 0, 3), 3);
        assert_eq!(words_at("a b c d e f g h", 1000, 3), 8);
        assert_eq!(rove(0, 3, "ArrowLeft"), 0);
        assert_eq!(rove(1, 3, "End"), 2);
        assert_eq!(nth("mc-model-2", "mc-model-"), Some(2));
        assert_eq!(AI_SLUGS.len(), 33);
    }
}
