//! Scroll area with stick-to-bottom (`scroll-area.md`) and pagination (`pagination.md`).
//!
//! - **Scroll area** (Arc UI Scroll area): native scrolling with thin overlay scrollbars that show
//!   while scrolling or on hover and fade after `hide_delay` (900 ms), and edge fades that grow
//!   with the content left beyond each edge. Pressing the track pages 90 % of the viewport toward
//!   the pointer. Oxyn plus: **stick to bottom** for streaming transcripts: it follows new content
//!   while the reader is at the bottom, lets go when they scroll up, counts what arrived since,
//!   and offers a "Jump to latest" pill.
//! - **Pagination** (Arc UI Pagination): previous and next around a sliding five-page window with
//!   a travelling current mark; when the window shifts the numbers slide like a belt by the number
//!   of slots moved. Oxyn plus: an ellipsis layout with first and last, a page-size picker with a
//!   range label, and a cursor mode (no total: just previous / next).
//!
//! Rebuilt from the public docs; no source used.

use std::time::Duration;

use crate::ClockMs;

/// Scroll timings.
pub mod timing {
    use std::time::Duration;
    /// Scrollbars linger after scrolling stops.
    pub const HIDE_DELAY: Duration = Duration::from_millis(900);
    /// Default edge fade length px.
    pub const FADE: f32 = 28.0;
    /// Track press pages this fraction of the viewport.
    pub const PAGE: f32 = 0.9;
    /// Minimum thumb length px.
    pub const MIN_THUMB: f32 = 24.0;
    /// Within this many px of the bottom counts as "at the bottom".
    pub const STICK_SLOP: f32 = 24.0;
    /// Pagination window.
    pub const WINDOW: usize = 5;
}

/// Measured scroll state of one viewport.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Metrics {
    /// `scrollTop`.
    pub top: f32,
    /// `scrollLeft`.
    pub left: f32,
    /// `scrollHeight`.
    pub height: f32,
    /// `scrollWidth`.
    pub width: f32,
    /// `clientHeight`.
    pub view_h: f32,
    /// `clientWidth`.
    pub view_w: f32,
}

impl Metrics {
    /// Vertical metrics only.
    pub fn vertical(top: f32, height: f32, view_h: f32) -> Self {
        Self {
            top,
            height,
            view_h,
            ..Default::default()
        }
    }
    /// Content left above / below / left / right px.
    pub fn beyond(&self) -> (f32, f32, f32, f32) {
        (
            self.top.max(0.0),
            (self.height - self.view_h - self.top).max(0.0),
            self.left.max(0.0),
            (self.width - self.view_w - self.left).max(0.0),
        )
    }
    /// Overflows vertically.
    pub fn overflows_y(&self) -> bool {
        self.height > self.view_h + 0.5
    }
    /// Overflows horizontally.
    pub fn overflows_x(&self) -> bool {
        self.width > self.view_w + 0.5
    }
    /// Largest `top`.
    pub fn max_top(&self) -> f32 {
        (self.height - self.view_h).max(0.0)
    }
}

/// Which edges have content beyond them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges {
    /// Content above.
    pub top: bool,
    /// Content below.
    pub bottom: bool,
    /// Content to the left.
    pub left: bool,
    /// Content to the right.
    pub right: bool,
}

/// Edges with content beyond them (1 px slop).
pub fn edges(m: &Metrics) -> Edges {
    let (t, b, l, r) = m.beyond();
    Edges {
        top: t > 1.0,
        bottom: b > 1.0,
        left: l > 1.0,
        right: r > 1.0,
    }
}

/// Fade lengths px for each edge: grow with the content left beyond, up to `fade`.
pub fn fades(m: &Metrics, fade: f32) -> (f32, f32, f32, f32) {
    let (t, b, l, r) = m.beyond();
    let f = |x: f32| x.min(fade).max(0.0);
    (f(t), f(b), f(l), f(r))
}

/// Thumb (length, offset) along a track of `track` px for a viewport of `view` px showing
/// content of `content` px scrolled by `pos`.
pub fn thumb(pos: f32, content: f32, view: f32, track: f32) -> (f32, f32) {
    if content <= view || content <= 0.0 {
        return (track, 0.0);
    }
    let len = (view / content * track).clamp(timing::MIN_THUMB.min(track), track);
    let max_pos = content - view;
    let off = (pos / max_pos).clamp(0.0, 1.0) * (track - len);
    (len, off)
}

/// Track press at `at` px along the track: pages 90 % of the viewport toward it.
pub fn page_toward(pos: f32, content: f32, view: f32, track: f32, at: f32) -> f32 {
    let (len, off) = thumb(pos, content, view, track);
    let max = (content - view).max(0.0);
    if at < off {
        (pos - view * timing::PAGE).max(0.0)
    } else if at > off + len {
        (pos + view * timing::PAGE).min(max)
    } else {
        pos
    }
}

/// Scrollbar visibility.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Scrollbars {
    /// While scrolling or hovered.
    #[default]
    Auto,
    /// Whenever content overflows.
    Always,
    /// Oxyn plus: never drawn (native still works).
    Hidden,
}

impl Scrollbars {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "always" => Self::Always,
            "hidden" => Self::Hidden,
            _ => Self::Auto,
        }
    }
    /// Shown at `now` given the last scroll and hover.
    pub fn shown(
        self,
        overflows: bool,
        hovered: bool,
        last_scroll: Option<ClockMs>,
        hide: Duration,
        now: ClockMs,
    ) -> bool {
        overflows
            && match self {
                Self::Always => true,
                Self::Hidden => false,
                Self::Auto => hovered || last_scroll.is_some_and(|t| now.since(t) < hide),
            }
    }
}

/// What the stick-to-bottom controller wants after an event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stick {
    /// Nothing.
    Stay,
    /// Scroll to `top` (follow new content / jump to latest).
    ScrollTo(f32),
}

/// Stick to bottom (Oxyn plus) for transcripts that stream.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StickToBottom {
    /// Following new content.
    pub following: bool,
    /// Items that arrived since the reader let go.
    pub unread: usize,
    /// Slop px.
    pub slop: f32,
    last_top: f32,
}

impl Default for StickToBottom {
    fn default() -> Self {
        Self {
            following: true,
            unread: 0,
            slop: timing::STICK_SLOP,
            last_top: 0.0,
        }
    }
}

impl StickToBottom {
    /// At the bottom within the slop.
    pub fn at_bottom(&self, m: &Metrics) -> bool {
        m.max_top() - m.top <= self.slop
    }
    /// The reader scrolled: scrolling up lets go; reaching the bottom re-attaches.
    pub fn scrolled(&mut self, m: &Metrics) {
        if self.at_bottom(m) {
            self.following = true;
            self.unread = 0;
        } else if m.top < self.last_top - 0.5 {
            self.following = false;
        }
        self.last_top = m.top;
    }
    /// Content grew by `added` items (the metrics are after the growth).
    pub fn content_changed(&mut self, m: &Metrics, added: usize) -> Stick {
        if self.following {
            self.last_top = m.max_top();
            Stick::ScrollTo(m.max_top())
        } else {
            self.unread += added;
            Stick::Stay
        }
    }
    /// The pill was pressed.
    pub fn jump(&mut self, m: &Metrics) -> Stick {
        self.following = true;
        self.unread = 0;
        self.last_top = m.max_top();
        Stick::ScrollTo(m.max_top())
    }
    /// Show the pill.
    pub fn pill(&self) -> bool {
        !self.following
    }
    /// Pill text.
    pub fn pill_label(&self) -> String {
        match self.unread {
            0 => "Jump to latest".into(),
            1 => "1 new message".into(),
            n => format!("{n} new messages"),
        }
    }
}

/// Pagination layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PageMode {
    /// Arc: a sliding five-page window, same width always.
    #[default]
    Window,
    /// Oxyn plus: first, last and neighbours with gaps.
    Ellipsis,
    /// Oxyn plus: cursor-based, no total ("Page 3", previous / next).
    Cursor,
    /// Oxyn plus: a "Load more" button with a count.
    LoadMore,
}

impl PageMode {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "ellipsis" => Self::Ellipsis,
            "cursor" => Self::Cursor,
            "load-more" => Self::LoadMore,
            _ => Self::Window,
        }
    }
}

/// One button slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageSlot {
    /// A page number.
    Page(usize),
    /// A gap (…).
    Gap,
}

/// Pagination state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pagination {
    /// Current page, 1-based.
    pub page: usize,
    /// Total pages (ignored in cursor mode).
    pub count: usize,
    /// Window size (5).
    pub window: usize,
    /// Layout.
    pub mode: PageMode,
    /// Cursor mode: a next page exists.
    pub has_next: bool,
}

impl Pagination {
    /// A pager.
    pub fn new(page: usize, count: usize) -> Self {
        Self {
            page: page.clamp(1, count.max(1)),
            count,
            window: timing::WINDOW,
            mode: PageMode::Window,
            has_next: true,
        }
    }
    /// First page of the sliding window.
    pub fn window_start(&self) -> usize {
        let w = self.window.max(1);
        if self.count <= w {
            return 1;
        }
        let half = w / 2;
        self.page.saturating_sub(half).clamp(1, self.count - w + 1)
    }
    /// Slots to draw.
    pub fn slots(&self) -> Vec<PageSlot> {
        if self.count == 0 {
            return vec![];
        }
        match self.mode {
            PageMode::Cursor | PageMode::LoadMore => vec![PageSlot::Page(self.page)],
            PageMode::Window => {
                let s = self.window_start();
                (s..(s + self.window).min(self.count + 1))
                    .map(PageSlot::Page)
                    .collect()
            }
            PageMode::Ellipsis => {
                let n = self.count;
                if n <= 7 {
                    return (1..=n).map(PageSlot::Page).collect();
                }
                let p = self.page;
                let mut v = vec![PageSlot::Page(1)];
                let (a, b) = if p <= 4 {
                    (2, 5)
                } else if p >= n - 3 {
                    (n - 4, n - 1)
                } else {
                    (p - 1, p + 1)
                };
                if a > 2 {
                    v.push(PageSlot::Gap);
                }
                v.extend((a..=b).map(PageSlot::Page));
                if b < n - 1 {
                    v.push(PageSlot::Gap);
                }
                v.push(PageSlot::Page(n));
                v
            }
        }
    }
    /// Goes to `p` (clamped); returns how many slots the window belt moved (signed).
    pub fn go(&mut self, p: usize) -> isize {
        let before = self.window_start() as isize;
        self.page = match self.mode {
            PageMode::Cursor | PageMode::LoadMore => p.max(1),
            _ => p.clamp(1, self.count.max(1)),
        };
        self.window_start() as isize - before
    }
    /// Previous.
    pub fn back(&mut self) -> isize {
        self.go(self.page.saturating_sub(1).max(1))
    }
    /// Next.
    pub fn forward(&mut self) -> isize {
        if self.next_disabled() {
            return 0;
        }
        self.go(self.page + 1)
    }
    /// Previous is disabled.
    pub fn prev_disabled(&self) -> bool {
        self.page <= 1
    }
    /// Next is disabled.
    pub fn next_disabled(&self) -> bool {
        match self.mode {
            PageMode::Cursor | PageMode::LoadMore => !self.has_next,
            _ => self.page >= self.count,
        }
    }
    /// Index of the current page among the slots (where the mark sits).
    pub fn mark_index(&self) -> Option<usize> {
        self.slots()
            .iter()
            .position(|s| *s == PageSlot::Page(self.page))
    }
    /// Label for a page button.
    pub fn page_label(&self, p: usize) -> String {
        if p == self.page {
            format!("Page {p}, current page")
        } else {
            format!("Go to page {p}")
        }
    }
}

/// "21–40 of 312" for a page of `size` items.
pub fn range_label(page: usize, size: usize, total: usize) -> String {
    if total == 0 {
        return "No results".into();
    }
    let a = (page.max(1) - 1) * size + 1;
    let b = (page.max(1) * size).min(total);
    format!("{a}–{b} of {total}")
}

/// New page after changing the page size so the first visible item stays on screen.
pub fn page_for_size(page: usize, old_size: usize, new_size: usize) -> usize {
    let first = (page.max(1) - 1) * old_size;
    first / new_size.max(1) + 1
}

/// Pages for `total` items of `size`.
pub fn page_count(total: usize, size: usize) -> usize {
    total.div_ceil(size.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fades_and_thumb() {
        let m = Metrics::vertical(10.0, 1000.0, 200.0);
        let e = edges(&m);
        assert!(e.top && e.bottom && !e.left);
        let (t, b, ..) = fades(&m, 28.0);
        assert_eq!((t, b), (10.0, 28.0));
        let (len, off) = thumb(400.0, 1000.0, 200.0, 200.0);
        assert_eq!(len, 40.0);
        assert_eq!(off, 80.0);
        assert_eq!(page_toward(400.0, 1000.0, 200.0, 200.0, 190.0), 580.0);
        assert_eq!(page_toward(100.0, 1000.0, 200.0, 200.0, 0.0), 0.0);
        assert_eq!(thumb(0.0, 100.0, 200.0, 200.0), (200.0, 0.0));
    }

    #[test]
    fn scrollbar_visibility() {
        let h = Duration::from_millis(900);
        assert!(Scrollbars::Auto.shown(true, false, Some(ClockMs(0)), h, ClockMs(800)));
        assert!(!Scrollbars::Auto.shown(true, false, Some(ClockMs(0)), h, ClockMs(1000)));
        assert!(Scrollbars::Always.shown(true, false, None, h, ClockMs(0)));
        assert!(!Scrollbars::Always.shown(false, true, None, h, ClockMs(0)));
    }

    #[test]
    fn stick_to_bottom() {
        let mut s = StickToBottom::default();
        let m = Metrics::vertical(800.0, 1000.0, 200.0);
        assert_eq!(
            s.content_changed(&Metrics::vertical(800.0, 1100.0, 200.0), 1),
            Stick::ScrollTo(900.0)
        );
        s.scrolled(&Metrics::vertical(900.0, 1100.0, 200.0));
        s.scrolled(&Metrics::vertical(500.0, 1100.0, 200.0));
        assert!(!s.following && s.pill());
        assert_eq!(
            s.content_changed(&Metrics::vertical(500.0, 1300.0, 200.0), 3),
            Stick::Stay
        );
        assert_eq!(s.pill_label(), "3 new messages");
        assert_eq!(
            s.jump(&Metrics::vertical(500.0, 1300.0, 200.0)),
            Stick::ScrollTo(1100.0)
        );
        assert!(s.following && s.unread == 0);
        let _ = m;
    }

    #[test]
    fn window_slides_like_a_belt() {
        let mut p = Pagination::new(1, 20);
        assert_eq!(p.slots(), (1..=5).map(PageSlot::Page).collect::<Vec<_>>());
        assert_eq!(p.go(3), 0);
        assert_eq!(p.go(6), 3);
        assert_eq!(p.slots()[0], PageSlot::Page(4));
        assert_eq!(p.mark_index(), Some(2));
        assert_eq!(p.go(99), 12);
        assert!(p.next_disabled() && p.page == 20);
        assert_eq!(p.slots().last(), Some(&PageSlot::Page(20)));
        let mut small = Pagination::new(2, 3);
        assert_eq!(small.slots().len(), 3);
        assert_eq!(small.back(), 0);
        assert!(small.prev_disabled());
        assert!(Pagination::new(1, 0).slots().is_empty());
    }

    #[test]
    fn ellipsis_cursor_and_sizes() {
        let mut p = Pagination::new(10, 20);
        p.mode = PageMode::Ellipsis;
        use PageSlot::*;
        assert_eq!(
            p.slots(),
            vec![Page(1), Gap, Page(9), Page(10), Page(11), Gap, Page(20)]
        );
        p.go(2);
        assert_eq!(
            p.slots(),
            vec![Page(1), Page(2), Page(3), Page(4), Page(5), Gap, Page(20)]
        );
        p.go(19);
        assert_eq!(
            p.slots(),
            vec![
                Page(1),
                Gap,
                Page(16),
                Page(17),
                Page(18),
                Page(19),
                Page(20)
            ]
        );
        let mut c = Pagination::new(3, 0);
        c.mode = PageMode::Cursor;
        c.page = 3;
        c.has_next = false;
        assert!(c.next_disabled() && !c.prev_disabled());
        assert_eq!(range_label(2, 20, 312), "21–40 of 312");
        assert_eq!(range_label(16, 20, 312), "301–312 of 312");
        assert_eq!(page_for_size(3, 20, 50), 1);
        assert_eq!(page_count(312, 20), 16);
    }
}
