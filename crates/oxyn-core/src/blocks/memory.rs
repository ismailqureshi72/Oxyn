//! Assistant memory: what an assistant remembers about you. Filter by group, search, edit or
//! forget a memory in place (with Undo), file the memory the current chat just made, add one by
//! typing (sorted into a group by a keyword guess) and switch saved memories or chat history off.

use super::{nth, rove};

/// Groups (the first filter is All).
pub const GROUPS: [&str; 4] = ["About you", "Preferences", "Projects", "People"];

/// Starting memories: text, group, source.
pub const MEMORIES: [(&str, usize, &str); 10] = [
    (
        "Works as a product designer on a payments team",
        0,
        "Chat on Nov 2",
    ),
    (
        "Lives in Lisbon and works on Western European Time",
        0,
        "Chat on Oct 18",
    ),
    (
        "Prefers metric units and 24 hour times",
        1,
        "Settings chat, Oct 9",
    ),
    (
        "Likes answers as short bullet lists, no emoji",
        1,
        "Chat on Nov 11",
    ),
    ("Writes code samples in TypeScript", 1, "Chat on Sep 30"),
    (
        "Redesigning the checkout flow, launch planned for March",
        2,
        "Chat on Nov 14",
    ),
    (
        "Keeps a reading list of design systems books",
        2,
        "Chat on Oct 3",
    ),
    (
        "Priya is their manager and reviews every Friday",
        3,
        "Chat on Nov 6",
    ),
    (
        "Tomás handles front-end for the checkout redesign",
        3,
        "Chat on Nov 14",
    ),
    ("Has a dog called Biscuit", 0, "Chat on Sep 12"),
];

/// The memory the current chat makes (the Memory updated pill).
pub const CAPTURE: (&str, usize) = ("Wants dates written as 21 Nov 2026", 1);

/// Memory capacity, in memories, for the meter.
pub const CAPACITY: usize = 16;
/// How long a forgotten memory can be restored, ms.
pub const UNDO_MS: u64 = 6_000;
/// How long a filed memory stays highlighted, ms.
pub const GLOW_MS: u64 = 1_600;

/// One memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mem {
    /// Stable id.
    pub id: usize,
    /// The sentence.
    pub text: String,
    /// Group index.
    pub group: usize,
    /// Where it came from.
    pub source: String,
    /// Filed at (for the highlight).
    pub at: Option<u64>,
}

/// Sorts a typed memory into a group by keywords (the preview's guess).
pub fn classify(t: &str) -> usize {
    let l = t.to_lowercase();
    let has = |w: &[&str]| w.iter().any(|x| l.contains(x));
    if has(&[
        "prefer", "like", "always", "never", "units", "format", "write in",
    ]) {
        1
    } else if has(&[
        "project",
        "launch",
        "working on",
        "building",
        "redesign",
        "deadline",
    ]) {
        2
    } else if has(&[
        "manager",
        "partner",
        "friend",
        "colleague",
        "sister",
        "brother",
        "mother",
        "father",
        " is my",
    ]) {
        3
    } else {
        0
    }
}

/// The memory card.
#[derive(Clone, Debug, PartialEq)]
pub struct Memory {
    /// Memories, newest filed last in their group.
    pub list: Vec<Mem>,
    /// Filter: 0 All, then group + 1.
    pub filter: usize,
    /// Search text, when the field is open.
    pub search: Option<String>,
    /// Saved memories on.
    pub saved: bool,
    /// Chat history used.
    pub history: bool,
    /// The chat's new memory is waiting to be filed.
    pub pending: bool,
    /// Editing: id and draft.
    pub editing: Option<(usize, String)>,
    /// Forgotten: memory, position and when.
    pub forgotten: Option<(Mem, usize, u64)>,
    /// Result of the last add: ok and words.
    pub added: Option<(bool, String)>,
    next: usize,
}

impl Memory {
    /// A card; `full` starts near capacity (warning), `paused` with saved memories off.
    pub fn new(full: bool, paused: bool) -> Self {
        let mut list: Vec<Mem> = MEMORIES
            .iter()
            .enumerate()
            .map(|(i, &(t, g, s))| Mem {
                id: i,
                text: t.into(),
                group: g,
                source: s.into(),
                at: None,
            })
            .collect();
        if full {
            for (k, t) in [
                "Uses a split keyboard",
                "Drinks oat flat whites",
                "Runs on Sunday mornings",
                "Speaks Portuguese and English",
            ]
            .iter()
            .enumerate()
            {
                list.push(Mem {
                    id: 20 + k,
                    text: (*t).into(),
                    group: if k == 3 { 0 } else { 1 },
                    source: "Chat on Nov 19".into(),
                    at: None,
                });
            }
        }
        Self {
            next: 40,
            list,
            filter: 0,
            search: None,
            saved: !paused,
            history: true,
            pending: !paused,
            editing: None,
            forgotten: None,
            added: None,
        }
    }

    /// How full memory is, 0..=1.
    pub fn usage(&self) -> f64 {
        (self.list.len() as f64 / CAPACITY as f64).min(1.0)
    }

    /// Warn from 80% full.
    pub fn warn(&self) -> bool {
        self.usage() >= 0.8
    }

    /// Memories shown: by group (in group order), filtered and searched.
    pub fn shown(&self) -> Vec<(usize, Vec<&Mem>)> {
        let q = self.search.as_deref().unwrap_or("").trim().to_lowercase();
        (0..GROUPS.len())
            .filter(|g| self.filter == 0 || self.filter == g + 1)
            .map(|g| {
                (
                    g,
                    self.list
                        .iter()
                        .filter(|m| {
                            m.group == g && (q.is_empty() || m.text.to_lowercase().contains(&q))
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_, v)| !v.is_empty())
            .collect()
    }

    /// Count per filter (All first).
    pub fn counts(&self) -> [usize; 5] {
        let mut c = [self.list.len(), 0, 0, 0, 0];
        for m in &self.list {
            c[m.group + 1] += 1;
        }
        c
    }

    /// The undo window is open at `now`.
    pub fn can_undo(&self, now: u64) -> bool {
        self.forgotten.as_ref().is_some_and(|f| now < f.2 + UNDO_MS)
    }

    /// Seconds left to undo.
    pub fn undo_left(&self, now: u64) -> u64 {
        self.forgotten
            .as_ref()
            .map_or(0, |f| (f.2 + UNDO_MS).saturating_sub(now).div_ceil(1000))
    }

    /// Highlighted (just filed) at `now`.
    pub fn glow(&self, m: &Mem, now: u64) -> bool {
        m.at.is_some_and(|a| now < a + GLOW_MS)
    }

    /// Set the search text.
    pub fn set_query(&mut self, q: &str) -> Option<String> {
        self.search = Some(q.to_string());
        let n: usize = self.shown().iter().map(|g| g.1.len()).sum();
        Some(format!(
            "{n} {} match",
            if n == 1 { "memory" } else { "memories" }
        ))
    }

    /// Set the edit draft.
    pub fn set_draft(&mut self, d: &str) {
        if let Some(e) = self.editing.as_mut() {
            e.1 = d.to_string();
        }
    }

    /// Adds a typed memory, sorted into a group.
    pub fn add(&mut self, t: &str, now: u64) -> Option<String> {
        let t = t.trim();
        if t.is_empty() {
            return None;
        }
        if !self.saved {
            self.added = Some((false, "Saved memories are off".into()));
            return Some("Couldn't add: saved memories are off".into());
        }
        if self.list.len() >= CAPACITY {
            self.added = Some((false, "Memory is full. Forget something first".into()));
            return Some("Couldn't add: memory is full".into());
        }
        let g = classify(t);
        self.next += 1;
        self.list.push(Mem {
            id: self.next,
            text: t.into(),
            group: g,
            source: "Added by you".into(),
            at: Some(now),
        });
        self.added = Some((true, format!("Added to {}", GROUPS[g])));
        Some(format!(
            "Added to {}. The preview sorts with a keyword guess",
            GROUPS[g]
        ))
    }

    fn pos(&self, id: usize) -> Option<usize> {
        self.list.iter().position(|m| m.id == id)
    }

    /// Presses: `mm-saved`, `mm-history`, `mm-f-<i>`, `mm-search`, `mm-file`, `mm-edit-<id>`,
    /// `mm-save`, `mm-cancel`, `mm-forget-<id>`, `mm-undo`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "mm-saved" => {
                self.saved = !self.saved;
                return Some(
                    if self.saved {
                        "Saved memories on"
                    } else {
                        "Saved memories off. Nothing new will be saved"
                    }
                    .into(),
                );
            }
            "mm-history" => {
                self.history = !self.history;
                return Some(
                    if self.history {
                        "Chat history on"
                    } else {
                        "Chat history off"
                    }
                    .into(),
                );
            }
            "mm-search" => {
                if self.search.is_some() {
                    self.search = None;
                    return Some("Search closed".into());
                }
                self.search = Some(String::new());
                return None;
            }
            "mm-file" if self.pending => {
                self.pending = false;
                self.next += 1;
                self.list.push(Mem {
                    id: self.next,
                    text: CAPTURE.0.into(),
                    group: CAPTURE.1,
                    source: "This chat".into(),
                    at: Some(now),
                });
                return Some(format!("Saved to {}", GROUPS[CAPTURE.1]));
            }
            "mm-save" => {
                let (id, d) = self.editing.take()?;
                let d = d.trim().to_string();
                if let (Some(p), false) = (self.pos(id), d.is_empty()) {
                    self.list[p].text = d;
                    return Some("Memory saved".into());
                }
                return Some("Edit cancelled".into());
            }
            "mm-cancel" => {
                self.editing.take()?;
                return Some("Edit cancelled".into());
            }
            "mm-undo" if self.can_undo(now) => {
                let (m, p, _) = self.forgotten.take()?;
                let t = m.text.clone();
                self.list.insert(p.min(self.list.len()), m);
                return Some(format!("Restored: {t}"));
            }
            _ => {}
        }
        if let Some(i) = nth(id, "mm-f-") {
            if i > GROUPS.len() {
                return None;
            }
            self.filter = i;
            return Some(if i == 0 {
                "All memories".into()
            } else {
                GROUPS[i - 1].to_string()
            });
        }
        if let Some(i) = nth(id, "mm-edit-") {
            let p = self.pos(i)?;
            self.editing = Some((i, self.list[p].text.clone()));
            return None;
        }
        if let Some(i) = nth(id, "mm-forget-") {
            let p = self.pos(i)?;
            let m = self.list.remove(p);
            let t = m.text.clone();
            self.forgotten = Some((m, p, now));
            return Some(format!("Forgot: {t}. Undo for 6 seconds"));
        }
        None
    }

    /// Keys: Left/Right/Home/End move the filter; Escape cancels an edit or closes the search.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let k = super::norm_key(key);
        match k {
            "Escape" if self.editing.is_some() => self.press("mm-cancel", now),
            "Escape" if self.search.is_some() => self.press("mm-search", now),
            "ArrowLeft" | "ArrowRight" | "Home" | "End" => {
                let i = rove(self.filter, GROUPS.len() + 1, k);
                self.press(&format!("mm-f-{i}"), now)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_edit_forget_undo_add() {
        let mut m = Memory::new(false, false);
        assert_eq!(m.counts()[0], 10);
        m.press("mm-file", 0);
        assert_eq!(m.counts()[2], 4);
        assert!(m.glow(m.list.last().unwrap(), 100));
        m.press("mm-edit-2", 0);
        m.set_draft("Prefers metric units");
        m.press("mm-save", 0);
        assert_eq!(m.list[2].text, "Prefers metric units");
        m.press("mm-forget-0", 1_000);
        assert_eq!(m.list.len(), 10);
        assert_eq!(m.undo_left(1_500), 6);
        m.press("mm-undo", 2_000);
        assert_eq!(m.list[0].id, 0);
        m.add("Sam is my sister", 0);
        assert_eq!(m.list.last().unwrap().group, 3);
        m.set_query("checkout");
        assert_eq!(m.shown().iter().map(|g| g.1.len()).sum::<usize>(), 2);
        m.key("escape", 0);
        assert!(m.search.is_none());
        m.key("end", 0);
        assert_eq!(m.filter, 4);
        let f = Memory::new(true, true);
        assert!(f.warn());
        assert!(!f.pending);
    }
}
