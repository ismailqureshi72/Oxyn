//! Storage family (variants 2): disk and context usage drawn as dot matrices – the Storage meter
//! (capacity bar of dots, category grid, rescan and clear), Storage cleanup (unused skills and
//! commands an agent loads every session), Storage waffle and Storage forecast (Oxyn originals).
//! Behaviour only: categories, figures and every line of copy are our own.

/// One storage category.
#[derive(Clone, Debug, PartialEq)]
pub struct StoreCat {
    /// Stable key (`apps`, `dev`, `photos`, `video`, `docs`, `music`, `system`, `caches`, `trash`).
    pub key: &'static str,
    /// Label.
    pub label: &'static str,
    /// Size in GB.
    pub gb: f64,
    /// How many things it holds ("126 apps").
    pub count: &'static str,
    /// What is left after clearing it (`None` when it cannot be cleared here).
    pub floor: Option<f64>,
}

/// The demo disk: 288.2 GB used of 512 GB.
pub fn store_demo() -> Vec<StoreCat> {
    let c = |key, label, gb, count, floor| StoreCat {
        key,
        label,
        gb,
        count,
        floor,
    };
    vec![
        c("apps", "Apps", 48.2, "126 apps", None),
        c("dev", "Developer", 61.7, "9 toolchains, 4 simulators", None),
        c("photos", "Photos", 39.5, "11,204 photos", None),
        c("video", "Video", 33.0, "86 clips", None),
        c("docs", "Documents", 27.4, "3,912 files", None),
        c("music", "Music", 9.8, "2,140 songs", None),
        c("system", "System", 31.6, "macOS and its data", None),
        c(
            "caches",
            "Caches",
            22.9,
            "Rebuilt by apps on demand",
            Some(1.1),
        ),
        c("trash", "Trash", 14.1, "418 items", Some(0.0)),
    ]
}

/// The category colour (`RRGGBB`) – hue marks categories only.
pub fn store_hex(key: &str) -> &'static str {
    match key {
        "apps" => "3B82F6",
        "dev" => "8B5CF6",
        "photos" => "EC4899",
        "video" => "06B6D4",
        "docs" => "22C55E",
        "music" => "EAB308",
        "system" => "8A8A93",
        "caches" => "F97316",
        "trash" => "EF4444",
        _ => "8A8A93",
    }
}

/// GB used by every category.
pub fn store_used(c: &[StoreCat]) -> f64 {
    c.iter().map(|x| x.gb).sum()
}

/// A size: `288.2 GB`, `512 GB`, `1.2 TB`, `840 MB`.
pub fn fmt_gb(gb: f64) -> String {
    if gb >= 1000.0 {
        let t = gb / 1000.0;
        return trim(format!("{t:.1}"), " TB");
    }
    if gb < 1.0 {
        return format!("{} MB", (gb * 1000.0).round() as u32);
    }
    trim(format!("{gb:.1}"), " GB")
}

fn trim(n: String, unit: &str) -> String {
    let n = n.strip_suffix(".0").map(str::to_string).unwrap_or(n);
    format!("{n}{unit}")
}

/// Dots per category out of `dots` for a disk of `cap` GB (largest remainder, at least one dot for
/// anything above zero). The rest is free.
pub fn dot_counts(c: &[StoreCat], cap: f64, dots: usize) -> Vec<usize> {
    let cap = cap.max(0.001);
    let raw: Vec<f64> = c.iter().map(|x| x.gb / cap * dots as f64).collect();
    let mut n: Vec<usize> = raw
        .iter()
        .zip(c)
        .map(|(r, x)| {
            if x.gb > 0.0 {
                (r.floor() as usize).max(1)
            } else {
                0
            }
        })
        .collect();
    let target = (store_used(c) / cap * dots as f64).round() as usize;
    let mut order: Vec<usize> = (0..c.len()).collect();
    order.sort_by(|a, b| {
        (raw[*b] - raw[*b].floor())
            .partial_cmp(&(raw[*a] - raw[*a].floor()))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut sum: usize = n.iter().sum();
    for i in order {
        if sum >= target.min(dots) {
            break;
        }
        if c[i].gb > 0.0 {
            n[i] += 1;
            sum += 1;
        }
    }
    n
}

/// The dots laid out in order: category index or `None` (free).
pub fn dot_runs(c: &[StoreCat], cap: f64, dots: usize) -> Vec<Option<usize>> {
    let mut v = Vec::with_capacity(dots);
    for (i, n) in dot_counts(c, cap, dots).into_iter().enumerate() {
        v.extend(std::iter::repeat_n(Some(i), n));
    }
    v.truncate(dots);
    v.resize(dots, None);
    v
}

/// The detail line for a category: "Caches · 22.9 GB · Rebuilt by apps on demand".
pub fn store_detail(c: &StoreCat) -> String {
    format!("{} · {} · {}", c.label, fmt_gb(c.gb), c.count)
}

/// Clears category `key` down to its floor; returns the GB freed (0 when it cannot be cleared).
pub fn store_clear(c: &mut [StoreCat], key: &str) -> f64 {
    match c.iter_mut().find(|x| x.key == key) {
        Some(x) => match x.floor {
            Some(f) if x.gb > f => {
                let d = x.gb - f;
                x.gb = f;
                d
            }
            _ => 0.0,
        },
        None => 0.0,
    }
}

/// An item an agent loads every session (a skill or command) and when it was last used.
#[derive(Clone, Debug, PartialEq)]
pub struct CleanItem {
    /// Name.
    pub name: &'static str,
    /// Where it lives ("Shared", "Project").
    pub place: &'static str,
    /// Tokens it adds to every session.
    pub tokens: u32,
    /// Days since last use; `None` = never used.
    pub idle: Option<u32>,
}

/// The demo list of skills and commands.
pub fn clean_demo() -> Vec<CleanItem> {
    let c = |name, place, tokens, idle| CleanItem {
        name,
        place,
        tokens,
        idle,
    };
    vec![
        c("release-notes", "Shared", 212, Some(140)),
        c("sql-explain", "Shared", 168, None),
        c("slide-maker", "Shared", 241, Some(201)),
        c("bug-triage", "Project", 97, Some(12)),
        c("api-docs", "Shared", 186, Some(64)),
        c("a11y-audit", "Shared", 153, Some(95)),
        c("regex-helper", "Shared", 88, None),
        c("csv-clean", "Project", 121, Some(33)),
        c("unit-test-gen", "Shared", 204, Some(3)),
        c("seo-meta", "Shared", 132, Some(260)),
        c("pr-summary", "Project", 117, Some(1)),
        c("icon-export", "Shared", 109, Some(48)),
    ]
}

/// Indexes of items not used in the last `days` (never-used ones included).
pub fn unused(items: &[CleanItem], days: u32) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter(|(_, x)| x.idle.is_none_or(|d| d > days))
        .map(|(i, _)| i)
        .collect()
}

/// "Never used", "Used yesterday", "Used 3 days ago", "Used 4 months ago".
pub fn idle_line(idle: Option<u32>) -> String {
    match idle {
        None => "Never used".into(),
        Some(0) => "Used today".into(),
        Some(1) => "Used yesterday".into(),
        Some(d) if d < 45 => format!("Used {d} days ago"),
        Some(d) => format!("Used {} months ago", (d as f64 / 30.0).round() as u32),
    }
}

/// A number with thousands separators: `1,840`.
pub fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// Used GB on each of the last 14 days (oldest first) of the demo disk – ends at today's 288.2.
pub fn store_history() -> Vec<f64> {
    [
        251.0, 252.4, 255.1, 255.9, 259.3, 262.0, 262.8, 266.5, 270.2, 273.0, 276.9, 279.4, 284.1,
        288.2,
    ]
    .to_vec()
}

/// GB per day over the history (last minus first over the days between).
pub fn store_pace(h: &[f64]) -> f64 {
    if h.len() < 2 {
        0.0
    } else {
        (h[h.len() - 1] - h[0]) / (h.len() - 1) as f64
    }
}

/// Whole days until `cap` at `pace` GB per day from `used`; `None` when not growing.
pub fn days_until_full(used: f64, cap: f64, pace: f64) -> Option<u32> {
    if pace <= 0.0 {
        None
    } else {
        Some(((cap - used).max(0.0) / pace).floor() as u32)
    }
}

/// How many of `rows` dots a value of `gb` lights on a disk of `cap` GB (rounded, at least 1).
pub fn col_dots(gb: f64, cap: f64, rows: usize) -> usize {
    ((gb / cap.max(0.001) * rows as f64).round() as usize).clamp(1, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        let d = store_demo();
        assert!((store_used(&d) - 288.2).abs() < 1e-6);
        assert_eq!(fmt_gb(288.2), "288.2 GB");
        assert_eq!(fmt_gb(512.0), "512 GB");
        assert_eq!(fmt_gb(0.84), "840 MB");
        assert_eq!(fmt_gb(1200.0), "1.2 TB");
        assert_eq!(
            store_detail(&d[7]),
            "Caches · 22.9 GB · Rebuilt by apps on demand"
        );
    }

    #[test]
    fn dots() {
        let d = store_demo();
        let n = dot_counts(&d, 512.0, 200);
        assert_eq!(n.iter().sum::<usize>(), 113);
        let r = dot_runs(&d, 512.0, 200);
        assert_eq!(r.len(), 200);
        assert_eq!(r[0], Some(0));
        assert_eq!(r[199], None);
        let mut d2 = d.clone();
        assert!((store_clear(&mut d2, "caches") - 21.8).abs() < 1e-6);
        assert_eq!(store_clear(&mut d2, "apps"), 0.0);
        assert_eq!(store_clear(&mut d2, "caches"), 0.0);
    }

    #[test]
    fn cleanup_and_forecast() {
        let c = clean_demo();
        assert_eq!(unused(&c, 30).len(), 9);
        assert_eq!(unused(&c, 90).len(), 6);
        assert_eq!(unused(&c, 180).len(), 4);
        assert_eq!(idle_line(None), "Never used");
        assert_eq!(idle_line(Some(140)), "Used 5 months ago");
        assert_eq!(idle_line(Some(12)), "Used 12 days ago");
        assert_eq!(thousands(1840), "1,840");
        let h = store_history();
        let p = store_pace(&h);
        assert!((p - 2.86).abs() < 0.01);
        assert_eq!(days_until_full(288.2, 512.0, p), Some(78));
        assert_eq!(days_until_full(288.2, 512.0, 0.0), None);
        assert_eq!(col_dots(288.2, 512.0, 12), 7);
    }
}
