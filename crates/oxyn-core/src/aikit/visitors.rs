//! Live visitors: a realtime card for a marketing site. A live status, a count that rolls digit
//! by digit, a dotted world map where arrivals ping in, the top pages with bars, and a ticker of
//! signups and upgrades with country flags. The sample is a simulated stream on a fixed tick.

use super::hash;

/// The stream moves every this many ms.
pub const TICK_MS: u64 = 1_600;
/// Map grid, columns × rows.
pub const MAP: (usize, usize) = (72, 30);
/// Latitudes the map shows (north, south).
pub const LATS: (f32, f32) = (78.0, -56.0);

/// Paths and their base audience.
pub const PAGES: [(&str, u32); 6] = [
    ("/", 34),
    ("/pricing", 22),
    ("/docs/quickstart", 17),
    ("/blog/offline-sync", 14),
    ("/changelog", 9),
    ("/customers", 7),
];

/// Cities: name, country code, latitude, longitude.
pub const CITIES: [(&str, &str, f32, f32); 12] = [
    ("Berlin", "DE", 52.5, 13.4),
    ("Paris", "FR", 48.9, 2.35),
    ("Milan", "IT", 45.5, 9.2),
    ("Amsterdam", "NL", 52.4, 4.9),
    ("Tokyo", "JP", 35.7, 139.7),
    ("Dublin", "IE", 53.3, -6.3),
    ("Bengaluru", "IN", 12.97, 77.6),
    ("Stockholm", "SE", 59.3, 18.1),
    ("Lahore", "PK", 31.5, 74.3),
    ("Munich", "DE", 48.1, 11.6),
    ("Lyon", "FR", 45.8, 4.8),
    ("Osaka", "JP", 34.7, 135.5),
];

/// Conversions.
pub const ACTIONS: [&str; 3] = ["Signed up", "Started a Pro trial", "Upgraded to Team"];

/// A flag drawn with stripes: `h` (horizontal), `v` (vertical), `dot` (a disc on white),
/// `cross` (a Nordic cross) or `crescent`; colours in order.
pub fn flag(code: &str) -> (&'static str, [u32; 3]) {
    match code {
        "DE" => ("h", [0x000000, 0xdd0000, 0xffce00]),
        "FR" => ("v", [0x0055a4, 0xffffff, 0xef4135]),
        "IT" => ("v", [0x009246, 0xffffff, 0xce2b37]),
        "NL" => ("h", [0xae1c28, 0xffffff, 0x21468b]),
        "JP" => ("dot", [0xffffff, 0xbc002d, 0xffffff]),
        "IE" => ("v", [0x169b62, 0xffffff, 0xff883e]),
        "IN" => ("h", [0xff9933, 0xffffff, 0x138808]),
        "SE" => ("cross", [0x006aa7, 0xfecc00, 0x006aa7]),
        "PK" => ("crescent", [0x01411c, 0xffffff, 0xffffff]),
        _ => ("h", [0x999999, 0xcccccc, 0x999999]),
    }
}

const LAND: [&[(f32, f32)]; 11] = [
    &[
        (-168.0, 66.0),
        (-140.0, 70.0),
        (-95.0, 72.0),
        (-80.0, 62.0),
        (-60.0, 55.0),
        (-52.0, 47.0),
        (-66.0, 44.0),
        (-75.0, 35.0),
        (-81.0, 25.0),
        (-97.0, 26.0),
        (-105.0, 20.0),
        (-92.0, 15.0),
        (-83.0, 9.0),
        (-79.0, 8.0),
        (-87.0, 13.0),
        (-105.0, 22.0),
        (-117.0, 32.0),
        (-124.0, 40.0),
        (-124.0, 48.0),
        (-135.0, 57.0),
        (-152.0, 58.0),
        (-165.0, 60.0),
    ],
    &[
        (-55.0, 82.0),
        (-25.0, 82.0),
        (-20.0, 70.0),
        (-40.0, 60.0),
        (-52.0, 64.0),
        (-60.0, 76.0),
    ],
    &[
        (-80.0, 10.0),
        (-62.0, 11.0),
        (-50.0, 0.0),
        (-35.0, -6.0),
        (-39.0, -15.0),
        (-48.0, -26.0),
        (-58.0, -38.0),
        (-66.0, -55.0),
        (-73.0, -50.0),
        (-71.0, -30.0),
        (-76.0, -15.0),
        (-81.0, -5.0),
    ],
    &[
        (-10.0, 36.0),
        (-9.0, 43.0),
        (-2.0, 44.0),
        (-5.0, 48.0),
        (3.0, 51.0),
        (8.0, 54.0),
        (5.0, 58.0),
        (10.0, 63.0),
        (20.0, 70.0),
        (30.0, 70.0),
        (40.0, 66.0),
        (45.0, 55.0),
        (40.0, 45.0),
        (28.0, 41.0),
        (22.0, 37.0),
        (15.0, 38.0),
        (12.0, 44.0),
        (3.0, 42.0),
    ],
    &[
        (-17.0, 21.0),
        (-10.0, 30.0),
        (-5.0, 36.0),
        (10.0, 37.0),
        (32.0, 31.0),
        (35.0, 28.0),
        (43.0, 12.0),
        (51.0, 12.0),
        (40.0, -2.0),
        (40.0, -15.0),
        (33.0, -27.0),
        (20.0, -35.0),
        (17.0, -28.0),
        (12.0, -18.0),
        (13.0, -5.0),
        (9.0, 4.0),
        (-8.0, 5.0),
        (-16.0, 12.0),
    ],
    &[
        (26.0, 40.0),
        (30.0, 46.0),
        (45.0, 55.0),
        (40.0, 66.0),
        (60.0, 70.0),
        (80.0, 73.0),
        (110.0, 76.0),
        (140.0, 72.0),
        (170.0, 68.0),
        (180.0, 66.0),
        (160.0, 60.0),
        (140.0, 55.0),
        (135.0, 43.0),
        (122.0, 40.0),
        (122.0, 30.0),
        (110.0, 20.0),
        (105.0, 10.0),
        (100.0, 5.0),
        (98.0, 15.0),
        (92.0, 22.0),
        (80.0, 8.0),
        (73.0, 20.0),
        (66.0, 25.0),
        (57.0, 25.0),
        (50.0, 30.0),
        (56.0, 18.0),
        (45.0, 12.0),
        (35.0, 30.0),
        (36.0, 36.0),
    ],
    &[
        (130.0, 31.0),
        (141.0, 35.0),
        (142.0, 45.0),
        (140.0, 41.0),
        (133.0, 34.0),
    ],
    &[
        (114.0, -22.0),
        (122.0, -17.0),
        (131.0, -11.0),
        (137.0, -12.0),
        (142.0, -11.0),
        (146.0, -19.0),
        (153.0, -26.0),
        (150.0, -37.0),
        (141.0, -38.0),
        (131.0, -31.0),
        (115.0, -34.0),
    ],
    &[(-6.0, 50.0), (2.0, 51.0), (-2.0, 58.0), (-6.0, 57.0)],
    &[
        (95.0, 5.0),
        (106.0, -6.0),
        (115.0, -8.0),
        (120.0, -5.0),
        (110.0, 2.0),
    ],
    &[
        (166.0, -46.0),
        (174.0, -37.0),
        (178.0, -38.0),
        (172.0, -45.0),
    ],
];

fn inside(p: (f32, f32), poly: &[(f32, f32)]) -> bool {
    let mut c = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[j]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1) + a.0 {
            c = !c;
        }
        j = i;
    }
    c
}

/// Map position 0..1 of a latitude and longitude.
pub fn project(lat: f32, lon: f32) -> (f32, f32) {
    ((lon + 180.0) / 360.0, (LATS.0 - lat) / (LATS.0 - LATS.1))
}

/// The land dots of the map, as grid cells (column, row).
pub fn land() -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for r in 0..MAP.1 {
        for c in 0..MAP.0 {
            let lon = (c as f32 + 0.5) / MAP.0 as f32 * 360.0 - 180.0;
            let lat = LATS.0 - (r as f32 + 0.5) / MAP.1 as f32 * (LATS.0 - LATS.1);
            if LAND.iter().any(|p| inside((lon, lat), p)) {
                out.push((c, r));
            }
        }
    }
    out
}

/// A conversion in the ticker.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// Tick it happened on (also its id).
    pub tick: u64,
    /// What happened.
    pub label: &'static str,
    /// City index.
    pub city: usize,
}

/// The live state of the card.
#[derive(Clone, Debug, PartialEq)]
pub struct Visitors {
    /// Stream start.
    pub t0: u64,
    /// Paused at.
    pub paused: Option<u64>,
    /// Ms spent paused.
    pub paused_ms: u64,
    /// `live`, `connecting` or `reconnecting`.
    pub connection: &'static str,
}

impl Visitors {
    /// A stream from `now`.
    pub fn new(connection: &'static str, paused: bool, now: u64) -> Self {
        Self {
            t0: now,
            paused: paused.then_some(now),
            paused_ms: 0,
            connection,
        }
    }

    /// The stream's tick.
    pub fn tick(&self, now: u64) -> u64 {
        let end = self.paused.unwrap_or(now);
        end.saturating_sub(self.t0).saturating_sub(self.paused_ms) / TICK_MS + 8
    }

    /// Ms into the current tick (for pings).
    pub fn into_tick(&self, now: u64) -> u64 {
        let end = self.paused.unwrap_or(now);
        end.saturating_sub(self.t0).saturating_sub(self.paused_ms) % TICK_MS
    }

    /// People on the site at a tick.
    pub fn count_at(k: u64) -> u64 {
        let wave = ((k as f64) * 0.35).sin() * 9.0;
        let j = (hash(&format!("c{k}")) % 7) as f64;
        (128.0 + wave + j) as u64
    }

    /// People on the site now.
    pub fn count(&self, now: u64) -> u64 {
        Self::count_at(self.tick(now))
    }

    /// Top pages, most visitors first.
    pub fn pages(&self, now: u64, max: usize) -> Vec<(&'static str, u64)> {
        let k = self.tick(now);
        let total = self.count(now);
        let mut v: Vec<(&'static str, u64)> = PAGES
            .iter()
            .enumerate()
            .map(|(i, (p, w))| {
                let j = (hash(&format!("p{i}-{}", k / 2)) % 9) as u64;
                (*p, (u64::from(*w) * total / 103 + j).max(1))
            })
            .collect();
        v.sort_by_key(|x| std::cmp::Reverse(x.1));
        v.truncate(max);
        v
    }

    /// Arrivals on the map, newest last: (city index, tick, is a conversion).
    pub fn pings(&self, now: u64) -> Vec<(usize, u64, bool)> {
        let k = self.tick(now);
        (k.saturating_sub(7)..=k)
            .map(|t| {
                let h = hash(&format!("v{t}"));
                (h as usize % CITIES.len(), t, t % 3 == 0)
            })
            .collect()
    }

    /// Conversions, newest first.
    pub fn events(&self, now: u64, max: usize) -> Vec<Event> {
        let k = self.tick(now);
        (0..=k)
            .rev()
            .filter(|t| t % 3 == 0)
            .take(max)
            .map(|t| {
                let h = hash(&format!("v{t}"));
                Event {
                    tick: t,
                    label: ACTIONS[(h / 7) as usize % 3],
                    city: h as usize % CITIES.len(),
                }
            })
            .collect()
    }

    /// `just now`, `2 min ago` for an event tick.
    pub fn ago(&self, tick: u64, now: u64) -> String {
        let d = self.tick(now).saturating_sub(tick) * TICK_MS / 1000 * 20;
        if d < 30 {
            "just now".into()
        } else {
            format!("{} min ago", d.div_ceil(60))
        }
    }

    /// Presses: `lv-pause` (toggles).
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if id != "lv-pause" {
            return None;
        }
        if let Some(p) = self.paused.take() {
            self.paused_ms += now.saturating_sub(p);
            Some("Live updates resumed".into())
        } else {
            self.paused = Some(now);
            Some("Live updates paused".into())
        }
    }

    /// The status word.
    pub fn status(&self) -> &'static str {
        if self.paused.is_some() {
            "Paused"
        } else {
            match self.connection {
                "connecting" => "Connecting",
                "reconnecting" => "Reconnecting",
                _ => "Live",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_map_and_pause() {
        let mut v = Visitors::new("live", false, 0);
        let l = land();
        assert!(l.len() > 300 && l.len() < MAP.0 * MAP.1);
        let berlin = project(52.5, 13.4);
        assert!(berlin.0 > 0.5 && berlin.1 < 0.3);
        assert!(v.count(0) > 100);
        let p = v.pages(0, 4);
        assert_eq!(p.len(), 4);
        assert!(p[0].1 >= p[1].1);
        assert!(!v.events(0, 4).is_empty());
        assert_eq!(v.pings(0).len(), 8);
        v.press("lv-pause", 1_000);
        assert_eq!(v.tick(1_000), v.tick(50_000));
        assert_eq!(v.status(), "Paused");
        v.press("lv-pause", 50_000);
        assert!(v.tick(52_000) > v.tick(50_000));
        assert_eq!(flag("JP").0, "dot");
    }
}
