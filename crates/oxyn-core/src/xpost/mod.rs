//! Agent micro-interactions first seen in posts the founders saved (C2a): Test ticks, Triage
//! metro, Secret card, Proactive card, Write sweep, Done shockwave, Split confirm, Publish
//! popover, Agent prompt copy and Loop pipeline. Ideas only, rebuilt in Oxyn's own design. One
//! state machine per component, driven by press ids and keys with the host clock in ms, shared by
//! the Dioxus components (`oxyn_dioxus::xpost`) and the GPUI `XPostView`.

pub mod loops;
pub mod metro;
pub mod proactive;
pub mod promptcopy;
pub mod publish;
pub mod secret;
pub mod shock;
pub mod split;
pub mod sweep;
pub mod ticks;

/// Page slugs, in `XPostView` variant order.
pub const XP_SLUGS: [&str; 10] = [
    "test-ticks",
    "triage-metro",
    "secret-card",
    "proactive-card",
    "write-sweep",
    "done-shockwave",
    "split-confirm",
    "publish-popover",
    "agent-prompt-copy",
    "loop-pipeline",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_run_fail_and_fix() {
        let mut t = ticks::TestTicks::new(true, 1.0, 0);
        assert_eq!(t.tick(0, 0), ticks::Tick::Running);
        let end = t.span();
        assert!(t.done(end));
        assert_eq!(t.counts(end).1, ticks::FAILS.len());
        assert!(t.fix_open(end));
        t.press("tt-fix", end);
        assert!(!t.done(end + 100));
        let after = end + 500 + ticks::FAILS.len() as u64 * 450;
        assert!(t.done(after));
        assert_eq!(t.counts(after), (ticks::total(), 0, ticks::total()));
        t.key("ArrowRight", after);
        assert_eq!(t.sel, Some(0));
        t.key("ArrowRight", after);
        assert_eq!(t.sel, Some(1));
        t.press("tt-stop", after);
        assert!(t.stopped.is_none());
    }

    #[test]
    fn ticks_stop_midway() {
        let mut t = ticks::TestTicks::new(false, 2.0, 0);
        t.press("tt-stop", 1_000);
        let (_, _, n) = t.counts(50_000);
        assert!(n < ticks::total());
        assert!(t.status(50_000).starts_with("Stopped"));
    }

    #[test]
    fn metro_rides_and_reroutes() {
        let mut m = metro::Metro::new(false, 0);
        assert!(m.progress(1, 0).is_none());
        let p = m.point(0, 0).unwrap();
        assert_eq!(p, (metro::STATIONS[0].0, 100.0));
        let end = metro::GAP * 8 + metro::RIDE;
        assert_eq!(m.counts(end).iter().sum::<usize>(), 8);
        assert_eq!(m.counts(end)[2], 2);
        m.press("tm-dot-2", end);
        assert_eq!(m.open, Some(2));
        m.press("tm-patch", end);
        assert_eq!(m.outcome(2), 0);
        assert_eq!(m.counts(end + 1_300)[0], 5);
        let last = m.point(2, end + 2_000).unwrap();
        assert!((last.1 - metro::TERM_Y[0]).abs() < 0.01);
        m.key("Escape", end);
        assert_eq!(m.open, None);
        m.press("tm-pause", end);
        assert!(m.paused.is_some());
    }

    #[test]
    fn metro_along_is_on_the_path() {
        let pts = metro::path(1);
        assert_eq!(metro::along(&pts, 1.0), pts[3]);
        assert_eq!(metro::along(&pts, 0.0), pts[0]);
    }

    #[test]
    fn secret_reveal_rotate_revoke() {
        let mut s = secret::Secret::new(false);
        assert!(s.value(0).contains('•'));
        s.press("sc-reveal", 0);
        assert_eq!(s.ask, secret::Ask::Reveal);
        s.key("Enter", 10);
        assert!(s.shown(100));
        assert!(!s.shown(10 + secret::REVEAL_MS));
        s.press("sc-rotate", 6_000);
        s.press("sc-confirm", 6_000);
        assert!(s.rotation(6_500).is_some());
        assert_eq!(s.tail(9_000), secret::ROTATED);
        s.press("sc-revoke", 9_000);
        s.press("sc-confirm", 9_000);
        assert!(s.is_revoked() && s.can_undo(10_000));
        s.press("sc-undo", 10_000);
        assert!(!s.is_revoked());
        assert_eq!(secret::Secret::handle(), "{{secrets.STRIPE_SECRET_KEY}}");
    }

    #[test]
    fn proactive_accept_snooze_and_fail() {
        let mut p = proactive::Proactive::new(false);
        assert_eq!(p.phase(0), proactive::Phase::Offer);
        p.press("pc-accept", 0);
        assert_eq!(p.phase(500), proactive::Phase::Working);
        assert_eq!(p.phase(proactive::STEP_MS * 3), proactive::Phase::Result);
        let mut q = proactive::Proactive::new(false);
        q.press("pc-later", 0);
        q.key("ArrowDown", 0);
        q.key("Enter", 0);
        assert_eq!(q.snoozed, Some(1));
        assert_eq!(q.phase(0), proactive::Phase::Snoozed);
        q.press("pc-undo", 0);
        assert_eq!(q.phase(0), proactive::Phase::Offer);
        let mut f = proactive::Proactive::new(true);
        f.press("pc-accept", 0);
        assert_eq!(f.phase(proactive::STEP_MS + 1), proactive::Phase::Failed);
        f.press("pc-retry", 5_000);
        assert_eq!(
            f.phase(5_000 + proactive::STEP_MS * 3),
            proactive::Phase::Result
        );
    }

    #[test]
    fn sweep_crosses_rows_and_undoes() {
        let mut s = sweep::Sweep::new(1_200, false, 0);
        assert_eq!(s.value(1, 0), sweep::ROWS[1].1);
        s.press("ws-run", 0);
        assert!(s.band(600).is_some());
        assert_eq!(s.value(1, 600), sweep::ROWS[1].2);
        assert_eq!(s.value(4, 600), sweep::ROWS[4].1);
        assert!(s.done(1_200));
        s.key("undo", 1_300);
        assert_eq!(s.value(1, 1_400), sweep::ROWS[1].1);
        assert_eq!(sweep::changed().len(), 4);
    }

    #[test]
    fn shock_rings_after_done() {
        let mut s = shock::Shock::new(false, 1.0, 1.0, false, 0);
        assert_eq!(s.phase(0), shock::Phase::Idle);
        s.key("Enter", 0);
        assert_eq!(s.phase(10), shock::Phase::Running);
        assert!(s.rings(10).is_empty());
        let end = shock::TASK.2;
        assert_eq!(s.rings(end + 200).len(), 2);
        assert!(s.rings(end + 2_000).is_empty());
        let mut f = shock::Shock::new(true, 1.0, 1.0, true, 0);
        assert_eq!(f.phase(5_000), shock::Phase::Failed);
        f.press("sw-retry", 5_000);
        assert_eq!(f.phase(5_000 + end), shock::Phase::Done);
    }

    #[test]
    fn split_stretches_and_undoes() {
        let mut s = split::Split::new(0, false);
        s.press("sp-go", 0);
        assert!(s.scale(130) > 1.1);
        assert!(s.bubble(400).is_some());
        assert_eq!(s.text(1_000), "Approved");
        assert!(s.can_undo(1_000));
        s.key("Backspace", 1_000);
        assert_eq!(s.phase(1_000), split::Phase::Idle);
        let mut f = split::Split::new(1, true);
        f.key("Enter", 0);
        assert_eq!(f.phase(1_000), split::Phase::Failed);
        f.key("Enter", 1_000);
        assert_eq!(f.text(3_000), "Sent");
    }

    #[test]
    fn publish_steps_and_taken_slug() {
        let mut p = publish::Publish::new(true, false);
        p.set_slug("Launch Plan Q4!");
        p.key("", "Enter", 0);
        assert_eq!(p.slug, "launch-plan-q4");
        assert_eq!(p.phase(100), publish::Phase::Publishing(0));
        assert_eq!(p.phase(5_000), publish::Phase::Live);
        p.key("", "Enter", 5_000);
        assert!(p.copied.is_some());
        p.key("host", "ArrowRight", 5_000);
        assert_eq!(p.host, 1);
        let mut t = publish::Publish::new(true, true);
        t.press("pp-go", 0);
        assert_eq!(t.phase(1_000), publish::Phase::Taken);
        t.press("pp-go", 1_000);
        assert!(t.slug.ends_with("-2"));
        assert_eq!(t.phase(5_000), publish::Phase::Live);
    }

    #[test]
    fn prompt_copy_menu_and_formats() {
        let mut c = promptcopy::PromptCopy::new(0, 0, false);
        assert!(c.text().contains("oxyn add test-ticks"));
        c.key("ArrowDown", 0);
        assert_eq!(c.menu, Some(0));
        c.key("ArrowDown", 0);
        c.key("ArrowDown", 0);
        c.key("Enter", 0);
        assert_eq!(c.agent, 2);
        c.press("ga-format-2", 0);
        assert!(c.text().contains("--agent cursor"));
        c.key("Escape", 0);
        c.key("Enter", 100);
        assert!(c.copied(200));
    }

    #[test]
    fn loop_exits_on_target_or_runs_out() {
        let l = loops::Loops::new(85, 5, 0);
        assert_eq!(l.exit_round(), Some(4));
        assert!(l.passed(100_000));
        assert_eq!(l.scored(100_000), vec![62, 74, 81, 88]);
        let mut o = loops::Loops::new(95, 3, 0);
        assert!(o.out(100_000));
        assert_eq!(o.best(100_000), 81);
        o.press("lp-more", 100_000);
        assert!(o.at(100_000).is_some());
        assert_eq!(o.scored(200_000).len(), 4);
        o.key("ArrowLeft", 200_000);
        assert_eq!(o.sel, Some(2));
    }
}
