//! One loader animates per busy region (`loaders.md` §2.3).

/// A stable loader id chosen by the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LoaderId(pub u64);

/// Where a loader sits. Each region has at most one animating loader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BusyRegion {
    /// The composer (status strip dot, border light, composer mascot).
    Composer,
    /// The end of the transcript.
    TranscriptTail,
    /// One tool row.
    ToolRow(u64),
    /// One list row (sidebar thread, sub-agent, background job).
    ListRow(u64),
    /// An avatar.
    Avatar(u64),
    /// The whole window (edge glow).
    Window,
}

/// How strongly a loader wants to lead its region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BusyPriority {
    /// Only mirrors the lead (still frame).
    Mirror,
    /// Default.
    Normal,
    /// Takes the region (border light, mascot).
    Lead,
}

/// At most this many list rows animate at once; the rest show their still frame.
pub const LIST_ANIMATION_CAP: usize = 8;

#[derive(Clone, Copy, Debug)]
struct Claim {
    id: LoaderId,
    region: BusyRegion,
    priority: BusyPriority,
    seq: u64,
    /// For list rows: distance from the viewport centre, nearest first.
    rank: u32,
}

/// Decides which loaders animate. Everyone else draws the still frame of the same phase.
#[derive(Debug, Default)]
pub struct BusyArbiter {
    claims: Vec<Claim>,
    seq: u64,
}

impl BusyArbiter {
    /// An empty arbiter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Claims (or re-claims) a region. Ties go to the most recent claim.
    pub fn claim(&mut self, id: LoaderId, region: BusyRegion, priority: BusyPriority) {
        self.release(id);
        self.seq += 1;
        self.claims.push(Claim {
            id,
            region,
            priority,
            seq: self.seq,
            rank: u32::MAX,
        });
    }

    /// Sets a list row's distance from the viewport centre (smaller animates first).
    pub fn set_rank(&mut self, id: LoaderId, rank: u32) {
        if let Some(c) = self.claims.iter_mut().find(|c| c.id == id) {
            c.rank = rank;
        }
    }

    /// Drops a claim (unmount, or the loader went hidden).
    pub fn release(&mut self, id: LoaderId) {
        self.claims.retain(|c| c.id != id);
    }

    /// True if `id` may animate.
    pub fn animates(&self, id: LoaderId) -> bool {
        let Some(me) = self.claims.iter().find(|c| c.id == id) else {
            return false;
        };
        if me.priority == BusyPriority::Mirror || self.lead(me.region) != Some(id) {
            return false;
        }
        if let BusyRegion::ListRow(_) = me.region {
            let mut rows: Vec<&Claim> = self
                .claims
                .iter()
                .filter(|c| matches!(c.region, BusyRegion::ListRow(_)))
                .filter(|c| self.lead(c.region) == Some(c.id) && c.priority != BusyPriority::Mirror)
                .collect();
            rows.sort_by_key(|c| (c.rank, std::cmp::Reverse(c.seq)));
            return rows.iter().take(LIST_ANIMATION_CAP).any(|c| c.id == id);
        }
        // A window glow only leads when no composer loader is visible.
        if me.region == BusyRegion::Window
            && self.claims.iter().any(|c| c.region == BusyRegion::Composer)
        {
            return false;
        }
        true
    }

    /// The loader leading `region`, if any.
    pub fn lead(&self, region: BusyRegion) -> Option<LoaderId> {
        self.claims
            .iter()
            .filter(|c| c.region == region)
            .max_by_key(|c| (c.priority, c.seq))
            .map(|c| c.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highest_priority_then_newest_leads() {
        let mut a = BusyArbiter::new();
        a.claim(LoaderId(1), BusyRegion::Composer, BusyPriority::Normal);
        a.claim(LoaderId(2), BusyRegion::Composer, BusyPriority::Normal);
        assert!(a.animates(LoaderId(2)) && !a.animates(LoaderId(1)));
        a.claim(LoaderId(3), BusyRegion::Composer, BusyPriority::Lead);
        a.claim(LoaderId(4), BusyRegion::Composer, BusyPriority::Normal);
        assert!(a.animates(LoaderId(3)) && !a.animates(LoaderId(4)));
        a.release(LoaderId(3));
        assert!(a.animates(LoaderId(4)));
    }

    #[test]
    fn at_most_eight_list_rows_animate_nearest_first() {
        let mut a = BusyArbiter::new();
        for i in 0..20 {
            a.claim(LoaderId(i), BusyRegion::ListRow(i), BusyPriority::Normal);
            a.set_rank(LoaderId(i), (i as i64 - 10).unsigned_abs() as u32);
        }
        let animating: Vec<u64> = (0..20).filter(|i| a.animates(LoaderId(*i))).collect();
        assert_eq!(animating.len(), LIST_ANIMATION_CAP);
        assert!(animating.contains(&10) && !animating.contains(&0));
    }

    #[test]
    fn window_glow_yields_to_composer() {
        let mut a = BusyArbiter::new();
        a.claim(LoaderId(1), BusyRegion::Window, BusyPriority::Lead);
        assert!(a.animates(LoaderId(1)));
        a.claim(LoaderId(2), BusyRegion::Composer, BusyPriority::Normal);
        assert!(!a.animates(LoaderId(1)));
    }
}
