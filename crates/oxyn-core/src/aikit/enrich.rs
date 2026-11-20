//! Enrich column: one AI prompt runs down every row of a table column. Cells shimmer while they
//! work (a few at a time), then settle into a value with its source, or a failure to retry.

use super::{norm_key, nth};

/// One cell takes this long, ms.
pub const CELL_MS: u64 = 900;

/// Companies (fictional): name, domain.
pub const ROWS: [(&str, &str); 6] = [
    ("Fernhill Studio", "fernhill.studio"),
    ("Corvel Labs", "corvel.dev"),
    ("Quayside Freight", "quayside.io"),
    ("Pellam Health", "pellam.health"),
    ("Brightmoor Foods", "brightmoor.co"),
    ("Okapi Robotics", "okapi.ai"),
];

/// Column prompts: header, prompt, values per row (empty value means the lookup fails).
pub const COLUMNS: [(&str, &str, [&str; 6]); 3] = [
    (
        "Industry",
        "What industry is this company in?",
        [
            "Design agency",
            "Developer tools",
            "Logistics",
            "",
            "Food and beverage",
            "Robotics",
        ],
    ),
    (
        "Employees",
        "About how many people work there?",
        ["24", "130", "1,900", "410", "", "75"],
    ),
    (
        "HQ city",
        "Which city is the head office in?",
        ["Lisbon", "Berlin", "Rotterdam", "Boston", "Leeds", ""],
    ),
];

/// A cell's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellSt {
    /// Not run.
    Empty,
    /// Waiting its turn.
    Queued,
    /// Working.
    Running,
    /// Value found.
    Done,
    /// Lookup failed.
    Failed,
}

/// The live state of the enrich column.
#[derive(Clone, Debug, PartialEq)]
pub struct Enrich {
    /// Column prompt.
    pub column: usize,
    /// Run start per cell (staggered by concurrency).
    pub starts: [Option<u64>; 6],
    /// Retries per cell (a retried cell succeeds).
    pub retried: [bool; 6],
    /// Cells at once.
    pub concurrency: usize,
    /// Stopped at.
    pub stopped: Option<u64>,
    /// Column menu open.
    pub menu: bool,
}

impl Enrich {
    /// `auto` runs the column at `now`.
    pub fn new(column: usize, concurrency: usize, auto: bool, now: u64) -> Self {
        let mut e = Self {
            column: column.min(2),
            starts: [None; 6],
            retried: [false; 6],
            concurrency: concurrency.clamp(1, 6),
            stopped: None,
            menu: false,
        };
        if auto {
            e.run(now);
        }
        e
    }

    fn run(&mut self, now: u64) {
        for i in 0..6 {
            self.starts[i] = Some(now + (i / self.concurrency) as u64 * CELL_MS);
        }
        self.retried = [false; 6];
        self.stopped = None;
    }

    /// State of cell `i`.
    pub fn st(&self, i: usize, now: u64) -> CellSt {
        let Some(s) = self.starts[i] else {
            return CellSt::Empty;
        };
        let t = self.stopped.unwrap_or(now);
        if t < s {
            if self.stopped.is_some() {
                CellSt::Empty
            } else {
                CellSt::Queued
            }
        } else if t < s + CELL_MS {
            if self.stopped.is_some() {
                CellSt::Empty
            } else {
                CellSt::Running
            }
        } else if COLUMNS[self.column].2[i].is_empty() && !self.retried[i] {
            CellSt::Failed
        } else {
            CellSt::Done
        }
    }

    /// Value of cell `i` once done (a retried failure finds a value).
    pub fn value(&self, i: usize) -> &'static str {
        let v = COLUMNS[self.column].2[i];
        if v.is_empty() {
            ["Healthcare", "980", "Manchester"][self.column]
        } else {
            v
        }
    }

    /// Running, done and failed counts.
    pub fn counts(&self, now: u64) -> (usize, usize, usize) {
        let s: Vec<CellSt> = (0..6).map(|i| self.st(i, now)).collect();
        (
            s.iter()
                .filter(|x| matches!(x, CellSt::Running | CellSt::Queued))
                .count(),
            s.iter().filter(|x| **x == CellSt::Done).count(),
            s.iter().filter(|x| **x == CellSt::Failed).count(),
        )
    }

    /// Footer line (`4 of 6 filled · 1 failed · about $0.002`).
    pub fn footer(&self, now: u64) -> String {
        let (r, d, f) = self.counts(now);
        if r > 0 {
            return format!("Enriching {} of 6…", d + f + 1);
        }
        let mut s = format!("{d} of 6 filled");
        if f > 0 {
            s.push_str(&format!(" · {f} failed"));
        }
        s.push_str(&format!(" · about ${:.3}", d as f64 * 0.0004));
        s
    }

    /// Presses: `ec-run`, `ec-stop`, `ec-retry-<i>`, `ec-col-<i>`, `ec-menu`, `ec-conc-<n>`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "ec-retry-") {
            if i >= 6 || self.st(i, now) != CellSt::Failed {
                return None;
            }
            self.retried[i] = true;
            self.starts[i] = Some(now);
            return Some(format!("Retrying {}", ROWS[i].0));
        }
        if let Some(i) = nth(id, "ec-col-") {
            self.column = i.min(2);
            self.menu = false;
            self.starts = [None; 6];
            return Some(format!("Column {}", COLUMNS[self.column].0));
        }
        if let Some(n) = nth(id, "ec-conc-") {
            self.concurrency = n.clamp(1, 6);
            return Some(format!("{} at a time", self.concurrency));
        }
        match id {
            "ec-run" => {
                self.run(now);
                Some(format!(
                    "Enriching {} for 6 rows, simulated",
                    COLUMNS[self.column].0
                ))
            }
            "ec-stop" => {
                if self.counts(now).0 == 0 {
                    return None;
                }
                self.stopped = Some(now);
                Some("Stopped, filled cells are kept".into())
            }
            "ec-menu" => {
                self.menu = !self.menu;
                None
            }
            _ => None,
        }
    }

    /// Cmd/Ctrl+Enter runs (`Enter` with `cmd`), Escape stops or closes the menu.
    pub fn key(&mut self, key: &str, cmd: bool, now: u64) -> Option<String> {
        match norm_key(key) {
            "Enter" if cmd => self.press("ec-run", now),
            "Escape" => {
                if self.menu {
                    self.menu = false;
                    None
                } else {
                    self.press("ec-stop", now)
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enrich_runs_fails_retries_and_stops() {
        let mut e = Enrich::new(0, 3, true, 0);
        assert_eq!(e.st(0, 100), CellSt::Running);
        assert_eq!(e.st(4, 100), CellSt::Queued);
        assert_eq!(e.st(3, 5_000), CellSt::Failed);
        assert_eq!(e.counts(5_000), (0, 5, 1));
        e.press("ec-retry-3", 5_000);
        assert_eq!(e.st(3, 7_000), CellSt::Done);
        assert_eq!(e.value(3), "Healthcare");
        e.key("Enter", true, 10_000);
        e.key("Escape", false, 10_100);
        assert_eq!(e.st(4, 20_000), CellSt::Empty);
        e.press("ec-col-2", 0);
        assert_eq!(e.st(0, 0), CellSt::Empty);
        assert!(e.footer(0).contains("0 of 6"));
    }
}
