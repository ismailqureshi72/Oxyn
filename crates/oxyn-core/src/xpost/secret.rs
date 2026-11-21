//! Secret card: an API key the agent may use but never sees. The value is masked down to its last
//! four characters; the agent and the chat only get a handle (`{{secrets.NAME}}`). Revealing needs
//! a confirm and re-masks itself after a few seconds; rotating swaps the tail after a short
//! progress; revoking pauses every routine that used it (with Undo for a few seconds).

/// The sample secret: (name, brand mark, prefix, tail).
pub const SECRET: (&str, &str, &str, &str) = ("STRIPE_SECRET_KEY", "stripe", "sk_live_", "4f2a");
/// A tail after rotation.
pub const ROTATED: &str = "9c1e";
/// Scopes granted.
pub const SCOPES: [&str; 3] = ["Read charges", "Create refunds", "Read customers"];
/// Routines that use it: (name, minutes since last use).
pub const USED_BY: [(&str, u64); 2] = [("Refund routine", 6), ("Weekly revenue report", 1_440)];
/// Ms a reveal lasts.
pub const REVEAL_MS: u64 = 5_000;
/// Ms a rotation takes.
pub const ROTATE_MS: u64 = 1_600;
/// Ms Undo stays after revoking.
pub const UNDO_MS: u64 = 5_000;

/// What is waiting for a confirm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ask {
    /// Nothing.
    None,
    /// Reveal the value.
    Reveal,
    /// Rotate the key.
    Rotate,
    /// Revoke the key.
    Revoke,
}

/// The card.
#[derive(Clone, Debug, PartialEq)]
pub struct Secret {
    /// Pending confirm.
    pub ask: Ask,
    /// Revealed at.
    pub revealed: Option<u64>,
    /// Rotation started at.
    pub rotating: Option<u64>,
    /// Rotations done.
    pub rotations: u32,
    /// Revoked at.
    pub revoked: Option<u64>,
    /// Handle copied at.
    pub copied: Option<u64>,
}

impl Secret {
    /// A card; `revoked` starts it revoked.
    pub fn new(revoked: bool) -> Self {
        Self {
            ask: Ask::None,
            revealed: None,
            rotating: None,
            rotations: 0,
            revoked: if revoked { Some(0) } else { None },
            copied: None,
        }
    }

    /// The handle the agent gets.
    pub fn handle() -> String {
        format!("{{{{secrets.{}}}}}", SECRET.0)
    }

    /// Rotation in progress (0..1).
    pub fn rotation(&self, now: u64) -> Option<f64> {
        let r = self.rotating?;
        let e = now.saturating_sub(r);
        (e < ROTATE_MS).then(|| e as f64 / ROTATE_MS as f64)
    }

    /// The current tail.
    pub fn tail(&self, now: u64) -> &'static str {
        let done =
            self.rotations + u32::from(self.rotating.is_some() && self.rotation(now).is_none());
        if done % 2 == 1 { ROTATED } else { SECRET.3 }
    }

    /// Shown in full right now.
    pub fn shown(&self, now: u64) -> bool {
        self.revoked.is_none() && self.revealed.is_some_and(|r| now < r + REVEAL_MS)
    }

    /// Seconds left on a reveal.
    pub fn reveal_left(&self, now: u64) -> u64 {
        self.revealed
            .map(|r| (r + REVEAL_MS).saturating_sub(now).div_ceil(1000))
            .unwrap_or(0)
    }

    /// The value as displayed.
    pub fn value(&self, now: u64) -> String {
        if self.shown(now) {
            format!("{}51Hq8Lm2Xv7Rk0Tz{}", SECRET.2, self.tail(now))
        } else {
            format!("{}{}{}", SECRET.2, "•".repeat(12), self.tail(now))
        }
    }

    /// Revoked (Undo window included).
    pub fn is_revoked(&self) -> bool {
        self.revoked.is_some()
    }

    /// Undo is still offered.
    pub fn can_undo(&self, now: u64) -> bool {
        self.revoked
            .is_some_and(|r| r > 0 && now.saturating_sub(r) < UNDO_MS)
    }

    /// Status line.
    pub fn status(&self, now: u64) -> String {
        if self.is_revoked() {
            return format!("Revoked · {} routines paused", USED_BY.len());
        }
        if self.rotation(now).is_some() {
            return "Rotating · routines keep the old key until it finishes".into();
        }
        if self.shown(now) {
            return format!("Visible to you only · hides in {}s", self.reveal_left(now));
        }
        "Stays out of the chat · the agent gets a handle".into()
    }

    /// Handles a press id.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "sc-reveal" if !self.is_revoked() => {
                if self.shown(now) {
                    self.revealed = None;
                    return Some("Hidden".into());
                }
                self.ask = Ask::Reveal;
                Some("Confirm to show the key".into())
            }
            "sc-rotate" if !self.is_revoked() && self.rotation(now).is_none() => {
                self.ask = Ask::Rotate;
                Some("Confirm to rotate the key".into())
            }
            "sc-revoke" if !self.is_revoked() => {
                self.ask = Ask::Revoke;
                Some("Confirm to revoke the key".into())
            }
            "sc-cancel" => {
                self.ask = Ask::None;
                Some("Cancelled".into())
            }
            "sc-confirm" => {
                let a = self.ask;
                self.ask = Ask::None;
                match a {
                    Ask::Reveal => {
                        self.revealed = Some(now);
                        Some("Key shown for 5 seconds".into())
                    }
                    Ask::Rotate => {
                        if self.rotating.is_some() {
                            self.rotations += 1;
                        }
                        self.rotating = Some(now);
                        self.revealed = None;
                        Some("Rotating the key".into())
                    }
                    Ask::Revoke => {
                        self.revoked = Some(now.max(1));
                        self.revealed = None;
                        Some("Key revoked. Undo is available for 5 seconds".into())
                    }
                    Ask::None => None,
                }
            }
            "sc-undo" if self.can_undo(now) => {
                self.revoked = None;
                Some("Key restored".into())
            }
            "sc-restore" if self.is_revoked() => {
                self.revoked = None;
                Some("Key restored".into())
            }
            "sc-copy" => {
                self.copied = Some(now);
                Some(format!("Copied {}", Self::handle()))
            }
            _ => None,
        }
    }

    /// Escape cancels a confirm or hides the value; Enter confirms.
    pub fn key(&mut self, k: &str, now: u64) -> Option<String> {
        match k {
            "Escape" if self.ask != Ask::None => self.press("sc-cancel", now),
            "Escape" if self.shown(now) => {
                self.revealed = None;
                Some("Hidden".into())
            }
            "Enter" if self.ask != Ask::None => self.press("sc-confirm", now),
            _ => None,
        }
    }
}
