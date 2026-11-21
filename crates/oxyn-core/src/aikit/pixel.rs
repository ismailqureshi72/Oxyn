//! Pixel mascot: Oxyn's pixel robot on a 24 × 24 grid. Every status is a sprite animation on a
//! fixed step clock ([`STEP`] ms, whole pixels only): a bob and a blink at rest, a thought bubble
//! of dots while thinking, a keyboard while writing, raised arms and stars when done, crossed
//! eyes and a shake on an error. A status change closes the eyes for two steps first, so it
//! never cuts.

use super::mascot::{Ink, Mascot, Mood};

/// One animation step, ms.
pub const STEP: u64 = 120;
/// The statuses the robot loops through when left alone.
pub const CYCLE: [Mood; 5] = [
    Mood::Idle,
    Mood::Thinking,
    Mood::Writing,
    Mood::Done,
    Mood::Error,
];

/// The robot at rest (B body, L light, S shade, F visor, M antenna bulb).
const BASE: [&str; 22] = [
    "........................",
    "...........MM...........",
    "...........MM...........",
    "............S...........",
    "......BBBBBBBBBBBB......",
    ".....BLLLBBBBBBBBBB.....",
    ".....BLBBBBBBBBBBBS.....",
    ".....BFFFFFFFFFFFFS.....",
    ".....BFFFFFFFFFFFFS.....",
    ".....BFFFFFFFFFFFFS.....",
    ".....BFFFFFFFFFFFFS.....",
    ".....BFFFFFFFFFFFFS.....",
    ".....BBBBBBBBBBBBBS.....",
    "......SSSSSSSSSSSS......",
    ".........BBBBBB.........",
    "........BBBBBBBBS.......",
    "........BLLBBBBBS.......",
    "........BBBBBBBBS.......",
    "........BBBBBBBBS.......",
    ".........BBBBBBS........",
    ".........SS..SS.........",
    ".........SS..SS.........",
];

/// A lit cell: column, row and ink.
pub type Cell = (i32, i32, Ink);

fn ink(c: char) -> Option<Ink> {
    match c {
        'B' => Some(Ink::Body),
        'L' => Some(Ink::Light),
        'S' => Some(Ink::Shade),
        'F' => Some(Ink::Face),
        'M' => Some(Ink::Mark),
        _ => None,
    }
}

/// The lit cells at `now` (later cells draw over earlier ones).
pub fn cells(m: &Mascot, now: u64) -> Vec<Cell> {
    let mood = m.mood(now);
    let (_, el) = m.change(now);
    let f = (m.paused.unwrap_or(now) / STEP) as i32;
    let step = (el / STEP) as i32;
    let mut out: Vec<Cell> = Vec::with_capacity(400);
    let bob = match mood {
        Mood::Idle | Mood::Writing => i32::from((f / 8) % 2 == 1),
        Mood::Done => -i32::from((f / 3) % 2 == 0),
        _ => 0,
    };
    let shake = if mood == Mood::Error && step < 8 {
        [1, -1, 1, -1, 1, 0, -1, 0][step as usize]
    } else {
        0
    };
    // Ground shadow does not bob.
    for x in 7..17 {
        out.push((x, 22, Ink::Ground));
    }
    let antenna_lit = match mood {
        Mood::Thinking => (f / 3) % 2 == 0,
        Mood::Error => true,
        _ => true,
    };
    for (y, row) in BASE.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if let Some(mut k) = ink(c) {
                if k == Ink::Mark {
                    if mood == Mood::Error {
                        k = Ink::Danger;
                    } else if !antenna_lit {
                        k = Ink::Shade;
                    }
                }
                out.push((x as i32 + shake, y as i32 + bob, k));
            }
        }
    }
    let px = |out: &mut Vec<Cell>, x: i32, y: i32, k: Ink| out.push((x + shake, y + bob, k));
    // Arms.
    match mood {
        Mood::Done => {
            for (x, y) in [(7, 14), (6, 13), (5, 12), (16, 14), (17, 13), (18, 12)] {
                px(&mut out, x, y, Ink::Body);
            }
        }
        Mood::Writing => {
            let tap = (f / 2) % 2;
            for (x, y) in [
                (7, 16),
                (7, 17),
                (8, 19 - tap),
                (16, 16),
                (16, 17),
                (15, 18 + tap),
            ] {
                px(&mut out, x, y, Ink::Body);
            }
        }
        _ => {
            for (x, y) in [(7, 15), (7, 16), (7, 17), (16, 15), (16, 16), (16, 17)] {
                px(&mut out, x, y, Ink::Shade);
            }
        }
    }
    // Eyes on the visor: two steps shut after a change, then the status's own eyes.
    let blink = (f % 30) == 0 && mood == Mood::Idle;
    let look = match mood {
        Mood::Idle => {
            if (f / 20) % 4 == 2 {
                1
            } else {
                0
            }
        }
        Mood::Thinking => 1,
        _ => 0,
    };
    let eye_y = match mood {
        Mood::Thinking => 8,
        Mood::Writing => 10,
        _ => 9,
    };
    if step < 2 || blink {
        for x in [8, 9, 14, 15] {
            px(&mut out, x + look, 10, Ink::Light);
        }
    } else {
        match mood {
            Mood::Done => {
                for (x, y) in [(7, 10), (8, 9), (9, 10), (13, 10), (14, 9), (15, 10)] {
                    px(&mut out, x + 1, y, Ink::Light);
                }
            }
            Mood::Error => {
                for (x, y) in [
                    (8, 8),
                    (10, 10),
                    (9, 9),
                    (8, 10),
                    (10, 8),
                    (13, 8),
                    (15, 10),
                    (14, 9),
                    (13, 10),
                    (15, 8),
                ] {
                    px(&mut out, x, y, Ink::Danger);
                }
            }
            _ => {
                for (x, y) in [
                    (8, 0),
                    (9, 0),
                    (8, 1),
                    (9, 1),
                    (14, 0),
                    (15, 0),
                    (14, 1),
                    (15, 1),
                ] {
                    px(&mut out, x + look, eye_y + y, Ink::Light);
                }
            }
        }
    }
    // Accessories.
    match mood {
        Mood::Thinking => {
            let n = (f / 4) % 4;
            let dots = [(19, 4), (20, 2), (22, 0)];
            for (i, (x, y)) in dots.iter().enumerate() {
                if (i as i32) < n {
                    out.push((*x + shake, *y, Ink::Mark));
                    if i == 2 {
                        out.push((*x - 1 + shake, *y, Ink::Mark));
                        out.push((*x - 1 + shake, *y + 1, Ink::Mark));
                        out.push((*x + shake, *y + 1, Ink::Mark));
                    }
                }
            }
        }
        Mood::Writing => {
            for x in 6..18 {
                out.push((x, 21, Ink::Shade));
            }
            let lit = 6 + (f % 12);
            out.push((lit, 21, Ink::Mark));
            for x in (7..17).step_by(2) {
                out.push((x, 20, Ink::Ground));
            }
        }
        Mood::Done => {
            let tw = (f / 3) % 2 == 0;
            let stars: &[(i32, i32)] = if tw {
                &[(3, 5), (20, 3), (21, 9)]
            } else {
                &[(2, 8), (20, 6), (19, 1)]
            };
            for (x, y) in stars {
                out.push((*x, *y, Ink::Mark));
                out.push((*x - 1, *y, Ink::Light));
                out.push((*x + 1, *y, Ink::Light));
                out.push((*x, *y - 1, Ink::Light));
                out.push((*x, *y + 1, Ink::Light));
            }
        }
        _ => {}
    }
    out.retain(|c| (0..24).contains(&c.0) && (0..24).contains(&c.1));
    out
}

/// Cells merged into horizontal runs per ink (`x`, `y`, `len`, ink), for fewer rectangles.
pub fn runs(cells: &[Cell]) -> Vec<(i32, i32, i32, Ink)> {
    let mut grid = [[None::<Ink>; 24]; 24];
    for &(x, y, k) in cells {
        grid[y as usize][x as usize] = Some(k);
    }
    let mut out = Vec::new();
    for (y, row) in grid.iter().enumerate() {
        let mut x = 0;
        while x < 24 {
            if let Some(k) = row[x] {
                let s = x;
                while x < 24 && row[x] == Some(k) {
                    x += 1;
                }
                out.push((s as i32, y as i32, (x - s) as i32, k));
            } else {
                x += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aikit::mascot::STEP_MS;

    #[test]
    fn pixel_sprites() {
        assert!(BASE.iter().all(|r| r.len() == 24));
        let m = Mascot::new(None, &CYCLE, 0);
        let idle = cells(&m, 500);
        let think = cells(&m, STEP_MS + 600);
        assert_ne!(idle, think);
        assert!(
            cells(&m, STEP_MS * 4 + 600)
                .iter()
                .any(|c| c.2 == Ink::Danger)
        );
        let r = runs(&idle);
        assert!(r.len() < idle.len());
    }
}
