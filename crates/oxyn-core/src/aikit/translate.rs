//! AI translate: a message card where picking a language streams the translation in place,
//! sentence by sentence over the dimmed original, with a detected-language chip, a Show original
//! toggle and a status line credited to the model.

use super::{norm_key, nth, rove};

/// The model thinks for this long before the first sentence, ms.
pub const THINK_MS: u64 = 450;

/// Languages: code, name.
pub const LANGS: [(&str, &str); 6] = [
    ("en", "English"),
    ("de", "German"),
    ("fr", "French"),
    ("es", "Spanish"),
    ("ja", "Japanese"),
    ("pt", "Portuguese"),
];

/// The message (English) as sentences, then each translation.
pub const TEXT: [[&str; 3]; 6] = [
    [
        "The launch moves to Tuesday so the EU tax check can finish.",
        "Please keep the pricing page in draft until then.",
        "I'll send the final copy on Monday afternoon.",
    ],
    [
        "Der Launch wird auf Dienstag verschoben, damit die EU-Steuerprüfung abgeschlossen werden kann.",
        "Bitte lass die Preisseite bis dahin im Entwurf.",
        "Ich schicke den finalen Text am Montagnachmittag.",
    ],
    [
        "Le lancement passe à mardi pour que la vérification de la TVA européenne soit terminée.",
        "Merci de laisser la page des tarifs en brouillon d'ici là.",
        "J'enverrai le texte final lundi après-midi.",
    ],
    [
        "El lanzamiento pasa al martes para que termine la revisión del IVA europeo.",
        "Mantén la página de precios en borrador hasta entonces.",
        "Enviaré el texto final el lunes por la tarde.",
    ],
    [
        "EUの税務チェックを終えるため、ローンチは火曜日に延期します。",
        "それまで料金ページは下書きのままにしてください。",
        "最終版の文章は月曜日の午後に送ります。",
    ],
    [
        "O lançamento passa para terça para que a verificação do imposto da UE termine.",
        "Mantenha a página de preços em rascunho até lá.",
        "Envio o texto final na segunda à tarde.",
    ],
];

/// Words (or about ten characters for Japanese) a chunk.
fn units(lang: usize, s: &str) -> usize {
    if lang == 4 {
        s.chars().count().div_ceil(10)
    } else {
        s.split_whitespace().count()
    }
}

/// The live state of the card.
#[derive(Clone, Debug, PartialEq)]
pub struct Translate {
    /// Target language (0 is the original).
    pub lang: usize,
    /// Translation started at.
    pub since: u64,
    /// Show the original instead of the translation.
    pub original: bool,
    /// Language menu open.
    pub menu: bool,
    /// Menu highlight.
    pub hi: usize,
    /// Show translation and original side by side (Oxyn extra).
    pub side: bool,
    /// The next translation fails (the error scenario).
    pub fail: bool,
}

impl Translate {
    /// `lang` 0 shows the original; another starts translating at `now`.
    pub fn new(lang: usize, fail: bool, now: u64) -> Self {
        Self {
            lang: lang.min(5),
            since: now,
            original: false,
            menu: false,
            hi: lang.min(5),
            side: false,
            fail,
        }
    }

    /// Units of sentence `i` visible at `now` (all of the original when lang is 0).
    pub fn shown(&self, i: usize, now: u64) -> usize {
        if self.lang == 0 {
            return 0;
        }
        let el = now.saturating_sub(self.since);
        if el < THINK_MS || (self.fail && el >= THINK_MS + 300) {
            return 0;
        }
        let per = 4;
        let ticks = (el - THINK_MS) / super::STREAM_TICK_MS + 1;
        let before: usize = (0..i).map(|j| units(self.lang, TEXT[self.lang][j])).sum();
        ((ticks as usize) * per)
            .saturating_sub(before)
            .min(units(self.lang, TEXT[self.lang][i]))
    }

    /// The visible part of sentence `i`.
    pub fn sentence(&self, i: usize, now: u64) -> String {
        let s = TEXT[self.lang][i];
        let n = self.shown(i, now);
        if self.lang == 4 {
            s.chars().take(n * 10).collect()
        } else {
            super::first_words(s, n)
        }
    }

    /// Sentence `i` finished in the new language.
    pub fn done_sentence(&self, i: usize, now: u64) -> bool {
        self.lang != 0 && self.shown(i, now) >= units(self.lang, TEXT[self.lang][i])
    }

    /// Status: `idle`, `translating`, `done`, `error`.
    pub fn status(&self, now: u64) -> &'static str {
        if self.lang == 0 {
            "idle"
        } else if self.fail && now >= self.since + THINK_MS + 300 {
            "error"
        } else if (0..3).all(|i| self.done_sentence(i, now)) {
            "done"
        } else {
            "translating"
        }
    }

    /// The status line.
    pub fn status_line(&self, now: u64) -> String {
        let name = LANGS[self.lang].1;
        match self.status(now) {
            "idle" => "Detected English".into(),
            "error" => format!("Couldn't translate to {name}. Try again."),
            "done" => format!("Translated to {name} by Claude Haiku 4.5"),
            _ => format!("Translating to {name}…"),
        }
    }

    /// Presses: `tr-lang-<i>`, `tr-menu`, `tr-orig`, `tr-retry`, `tr-side`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "tr-lang-") {
            let i = i.min(5);
            self.menu = false;
            self.hi = i;
            if i == self.lang {
                return None;
            }
            self.lang = i;
            self.since = now;
            self.original = false;
            return Some(if i == 0 {
                "Showing the original".into()
            } else {
                format!("Translating to {}", LANGS[i].1)
            });
        }
        match id {
            "tr-menu" => {
                self.menu = !self.menu;
                self.hi = self.lang;
                None
            }
            "tr-orig" if self.lang != 0 => {
                self.original = !self.original;
                Some(
                    if self.original {
                        "Showing the original"
                    } else {
                        "Showing the translation"
                    }
                    .into(),
                )
            }
            "tr-retry" => {
                self.fail = false;
                self.since = now;
                Some(format!("Translating to {}", LANGS[self.lang].1))
            }
            "tr-side" => {
                self.side = !self.side;
                None
            }
            _ => None,
        }
    }

    /// Keys in the menu: ↑/↓, Home, End move; Enter or Space picks; Escape closes.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        match key {
            "ArrowDown" | "ArrowUp" | "Home" | "End" => {
                if !self.menu {
                    self.menu = true;
                    return None;
                }
                self.hi = rove(self.hi, LANGS.len(), key);
                None
            }
            "Enter" | "Space" => {
                if self.menu {
                    self.press(&format!("tr-lang-{}", self.hi), now)
                } else {
                    self.menu = true;
                    None
                }
            }
            "Escape" => {
                self.menu = false;
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_streams_by_sentence() {
        let mut t = Translate::new(0, false, 0);
        assert_eq!(t.status(0), "idle");
        t.key("ArrowDown", 0);
        assert!(t.menu);
        t.key("ArrowDown", 0);
        t.key("Enter", 0);
        assert_eq!(t.lang, 1);
        assert_eq!(t.shown(0, 100), 0);
        assert!(t.shown(0, 500) > 0);
        assert_eq!(t.shown(2, 500), 0);
        assert_eq!(t.status(5_000), "done");
        t.press("tr-orig", 5_000);
        assert!(t.original);
        t.press("tr-lang-4", 6_000);
        assert!(t.sentence(0, 9_000).chars().count() > 10);
        let mut f = Translate::new(2, true, 0);
        assert_eq!(f.status(2_000), "error");
        f.press("tr-retry", 2_000);
        assert_eq!(f.status(9_000), "done");
    }
}
