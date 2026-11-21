//! Thinking Selector family, variant "Effort stairs" (an Oxyn original): rising bars, one per
//! level, with a rough thinking budget and wait for the level under the pointer. Behaviour only;
//! level logic is in [`super::thinkfid`].

/// A rough thinking budget for level `i` of `n`: (thinking tokens, seconds of wait). Doubles per
/// step from 1k tokens / 2 s; the top level is open-ended (`None`).
pub fn stairs_budget(i: usize, n: usize) -> Option<(u32, u32)> {
    if n == 0 || i + 1 >= n {
        return None;
    }
    let k = 1u32 << i.min(16);
    Some((1_000 * k, 2 * k))
}

/// The budget as one line: "≈ 4k thinking tokens · ~8 s", or "No cap · as long as it needs".
pub fn stairs_line(i: usize, n: usize) -> String {
    match stairs_budget(i, n) {
        Some((t, s)) => format!("≈ {}k thinking tokens · ~{s} s", t / 1_000),
        None => "No cap · as long as it needs".into(),
    }
}

/// Bar height share for level `i` of `n` (0.28 … 1.0).
pub fn stair_height(i: usize, n: usize) -> f32 {
    if n <= 1 {
        return 1.0;
    }
    0.28 + 0.72 * i.min(n - 1) as f32 / (n - 1) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budgets() {
        assert_eq!(stairs_budget(0, 5), Some((1_000, 2)));
        assert_eq!(stairs_budget(3, 5), Some((8_000, 16)));
        assert_eq!(stairs_budget(4, 5), None);
        assert_eq!(stairs_line(2, 5), "≈ 4k thinking tokens · ~8 s");
        assert_eq!(stairs_line(4, 5), "No cap · as long as it needs");
        assert_eq!(stair_height(0, 5), 0.28);
        assert_eq!(stair_height(4, 5), 1.0);
    }
}
