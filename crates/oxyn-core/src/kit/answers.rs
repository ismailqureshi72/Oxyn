//! affectUI fidelity pass (plan #32, overnight batch, family A): the models behind the exact
//! **AI Answer** (a streamed answer with source cards, citation chips and feedback), **Cited Answer**
//! (sentences wired to their sources and quotes), **Message Actions** (versions, feedback reasons),
//! **Thinking pill** (phases on a timer), **Thinking panel** (steps and sources revealed one by
//! one) and **Writing Assistant** (a sample writer and simulated dictation). Behaviour only; time is
//! passed in as elapsed milliseconds.

// ───────────────────────────────────────────────────────────────────── AI Answer ──

/// A source behind an answer: a card in the sources row and a row in a citation preview.
#[derive(Clone, Debug, PartialEq)]
pub struct AnswerSource {
    /// Brand id for the coloured mark (`notion`, `slack`, `wikipedia`, …).
    pub brand: String,
    /// Short name ("Notion").
    pub name: String,
    /// Title of the page.
    pub title: String,
    /// Link.
    pub url: String,
}

impl AnswerSource {
    /// A source.
    pub fn new(brand: &str, name: &str, title: &str, url: &str) -> Self {
        Self {
            brand: brand.into(),
            name: name.into(),
            title: title.into(),
            url: url.into(),
        }
    }

    /// The host of the link without `www.` ("notion.so").
    pub fn domain(&self) -> String {
        domain_of(&self.url)
    }
}

/// The host of a link without the scheme or `www.`.
pub fn domain_of(url: &str) -> String {
    let rest = url.split("//").last().unwrap_or(url);
    let host = rest.split('/').next().unwrap_or(rest);
    host.trim_start_matches("www.").to_string()
}

/// A piece of answer text.
#[derive(Clone, Debug, PartialEq)]
pub enum Seg {
    /// A word (with its trailing punctuation).
    Word(String),
    /// A bold word (bullet lead-ins).
    Bold(String),
    /// A citation chip for these sources (0-based).
    Cite(Vec<usize>),
}

/// A block of answer text.
#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    /// A paragraph.
    Para(Vec<Seg>),
    /// A bullet list.
    Bullets(Vec<Vec<Seg>>),
}

/// A parsed answer. Source text: blank lines split paragraphs, lines starting `- ` are bullets,
/// `**bold**` marks lead-ins and `[1,2]` is a citation chip for sources 1 and 2 (1-based).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnswerDoc {
    /// Blocks in order.
    pub blocks: Vec<Block>,
}

fn parse_segs(line: &str) -> Vec<Seg> {
    let mut out = Vec::new();
    let mut bold = false;
    for tok in line.split_whitespace() {
        if tok.starts_with('[') && tok.ends_with(']') && tok.len() > 2 {
            let ids: Vec<usize> = tok[1..tok.len() - 1]
                .split(',')
                .filter_map(|x| x.trim().parse::<usize>().ok())
                .filter(|x| *x > 0)
                .map(|x| x - 1)
                .collect();
            if !ids.is_empty() {
                out.push(Seg::Cite(ids));
                continue;
            }
        }
        let mut w = tok.to_string();
        let starts = w.starts_with("**");
        if starts {
            bold = true;
            w = w.trim_start_matches("**").to_string();
        }
        let ends = w.contains("**");
        let w2 = w.replace("**", "");
        if bold {
            out.push(Seg::Bold(w2));
        } else {
            out.push(Seg::Word(w2));
        }
        if ends {
            bold = false;
        }
    }
    out
}

impl AnswerDoc {
    /// Parses the markup described on the type.
    pub fn parse(src: &str) -> Self {
        let mut blocks = Vec::new();
        for para in src.split("\n\n") {
            let lines: Vec<&str> = para
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect();
            if lines.is_empty() {
                continue;
            }
            if lines.iter().all(|l| l.starts_with("- ")) {
                blocks.push(Block::Bullets(
                    lines.iter().map(|l| parse_segs(&l[2..])).collect(),
                ));
            } else {
                blocks.push(Block::Para(parse_segs(&lines.join(" "))));
            }
        }
        Self { blocks }
    }

    /// Words in the answer (chips are not words).
    pub fn words(&self) -> usize {
        let count = |s: &[Seg]| s.iter().filter(|x| !matches!(x, Seg::Cite(_))).count();
        self.blocks
            .iter()
            .map(|b| match b {
                Block::Para(s) => count(s),
                Block::Bullets(items) => items.iter().map(|s| count(s)).sum(),
            })
            .sum()
    }

    /// The first `n` words; a chip shows once the word before it has.
    pub fn upto(&self, n: usize) -> Self {
        let mut left = n;
        let mut blocks = Vec::new();
        let take = |segs: &[Seg], left: &mut usize| -> Vec<Seg> {
            let mut out = Vec::new();
            for s in segs {
                match s {
                    Seg::Cite(_) => {
                        if !out.is_empty() {
                            out.push(s.clone())
                        }
                    }
                    _ => {
                        if *left == 0 {
                            break;
                        }
                        *left -= 1;
                        out.push(s.clone());
                    }
                }
            }
            out
        };
        for b in &self.blocks {
            if left == 0 {
                break;
            }
            match b {
                Block::Para(s) => blocks.push(Block::Para(take(s, &mut left))),
                Block::Bullets(items) => {
                    let mut got = Vec::new();
                    for s in items {
                        if left == 0 {
                            break;
                        }
                        got.push(take(s, &mut left));
                    }
                    blocks.push(Block::Bullets(got));
                }
            }
        }
        Self { blocks }
    }

    /// Plain text for copying (chips dropped, bullets as `- `).
    pub fn plain(&self) -> String {
        let line = |s: &[Seg]| {
            s.iter()
                .filter_map(|x| match x {
                    Seg::Word(w) | Seg::Bold(w) => Some(w.as_str()),
                    Seg::Cite(_) => None,
                })
                .collect::<Vec<_>>()
                .join(" ")
        };
        self.blocks
            .iter()
            .map(|b| match b {
                Block::Para(s) => line(s),
                Block::Bullets(items) => items
                    .iter()
                    .map(|s| format!("- {}", line(s)))
                    .collect::<Vec<_>>()
                    .join("\n"),
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

/// How long the answer reads its sources before writing, ms.
pub const ANSWER_READ_MS: u64 = 1400;
/// Default streaming speed, words a second (affectUI's default).
pub const ANSWER_WPS: u64 = 38;

/// What the answer is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnswerPhase {
    /// Source cards pop in ("Reading N sources").
    Reading,
    /// Text streams ("Writing", a Stop button).
    Writing,
    /// Finished ("Answer").
    Done,
    /// Stopped by the user.
    Stopped,
}

/// Words shown `elapsed` ms after the start at `wps` words a second.
pub fn answer_words_at(elapsed: u64, total: usize, wps: u64) -> usize {
    if elapsed <= ANSWER_READ_MS {
        return 0;
    }
    ((elapsed - ANSWER_READ_MS).saturating_mul(wps.max(1)) / 1000).min(total as u64) as usize
}

/// The phase `elapsed` ms after the start.
pub fn answer_phase(elapsed: u64, total: usize, wps: u64) -> AnswerPhase {
    if elapsed < ANSWER_READ_MS {
        AnswerPhase::Reading
    } else if answer_words_at(elapsed, total, wps) < total {
        AnswerPhase::Writing
    } else {
        AnswerPhase::Done
    }
}

/// The reasons offered after a thumbs-down on an answer.
pub const ANSWER_REASONS: [&str; 4] = ["Not accurate", "Wrong source", "Too long", "Out of date"];

/// The demo answer: question, sources, hidden source count, answer markup, follow-ups.
pub fn demo_answer() -> (String, Vec<AnswerSource>, usize, String, Vec<String>) {
    (
        "Which city should host the spring team retreat?".into(),
        vec![
            AnswerSource::new(
                "notion",
                "Notion",
                "Retreat plan – venues and budget",
                "https://www.notion.so/retreat-plan",
            ),
            AnswerSource::new(
                "slack",
                "Slack",
                "#retreat – who can travel when",
                "https://slack.com/archives/retreat",
            ),
            AnswerSource::new(
                "wikipedia",
                "Wikipedia",
                "Ribeira – the riverside quarter of Porto",
                "https://en.wikipedia.org/wiki/Ribeira_(Porto)",
            ),
            AnswerSource::new(
                "airbnb",
                "Airbnb",
                "Two riverside flats in Ribeira",
                "https://www.airbnb.com/rooms/ribeira",
            ),
            AnswerSource::new(
                "google",
                "Google Flights",
                "Direct flights to Porto in April",
                "https://www.google.com/travel/flights",
            ),
        ],
        3,
        "Porto suits the 14-person retreat best: compact, calm in the mornings and inside the budget you set. [1,3]\n\n\
- **Dates:** 22–24 April works for everyone; two people join on day two. [2]\n\
- **Stay:** two linked flats by the river come to 2,640 EUR for three nights, about 18% under the hotel. [4,1]\n\
- **Flights:** direct flights from Berlin and Amsterdam land before 11:00, so day one can start after lunch. [5]\n\n\
Book the flats this week – only one April weekend is still open. [4]"
            .into(),
        vec![
            "Compare the flats with the hotel".into(),
            "Draft a two-day agenda".into(),
        ],
    )
}

// ───────────────────────────────────────────────────────────────── Cited Answer ──

/// A source of a cited answer, with the passage the answer leans on.
#[derive(Clone, Debug, PartialEq)]
pub struct CitedSource {
    /// Letter on its tile.
    pub mark: String,
    /// Tile tint, `RRGGBB`.
    pub tint: String,
    /// Publication.
    pub name: String,
    /// Site.
    pub domain: String,
    /// Published.
    pub date: String,
    /// Article title.
    pub title: String,
    /// The quoted passage.
    pub quote: String,
    /// Words of the quote that match the answer (marked).
    pub marked: Vec<String>,
    /// Link.
    pub url: String,
}

/// A sentence and the sources (0-based) it cites.
#[derive(Clone, Debug, PartialEq)]
pub struct CitedSentence {
    /// Text.
    pub text: String,
    /// Cited sources.
    pub cites: Vec<usize>,
}

/// Splits markup into paragraphs of sentences: blank lines split paragraphs and each run of
/// `[n]` markers (1-based) ends the sentence before it; text after the last marker is uncited.
pub fn parse_cited(src: &str) -> Vec<Vec<CitedSentence>> {
    let mut out = Vec::new();
    for para in src.split("\n\n") {
        let mut sents = Vec::new();
        let mut text = String::new();
        let mut cites: Vec<usize> = Vec::new();
        let mut rest = para.trim();
        while !rest.is_empty() {
            if let Some(i) = rest.find('[') {
                let (before, after) = rest.split_at(i);
                if let Some(j) = after.find(']')
                    && let Ok(n) = after[1..j].trim().parse::<usize>()
                {
                    if !before.trim().is_empty() {
                        if !cites.is_empty() {
                            sents.push(CitedSentence {
                                text: std::mem::take(&mut text).trim().to_string(),
                                cites: std::mem::take(&mut cites),
                            });
                        }
                        text.push_str(before);
                    }
                    if n > 0 {
                        cites.push(n - 1);
                    }
                    rest = &after[j + 1..];
                    continue;
                }
                text.push_str(&rest[..i + 1]);
                rest = &rest[i + 1..];
            } else {
                if !cites.is_empty() {
                    sents.push(CitedSentence {
                        text: std::mem::take(&mut text).trim().to_string(),
                        cites: std::mem::take(&mut cites),
                    });
                }
                text.push_str(rest);
                rest = "";
            }
        }
        if !text.trim().is_empty() || !cites.is_empty() {
            sents.push(CitedSentence {
                text: text.trim().to_string(),
                cites,
            });
        }
        if !sents.is_empty() {
            out.push(sents);
        }
    }
    out
}

/// Sentences citing source `i`.
pub fn sentences_citing(paras: &[Vec<CitedSentence>], i: usize) -> usize {
    paras
        .iter()
        .flatten()
        .filter(|s| s.cites.contains(&i))
        .count()
}

/// (sentences backed by a source, all sentences).
pub fn backed(paras: &[Vec<CitedSentence>]) -> (usize, usize) {
    let all: Vec<&CitedSentence> = paras.iter().flatten().collect();
    (
        all.iter().filter(|s| !s.cites.is_empty()).count(),
        all.len(),
    )
}

/// The quote split into (text, marked) runs.
pub fn quote_runs(quote: &str, marked: &[String]) -> Vec<(String, bool)> {
    let mut hits: Vec<(usize, usize)> = marked
        .iter()
        .filter(|m| !m.is_empty())
        .filter_map(|m| quote.find(m.as_str()).map(|i| (i, i + m.len())))
        .collect();
    hits.sort();
    let mut out = Vec::new();
    let mut at = 0;
    for (a, b) in hits {
        if a < at {
            continue;
        }
        if a > at {
            out.push((quote[at..a].to_string(), false));
        }
        out.push((quote[a..b].to_string(), true));
        at = b;
    }
    if at < quote.len() {
        out.push((quote[at..].to_string(), false));
    }
    out
}

/// "cited in 1 sentence" / "cited in 3 sentences".
pub fn cited_in(n: usize) -> String {
    format!("cited in {n} sentence{}", if n == 1 { "" } else { "s" })
}

/// The demo cited answer: header line, markup, sources.
pub fn demo_cited() -> (String, String, Vec<CitedSource>) {
    let s = |mark: &str,
             tint: &str,
             name: &str,
             domain: &str,
             date: &str,
             title: &str,
             quote: &str,
             marked: &[&str],
             url: &str| CitedSource {
        mark: mark.into(),
        tint: tint.into(),
        name: name.into(),
        domain: domain.into(),
        date: date.into(),
        title: title.into(),
        quote: quote.into(),
        marked: marked.iter().map(|m| m.to_string()).collect(),
        url: url.into(),
    };
    (
        "3 sources · 14 pages read".into(),
        "The quickest win is caching dependencies between runs: the runner restores compiled crates instead of building them again. [1] On large workspaces that can take a cold build from twelve minutes to about three. [1] The cache only hits while the lockfile stays the same, so pin the toolchain and land dependency bumps in their own commits. [2][1]\n\n\
Then split the test suite across parallel jobs: each shard finishes in about four minutes, [3] which halved the wait for most pull requests in one team's setup. [3] Keep the slow end-to-end tests for the main branch, so reviews get a quick first signal."
            .into(),
        vec![
            s(
                "B",
                "71717A",
                "Buildlog",
                "buildlog.dev",
                "Oct 3, 2026",
                "Dependency caching, explained: what gets stored and for how long",
                "Restoring the compiled dependency graph skips most of a cold build. On our largest workspace a twelve-minute build dropped to just over three once the cache was warm.",
                &["skips most of a cold build", "dropped to just over three"],
                "https://buildlog.dev/dependency-caching",
            ),
            s(
                "C",
                "16A34A",
                "Cache Notes",
                "cachenotes.dev",
                "Jul 9, 2026",
                "Keeping build caches warm",
                "Pin the toolchain and the lockfile, and land dependency bumps on their own. One changed version early in the graph invalidates every crate built after it.",
                &[
                    "Pin the toolchain and the lockfile",
                    "dependency bumps on their own",
                    "invalidates every crate built after it",
                ],
                "https://cachenotes.dev/warm-caches",
            ),
            s(
                "P",
                "0D9488",
                "Pipeline Weekly",
                "pipelineweekly.io",
                "Sep 21, 2026",
                "Sharding test suites without the flakes",
                "With the suite split into eight shards each job finished in about four minutes, and the median wait on a pull request fell by half.",
                &["about four minutes", "fell by half"],
                "https://pipelineweekly.io/sharding",
            ),
        ],
    )
}

// ────────────────────────────────────────────────────────────── Message Actions ──

/// The reasons offered after a thumbs-down on a message.
pub const MESSAGE_REASONS: [&str; 4] = [
    "Not accurate",
    "Too long",
    "Didn't follow instructions",
    "Other",
];

/// How long Regenerate spins before the new version lands, ms.
pub const REGENERATE_MS: u64 = 1200;

/// The demo message: prompt, model, its brand, the two versions it starts with, and the pool new
/// versions come from.
pub fn demo_message() -> (String, String, String, Vec<String>, Vec<String>) {
    (
        "Sum up the 3.1 changelog in one sentence.".into(),
        "Sonnet 5.5".into(),
        "claude".into(),
        vec![
            "Version 3.1 adds shared projects, makes search twice as fast and fixes the PDF export on Linux.".into(),
            "Shared projects land in 3.1, search is twice as fast, and PDF exports on Linux work again.".into(),
        ],
        vec![
            "In short: teams can share projects now, search takes half the time and Linux PDF exports are fixed.".into(),
            "3.1 brings shared projects, halves search time and repairs PDF export on Linux.".into(),
        ],
    )
}

// ────────────────────────────────────────────────────────────── Thinking pill ──

/// affectUI's default phases.
pub const LOADER_PHASES: [&str; 4] = [
    "Thinking",
    "Searching the web",
    "Reading sources",
    "Writing response",
];

/// Default time per phase, ms.
pub const LOADER_INTERVAL: u64 = 2200;

/// The phase index `elapsed` ms in, and whether it has run past the last phase (done when the
/// loader settles by itself). With `looped` it starts over instead.
pub fn loader_phase(elapsed: u64, interval: u64, n: usize, looped: bool) -> (usize, bool) {
    if n == 0 {
        return (0, true);
    }
    let i = (elapsed / interval.max(1)) as usize;
    if looped {
        (i % n, false)
    } else if i >= n {
        (n - 1, true)
    } else {
        (i, false)
    }
}

/// "Thought for 4s".
pub fn thought_for(secs: u64) -> String {
    format!("Thought for {secs}s")
}

// ─────────────────────────────────────────────────────────────── Thinking panel ──

/// A source chip of a search step.
#[derive(Clone, Debug, PartialEq)]
pub struct ThinkSource {
    /// Brand id for the coloured mark.
    pub brand: String,
    /// Label ("Notion – Retreat plan").
    pub label: String,
    /// Link.
    pub url: String,
}

/// A step of the thinking panel: a search with sources, or a plain thinking line.
#[derive(Clone, Debug, PartialEq)]
pub struct ThinkStep {
    /// Label ("Searched", "Comparing trains and flights").
    pub label: String,
    /// Source chips (a search step).
    pub sources: Vec<ThinkSource>,
    /// More sources behind "+N more".
    pub more: Vec<ThinkSource>,
}

impl ThinkStep {
    /// A thinking line.
    pub fn line(label: &str) -> Self {
        Self {
            label: label.into(),
            sources: vec![],
            more: vec![],
        }
    }
}

/// Default pause between two steps, ms.
pub const THINK_STEP_MS: u64 = 1300;

/// (steps shown, done, whole seconds) `elapsed` ms in; done one step after the last.
pub fn think_progress(elapsed: u64, step_ms: u64, n: usize) -> (usize, bool, u64) {
    let shown = ((elapsed / step_ms.max(1)) as usize + 1).min(n);
    let done = elapsed >= step_ms * n as u64;
    let secs = if done {
        (step_ms * n as u64).div_ceil(1000)
    } else {
        elapsed / 1000
    };
    (shown, done, secs)
}

/// The demo steps.
pub fn demo_think_steps() -> Vec<ThinkStep> {
    let src = |b: &str, l: &str, u: &str| ThinkSource {
        brand: b.into(),
        label: l.into(),
        url: u.into(),
    };
    vec![
        ThinkStep {
            label: "Searched".into(),
            sources: vec![
                src("slack", "Slack – #retreat", "https://slack.com"),
                src("x", "X – @portoguide", "https://x.com"),
                src("notion", "Notion – Retreat plan", "https://www.notion.so"),
                src(
                    "wikipedia",
                    "Wikipedia – Porto",
                    "https://en.wikipedia.org/wiki/Porto",
                ),
            ],
            more: vec![
                src(
                    "google",
                    "Google Flights – OPO",
                    "https://www.google.com/travel/flights",
                ),
                src(
                    "bookingdotcom",
                    "Booking.com – Porto",
                    "https://www.booking.com",
                ),
                src("airbnb", "Airbnb – Ribeira", "https://www.airbnb.com"),
            ],
        },
        ThinkStep::line("Reading 7 sources about the retreat"),
        ThinkStep::line("Comparing trains and flights"),
        ThinkStep::line("Checking dates against the team calendar"),
        ThinkStep::line("Drafting the retreat agenda"),
    ]
}

// ──────────────────────────────────────────────────────────── Writing Assistant ──

/// The sample writer: an answer for a message.
pub fn sample_reply(msg: &str) -> String {
    let m = msg.to_lowercase();
    if m.contains("shorter") {
        "Cut it to three lines: the ask, the date and one reason. Everything else moved to a P.S."
            .into()
    } else if m.contains("tone") {
        "I softened the opening, swapped \"ASAP\" for a date and ended on a thank-you, so it reads as a request, not a demand.".into()
    } else if m.contains("reply") {
        "Thanks for the update – Thursday works for us. I'll bring the revised mockups and we can decide on the colours there.".into()
    } else if m.contains("review") || m.contains("thursday") {
        "Hi all, quick heads-up: the design review moves to Thursday at 2 pm. Same room, same agenda – bring anything you'd like feedback on. Thanks!".into()
    } else {
        "I fixed two typos, split the long second paragraph and moved your main ask to the first line, so it's the first thing they read.".into()
    }
}

/// What the simulated dictation says.
pub const DICTATION: &str = "Write a short note that the design review moves to Thursday";

/// Characters a second the simulated dictation types.
pub const DICTATION_CPS: u64 = 36;

/// The dictation typed `elapsed` ms in (whole words).
pub fn dictated_at(elapsed: u64) -> &'static str {
    let n = (elapsed.saturating_mul(DICTATION_CPS) / 1000) as usize;
    if n >= DICTATION.len() {
        return DICTATION;
    }
    let cut = DICTATION[..n].rfind(' ').unwrap_or(0);
    &DICTATION[..cut]
}

/// Words a second the replies stream at.
pub const REPLY_WPS: u64 = 22;

/// How long the shimmering "Writing…" line holds before the reply streams, ms.
pub const REPLY_WAIT_MS: u64 = 700;

/// The first words of `text` `elapsed` ms after it was asked.
pub fn reply_at(text: &str, elapsed: u64) -> Option<String> {
    if elapsed < REPLY_WAIT_MS {
        return None;
    }
    let n = ((elapsed - REPLY_WAIT_MS).saturating_mul(REPLY_WPS) / 1000) as usize + 1;
    Some(text.split(' ').take(n).collect::<Vec<_>>().join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answer_parses_and_streams() {
        let (_, _, _, src, _) = demo_answer();
        let d = AnswerDoc::parse(&src);
        assert_eq!(d.blocks.len(), 3);
        assert!(matches!(&d.blocks[1], Block::Bullets(b) if b.len() == 3));
        let n = d.words();
        assert!(n > 40);
        assert_eq!(d.upto(n), d);
        assert_eq!(d.upto(3).words(), 3);
        assert_eq!(answer_words_at(ANSWER_READ_MS + 1000, n, 38), 38);
        assert_eq!(answer_phase(100, n, 38), AnswerPhase::Reading);
        assert_eq!(answer_phase(60_000, n, 38), AnswerPhase::Done);
        assert_eq!(answer_words_at(u64::MAX / 4, n, 38), n);
        assert!(!d.plain().contains('['));
    }

    #[test]
    fn cited_sentences() {
        let (_, src, srcs) = demo_cited();
        let p = parse_cited(&src);
        assert_eq!(p.len(), 2);
        assert_eq!(p[0][2].cites, vec![1, 0]);
        assert_eq!(sentences_citing(&p, 0), 3);
        assert_eq!(sentences_citing(&p, 1), 1);
        let (b, all) = backed(&p);
        assert_eq!((b, all), (5, 6));
        let r = quote_runs(&srcs[1].quote, &srcs[1].marked);
        assert_eq!(r.iter().filter(|x| x.1).count(), 3);
        assert_eq!(cited_in(1), "cited in 1 sentence");
    }

    #[test]
    fn loaders_and_steps() {
        assert_eq!(loader_phase(0, 2200, 4, false), (0, false));
        assert_eq!(loader_phase(9000, 2200, 4, false), (3, true));
        assert_eq!(loader_phase(9000, 2200, 4, true), (0, false));
        assert_eq!(think_progress(0, 1300, 5), (1, false, 0));
        assert_eq!(think_progress(6500, 1300, 5), (5, true, 7));
        assert_eq!(dictated_at(0), "");
        assert_eq!(dictated_at(60_000), DICTATION);
        assert!(reply_at("a b c", 100).is_none());
        assert_eq!(domain_of("https://www.notion.so/x"), "notion.so");
    }
}
