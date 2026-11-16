//! Thinking Selector family, variant "Nozzle line" (an Oxyn original, drawn by the user): a row of
//! short, closely spaced vertical lines with a hand-drawn wobble; a transparent ring – the nozzle –
//! slides along it and snaps to the levels. Behaviour only; level logic is in [`super::thinkfid`].

/// One stroke of the line: its height (share of the full height) and tilt in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NozzleTick {
    /// 0.6–1.0 of the full stroke height.
    pub height: f32,
    /// −6°…6°, like a pen stroke.
    pub tilt: f32,
}

/// `count` strokes with a steady, hand-drawn wobble (the same for the same `seed`).
pub fn nozzle_ticks(count: usize, seed: u32) -> Vec<NozzleTick> {
    let mut s = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453) | 1;
    let mut next = move || {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        (s % 10_000) as f32 / 10_000.0
    };
    (0..count)
        .map(|_| NozzleTick {
            height: 0.6 + next() * 0.4,
            tilt: next() * 12.0 - 6.0,
        })
        .collect()
}

/// The stroke a level sits on: level 0 on the first, the last level on the last, the rest spread
/// evenly between.
pub fn level_tick(level: usize, levels: usize, ticks: usize) -> usize {
    if levels <= 1 || ticks <= 1 {
        return 0;
    }
    let l = level.min(levels - 1);
    ((l as f32 / (levels - 1) as f32) * (ticks - 1) as f32).round() as usize
}

/// Whether stroke `t` is a level stop (it is drawn a touch taller and bolder).
pub fn is_stop(t: usize, levels: usize, ticks: usize) -> bool {
    (0..levels).any(|l| level_tick(l, levels, ticks) == t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strokes_and_stops() {
        let a = nozzle_ticks(30, 3);
        assert_eq!(a.len(), 30);
        assert_eq!(a, nozzle_ticks(30, 3));
        assert!(
            a.iter()
                .all(|t| (0.6..=1.0).contains(&t.height) && (-6.0..=6.0).contains(&t.tilt))
        );
        assert_eq!(level_tick(0, 5, 41), 0);
        assert_eq!(level_tick(4, 5, 41), 40);
        assert_eq!(level_tick(2, 5, 41), 20);
        assert!(is_stop(10, 5, 41));
        assert!(!is_stop(11, 5, 41));
        assert_eq!(level_tick(3, 1, 10), 0);
    }
}
