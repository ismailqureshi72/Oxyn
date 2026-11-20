//! Provider keys: a bring-your-own-key settings card. Each provider row shows its mark, the
//! masked key, a status and when it was last used; a key can be revealed for five seconds,
//! copied, rotated or removed (with Undo), and a new key is added inline and tested first.

use super::{nth, usd};

/// Reveal lasts this long, ms.
pub const REVEAL_MS: u64 = 5_000;
/// Undo after remove lasts this long, ms.
pub const UNDO_MS: u64 = 6_000;
/// Testing a key takes this long, ms.
pub const TEST_MS: u64 = 900;

/// Providers: id (mark), name, key prefix, example tail.
pub const PROVIDERS: [(&str, &str, &str); 6] = [
    ("openai", "OpenAI", "sk-proj-"),
    ("anthropic", "Anthropic", "sk-ant-"),
    ("gemini", "Google AI", "AIza"),
    ("mistral", "Mistral", "ms-"),
    ("groq", "Groq", "gsk_"),
    ("deepseek", "DeepSeek", "sk-ds-"),
];

/// A key's status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeySt {
    /// Working.
    Active,
    /// Rejected by the provider.
    Invalid,
    /// Close to its rate limit.
    Limited,
}

/// One saved key.
#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    /// Provider index.
    pub provider: usize,
    /// The secret (sample).
    pub secret: String,
    /// Status.
    pub status: KeySt,
    /// Last used (`2 min ago`).
    pub used: &'static str,
    /// Spend this month, USD (Oxyn extra).
    pub spend: f64,
    /// Requests per minute used of the limit (Oxyn extra).
    pub rpm: (u32, u32),
    /// Project scope (Oxyn extra).
    pub scope: &'static str,
}

impl Key {
    /// `sk-ant-…4f9a`.
    pub fn masked(&self) -> String {
        let p = PROVIDERS[self.provider].2;
        let tail: String = self
            .secret
            .chars()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("{p}••••••••{tail}")
    }
}

/// The add form.
#[derive(Clone, Debug, PartialEq)]
pub struct Draft {
    /// Provider.
    pub provider: usize,
    /// Typed key.
    pub secret: String,
    /// Test started at.
    pub testing: Option<u64>,
}

/// The live state of the card.
#[derive(Clone, Debug, PartialEq)]
pub struct Keys {
    /// Saved keys.
    pub keys: Vec<Key>,
    /// Revealed key and when.
    pub reveal: Option<(usize, u64)>,
    /// Removed key, its index and when (for Undo).
    pub removed: Option<(Key, usize, u64)>,
    /// The add form.
    pub draft: Option<Draft>,
    /// Rotating key and when.
    pub rotating: Option<(usize, u64)>,
    /// Loading skeleton.
    pub loading: bool,
}

impl Keys {
    /// The sample (or none for the empty state).
    pub fn new(empty: bool) -> Self {
        let keys = if empty {
            vec![]
        } else {
            vec![
                Key {
                    provider: 0,
                    secret: "sk-proj-8Hq2Lm4Xc9Vt3Rz7b1e4".into(),
                    status: KeySt::Active,
                    used: "2 min ago",
                    spend: 41.80,
                    rpm: (120, 500),
                    scope: "All projects",
                },
                Key {
                    provider: 1,
                    secret: "sk-ant-api03-Zp7Nw2Kd5Qs8Lx4f9a".into(),
                    status: KeySt::Limited,
                    used: "just now",
                    spend: 128.4,
                    rpm: (46, 50),
                    scope: "Support bot",
                },
                Key {
                    provider: 2,
                    secret: "AIzaSyB4kT8mQ2wN6pL0c3d7".into(),
                    status: KeySt::Invalid,
                    used: "12 days ago",
                    spend: 0.0,
                    rpm: (0, 60),
                    scope: "All projects",
                },
            ]
        };
        Self {
            keys,
            reveal: None,
            removed: None,
            draft: None,
            rotating: None,
            loading: false,
        }
    }

    /// Key `i` is revealed at `now`.
    pub fn revealed(&self, i: usize, now: u64) -> bool {
        self.reveal
            .is_some_and(|(k, t)| k == i && now < t + REVEAL_MS)
    }

    /// Seconds left on a reveal.
    pub fn reveal_left(&self, now: u64) -> u64 {
        self.reveal
            .map(|(_, t)| (t + REVEAL_MS).saturating_sub(now).div_ceil(1000))
            .unwrap_or(0)
    }

    /// Undo is still offered.
    pub fn can_undo(&self, now: u64) -> bool {
        self.removed.as_ref().is_some_and(|r| now < r.2 + UNDO_MS)
    }

    /// The add form's test result at `now`: `None` testing or not tested, `Some(ok)`.
    pub fn test_result(&self, now: u64) -> Option<bool> {
        let d = self.draft.as_ref()?;
        let t = d.testing?;
        (now >= t + TEST_MS).then(|| Self::valid(d.provider, &d.secret))
    }

    /// The add form is testing.
    pub fn testing(&self, now: u64) -> bool {
        self.draft
            .as_ref()
            .and_then(|d| d.testing)
            .is_some_and(|t| now < t + TEST_MS)
    }

    /// A key looks valid for its provider (prefix and length).
    pub fn valid(provider: usize, secret: &str) -> bool {
        secret.starts_with(PROVIDERS[provider].2) && secret.len() >= 16
    }

    /// Key `i` is rotating.
    pub fn rotating_now(&self, i: usize, now: u64) -> bool {
        self.rotating
            .is_some_and(|(k, t)| k == i && now < t + TEST_MS)
    }

    /// Spend line (`$170.20 this month across 3 keys`).
    pub fn footer(&self) -> String {
        let s: f64 = self.keys.iter().map(|k| k.spend).sum();
        format!(
            "{} this month across {} key{}",
            usd(s),
            self.keys.len(),
            if self.keys.len() == 1 { "" } else { "s" }
        )
    }

    /// Providers without a key (for the add form).
    pub fn free_providers(&self) -> Vec<usize> {
        (0..PROVIDERS.len())
            .filter(|&p| !self.keys.iter().any(|k| k.provider == p))
            .collect()
    }

    /// Presses: `pk-reveal-<i>`, `pk-copy-<i>`, `pk-rotate-<i>`, `pk-remove-<i>`, `pk-undo`,
    /// `pk-add`, `pk-cancel`, `pk-prov-<p>`, `pk-input:<text>`, `pk-test`, `pk-save`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(t) = id.strip_prefix("pk-input:") {
            if let Some(d) = self.draft.as_mut() {
                d.secret = t.trim().to_string();
                d.testing = None;
            }
            return None;
        }
        if let Some(i) = nth(id, "pk-reveal-") {
            if i >= self.keys.len() {
                return None;
            }
            if self.revealed(i, now) {
                self.reveal = None;
                return Some("Key hidden".into());
            }
            self.reveal = Some((i, now));
            return Some(format!(
                "{} key shown for 5 seconds",
                PROVIDERS[self.keys[i].provider].1
            ));
        }
        if let Some(i) = nth(id, "pk-copy-") {
            let k = self.keys.get(i)?;
            return Some(format!("Copied the {} key", PROVIDERS[k.provider].1));
        }
        if let Some(i) = nth(id, "pk-rotate-") {
            let k = self.keys.get_mut(i)?;
            let p = PROVIDERS[k.provider];
            let tail = format!("{:04x}", (now / 7 + i as u64 * 977) % 0xffff);
            k.secret = format!("{}rot{}{}", p.2, "x".repeat(10), tail);
            k.status = KeySt::Active;
            k.used = "never";
            self.rotating = Some((i, now));
            self.reveal = None;
            return Some(format!(
                "Rotated the {} key, the old one stops working in 24 hours",
                p.1
            ));
        }
        if let Some(i) = nth(id, "pk-remove-") {
            if i >= self.keys.len() {
                return None;
            }
            let k = self.keys.remove(i);
            let name = PROVIDERS[k.provider].1;
            self.removed = Some((k, i, now));
            self.reveal = None;
            return Some(format!("Removed the {name} key, Undo is available"));
        }
        if let Some(p) = nth(id, "pk-prov-") {
            if let Some(d) = self.draft.as_mut() {
                d.provider = p.min(PROVIDERS.len() - 1);
                d.testing = None;
            }
            return None;
        }
        match id {
            "pk-undo" => {
                if !self.can_undo(now) {
                    return None;
                }
                let (k, i, _) = self.removed.take()?;
                let name = PROVIDERS[k.provider].1;
                self.keys.insert(i.min(self.keys.len()), k);
                Some(format!("Restored the {name} key"))
            }
            "pk-add" => {
                let p = *self.free_providers().first()?;
                self.draft = Some(Draft {
                    provider: p,
                    secret: String::new(),
                    testing: None,
                });
                Some("Add a key".into())
            }
            "pk-cancel" => {
                self.draft = None;
                None
            }
            "pk-test" => {
                let d = self.draft.as_mut()?;
                if d.secret.is_empty() {
                    return Some("Paste a key first".into());
                }
                d.testing = Some(now);
                Some(format!("Testing the {} key", PROVIDERS[d.provider].1))
            }
            "pk-save" => {
                let ok = self.test_result(now)?;
                if !ok {
                    return Some("The key was rejected, check it and test again".into());
                }
                let d = self.draft.take()?;
                let name = PROVIDERS[d.provider].1;
                self.keys.push(Key {
                    provider: d.provider,
                    secret: d.secret,
                    status: KeySt::Active,
                    used: "never",
                    spend: 0.0,
                    rpm: (0, 60),
                    scope: "All projects",
                });
                Some(format!("Saved the {name} key"))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_reveal_remove_undo_and_add() {
        let mut k = Keys::new(false);
        assert!(k.keys[1].masked().ends_with("4f9a"));
        k.press("pk-reveal-0", 0);
        assert!(k.revealed(0, 4_000));
        assert!(!k.revealed(0, 5_000));
        assert_eq!(k.reveal_left(1_200), 4);
        k.press("pk-remove-2", 0);
        assert_eq!(k.keys.len(), 2);
        assert!(k.can_undo(1_000));
        k.press("pk-undo", 1_000);
        assert_eq!(k.keys.len(), 3);
        k.press("pk-rotate-2", 0);
        assert_eq!(k.keys[2].status, KeySt::Active);
        k.press("pk-add", 0);
        assert_eq!(k.draft.as_ref().unwrap().provider, 3);
        k.press("pk-input:nope", 0);
        k.press("pk-test", 0);
        assert!(k.testing(100));
        assert_eq!(k.test_result(1_000), Some(false));
        assert!(k.press("pk-save", 1_000).unwrap().contains("rejected"));
        k.press("pk-input:ms-1234567890abcdef", 1_000);
        k.press("pk-test", 1_000);
        k.press("pk-save", 2_000);
        assert_eq!(k.keys.len(), 4);
        assert!(k.footer().contains("4 keys"));
    }
}
