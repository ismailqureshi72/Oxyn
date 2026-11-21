//! Turns the slugs a user asked for into the registry items to install, dependencies first, and
//! explains why one can't be installed (Pro, unknown, not for this framework).

use std::collections::BTreeSet;

use oxyn_registry_schema::{Framework, IndexEntry, RegistryIndex, RegistryItem, Tier};

use crate::Error;
use crate::client::Client;

/// Items to install, in install order (dependencies before the items that need them).
pub fn resolve(
    client: &Client,
    index: &RegistryIndex,
    slugs: &[String],
    fw: Framework,
) -> Result<Vec<RegistryItem>, Error> {
    let mut out: Vec<RegistryItem> = Vec::new();
    let mut done: BTreeSet<String> = BTreeSet::new();
    let mut visiting: Vec<String> = Vec::new();
    for s in slugs {
        visit(
            client,
            index,
            s,
            fw,
            &mut out,
            &mut done,
            &mut visiting,
            None,
        )?;
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn visit(
    client: &Client,
    index: &RegistryIndex,
    slug: &str,
    fw: Framework,
    out: &mut Vec<RegistryItem>,
    done: &mut BTreeSet<String>,
    visiting: &mut Vec<String>,
    needed_by: Option<&str>,
) -> Result<(), Error> {
    if done.contains(slug) || visiting.iter().any(|s| s == slug) {
        return Ok(());
    }
    let entry = index
        .get(slug)
        .ok_or_else(|| Error::UnknownItem(slug.to_string(), suggest(index, slug)))?;
    check_entry(entry, fw, needed_by)?;
    let item = client.item(slug)?.ok_or_else(|| {
        Error::Invalid(format!(
            "the registry lists `{slug}` but has no {slug}.json; try again later"
        ))
    })?;
    let build = item
        .build(fw)
        .ok_or_else(|| Error::NoFramework(slug.to_string(), fw, item.framework_list()))?;
    visiting.push(slug.to_string());
    for d in build.oxyn_deps.clone() {
        visit(client, index, &d, fw, out, done, visiting, Some(slug))?;
    }
    visiting.pop();
    done.insert(slug.to_string());
    out.push(item);
    Ok(())
}

/// Why an index entry can't be installed with the free registry, if it can't.
pub fn check_entry(
    entry: &IndexEntry,
    fw: Framework,
    needed_by: Option<&str>,
) -> Result<(), Error> {
    if entry.tier == Tier::Pro {
        return Err(match needed_by {
            Some(by) => Error::Invalid(format!(
                "`{by}` needs `{}`, which is part of Oxyn Pro ({}/)",
                entry.slug,
                crate::client::PRICING_URL
            )),
            None => Error::ProRequired(entry.slug.clone()),
        });
    }
    if !entry.installable {
        return Err(Error::Invalid(format!(
            "`{}` can't be added from the free registry: {}",
            entry.slug,
            entry
                .note
                .clone()
                .unwrap_or_else(|| "it has no installable build".into())
        )));
    }
    if !entry.frameworks.is_empty() && !entry.frameworks.contains(&fw) {
        return Err(Error::NoFramework(
            entry.slug.clone(),
            fw,
            entry.frameworks.clone(),
        ));
    }
    Ok(())
}

/// Up to three slugs close to a mistyped one.
pub fn suggest(index: &RegistryIndex, slug: &str) -> Vec<String> {
    let mut scored: Vec<(usize, &str)> = index
        .items
        .iter()
        .map(|e| {
            let d = distance(&e.slug, slug);
            let bonus = if e.slug.contains(slug) || slug.contains(&e.slug) {
                0
            } else {
                1
            };
            (d * 2 + bonus, e.slug.as_str())
        })
        .collect();
    scored.sort();
    scored
        .into_iter()
        .filter(|(d, _)| *d <= slug.len().max(4))
        .take(3)
        .map(|(_, s)| s.to_string())
        .collect()
}

fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            cur[j + 1] = (prev[j] + usize::from(ca != cb))
                .min(prev[j + 1] + 1)
                .min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Entries matching a search query, best first: slug and name matches before description ones.
pub fn search<'a>(index: &'a RegistryIndex, query: &str) -> Vec<&'a IndexEntry> {
    let q = query.to_lowercase();
    let words: Vec<&str> = q.split_whitespace().collect();
    let mut hits: Vec<(u32, &IndexEntry)> = index
        .items
        .iter()
        .filter_map(|e| {
            let slug = e.slug.to_lowercase();
            let name = e.name.to_lowercase();
            let desc = e.description.to_lowercase();
            let cat = e.category.to_lowercase();
            let mut score = 0;
            for w in &words {
                let s = if slug == *w || name == *w {
                    100
                } else if slug.starts_with(w) || name.starts_with(w) {
                    60
                } else if slug.contains(w) || name.contains(w) {
                    40
                } else if cat.contains(w) {
                    20
                } else if desc.contains(w) {
                    10
                } else {
                    return None;
                };
                score += s;
            }
            Some((score, e))
        })
        .collect();
    hits.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.slug.cmp(&b.1.slug)));
    hits.into_iter().map(|(_, e)| e).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fake_registry;

    #[test]
    fn resolves_dependencies_first_and_once() {
        let (_d, client) = fake_registry();
        let idx = client.index().unwrap();
        let items = resolve(
            &client,
            &idx,
            &["send-button".into(), "theme".into(), "send-button".into()],
            Framework::Dioxus,
        )
        .unwrap();
        let order: Vec<&str> = items.iter().map(|i| i.slug.as_str()).collect();
        assert_eq!(order, vec!["theme", "send-button"]);
    }

    #[test]
    fn pro_unknown_and_framework_errors() {
        let (_d, client) = fake_registry();
        let idx = client.index().unwrap();
        assert!(
            matches!(resolve(&client, &idx, &["fancy-screen".into()], Framework::Dioxus), Err(Error::ProRequired(s)) if s == "fancy-screen")
        );
        match resolve(&client, &idx, &["send-buton".into()], Framework::Dioxus) {
            Err(Error::UnknownItem(s, sug)) => {
                assert_eq!(s, "send-buton");
                assert_eq!(sug.first().map(String::as_str), Some("send-button"));
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            resolve(&client, &idx, &["send-button".into()], Framework::Gpui),
            Err(Error::NoFramework(..))
        ));
    }

    #[test]
    fn search_ranks_names_first() {
        let (_d, client) = fake_registry();
        let idx = client.index().unwrap();
        let hits: Vec<&str> = search(&idx, "send")
            .iter()
            .map(|e| e.slug.as_str())
            .collect();
        assert_eq!(hits[0], "send-button");
        assert!(search(&idx, "nothing-like-this").is_empty());
        assert_eq!(distance("kitten", "sitting"), 3);
    }
}
