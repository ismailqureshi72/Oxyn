//! Menu Motion family (variants 2): the "gooey" menu motion – a menu that grows out of its button
//! like a drop of liquid (a stem stretches from the button, the panel swells from the nearest
//! corner, rows arrive top to bottom, a grey fill glides under the pointer, and closing runs it
//! back fast). Shared by the Gooey attach menu, Gooey model menu and Gooey composer, and by two
//! Oxyn originals, Liquid tabs and Liquid drops. Behaviour only: timings were tuned by eye; item
//! names, hints and every line of copy are our own.

/// One row of a gooey menu.
#[derive(Clone, Debug, PartialEq)]
pub struct GooItem {
    /// Stable key.
    pub key: &'static str,
    /// Label.
    pub label: &'static str,
    /// Muted hint on the same line (empty for none).
    pub hint: &'static str,
    /// What draws the icon: a file name (file-type icon), `brand:<id>` (a vetted brand mark),
    /// or `glyph:<name>` (a line glyph).
    pub icon: &'static str,
    /// Group heading this row sits under (empty = none).
    pub group: &'static str,
}

const fn it(
    key: &'static str,
    label: &'static str,
    hint: &'static str,
    icon: &'static str,
    group: &'static str,
) -> GooItem {
    GooItem {
        key,
        label,
        hint,
        icon,
        group,
    }
}

/// The attach drop-up: files by type, then sources.
pub fn goo_attach_items() -> Vec<GooItem> {
    vec![
        it("pdf", "PDF", "Read and cite", "file.pdf", "Add"),
        it(
            "image",
            "Image",
            "PNG, JPG or a screenshot",
            "file.png",
            "Add",
        ),
        it("sheet", "Excel", "Sheets and CSV", "file.xlsx", "Add"),
        it("doc", "Doc", "Word documents", "file.docx", "Add"),
        it("slides", "Slides", "Decks to summarise", "file.pptx", "Add"),
        it("code", "Code", "A file from the repo", "file.rs", "Add"),
        it(
            "drive",
            "Drive",
            "Pick from your drive",
            "brand:googledrive",
            "From",
        ),
        it(
            "github",
            "GitHub",
            "Issues, PRs and files",
            "brand:github",
            "From",
        ),
    ]
}

/// The model menu (vetted brand marks; names are the makers' product families).
pub fn goo_models() -> Vec<GooItem> {
    vec![
        it("auto", "Auto", "Picks per task", "glyph:spark", ""),
        it(
            "claude",
            "Claude",
            "Careful writing and code",
            "brand:claude",
            "",
        ),
        it("gpt", "GPT", "Quick everyday answers", "brand:openai", ""),
        it(
            "gemini",
            "Gemini",
            "Long files and video",
            "brand:gemini",
            "",
        ),
        it(
            "deepseek",
            "DeepSeek",
            "Low cost reasoning",
            "brand:deepseek",
            "",
        ),
        it("qwen", "Qwen", "Many languages", "brand:qwen", ""),
        it("mistral", "Mistral", "Small and fast", "brand:mistral", ""),
    ]
}

/// Row height, group-heading height and the panel's inner padding, in px.
pub const ROW_H: f32 = 36.0;
/// Group heading height in px.
pub const GROUP_H: f32 = 26.0;
/// Panel padding in px.
pub const PAD: f32 = 6.0;

/// The top of each row inside the panel (px), headings included where the group changes.
pub fn row_tops(items: &[GooItem]) -> Vec<f32> {
    let mut y = PAD;
    let mut last = "";
    let mut out = Vec::with_capacity(items.len());
    for i in items {
        if !i.group.is_empty() && i.group != last {
            y += GROUP_H;
        }
        last = i.group;
        out.push(y);
        y += ROW_H;
    }
    out
}

/// The panel's full height (px) for these rows, capped at `max` (it scrolls past that).
pub fn panel_height(items: &[GooItem], max: f32) -> f32 {
    let tops = row_tops(items);
    let h = tops.last().map_or(PAD * 2.0, |t| t + ROW_H + PAD);
    h.min(max)
}

/// Delay (ms) before row `i` arrives: rows cascade top to bottom after the panel starts to swell.
pub fn stagger_ms(i: usize) -> u32 {
    70 + 24 * i.min(12) as u32
}

/// Which way a menu opens: wherever there is more space (`up` when equal).
pub fn open_side(room_above: f32, room_below: f32) -> &'static str {
    if room_below > room_above {
        "down"
    } else {
        "up"
    }
}

/// The next highlighted row for a key step (±1 wraps; `i64::MIN` / `i64::MAX` jump to the ends).
pub fn goo_move(cur: usize, step: i64, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    match step {
        i64::MIN => 0,
        i64::MAX => n - 1,
        s => (cur as i64 + s).rem_euclid(n as i64) as usize,
    }
}

/// The liquid tab pill: left edge and width in percent for tab `i` of `n`.
pub fn tab_box(i: usize, n: usize) -> (f32, f32) {
    let n = n.max(1) as f32;
    let w = 100.0 / n;
    (i.min(n as usize - 1) as f32 * w, w)
}

/// Where action drop `i` of `n` rests above the button (px, upward), `gap` apart.
pub fn drop_rise(i: usize, gap: f32) -> f32 {
    (i + 1) as f32 * gap
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout() {
        let a = goo_attach_items();
        let t = row_tops(&a);
        assert_eq!(t[0], PAD + GROUP_H);
        assert_eq!(t[1], PAD + GROUP_H + ROW_H);
        assert_eq!(t[6], PAD + 2.0 * GROUP_H + 6.0 * ROW_H);
        assert_eq!(
            panel_height(&a, 1000.0),
            PAD * 2.0 + 2.0 * GROUP_H + 8.0 * ROW_H
        );
        assert_eq!(panel_height(&a, 200.0), 200.0);
        let m = goo_models();
        assert_eq!(row_tops(&m)[0], PAD);
        assert_eq!(panel_height(&m, 1000.0), PAD * 2.0 + 7.0 * ROW_H);
    }

    #[test]
    fn motion_and_keys() {
        assert_eq!(stagger_ms(0), 70);
        assert_eq!(stagger_ms(3), 142);
        assert_eq!(open_side(300.0, 100.0), "up");
        assert_eq!(open_side(100.0, 300.0), "down");
        assert_eq!(goo_move(0, -1, 5), 4);
        assert_eq!(goo_move(4, 1, 5), 0);
        assert_eq!(goo_move(2, i64::MAX, 5), 4);
        assert_eq!(tab_box(2, 4), (50.0, 25.0));
        assert_eq!(drop_rise(2, 52.0), 156.0);
    }
}
