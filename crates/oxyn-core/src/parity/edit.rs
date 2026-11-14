//! A single-line text buffer with a selection and an IME composition (marked) range, shared by
//! the GPUI `ImeInput` and any host that needs the platform input-handler contract. Offsets are
//! UTF-8 byte offsets internally; the `*_utf16` helpers convert for platform IME APIs, which speak
//! UTF-16. Cursor moves step over whole chars (grapheme clusters are the host's refinement).

use std::ops::Range;

/// Text, selection and composition.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextBuffer {
    /// Content.
    pub text: String,
    /// Selected byte range (empty = caret).
    pub selected: Range<usize>,
    /// The caret sits at the start of the selection.
    pub reversed: bool,
    /// IME composition range (underlined while composing).
    pub marked: Option<Range<usize>>,
}

impl TextBuffer {
    /// A buffer with the caret at the end.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.into(),
            selected: text.len()..text.len(),
            reversed: false,
            marked: None,
        }
    }
    /// Caret byte offset.
    pub fn cursor(&self) -> usize {
        if self.reversed {
            self.selected.start
        } else {
            self.selected.end
        }
    }
    /// UTF-16 offset → byte offset.
    pub fn from_utf16(&self, off: usize) -> usize {
        let (mut b, mut u) = (0, 0);
        for ch in self.text.chars() {
            if u >= off {
                break;
            }
            u += ch.len_utf16();
            b += ch.len_utf8();
        }
        b
    }
    /// Byte offset → UTF-16 offset.
    pub fn to_utf16(&self, off: usize) -> usize {
        let (mut b, mut u) = (0, 0);
        for ch in self.text.chars() {
            if b >= off {
                break;
            }
            b += ch.len_utf8();
            u += ch.len_utf16();
        }
        u
    }
    /// Range in UTF-16.
    pub fn range_to_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.to_utf16(r.start)..self.to_utf16(r.end)
    }
    /// Range from UTF-16.
    pub fn range_from_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.from_utf16(r.start)..self.from_utf16(r.end)
    }
    /// Length in UTF-16 code units.
    pub fn len_utf16(&self) -> usize {
        self.text.encode_utf16().count()
    }
    /// Previous char boundary.
    pub fn prev_boundary(&self, off: usize) -> usize {
        self.text[..off.min(self.text.len())]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
    /// Next char boundary.
    pub fn next_boundary(&self, off: usize) -> usize {
        self.text[off.min(self.text.len())..]
            .chars()
            .next()
            .map(|c| off + c.len_utf8())
            .unwrap_or(self.text.len())
    }
    /// Previous word start (⌥←).
    pub fn prev_word(&self, off: usize) -> usize {
        let s = &self.text[..off];
        let t = s.trim_end();
        t.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0)
    }
    /// Next word end (⌥→).
    pub fn next_word(&self, off: usize) -> usize {
        let s = &self.text[off..];
        let lead = s.len() - s.trim_start().len();
        let rest = &s[lead..];
        off + lead + rest.find(char::is_whitespace).unwrap_or(rest.len())
    }
    /// Moves the caret (collapses the selection).
    pub fn move_to(&mut self, off: usize) {
        let off = off.min(self.text.len());
        self.selected = off..off;
        self.reversed = false;
    }
    /// Extends the selection to `off`.
    pub fn select_to(&mut self, off: usize) {
        let off = off.min(self.text.len());
        if self.reversed {
            self.selected.start = off;
        } else {
            self.selected.end = off;
        }
        if self.selected.end < self.selected.start {
            self.reversed = !self.reversed;
            self.selected = self.selected.end..self.selected.start;
        }
    }
    /// Left arrow.
    pub fn left(&mut self) {
        if self.selected.is_empty() {
            self.move_to(self.prev_boundary(self.cursor()));
        } else {
            self.move_to(self.selected.start);
        }
    }
    /// Right arrow.
    pub fn right(&mut self) {
        if self.selected.is_empty() {
            self.move_to(self.next_boundary(self.selected.end));
        } else {
            self.move_to(self.selected.end);
        }
    }
    /// Select all.
    pub fn select_all(&mut self) {
        self.selected = 0..self.text.len();
        self.reversed = false;
    }
    /// Selected text.
    pub fn selection_text(&self) -> &str {
        &self.text[self.selected.clone()]
    }
    /// Replaces `range` (or the marked range, or the selection) with `new` and ends composition.
    pub fn replace(&mut self, range: Option<Range<usize>>, new: &str) {
        let r = range
            .or(self.marked.clone())
            .unwrap_or(self.selected.clone());
        let r = r.start.min(self.text.len())..r.end.min(self.text.len());
        self.text.replace_range(r.clone(), new);
        let c = r.start + new.len();
        self.selected = c..c;
        self.reversed = false;
        self.marked = None;
    }
    /// IME composition: replaces and marks `new`; `sel` is the selection inside `new` in UTF-16.
    pub fn replace_and_mark(
        &mut self,
        range: Option<Range<usize>>,
        new: &str,
        sel: Option<Range<usize>>,
    ) {
        let r = range
            .or(self.marked.clone())
            .unwrap_or(self.selected.clone());
        let r = r.start.min(self.text.len())..r.end.min(self.text.len());
        self.text.replace_range(r.clone(), new);
        self.marked = (!new.is_empty()).then(|| r.start..r.start + new.len());
        let start16 = self.to_utf16(r.start);
        let n16 = new.encode_utf16().count();
        self.selected = match sel {
            Some(s) => {
                self.range_from_utf16(&(start16 + s.start.min(n16)..start16 + s.end.min(n16)))
            }
            None => r.start + new.len()..r.start + new.len(),
        };
        self.reversed = false;
    }
    /// Ends composition, keeping the text.
    pub fn unmark(&mut self) {
        self.marked = None;
    }
    /// Backspace.
    pub fn backspace(&mut self) -> bool {
        if self.selected.is_empty() {
            let p = self.prev_boundary(self.cursor());
            if p == self.cursor() {
                return false;
            }
            self.select_to(p);
        }
        self.replace(None, "");
        true
    }
    /// Delete forward.
    pub fn delete(&mut self) -> bool {
        if self.selected.is_empty() {
            let n = self.next_boundary(self.cursor());
            if n == self.cursor() {
                return false;
            }
            self.select_to(n);
        }
        self.replace(None, "");
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_and_compose() {
        let mut b = TextBuffer::new("héllo");
        assert_eq!(b.len_utf16(), 5);
        b.left();
        b.backspace();
        assert_eq!(b.text, "hélo");
        b.move_to(0);
        b.right();
        b.right();
        assert_eq!(b.cursor(), 3);
        // compose "にほん" then commit "日本"
        b.move_to(b.text.len());
        b.replace_and_mark(None, "にほ", None);
        assert_eq!(b.marked, Some(5..11));
        b.replace_and_mark(None, "にほん", Some(3..3));
        assert_eq!(b.text, "héloにほん");
        b.replace(None, "日本");
        assert_eq!(b.text, "hélo日本");
        assert!(b.marked.is_none());
        assert_eq!(b.to_utf16(b.text.len()), 6);
        b.select_to(0);
        assert!(b.reversed);
        assert_eq!(b.selection_text(), "hélo日本");
        assert_eq!(TextBuffer::new("ab cd ef").prev_word(5), 3);
        assert_eq!(TextBuffer::new("ab cd ef").next_word(2), 5);
    }
}
