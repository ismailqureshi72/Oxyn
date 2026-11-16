//! affectUI fidelity pass, family B part 9 (plan #32): the **Memory Panel** model. What an
//! assistant remembers about you, by category: each memory has its text, where it came from and
//! when, and can be paused (kept but not used), edited or forgotten (with a short undo). A master
//! switch turns memory off without deleting anything. Behaviour only: the views live in
//! `oxyn-dioxus` / `oxyn-gpui` (`parity/memoryfid.rs`).

/// One memory.
#[derive(Clone, Debug, PartialEq)]
pub struct MpMemory {
    /// Stable id.
    pub id: String,
    /// What is remembered.
    pub text: String,
    /// Its category.
    pub category: String,
    /// Where it came from ("From chat", "You added").
    pub source: String,
    /// When ("Oct 12").
    pub date: String,
    /// Kept but not used.
    pub paused: bool,
}

/// The demo categories in display order.
pub fn mp_categories() -> Vec<String> {
    vec!["Preferences".into(), "Work".into(), "Projects".into()]
}

/// How many memories each category holds, in `cats` order.
pub fn mp_counts(m: &[MpMemory], cats: &[String]) -> Vec<(String, usize)> {
    cats.iter()
        .map(|c| (c.clone(), m.iter().filter(|x| x.category == *c).count()))
        .collect()
}

/// The visible memories grouped by category (in `cats` order): the category filter (`None` =
/// all) and a case-insensitive search over the text.
pub fn mp_groups<'a>(
    m: &'a [MpMemory],
    cats: &[String],
    filter: Option<&str>,
    query: &str,
) -> Vec<(String, Vec<&'a MpMemory>)> {
    let q = query.trim().to_lowercase();
    cats.iter()
        .filter(|c| filter.is_none_or(|f| f == c.as_str()))
        .map(|c| {
            let rows: Vec<&MpMemory> = m
                .iter()
                .filter(|x| x.category == *c)
                .filter(|x| q.is_empty() || x.text.to_lowercase().contains(&q))
                .collect();
            (c.clone(), rows)
        })
        .filter(|(_, r)| !r.is_empty())
        .collect()
}

/// The status line: "8 memories · 1 paused", "Memory is off", or "<category> · used in answers"
/// for the row being pointed at.
pub fn mp_status(m: &[MpMemory], enabled: bool, pointed: Option<&MpMemory>) -> String {
    if !enabled {
        return "Memory is off".into();
    }
    if let Some(x) = pointed {
        return if x.paused {
            format!("{} · paused, not used", x.category)
        } else {
            format!("{} · used in answers", x.category)
        };
    }
    let n = m.len();
    let p = m.iter().filter(|x| x.paused).count();
    let base = format!("{n} {}", if n == 1 { "memory" } else { "memories" });
    if p > 0 {
        format!("{base} · {p} paused")
    } else {
        base
    }
}

/// A short quote of a memory for the undo line ("Prefers TypeScript examples …").
pub fn mp_quote(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let cut: String = text.chars().take(max).collect();
    let cut = cut
        .rsplit_once(' ')
        .map_or(cut.as_str(), |(a, _)| a)
        .to_string();
    format!("{cut} …")
}

fn mm(id: &str, text: &str, cat: &str, src: &str, date: &str, paused: bool) -> MpMemory {
    MpMemory {
        id: id.into(),
        text: text.into(),
        category: cat.into(),
        source: src.into(),
        date: date.into(),
        paused,
    }
}

/// The demo memories (8, one paused).
pub fn mp_demo() -> Vec<MpMemory> {
    vec![
        mm(
            "m1",
            "Likes answers short, with the code first",
            "Preferences",
            "From chat",
            "Oct 12",
            false,
        ),
        mm(
            "m2",
            "Writes Rust examples, not pseudocode",
            "Preferences",
            "You added",
            "Oct 2",
            false,
        ),
        mm(
            "m3",
            "Uses metric units",
            "Preferences",
            "From chat",
            "Sep 28",
            true,
        ),
        mm(
            "m4",
            "Works on the billing team",
            "Work",
            "From chat",
            "Oct 9",
            false,
        ),
        mm(
            "m5",
            "Reviews pull requests on Tuesday mornings",
            "Work",
            "From chat",
            "Oct 6",
            false,
        ),
        mm(
            "m6",
            "Deploys go through the staging cluster first",
            "Work",
            "You added",
            "Sep 30",
            false,
        ),
        mm(
            "m7",
            "Design tokens live in one JSON file",
            "Projects",
            "From chat",
            "Oct 11",
            false,
        ),
        mm(
            "m8",
            "The mobile app ships every second Thursday",
            "Projects",
            "From chat",
            "Oct 4",
            false,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_counts_status() {
        let m = mp_demo();
        let cats = mp_categories();
        let c = mp_counts(&m, &cats);
        assert_eq!(c[0], ("Preferences".to_string(), 3));
        assert_eq!(c[2].1, 2);
        let g = mp_groups(&m, &cats, None, "");
        assert_eq!(g.len(), 3);
        let g = mp_groups(&m, &cats, Some("Projects"), "");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].1.len(), 2);
        let g = mp_groups(&m, &cats, None, "TOKENS");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].1[0].id, "m7");
        assert!(mp_groups(&m, &cats, Some("Work"), "tokens").is_empty());
        assert_eq!(mp_status(&m, true, None), "8 memories · 1 paused");
        assert_eq!(mp_status(&m, false, None), "Memory is off");
        assert_eq!(mp_status(&m, true, Some(&m[3])), "Work · used in answers");
        assert_eq!(mp_status(&m[..1], true, None), "1 memory");
        assert_eq!(
            mp_quote("Writes Rust examples, not pseudocode", 20),
            "Writes Rust …"
        );
        assert_eq!(mp_quote("short", 20), "short");
    }
}
