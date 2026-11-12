//! Glyph loaders (render tier T1, `docs/specs/loader-glyph.md`): one renderer, many frame tables.

use std::time::Duration;

use oxyn_tokens::ColorRole;

use super::{LoaderPhase, TempoClock, timing};
use crate::{ClockMs, state::AgentState};

/// How a table produces its frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GlyphKind {
    /// A frame list, with optional per-frame holds (in ticks) and per-frame alpha.
    Frames {
        /// The frames. Every frame has the same number of cells.
        frames: &'static [&'static str],
        /// Ticks each frame is held (default 1).
        holds: Option<&'static [u8]>,
        /// Alpha per frame (default 1).
        alphas: Option<&'static [f32]>,
    },
    /// A head bouncing across `width` cells with a fading trail, holding `hold` ticks at each end.
    Scanner {
        /// Cells.
        width: u8,
        /// Ticks held at each end.
        hold: u8,
    },
    /// One `●` breathing in brightness, quantised to the frame rate.
    Pulse {
        /// Breath period.
        period_ms: u32,
    },
}

/// A glyph loader variant: a frame table plus its fallback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphTable {
    /// Registry name without the family prefix (`glyph-braille` → `loader/glyph-braille`).
    pub id: &'static str,
    /// Frames.
    pub kind: GlyphKind,
    /// Milliseconds per tick at tempo 1.
    pub frame_ms: u32,
    /// Table to use when the font or terminal can't draw this one (ends at `glyph-line`).
    pub fallback: Option<&'static str>,
}

/// Still glyphs (reduced motion, mirrored loaders, posters, hard stall).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Still {
    /// `◌`: working, stalled.
    Working,
    /// `⧗`: waiting on the user.
    Waiting,
    /// `✓`.
    Done,
    /// `✕`.
    Failed,
}

impl Still {
    /// The glyph.
    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Working => "◌",
            Self::Waiting => "⧗",
            Self::Done => "✓",
            Self::Failed => "✕",
        }
    }
}

const LINE: &str = "glyph-line";

/// The 14 free glyph loaders (`loader-glyph.md` §5.1), in catalogue order.
pub const FREE_GLYPH_TABLES: [GlyphTable; 14] = [
    GlyphTable {
        id: "glyph-braille",
        kind: GlyphKind::Frames {
            frames: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            holds: None,
            alphas: None,
        },
        frame_ms: 80,
        fallback: Some(LINE),
    },
    GlyphTable {
        id: "glyph-sand",
        kind: GlyphKind::Frames {
            frames: &["⠛", "⠫", "⡋", "⡍", "⣉", "⣅", "⣤", "⣤", "⣄", "⣀", "⡀", "⠀"],
            holds: Some(&[1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1]),
            alphas: None,
        },
        frame_ms: 150,
        fallback: Some(LINE),
    },
    GlyphTable {
        id: "glyph-block",
        kind: GlyphKind::Frames {
            frames: &["░░░", "▒░░", "▓▒░", "█▓▒", "▓█▓", "▒▓█", "░▒▓", "░░▒"],
            holds: None,
            alphas: None,
        },
        frame_ms: 100,
        fallback: Some("glyph-dots"),
    },
    GlyphTable {
        id: "glyph-bar",
        kind: GlyphKind::Frames {
            frames: &[
                "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃", "▂",
            ],
            holds: Some(&[2, 2, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2]),
            alphas: None,
        },
        frame_ms: 80,
        fallback: Some(LINE),
    },
    GlyphTable {
        id: "glyph-dots",
        kind: GlyphKind::Frames {
            frames: &["·   ", "··  ", "··· ", " ·· ", "  · ", "    "],
            holds: None,
            alphas: None,
        },
        frame_ms: 180,
        fallback: Some(LINE),
    },
    GlyphTable {
        id: "glyph-line",
        kind: GlyphKind::Frames {
            frames: &["|", "/", "-", "\\"],
            holds: None,
            alphas: None,
        },
        frame_ms: 100,
        fallback: None,
    },
    GlyphTable {
        id: "glyph-arc",
        kind: GlyphKind::Frames {
            frames: &["◜", "◠", "◝", "◞", "◡", "◟"],
            holds: None,
            alphas: None,
        },
        frame_ms: 100,
        fallback: Some(LINE),
    },
    GlyphTable {
        id: "glyph-quadrant",
        kind: GlyphKind::Frames {
            frames: &["◰", "◳", "◲", "◱"],
            holds: None,
            alphas: None,
        },
        frame_ms: 160,
        fallback: Some(LINE),
    },
    GlyphTable {
        id: "glyph-bloom",
        kind: GlyphKind::Frames {
            frames: &["▫", "▪", "▣", "■", "▣", "▪"],
            holds: None,
            alphas: None,
        },
        frame_ms: 120,
        fallback: Some(LINE),
    },
    GlyphTable {
        id: "glyph-half",
        kind: GlyphKind::Frames {
            frames: &["◐", "◓", "◑", "◒"],
            holds: None,
            alphas: None,
        },
        frame_ms: 150,
        fallback: Some(LINE),
    },
    GlyphTable {
        id: "glyph-bullet",
        kind: GlyphKind::Frames {
            frames: &["•", "◦"],
            holds: None,
            alphas: Some(&[1.0, 0.6]),
        },
        frame_ms: 600,
        fallback: Some(LINE),
    },
    GlyphTable {
        id: "glyph-scanner",
        kind: GlyphKind::Scanner { width: 8, hold: 6 },
        frame_ms: 50,
        fallback: Some("glyph-dots"),
    },
    GlyphTable {
        id: "glyph-spark",
        kind: GlyphKind::Frames {
            frames: &["·", "⋆", "✧", "✦", "✧", "⋆"],
            holds: None,
            alphas: None,
        },
        frame_ms: 110,
        fallback: Some(LINE),
    },
    GlyphTable {
        id: "glyph-pulse",
        kind: GlyphKind::Pulse { period_ms: 1600 },
        // 20 fps cap on the brightness loop.
        frame_ms: 50,
        fallback: Some("glyph-bullet"),
    },
];

impl GlyphTable {
    /// A free table by id (`glyph-braille` or `loader/glyph-braille`).
    pub fn by_id(id: &str) -> Option<&'static GlyphTable> {
        let id = id.strip_prefix("loader/").unwrap_or(id);
        FREE_GLYPH_TABLES.iter().find(|t| t.id == id)
    }

    /// The default `Waiting` table for every glyph loader.
    pub fn sand() -> &'static GlyphTable {
        &FREE_GLYPH_TABLES[1]
    }

    /// Cells every frame occupies.
    pub fn cells(&self) -> usize {
        match self.kind {
            GlyphKind::Frames { frames, .. } => frames[0].chars().count(),
            GlyphKind::Scanner { width, .. } => usize::from(width),
            GlyphKind::Pulse { .. } => 1,
        }
    }

    /// One full loop at tempo 1.
    pub fn loop_duration(&self) -> Duration {
        Duration::from_millis(u64::from(self.frame_ms) * self.ticks())
    }

    fn ticks(&self) -> u64 {
        match self.kind {
            GlyphKind::Frames { frames, holds, .. } => match holds {
                Some(h) => h.iter().map(|&x| u64::from(x)).sum(),
                None => frames.len() as u64,
            },
            GlyphKind::Scanner { width, hold } => 2 * u64::from(width) - 1 + 2 * u64::from(hold),
            GlyphKind::Pulse { period_ms } => u64::from(period_ms / self.frame_ms.max(1)),
        }
    }

    /// The frame at `beats` (milliseconds at tempo 1): its text and per-cell alpha.
    pub fn frame_at(&self, beats: f64) -> (String, Vec<f32>) {
        let tick = (beats.max(0.0) / f64::from(self.frame_ms.max(1))).floor() as u64;
        let t = tick % self.ticks().max(1);
        match self.kind {
            GlyphKind::Frames {
                frames,
                holds,
                alphas,
            } => {
                let idx = match holds {
                    None => t as usize,
                    Some(h) => {
                        let mut acc = 0u64;
                        h.iter()
                            .position(|&x| {
                                acc += u64::from(x);
                                t < acc
                            })
                            .unwrap_or(0)
                    }
                };
                let text = frames[idx];
                let a = alphas.map_or(1.0, |a| a[idx]);
                (text.to_string(), vec![a; text.chars().count()])
            }
            GlyphKind::Scanner { width, hold } => scanner(width, hold, t),
            GlyphKind::Pulse { period_ms } => {
                let x = (t * u64::from(self.frame_ms)) as f32 / period_ms as f32;
                let a = 0.3 + 0.7 * ((x * std::f32::consts::TAU).cos() + 1.0) / 2.0;
                ("●".into(), vec![a])
            }
        }
    }
}

/// Scanner: forward 0..w, hold at the end, back to 0, hold. Trail levels from the spec.
fn scanner(width: u8, hold: u8, t: u64) -> (String, Vec<f32>) {
    let w = i64::from(width);
    let h = i64::from(hold);
    let t = t as i64;
    let (pos, dir) = if t < w {
        (t, 1)
    } else if t < w + h {
        (w - 1, 1)
    } else if t < 2 * w - 1 + h {
        (w - 2 - (t - w - h), -1)
    } else {
        (0, -1)
    };
    const TRAIL: [(&str, f32); 3] = [("▪", 0.6), ("▪", 0.35), ("·", 0.15)];
    let mut text = String::new();
    let mut alpha = Vec::with_capacity(width.into());
    for i in 0..w {
        let behind = (pos - i) * dir;
        let (g, a) = if i == pos {
            ("■", 1.0)
        } else if (1..=3).contains(&behind) {
            TRAIL[(behind - 1) as usize]
        } else {
            ("·", 0.1)
        };
        text.push_str(g);
        alpha.push(a);
    }
    (text, alpha)
}

/// What to draw this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphFrame {
    /// The cells, padded to the table's width so the line never moves.
    pub text: String,
    /// Alpha per cell (`text.chars().count()` entries).
    pub cell_alpha: Vec<f32>,
    /// Colour role now being eased to.
    pub color: ColorRole,
    /// Colour role being eased from, and the progress `t` toward `color` (mix in OKLab).
    pub color_from: ColorRole,
    /// Progress from `color_from` to `color`, `[0, 1]`.
    pub color_t: f32,
    /// Whole-loader opacity (enter fade, submitted dim, cancelled dim).
    pub opacity: f32,
    /// True when this is a still frame (no further motion until the phase changes).
    pub still: bool,
}

/// A glyph loader instance. Holds phase, tempo and hysteresis; renderers only draw its frames.
#[derive(Clone, Debug)]
pub struct GlyphLoader {
    table: &'static GlyphTable,
    phase: LoaderPhase,
    prev: LoaderPhase,
    since: ClockMs,
    tempo: TempoClock,
    loop_since: Option<ClockMs>,
    was_shown: bool,
    frozen: Option<(String, Vec<f32>)>,
}

impl GlyphLoader {
    /// A hidden loader for `table`.
    pub fn new(table: &'static GlyphTable) -> Self {
        Self {
            table,
            phase: LoaderPhase::Hidden,
            prev: LoaderPhase::Hidden,
            since: ClockMs(0),
            tempo: TempoClock::new(1.0),
            loop_since: None,
            was_shown: false,
            frozen: None,
        }
    }

    /// The variant.
    pub fn table(&self) -> &'static GlyphTable {
        self.table
    }

    /// Current phase.
    pub fn phase(&self) -> LoaderPhase {
        self.phase
    }

    /// Feeds the current agent state. Cheap when nothing changed; call it every frame.
    pub fn update(&mut self, state: &AgentState, now: ClockMs) {
        let next = LoaderPhase::from_state(state);
        if self
            .loop_since
            .is_some_and(|s| now.since(s) >= timing::SHOW_DELAY)
        {
            self.was_shown = true;
        }
        if next == self.phase {
            return;
        }
        if next == LoaderPhase::Cancelled {
            self.frozen = self.loop_cells(now);
        }
        self.prev = self.phase;
        self.phase = next;
        self.since = now;
        if next.loops() {
            let over = if next == LoaderPhase::StalledSoft {
                timing::STALL_RAMP
            } else {
                timing::TEMPO_EASE
            };
            self.tempo.set_speed(next.tempo(), now, over);
            self.loop_since.get_or_insert(now);
        } else if next == LoaderPhase::Hidden {
            self.loop_since = None;
            self.was_shown = false;
            self.frozen = None;
        }
    }

    fn table_for(&self, phase: LoaderPhase) -> &'static GlyphTable {
        if phase == LoaderPhase::Waiting {
            GlyphTable::sand()
        } else {
            self.table
        }
    }

    fn shown(&self, now: ClockMs) -> bool {
        self.was_shown
            || self
                .loop_since
                .is_some_and(|s| now.since(s) >= timing::SHOW_DELAY)
    }

    fn loop_cells(&self, now: ClockMs) -> Option<(String, Vec<f32>)> {
        if !self.phase.loops() || !self.shown(now) {
            return None;
        }
        Some(self.table_for(self.phase).frame_at(self.tempo.beats(now)))
    }

    fn pad(&self, (mut text, mut alpha): (String, Vec<f32>)) -> (String, Vec<f32>) {
        let width = self.table.cells();
        let n = text.chars().count();
        if n < width {
            // Centre narrower glyphs (sand, still glyphs) in the reserved box.
            let left = (width - n) / 2;
            let right = width - n - left;
            text = format!("{}{text}{}", " ".repeat(left), " ".repeat(right));
            let mut a = vec![0.0; left];
            a.append(&mut alpha);
            a.extend(std::iter::repeat_n(0.0, right));
            alpha = a;
        }
        (text, alpha)
    }

    /// The frame to draw at `now`, or `None` when nothing shows. `still` draws the still frame of
    /// the current phase (reduced motion, mirrored loader, poster).
    pub fn frame(&self, now: ClockMs, still: bool) -> Option<GlyphFrame> {
        use LoaderPhase as P;
        let in_phase = now.since(self.since);
        let (cells, opacity, is_still) = match self.phase {
            P::Hidden => return None,
            P::Submitted => {
                let p = (in_phase.as_secs_f32() / timing::SUBMITTED.as_secs_f32()).min(1.0);
                let first = if still {
                    (Still::Working.glyph().into(), vec![1.0])
                } else {
                    self.table.frame_at(0.0)
                };
                (
                    first,
                    1.0 - 0.4 * (1.0 - (1.0 - p).powi(2)),
                    still || p >= 1.0,
                )
            }
            phase if phase.loops() => {
                if !self.shown(now) {
                    return None;
                }
                let start = self.loop_since.unwrap_or(self.since);
                let enter = now.since(start).saturating_sub(timing::SHOW_DELAY);
                let fade = if self.was_shown && self.prev.loops() || still {
                    1.0
                } else {
                    (enter.as_secs_f32() / timing::ENTER.as_secs_f32()).min(1.0)
                };
                if still {
                    let g = if phase == P::Waiting {
                        Still::Waiting
                    } else {
                        Still::Working
                    };
                    ((g.glyph().into(), vec![1.0]), 1.0, true)
                } else {
                    (
                        self.table_for(phase).frame_at(self.tempo.beats(now)),
                        fade,
                        false,
                    )
                }
            }
            P::StalledHard => ((Still::Working.glyph().into(), vec![1.0]), 1.0, true),
            P::Done => {
                if !self.was_shown || in_phase >= timing::DONE_HOLD {
                    return None;
                }
                let fade = (in_phase.as_secs_f32() / timing::ENTER.as_secs_f32()).min(1.0);
                (
                    (Still::Done.glyph().into(), vec![1.0]),
                    if still { 1.0 } else { fade },
                    true,
                )
            }
            P::Failed => ((Still::Failed.glyph().into(), vec![1.0]), 1.0, true),
            P::Cancelled => {
                if in_phase >= timing::CANCEL_CLEAR {
                    return None;
                }
                (self.frozen.clone()?, 0.4, true)
            }
            _ => return None,
        };
        let (text, cell_alpha) = self.pad(cells);
        let ease = self.phase.color_ease(self.prev);
        let color_t = if self.prev == P::Hidden || still {
            1.0
        } else {
            (in_phase.as_secs_f32() / ease.as_secs_f32()).min(1.0)
        };
        Some(GlyphFrame {
            text,
            cell_alpha,
            color: self.phase.color(),
            color_from: self.prev.color(),
            color_t,
            opacity,
            still: is_still,
        })
    }

    /// How long until the drawing changes, so hosts repaint only when needed (`None`: never,
    /// until the next state change). Loops repaint at frame boundaries, not 60 fps.
    pub fn next_due(&self, now: ClockMs, still: bool) -> Option<Duration> {
        const FRAME: Duration = Duration::from_millis(16);
        let in_phase = now.since(self.since);
        let colour_moving = self.prev != LoaderPhase::Hidden
            && in_phase < self.phase.color_ease(self.prev)
            && !still;
        match self.phase {
            LoaderPhase::Hidden | LoaderPhase::Failed | LoaderPhase::StalledHard => {
                colour_moving.then_some(FRAME)
            }
            LoaderPhase::Submitted => (in_phase < timing::SUBMITTED).then_some(FRAME),
            LoaderPhase::Done => {
                (in_phase < timing::DONE_HOLD).then(|| FRAME.min(timing::DONE_HOLD - in_phase))
            }
            LoaderPhase::Cancelled => (in_phase < timing::CANCEL_CLEAR).then_some(FRAME),
            phase => {
                let start = self.loop_since.unwrap_or(self.since);
                let to_show = timing::SHOW_DELAY.saturating_sub(now.since(start));
                if !to_show.is_zero() {
                    return Some(to_show);
                }
                if still {
                    return None;
                }
                if colour_moving || now.since(start) < timing::SHOW_DELAY + timing::ENTER {
                    return Some(FRAME);
                }
                let fm = f64::from(self.table_for(phase).frame_ms.max(1));
                let beats = self.tempo.beats(now);
                let left = fm - beats % fm;
                let speed = f64::from(self.tempo.speed(now).max(0.05));
                Some(Duration::from_micros(((left / speed) * 1000.0).ceil() as u64).max(FRAME))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentEvent, AgentMachine};

    #[test]
    fn every_free_table_keeps_its_width_and_matches_the_spec_loop() {
        let loops = [
            800, 1950, 800, 1440, 1080, 400, 600, 640, 720, 600, 1200, 1350, 660, 1600,
        ];
        for (t, ms) in FREE_GLYPH_TABLES.iter().zip(loops) {
            assert_eq!(t.loop_duration().as_millis() as u64, ms, "{}", t.id);
            let w = t.cells();
            for tick in 0..t.ticks() * 2 {
                let (text, alpha) = t.frame_at((tick * u64::from(t.frame_ms)) as f64);
                assert_eq!(text.chars().count(), w, "{} tick {tick}", t.id);
                assert_eq!(alpha.len(), w);
            }
            // Fallback chains end at the ASCII line.
            let mut cur = *t;
            for _ in 0..4 {
                match cur.fallback {
                    Some(f) => cur = *GlyphTable::by_id(f).unwrap(),
                    None => break,
                }
            }
            assert_eq!(cur.id, "glyph-line", "{}", t.id);
        }
    }

    #[test]
    fn holds_and_scanner_sequence() {
        let bar = GlyphTable::by_id("loader/glyph-bar").unwrap();
        assert_eq!(bar.frame_at(0.0).0, "▁");
        assert_eq!(bar.frame_at(80.0).0, "▁"); // held 2 ticks
        assert_eq!(bar.frame_at(160.0).0, "▂");
        let sc = GlyphTable::by_id("glyph-scanner").unwrap();
        assert_eq!(sc.frame_at(0.0).0, "■·······");
        assert_eq!(sc.frame_at(3.0 * 50.0).0, "▪▪▪■····".replacen('▪', "·", 1));
        assert_eq!(sc.frame_at(7.0 * 50.0).0, "····▪▪▪■".replacen('▪', "·", 1));
    }

    fn run() -> (AgentMachine, GlyphLoader) {
        let mut m = AgentMachine::default();
        let l = GlyphLoader::new(GlyphTable::by_id("glyph-braille").unwrap());
        m.apply(AgentEvent::Submit, ClockMs(1000));
        (m, l)
    }

    #[test]
    fn quick_thinking_never_shows_and_done_stays_quiet() {
        let (mut m, mut l) = run();
        l.update(m.state(), ClockMs(1000));
        assert!(l.frame(ClockMs(1000), false).is_some()); // submitted pulse
        m.apply(AgentEvent::Thinking { header: None }, ClockMs(1100));
        l.update(m.state(), ClockMs(1100));
        assert!(l.frame(ClockMs(1300), false).is_none()); // inside show_delay
        m.apply(AgentEvent::Finished, ClockMs(1350));
        l.update(m.state(), ClockMs(1350));
        assert!(l.frame(ClockMs(1360), false).is_none());
    }

    #[test]
    fn waiting_swaps_to_sand_padded_and_stall_goes_still() {
        let mut l = GlyphLoader::new(GlyphTable::by_id("glyph-block").unwrap());
        let mut m = AgentMachine::default();
        m.apply(AgentEvent::Submit, ClockMs(0));
        m.apply(AgentEvent::Thinking { header: None }, ClockMs(0));
        l.update(m.state(), ClockMs(0));
        let f = l.frame(ClockMs(1000), false).unwrap();
        assert_eq!(f.text.chars().count(), 3);
        assert_eq!(f.color, ColorRole::LoaderAccent);
        // Hard stall: still ◌ in warn, padded to the block's 3 cells.
        l.update(m.tick(ClockMs(46_000)), ClockMs(46_000));
        let f = l.frame(ClockMs(46_000), false).unwrap();
        assert_eq!(f.text, " ◌ ");
        assert!(f.still && f.color == ColorRole::LoaderWarn);
        assert_eq!(l.next_due(ClockMs(50_000), false), None);
    }

    #[test]
    fn reduced_motion_draws_still_glyphs() {
        let mut l = GlyphLoader::new(GlyphTable::by_id("glyph-braille").unwrap());
        let mut m = AgentMachine::default();
        m.apply(AgentEvent::Submit, ClockMs(0));
        m.apply(AgentEvent::Thinking { header: None }, ClockMs(0));
        l.update(m.state(), ClockMs(0));
        let f = l.frame(ClockMs(2000), true).unwrap();
        assert_eq!(f.text, "◌");
        assert!(f.still);
        assert_eq!(l.next_due(ClockMs(2000), true), None);
    }

    #[test]
    fn repaints_only_at_frame_boundaries() {
        let mut l = GlyphLoader::new(GlyphTable::by_id("glyph-bullet").unwrap());
        let mut m = AgentMachine::default();
        m.apply(AgentEvent::Submit, ClockMs(0));
        m.apply(AgentEvent::Thinking { header: None }, ClockMs(0));
        l.update(m.state(), ClockMs(0));
        // 600 ms frames: at t = 10 100 the next boundary is 10 200.
        assert_eq!(
            l.next_due(ClockMs(10_100), false),
            Some(Duration::from_millis(100))
        );
    }
}
