//! Phone input (`phone-input.md`): a phone field with a searchable country picker that grows out
//! of the flag button, formatting as you type, and E.164 output. `+` in the number opens the
//! picker with a calling-code search; a pasted international number switches the country.
//!
//! Behaviour follows the Arc UI Phone input docs (rebuilt, no source used; the country table is
//! Oxyn's own, from public ITU calling codes). Oxyn plus: a masked display (`•••• 4821`) and a
//! country picked from the user's locale (`en-GB` → United Kingdom).

/// One country.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Country {
    /// ISO 3166 alpha-2.
    pub iso: &'static str,
    /// Name.
    pub name: &'static str,
    /// Calling code without `+`.
    pub code: &'static str,
    /// National format; `#` is a digit.
    pub pattern: &'static str,
    /// Trunk prefix dialled at home (`0`), dropped in E.164.
    pub trunk: Option<char>,
    /// Example national number.
    pub example: &'static str,
}

impl Country {
    /// National significant digits for a valid number.
    pub fn digits(&self) -> usize {
        self.pattern.chars().filter(|&c| c == '#').count()
    }
    /// Flag emoji from the ISO code.
    pub fn flag(&self) -> String {
        flag_of(self.iso)
    }
}

/// Flag emoji for an ISO code (`GB` → 🇬🇧).
pub fn flag_of(iso: &str) -> String {
    iso.chars()
        .filter(char::is_ascii_alphabetic)
        .filter_map(|c| char::from_u32(0x1F1E6 + (c.to_ascii_uppercase() as u32 - 'A' as u32)))
        .collect()
}

macro_rules! c {
    ($iso:literal, $name:literal, $code:literal, $pat:literal, $trunk:expr, $ex:literal) => {
        Country {
            iso: $iso,
            name: $name,
            code: $code,
            pattern: $pat,
            trunk: $trunk,
            example: $ex,
        }
    };
}

/// The built-in table (32 countries).
pub const COUNTRIES: [Country; 32] = [
    c!(
        "US",
        "United States",
        "1",
        "(###) ###-####",
        None,
        "4155550132"
    ),
    c!("CA", "Canada", "1", "(###) ###-####", None, "6045550199"),
    c!(
        "GB",
        "United Kingdom",
        "44",
        "#### ######",
        Some('0'),
        "7400123456"
    ),
    c!(
        "IE",
        "Ireland",
        "353",
        "## ### ####",
        Some('0'),
        "851234567"
    ),
    c!(
        "FR",
        "France",
        "33",
        "# ## ## ## ##",
        Some('0'),
        "612345678"
    ),
    c!(
        "DE",
        "Germany",
        "49",
        "### ########",
        Some('0'),
        "15123456789"
    ),
    c!("ES", "Spain", "34", "### ### ###", None, "612345678"),
    c!("IT", "Italy", "39", "### ### ####", None, "3123456789"),
    c!(
        "NL",
        "Netherlands",
        "31",
        "# ########",
        Some('0'),
        "612345678"
    ),
    c!("SE", "Sweden", "46", "## ### ## ##", Some('0'), "701234567"),
    c!(
        "CH",
        "Switzerland",
        "41",
        "## ### ## ##",
        Some('0'),
        "781234567"
    ),
    c!("PL", "Poland", "48", "### ### ###", None, "512345678"),
    c!("PT", "Portugal", "351", "### ### ###", None, "912345678"),
    c!(
        "TR",
        "Türkiye",
        "90",
        "### ### ## ##",
        Some('0'),
        "5012345678"
    ),
    c!(
        "AE",
        "United Arab Emirates",
        "971",
        "## ### ####",
        Some('0'),
        "501234567"
    ),
    c!(
        "SA",
        "Saudi Arabia",
        "966",
        "## ### ####",
        Some('0'),
        "512345678"
    ),
    c!(
        "PK",
        "Pakistan",
        "92",
        "### #######",
        Some('0'),
        "3012345678"
    ),
    c!("IN", "India", "91", "##### #####", Some('0'), "9123456789"),
    c!(
        "BD",
        "Bangladesh",
        "880",
        "####-######",
        Some('0'),
        "1812345678"
    ),
    c!(
        "CN",
        "China",
        "86",
        "### #### ####",
        Some('0'),
        "13123456789"
    ),
    c!("JP", "Japan", "81", "##-####-####", Some('0'), "9012345678"),
    c!(
        "KR",
        "South Korea",
        "82",
        "##-####-####",
        Some('0'),
        "1023456789"
    ),
    c!("SG", "Singapore", "65", "#### ####", None, "81234567"),
    c!(
        "ID",
        "Indonesia",
        "62",
        "###-####-####",
        Some('0'),
        "81234567890"
    ),
    c!(
        "AU",
        "Australia",
        "61",
        "### ### ###",
        Some('0'),
        "412345678"
    ),
    c!(
        "NZ",
        "New Zealand",
        "64",
        "## ### ####",
        Some('0'),
        "211234567"
    ),
    c!(
        "BR",
        "Brazil",
        "55",
        "(##) #####-####",
        Some('0'),
        "11912345678"
    ),
    c!("MX", "Mexico", "52", "## #### ####", None, "5512345678"),
    c!(
        "AR",
        "Argentina",
        "54",
        "## ####-####",
        Some('0'),
        "1123456789"
    ),
    c!(
        "ZA",
        "South Africa",
        "27",
        "## ### ####",
        Some('0'),
        "711234567"
    ),
    c!(
        "NG",
        "Nigeria",
        "234",
        "### ### ####",
        Some('0'),
        "8021234567"
    ),
    c!("EG", "Egypt", "20", "### ### ####", Some('0'), "1001234567"),
];

/// Looks up a country by ISO code.
pub fn country(iso: &str) -> Option<&'static Country> {
    COUNTRIES.iter().find(|c| c.iso.eq_ignore_ascii_case(iso))
}

/// Oxyn plus: the country from a BCP 47 locale (`en-GB`, `pt_BR`).
pub fn country_from_locale(locale: &str) -> Option<&'static Country> {
    let region = locale.split(['-', '_']).nth(1)?;
    country(region)
}

/// Formats national digits for `c`, keeping a typed trunk prefix in front.
pub fn format_national(c: &Country, digits: &str) -> String {
    let (trunk, body) = match (c.trunk, digits.chars().next()) {
        (Some(t), Some(f)) if f == t => (Some(t), &digits[1..]),
        _ => (None, digits),
    };
    let mut out = String::new();
    if let Some(t) = trunk {
        out.push(t);
        if !body.is_empty() {
            out.push(' ');
        }
    }
    let mut it = body.chars().peekable();
    let mut pending = String::new();
    for p in c.pattern.chars() {
        if it.peek().is_none() {
            break;
        }
        if p == '#' {
            out.push_str(&pending);
            pending.clear();
            out.extend(it.next());
        } else {
            pending.push(p);
        }
    }
    // Digits past the pattern are appended so nothing typed is lost.
    out.extend(it);
    out
}

/// Parses `+44 (0)7911 123456`, `0044 7911…` or `+1 415…` into the country and national digits.
pub fn parse(input: &str) -> Option<(&'static Country, String)> {
    let t = input.trim();
    let intl = t.starts_with('+') || t.starts_with("00");
    if !intl {
        return None;
    }
    // Drop a "(0)" trunk written inside an international number.
    let cleaned = t.replace("(0)", "");
    let mut digits: String = cleaned.chars().filter(char::is_ascii_digit).collect();
    if t.starts_with("00") {
        digits.drain(..2);
    }
    // Longest code first (1–3 digits); +1 goes to the US unless the area code says Canada.
    for len in (1..=3).rev() {
        if digits.len() <= len {
            continue;
        }
        let (code, rest) = digits.split_at(len);
        if let Some(c) = COUNTRIES.iter().find(|c| c.code == code) {
            let c = if code == "1" && CANADA_AREA.contains(&&rest[..rest.len().min(3)]) {
                country("CA").unwrap_or(c)
            } else {
                c
            };
            return Some((c, rest.to_string()));
        }
    }
    None
}

const CANADA_AREA: [&str; 12] = [
    "204", "236", "250", "289", "306", "403", "416", "438", "514", "604", "613", "647",
];

/// How the number reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PhoneStatus {
    /// No digits.
    #[default]
    Empty,
    /// Too few digits for the country.
    Incomplete,
    /// The right length.
    Valid,
    /// Too many digits.
    TooLong,
}

/// A row of the country list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CountryRow {
    /// Group title (`Suggested`, `All countries`).
    Group(&'static str),
    /// A country (index into [`COUNTRIES`]).
    Country(usize),
    /// Nothing matches.
    Empty,
}

/// The phone field.
#[derive(Clone, Debug, PartialEq)]
pub struct PhoneInput {
    /// Selected country.
    pub country: &'static Country,
    /// National digits as typed (may start with the trunk prefix).
    pub digits: String,
    /// Picker open.
    pub open: bool,
    /// Picker search.
    pub query: String,
    /// Highlighted row in the picker.
    pub active: Option<usize>,
    /// Pinned at the top under Suggested.
    pub preferred: Vec<&'static str>,
    /// Limit the picker to these ISO codes.
    pub allowed: Option<Vec<&'static str>>,
    /// Blurred at least once (errors show after blur).
    pub touched: bool,
    /// Show a message after blur on the wrong length.
    pub validate: bool,
    /// Disabled.
    pub disabled: bool,
    /// Oxyn plus: masked display when not focused.
    pub masked: bool,
    /// Which way the flag rolled on the last change (`true` = down the list).
    pub rolled_down: bool,
    /// Live-region line.
    pub announce: String,
}

impl PhoneInput {
    /// A field for a country (falls back to the US).
    pub fn new(iso: &str) -> Self {
        Self {
            country: country(iso).unwrap_or(&COUNTRIES[0]),
            digits: String::new(),
            open: false,
            query: String::new(),
            active: None,
            preferred: vec!["US", "CA", "GB"],
            allowed: None,
            touched: false,
            validate: true,
            disabled: false,
            masked: false,
            rolled_down: true,
            announce: String::new(),
        }
    }
    /// From an E.164 value.
    pub fn from_e164(v: &str, fallback: &str) -> Self {
        let mut p = Self::new(fallback);
        if let Some((c, d)) = parse(v) {
            p.country = c;
            p.digits = d;
        }
        p
    }
    fn national(&self) -> &str {
        match (self.country.trunk, self.digits.chars().next()) {
            (Some(t), Some(f)) if f == t => &self.digits[1..],
            _ => &self.digits,
        }
    }
    /// E.164 (`+447400123456`), empty without digits.
    pub fn e164(&self) -> String {
        let n = self.national();
        if n.is_empty() {
            String::new()
        } else {
            format!("+{}{}", self.country.code, n)
        }
    }
    /// The formatted national number shown in the field.
    pub fn formatted(&self) -> String {
        format_national(self.country, &self.digits)
    }
    /// `+44 7400 123456`.
    pub fn international(&self) -> String {
        let n = self.national();
        format!(
            "+{} {}",
            self.country.code,
            format_national(self.country, n)
        )
    }
    /// Masked (Oxyn plus): `•••• 3456`.
    pub fn masked_text(&self) -> String {
        let n = self.national();
        let tail: String = n
            .chars()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if n.len() <= 4 {
            n.into()
        } else {
            format!("•••• {tail}")
        }
    }
    /// Status.
    pub fn status(&self) -> PhoneStatus {
        let n = self.national().len();
        let want = self.country.digits();
        match n {
            0 => PhoneStatus::Empty,
            n if n < want => PhoneStatus::Incomplete,
            n if n == want => PhoneStatus::Valid,
            _ => PhoneStatus::TooLong,
        }
    }
    /// The message after blur.
    pub fn error(&self) -> Option<String> {
        if !self.validate || !self.touched {
            return None;
        }
        match self.status() {
            PhoneStatus::Incomplete => {
                Some(format!("Too short for a {} number", self.country.name))
            }
            PhoneStatus::TooLong => Some(format!("Too long for a {} number", self.country.name)),
            _ => None,
        }
    }
    /// Typing in the number. A leading `+` opens the picker with a code search instead; a pasted
    /// international number switches the country. Returns `true` when the country changed.
    pub fn input(&mut self, text: &str) -> bool {
        if self.disabled {
            return false;
        }
        let t = text.trim();
        if t == "+" {
            self.open_picker("+");
            return false;
        }
        if let Some((c, d)) = parse(t) {
            let changed = c.iso != self.country.iso;
            self.set_country(c);
            self.digits = d;
            if changed {
                self.announce = format!("Country changed to {}", c.name);
            }
            return changed;
        }
        self.digits = t.chars().filter(char::is_ascii_digit).take(15).collect();
        if self.status() == PhoneStatus::Valid {
            self.announce = format!("Valid {} number", self.country.name);
        }
        false
    }
    /// Backspace over a separator deletes the digit beside it: removes the last digit.
    pub fn backspace(&mut self) {
        self.digits.pop();
    }
    /// Blur.
    pub fn blur(&mut self) {
        self.touched = true;
    }
    fn set_country(&mut self, c: &'static Country) {
        let a = COUNTRIES.iter().position(|x| x.iso == self.country.iso);
        let b = COUNTRIES.iter().position(|x| x.iso == c.iso);
        self.rolled_down = b >= a;
        self.country = c;
    }
    /// Opens the picker (optionally seeded with a search, e.g. a typed letter or `+`).
    pub fn open_picker(&mut self, seed: &str) {
        if self.disabled {
            return;
        }
        self.open = true;
        self.query = seed.into();
        self.active = self.first_row();
    }
    /// Closes the picker.
    pub fn close_picker(&mut self) {
        self.open = false;
        self.query.clear();
        self.active = None;
    }
    fn pool(&self) -> impl Iterator<Item = (usize, &'static Country)> + '_ {
        COUNTRIES
            .iter()
            .enumerate()
            .filter(|(_, c)| self.allowed.as_ref().is_none_or(|a| a.contains(&c.iso)))
    }
    /// Picker rows: Suggested, then All countries; a search shows one flat list.
    pub fn rows(&self) -> Vec<CountryRow> {
        let q = self.query.trim().to_lowercase();
        let mut out = vec![];
        if q.is_empty() {
            let pref: Vec<usize> = self
                .preferred
                .iter()
                .filter_map(|p| self.pool().find(|(_, c)| c.iso == *p).map(|(i, _)| i))
                .collect();
            if !pref.is_empty() {
                out.push(CountryRow::Group("Suggested"));
                out.extend(pref.into_iter().map(CountryRow::Country));
                out.push(CountryRow::Group("All countries"));
            }
            out.extend(self.pool().map(|(i, _)| CountryRow::Country(i)));
            return out;
        }
        let code = q.trim_start_matches('+');
        let hits: Vec<usize> = self
            .pool()
            .filter(|(_, c)| {
                if q.starts_with('+') {
                    c.code.starts_with(code)
                } else {
                    c.name.to_lowercase().starts_with(&q)
                        || c.name
                            .to_lowercase()
                            .split_whitespace()
                            .any(|w| w.starts_with(&q))
                        || c.iso.eq_ignore_ascii_case(&q)
                        || c.code == code
                }
            })
            .map(|(i, _)| i)
            .collect();
        if hits.is_empty() {
            return vec![CountryRow::Empty];
        }
        hits.into_iter().map(CountryRow::Country).collect()
    }
    fn first_row(&self) -> Option<usize> {
        let rows = self.rows();
        if self.query.is_empty()
            && let Some(p) = rows.iter().position(
                |r| matches!(r, CountryRow::Country(i) if COUNTRIES[*i].iso == self.country.iso),
            )
        {
            return Some(p);
        }
        rows.iter()
            .position(|r| matches!(r, CountryRow::Country(_)))
    }
    /// Search typing.
    pub fn search(&mut self, q: &str) {
        self.query = q.into();
        self.active = self.first_row();
        let n = self
            .rows()
            .iter()
            .filter(|r| matches!(r, CountryRow::Country(_)))
            .count();
        self.announce = format!("{n} result{}", if n == 1 { "" } else { "s" });
    }
    /// ↑ / ↓ / PageUp / PageDown (`by` rows) / Home / End in the picker.
    pub fn move_active(&mut self, by: isize) {
        let rows = self.rows();
        let idx: Vec<usize> = (0..rows.len())
            .filter(|&i| matches!(rows[i], CountryRow::Country(_)))
            .collect();
        if idx.is_empty() {
            return;
        }
        let pos = self
            .active
            .and_then(|a| idx.iter().position(|&x| x == a))
            .unwrap_or(0) as isize;
        let next = (pos + by).clamp(0, idx.len() as isize - 1) as usize;
        self.active = Some(idx[next]);
    }
    /// Picks the highlighted (or a clicked) row. Returns the new ISO code when it changed.
    pub fn pick(&mut self, row: Option<usize>) -> Option<&'static str> {
        let rows = self.rows();
        let r = row.or(self.active)?;
        let CountryRow::Country(i) = rows.get(r)? else {
            return None;
        };
        let c = &COUNTRIES[*i];
        let changed = c.iso != self.country.iso;
        self.set_country(c);
        self.close_picker();
        self.announce = format!("{} +{}", c.name, c.code);
        changed.then_some(c.iso)
    }
    /// The country button's name: `Country, United Kingdom +44`.
    pub fn button_label(&self) -> String {
        format!("Country, {} +{}", self.country.name, self.country.code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_and_outputs_e164() {
        let mut p = PhoneInput::new("US");
        p.input("4155550132");
        assert_eq!(p.formatted(), "(415) 555-0132");
        assert_eq!(p.e164(), "+14155550132");
        assert_eq!(p.status(), PhoneStatus::Valid);
        assert_eq!(p.announce, "Valid United States number");
        let mut g = PhoneInput::new("GB");
        g.input("07400 123456");
        assert_eq!(g.formatted(), "0 7400 123456");
        assert_eq!(g.e164(), "+447400123456", "trunk dropped");
        assert_eq!(g.international(), "+44 7400 123456");
        assert_eq!(g.masked_text(), "•••• 3456");
        g.backspace();
        g.blur();
        assert_eq!(
            g.error().as_deref(),
            Some("Too short for a United Kingdom number")
        );
    }

    #[test]
    fn paste_switches_country() {
        let mut p = PhoneInput::new("US");
        assert!(p.input("+44 (0)7911 123456"));
        assert_eq!(p.country.iso, "GB");
        assert_eq!(p.e164(), "+447911123456");
        assert!(p.input("0092 301 2345678"));
        assert_eq!(p.country.iso, "PK");
        assert_eq!(parse("+1 604 555 0199").unwrap().0.iso, "CA");
        assert_eq!(parse("+1 415 555 0132").unwrap().0.iso, "US");
        let q = PhoneInput::from_e164("+919123456789", "US");
        assert_eq!(q.formatted(), "91234 56789");
    }

    #[test]
    fn picker_search_and_keys() {
        let mut p = PhoneInput::new("US");
        p.input("+");
        assert!(p.open && p.query == "+");
        p.search("+4");
        let rows = p.rows();
        assert!(
            rows.iter().all(
                |r| matches!(r, CountryRow::Country(i) if COUNTRIES[*i].code.starts_with('4'))
            )
        );
        p.search("united");
        assert_eq!(p.rows().len(), 3);
        p.move_active(1);
        assert_eq!(p.pick(None), Some("GB"));
        assert!(!p.rolled_down || p.country.iso == "GB");
        p.open_picker("");
        assert_eq!(p.rows()[0], CountryRow::Group("Suggested"));
        assert!(
            matches!(p.rows()[p.active.unwrap()], CountryRow::Country(i) if COUNTRIES[i].iso == "GB")
        );
        p.search("zz");
        assert_eq!(p.rows(), vec![CountryRow::Empty]);
        assert_eq!(flag_of("gb"), "🇬🇧");
        assert_eq!(country_from_locale("en-PK").unwrap().name, "Pakistan");
        assert_eq!(p.button_label(), "Country, United Kingdom +44");
    }
}
