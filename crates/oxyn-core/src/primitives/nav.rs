//! Shared list navigation for every roving-focus primitive: menus, selects, segmented controls,
//! radio groups and button groups. Arrow keys, Home / End, wrap, disabled items skipped, and
//! type-ahead (A–Z) with a reset window.

use std::time::Duration;

use crate::ClockMs;

/// A key a list understands. Hosts map their own key events onto this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavKey {
    /// ↑ (or ← in a horizontal list).
    Prev,
    /// ↓ (or → in a horizontal list).
    Next,
    /// Home.
    First,
    /// End.
    Last,
    /// Page Up / Shift+Arrow: a large step back.
    PageBack,
    /// Page Down / Shift+Arrow: a large step forward.
    PageForward,
}

/// Type-ahead buffer: letters typed within [`TYPEAHEAD_RESET`] of each other build a prefix.
pub const TYPEAHEAD_RESET: Duration = Duration::from_millis(500);

/// Moves `from` by `key` over `len` items, skipping those `enabled` says no to.
///
/// `wrap` makes ↓ on the last item go to the first. Returns `None` when nothing is enabled.
pub fn step(
    from: Option<usize>,
    key: NavKey,
    len: usize,
    wrap: bool,
    enabled: impl Fn(usize) -> bool,
) -> Option<usize> {
    if len == 0 || !(0..len).any(&enabled) {
        return None;
    }
    let first = (0..len).find(|&i| enabled(i));
    let last = (0..len).rev().find(|&i| enabled(i));
    match key {
        NavKey::First => first,
        NavKey::Last => last,
        NavKey::Next | NavKey::Prev => {
            let fwd = key == NavKey::Next;
            let Some(start) = from else {
                return if fwd { first } else { last };
            };
            let mut i = start;
            for _ in 0..len {
                i = if fwd {
                    if i + 1 >= len {
                        if !wrap {
                            return Some(start).filter(|&s| enabled(s)).or(last);
                        }
                        0
                    } else {
                        i + 1
                    }
                } else if i == 0 {
                    if !wrap {
                        return Some(start).filter(|&s| enabled(s)).or(first);
                    }
                    len - 1
                } else {
                    i - 1
                };
                if enabled(i) {
                    return Some(i);
                }
            }
            Some(start)
        }
        NavKey::PageBack | NavKey::PageForward => {
            let fwd = key == NavKey::PageForward;
            let start = from.unwrap_or(0) as isize;
            let target = (start + if fwd { 10 } else { -10 }).clamp(0, len as isize - 1) as usize;
            // Nearest enabled item at or before (forward) / after (back) the target.
            let near = if fwd {
                (0..=target).rev().find(|&i| enabled(i))
            } else {
                (target..len).find(|&i| enabled(i))
            };
            near.or(if fwd { last } else { first })
        }
    }
}

/// Type-ahead over item labels.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Typeahead {
    buf: String,
    last: Option<ClockMs>,
}

impl Typeahead {
    /// Feeds a printable character; returns the first enabled item (after `from`, wrapping) whose
    /// label starts with the buffer, case-insensitively. Repeating one letter cycles its matches.
    pub fn feed<'a>(
        &mut self,
        ch: char,
        now: ClockMs,
        from: Option<usize>,
        labels: impl Iterator<Item = (&'a str, bool)>,
    ) -> Option<usize> {
        if self.last.is_none_or(|t| now.since(t) > TYPEAHEAD_RESET) {
            self.buf.clear();
        }
        self.last = Some(now);
        self.buf.extend(ch.to_lowercase());
        let labels: Vec<(String, bool)> = labels.map(|(l, e)| (l.to_lowercase(), e)).collect();
        let n = labels.len();
        if n == 0 {
            return None;
        }
        // "aaa" with one letter repeated cycles through items starting with "a".
        let first = self.buf.chars().next().unwrap_or(ch);
        let cycling = self.buf.chars().all(|c| c == first) && self.buf.chars().count() > 1;
        let needle: String = if cycling {
            first.to_string()
        } else {
            self.buf.clone()
        };
        let start = match (from, cycling || self.buf.chars().count() == 1) {
            (Some(f), true) => f + 1,
            (Some(f), false) => f,
            (None, _) => 0,
        };
        (0..n)
            .map(|k| (start + k) % n)
            .find(|&i| labels[i].1 && labels[i].0.starts_with(&needle))
    }

    /// Current buffer (for a "searching: ab" hint).
    pub fn buffer(&self) -> &str {
        &self.buf
    }

    /// Clears the buffer.
    pub fn reset(&mut self) {
        self.buf.clear();
        self.last = None;
    }
}

/// Maps a pointer position along a track to the item under it, for drag-to-select on
/// segmented controls and button groups. `edges` are the item start offsets plus the end.
pub fn index_at(edges: &[f32], x: f32) -> Option<usize> {
    if edges.len() < 2 {
        return None;
    }
    if x >= edges[edges.len() - 1] {
        return Some(edges.len() - 2);
    }
    let x = x.max(edges[0]);
    edges.windows(2).position(|w| x >= w[0] && x < w[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_skips_disabled_and_wraps() {
        let en = |i: usize| i != 1;
        assert_eq!(step(Some(0), NavKey::Next, 4, true, en), Some(2));
        assert_eq!(step(Some(3), NavKey::Next, 4, true, en), Some(0));
        assert_eq!(step(Some(3), NavKey::Next, 4, false, en), Some(3));
        assert_eq!(step(Some(0), NavKey::Prev, 4, true, en), Some(3));
        assert_eq!(step(None, NavKey::Prev, 4, true, en), Some(3));
        assert_eq!(step(None, NavKey::First, 4, true, |i| i > 1), Some(2));
        assert_eq!(step(Some(2), NavKey::Last, 4, true, en), Some(3));
        assert_eq!(step(Some(0), NavKey::Next, 3, true, |_| false), None);
        assert_eq!(step(Some(0), NavKey::PageForward, 30, false, en), Some(10));
        assert_eq!(step(Some(5), NavKey::PageBack, 30, false, en), Some(0));
    }

    #[test]
    fn typeahead_prefix_cycle_and_reset() {
        let items = [
            ("Copy", true),
            ("Cut", true),
            ("Paste", true),
            ("Code", false),
            ("Close", true),
        ];
        let mut t = Typeahead::default();
        assert_eq!(
            t.feed('c', ClockMs(0), None, items.iter().copied()),
            Some(0)
        );
        assert_eq!(
            t.feed('l', ClockMs(100), Some(0), items.iter().copied()),
            Some(4)
        );
        // After the reset window a new letter starts over.
        assert_eq!(
            t.feed('p', ClockMs(900), Some(4), items.iter().copied()),
            Some(2)
        );
        // Repeating one letter cycles and skips disabled "Code".
        let mut t = Typeahead::default();
        assert_eq!(
            t.feed('c', ClockMs(0), None, items.iter().copied()),
            Some(0)
        );
        assert_eq!(
            t.feed('c', ClockMs(50), Some(0), items.iter().copied()),
            Some(1)
        );
        assert_eq!(
            t.feed('c', ClockMs(90), Some(1), items.iter().copied()),
            Some(4)
        );
        assert_eq!(t.buffer(), "ccc");
    }

    #[test]
    fn index_under_pointer() {
        let e = [0.0, 50.0, 120.0, 200.0];
        assert_eq!(index_at(&e, -5.0), Some(0));
        assert_eq!(index_at(&e, 60.0), Some(1));
        assert_eq!(index_at(&e, 500.0), Some(2));
        assert_eq!(index_at(&[0.0], 1.0), None);
    }
}
