//! Where a floating surface goes: tooltip, popover, dropdown menu, select list, split-button menu.
//! Side + align + offset against an anchor rect, flipped to the opposite side when it would leave
//! the viewport, then shifted along the cross axis to stay inside (collision padding).

/// A rectangle in host pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Left.
    pub x: f32,
    /// Top.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

impl Rect {
    /// A rect.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
    /// Right edge.
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    /// Bottom edge.
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

/// Which side of the anchor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Side {
    /// Above.
    Top,
    /// Below.
    #[default]
    Bottom,
    /// Left.
    Left,
    /// Right.
    Right,
}

impl Side {
    /// The other side.
    pub fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
    /// `top`, `bottom`, `left`, `right`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
    /// Parses an id.
    pub fn from_id(s: &str) -> Option<Self> {
        Some(match s {
            "top" => Self::Top,
            "bottom" => Self::Bottom,
            "left" => Self::Left,
            "right" => Self::Right,
            _ => return None,
        })
    }
    fn vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

/// Alignment along the anchor's edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Align {
    /// Start edges line up.
    #[default]
    Start,
    /// Centred.
    Center,
    /// End edges line up.
    End,
}

impl Align {
    /// `start`, `center`, `end`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
    /// Parses an id.
    pub fn from_id(s: &str) -> Option<Self> {
        Some(match s {
            "start" => Self::Start,
            "center" => Self::Center,
            "end" => Self::End,
            _ => return None,
        })
    }
}

/// The result: where to draw, the side actually used (after a flip) and the transform origin the
/// enter animation scales from (the anchored corner).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    /// Final rect.
    pub rect: Rect,
    /// Side after any flip.
    pub side: Side,
    /// True when the preferred side did not fit.
    pub flipped: bool,
    /// Transform origin as fractions of the floating rect (0,0 = top-left).
    pub origin: (f32, f32),
    /// Where an arrow would point, along the cross axis, in floating-rect pixels.
    pub arrow: f32,
}

/// Places a `size` (w, h) surface against `anchor` inside `viewport`.
pub fn place(
    anchor: Rect,
    size: (f32, f32),
    viewport: Rect,
    side: Side,
    align: Align,
    offset: f32,
    padding: f32,
) -> Placed {
    let (w, h) = size;
    let fits = |s: Side| match s {
        Side::Bottom => anchor.bottom() + offset + h <= viewport.bottom() - padding,
        Side::Top => anchor.y - offset - h >= viewport.y + padding,
        Side::Right => anchor.right() + offset + w <= viewport.right() - padding,
        Side::Left => anchor.x - offset - w >= viewport.x + padding,
    };
    let (side_used, flipped) = if fits(side) || !fits(side.opposite()) {
        (side, false)
    } else {
        (side.opposite(), true)
    };
    let (mut x, mut y);
    if side_used.vertical() {
        y = match side_used {
            Side::Bottom => anchor.bottom() + offset,
            _ => anchor.y - offset - h,
        };
        x = match align {
            Align::Start => anchor.x,
            Align::Center => anchor.x + (anchor.w - w) / 2.0,
            Align::End => anchor.right() - w,
        };
        x = x.clamp(
            viewport.x + padding,
            (viewport.right() - padding - w).max(viewport.x),
        );
    } else {
        x = match side_used {
            Side::Right => anchor.right() + offset,
            _ => anchor.x - offset - w,
        };
        y = match align {
            Align::Start => anchor.y,
            Align::Center => anchor.y + (anchor.h - h) / 2.0,
            Align::End => anchor.bottom() - h,
        };
        y = y.clamp(
            viewport.y + padding,
            (viewport.bottom() - padding - h).max(viewport.y),
        );
    }
    let along = match align {
        Align::Start => 0.0,
        Align::Center => 0.5,
        Align::End => 1.0,
    };
    let origin = match side_used {
        Side::Bottom => (along, 0.0),
        Side::Top => (along, 1.0),
        Side::Right => (0.0, along),
        Side::Left => (1.0, along),
    };
    let arrow = if side_used.vertical() {
        (anchor.x + anchor.w / 2.0 - x).clamp(8.0, (w - 8.0).max(8.0))
    } else {
        (anchor.y + anchor.h / 2.0 - y).clamp(8.0, (h - 8.0).max(8.0))
    };
    Placed {
        rect: Rect::new(x, y, w, h),
        side: side_used,
        flipped,
        origin,
        arrow,
    }
}

/// The few pixels a surface drifts in from, towards its anchor (tooltip 3 px, popover 5 px).
pub fn drift(side: Side, px: f32) -> (f32, f32) {
    match side {
        Side::Top => (0.0, px),
        Side::Bottom => (0.0, -px),
        Side::Left => (px, 0.0),
        Side::Right => (-px, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VP: Rect = Rect::new(0.0, 0.0, 400.0, 300.0);

    #[test]
    fn below_start_by_default() {
        let p = place(
            Rect::new(20.0, 20.0, 80.0, 30.0),
            (160.0, 100.0),
            VP,
            Side::Bottom,
            Align::Start,
            6.0,
            8.0,
        );
        assert_eq!(p.rect, Rect::new(20.0, 56.0, 160.0, 100.0));
        assert!(!p.flipped);
        assert_eq!(p.origin, (0.0, 0.0));
    }

    #[test]
    fn flips_and_clamps() {
        // Near the bottom: flips to the top and scales from the bottom edge.
        let p = place(
            Rect::new(350.0, 260.0, 40.0, 24.0),
            (160.0, 100.0),
            VP,
            Side::Bottom,
            Align::Start,
            6.0,
            8.0,
        );
        assert!(p.flipped);
        assert_eq!(p.side, Side::Top);
        assert_eq!(p.rect.y, 260.0 - 6.0 - 100.0);
        // Clamped so it never leaves the right edge (collision padding 8).
        assert_eq!(p.rect.x, 400.0 - 8.0 - 160.0);
        assert_eq!(p.origin.1, 1.0);
        // Neither side fits: keep the preferred one.
        let p = place(
            Rect::new(10.0, 100.0, 10.0, 10.0),
            (50.0, 290.0),
            VP,
            Side::Top,
            Align::Center,
            6.0,
            8.0,
        );
        assert_eq!(p.side, Side::Top);
        assert!(!p.flipped);
    }

    #[test]
    fn side_placement_and_arrow() {
        let p = place(
            Rect::new(100.0, 100.0, 40.0, 40.0),
            (120.0, 60.0),
            VP,
            Side::Right,
            Align::Center,
            6.0,
            8.0,
        );
        assert_eq!(p.rect.x, 146.0);
        assert_eq!(p.rect.y, 90.0);
        assert_eq!(p.arrow, 30.0);
        assert_eq!(drift(Side::Top, 3.0), (0.0, 3.0));
        assert_eq!(Side::from_id("left"), Some(Side::Left));
    }
}
