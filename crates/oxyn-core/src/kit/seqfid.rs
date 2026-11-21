//! affectUI fidelity pass (plan #32, overnight batch, family B part 4): the model behind the exact
//! **Sequence diagram** – one agent run as a sequence diagram: a column for you, every agent and every
//! tool, each message an arrow between lifelines (calls solid, replies dashed), a bar on a lifeline
//! while a participant works, and parallel or retried messages in labelled frames. Behaviour
//! only; times are milliseconds since the run started.

/// What a participant is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeqKind {
    /// A person (the full pixel mark in ink).
    Person,
    /// An agent (its own tinted pixel mark).
    Agent,
    /// A tool (its logo).
    Tool,
}

/// A column.
#[derive(Clone, Debug, PartialEq)]
pub struct SeqParticipant {
    /// Id.
    pub id: String,
    /// Name on the chip.
    pub name: String,
    /// Kind.
    pub kind: SeqKind,
    /// Brand mark for a tool.
    pub brand: String,
    /// Colour name: `ink`, `blue`, `purple`, `cyan`, `green`, `orange`.
    pub color: String,
    /// The model, as a tag in its card.
    pub model: String,
    /// A line on what it does.
    pub description: String,
}

/// A message's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeqStatus {
    /// Normal.
    Ok,
    /// Failed: red with a cross where it lands.
    Failed,
    /// Needs a person: amber.
    Waiting,
}

/// One arrow.
#[derive(Clone, Debug, PartialEq)]
pub struct SeqMessage {
    /// Id.
    pub id: String,
    /// Sender.
    pub from: String,
    /// Receiver.
    pub to: String,
    /// The label.
    pub text: String,
    /// A reply (dashed) – to the call with this id, when set.
    pub reply_to: Option<String>,
    /// State.
    pub status: SeqStatus,
    /// Sent at (ms since the run started).
    pub at: u64,
    /// Frame tag ("par", "retry"); consecutive messages with the same tag share a box.
    pub frame: String,
    /// What went along, for its card.
    pub detail: String,
}

impl SeqMessage {
    /// Whether it is a reply.
    pub fn is_reply(&self) -> bool {
        self.reply_to.is_some()
    }
}

/// "0:05" · "1:12" · "1:02:05".
pub fn seq_time(ms: u64) -> String {
    let s = ms / 1000;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// Row pitch and a frame's extra room for its tag (px).
pub const SEQ_ROW: f32 = 36.0;
/// Extra room above a frame's first row.
pub const SEQ_FRAME_GAP: f32 = 36.0;

/// Each message's y (px from the top of the rows), with a frame's first row pushed down for its
/// tag.
pub fn seq_rows(msgs: &[SeqMessage]) -> Vec<f32> {
    let mut y = SEQ_ROW / 2.0 + 4.0;
    let mut out = vec![];
    for (i, m) in msgs.iter().enumerate() {
        let starts = !m.frame.is_empty() && (i == 0 || msgs[i - 1].frame != m.frame);
        if starts {
            y += SEQ_FRAME_GAP;
        }
        out.push(y);
        let ends = !m.frame.is_empty() && msgs.get(i + 1).is_none_or(|n| n.frame != m.frame);
        y += SEQ_ROW + if ends { 8.0 } else { 0.0 };
    }
    out
}

/// The height the rows take.
pub fn seq_height(msgs: &[SeqMessage]) -> f32 {
    seq_rows(msgs).last().map_or(SEQ_ROW * 2.0, |y| y + SEQ_ROW)
}

/// A frame: (tag, first message index, last index).
pub fn seq_frames(msgs: &[SeqMessage]) -> Vec<(String, usize, usize)> {
    let mut out: Vec<(String, usize, usize)> = vec![];
    for (i, m) in msgs.iter().enumerate() {
        if m.frame.is_empty() {
            continue;
        }
        match out.last_mut() {
            Some(f) if f.0 == m.frame && f.2 + 1 == i => f.2 = i,
            _ => out.push((m.frame.clone(), i, i)),
        }
    }
    out
}

/// Activity bars: (participant id, from message index, to message index or `None` while open).
/// A call opens a bar on its receiver until the reply to it arrives.
pub fn seq_bars(msgs: &[SeqMessage]) -> Vec<(String, usize, Option<usize>)> {
    msgs.iter()
        .enumerate()
        .filter(|(_, m)| !m.is_reply() && m.from != m.to)
        .map(|(i, m)| {
            let end = msgs
                .iter()
                .position(|r| r.reply_to.as_deref() == Some(m.id.as_str()));
            (m.to.clone(), i, end)
        })
        .collect()
}

/// (working participants, failures) once these messages are in.
pub fn seq_status(msgs: &[SeqMessage]) -> (usize, usize) {
    let mut open: Vec<String> = seq_bars(msgs)
        .into_iter()
        .filter(|b| b.2.is_none())
        .map(|b| b.0)
        .collect();
    open.sort();
    open.dedup();
    (
        open.len(),
        msgs.iter()
            .filter(|m| m.status == SeqStatus::Failed)
            .count(),
    )
}

/// A message's accessible name: "Researcher to GitHub: search_code(\"…\"), 0:05".
pub fn seq_label(m: &SeqMessage, from: &str, to: &str) -> String {
    let st = match m.status {
        SeqStatus::Failed => ", failed",
        SeqStatus::Waiting => ", needs you",
        SeqStatus::Ok => "",
    };
    format!("{from} to {to}: {}{st}, {}", m.text, seq_time(m.at))
}

/// "Answered in 2s" / "Waiting for the reply" / "" for replies.
pub fn seq_answer(msgs: &[SeqMessage], i: usize) -> String {
    let m = &msgs[i];
    if m.is_reply() || m.from == m.to {
        return String::new();
    }
    match msgs
        .iter()
        .find(|r| r.reply_to.as_deref() == Some(m.id.as_str()))
    {
        Some(r) => format!("Answered in {}s", r.at.saturating_sub(m.at).div_ceil(1000)),
        None => "Waiting for the reply".into(),
    }
}

/// Per participant: (sent, received, tasks taken, busy ms).
pub fn seq_stats(msgs: &[SeqMessage], id: &str, now: u64) -> (usize, usize, usize, u64) {
    let sent = msgs.iter().filter(|m| m.from == id && m.to != id).count();
    let recv = msgs.iter().filter(|m| m.to == id && m.from != id).count();
    let mut tasks = 0;
    let mut busy = 0;
    for (p, a, b) in seq_bars(msgs) {
        if p == id {
            tasks += 1;
            let end = b.map_or(now, |b| msgs[b].at);
            busy += end.saturating_sub(msgs[a].at);
        }
    }
    (sent, recv, tasks, busy)
}

fn part(
    id: &str,
    name: &str,
    kind: SeqKind,
    brand: &str,
    color: &str,
    model: &str,
    d: &str,
) -> SeqParticipant {
    SeqParticipant {
        id: id.into(),
        name: name.into(),
        kind,
        brand: brand.into(),
        color: color.into(),
        model: model.into(),
        description: d.into(),
    }
}

#[allow(clippy::too_many_arguments)]
fn msg(
    id: &str,
    from: &str,
    to: &str,
    text: &str,
    reply_to: Option<&str>,
    at: u64,
    frame: &str,
    detail: &str,
) -> SeqMessage {
    SeqMessage {
        id: id.into(),
        from: from.into(),
        to: to.into(),
        text: text.into(),
        reply_to: reply_to.map(|s| s.to_string()),
        status: SeqStatus::Ok,
        at,
        frame: frame.into(),
        detail: detail.into(),
    }
}

/// The demo run's participants.
pub fn seq_demo_participants() -> Vec<SeqParticipant> {
    use SeqKind::*;
    vec![
        part("you", "You", Person, "", "ink", "", "Asked for the fix"),
        part(
            "lead",
            "Lead",
            Agent,
            "",
            "blue",
            "Opus 4.1",
            "Splits the work and checks it",
        ),
        part(
            "research",
            "Researcher",
            Agent,
            "",
            "purple",
            "Sonnet 4.5",
            "Finds where and why sign-ups fail",
        ),
        part(
            "coder",
            "Coder",
            Agent,
            "",
            "cyan",
            "GPT-5",
            "Writes the patch and ships a preview",
        ),
        part(
            "github",
            "GitHub",
            Tool,
            "github",
            "ink",
            "",
            "Code search and pull requests",
        ),
        part(
            "vercel",
            "Vercel",
            Tool,
            "vercel",
            "ink",
            "",
            "Preview deployments",
        ),
    ]
}

/// The demo run's fifteen messages.
pub fn seq_demo_messages() -> Vec<SeqMessage> {
    let mut v = vec![
        msg(
            "m1",
            "you",
            "lead",
            "Fix the failing sign-up",
            None,
            0,
            "",
            "Sign-ups with a plus in the email bounce",
        ),
        msg(
            "m2",
            "lead",
            "lead",
            "Plan: trace, patch, preview",
            None,
            2_000,
            "",
            "Three steps, two agents in parallel",
        ),
        msg(
            "m3",
            "lead",
            "research",
            "Trace the error",
            None,
            3_000,
            "par",
            "Start from the sign-up form",
        ),
        msg(
            "m4",
            "lead",
            "coder",
            "Open a branch",
            None,
            3_400,
            "par",
            "Branch off main",
        ),
        msg(
            "m5",
            "research",
            "github",
            "search_code(\"isValidEmail\")",
            None,
            5_000,
            "par",
            "{ \"query\": \"isValidEmail\", \"repo\": \"web/app\" }",
        ),
        msg(
            "m6",
            "github",
            "research",
            "4 matches",
            Some("m5"),
            6_000,
            "par",
            "auth/validate.ts, signup/form.tsx and 2 tests",
        ),
        msg(
            "m7",
            "coder",
            "lead",
            "fix/plus-emails ready",
            Some("m4"),
            8_000,
            "par",
            "Branch created from main at 9c41e2",
        ),
        msg(
            "m8",
            "research",
            "lead",
            "Regex drops plus signs",
            Some("m3"),
            11_000,
            "par",
            "auth/validate.ts:18 – the pattern has no + in the local part",
        ),
        msg(
            "m9",
            "lead",
            "coder",
            "Allow plus addresses",
            None,
            12_000,
            "",
            "Patch the pattern and add a test",
        ),
        msg(
            "m10",
            "coder",
            "vercel",
            "deploy_preview()",
            None,
            29_000,
            "retry",
            "{ \"branch\": \"fix/plus-emails\" }",
        ),
        msg(
            "m11",
            "vercel",
            "coder",
            "Build failed",
            Some("m10"),
            47_000,
            "retry",
            "Type error in signup/form.tsx:42",
        ),
        msg(
            "m12",
            "coder",
            "vercel",
            "deploy_preview()",
            None,
            50_000,
            "retry",
            "{ \"branch\": \"fix/plus-emails\", \"attempt\": 2 }",
        ),
        msg(
            "m13",
            "vercel",
            "coder",
            "Preview ready",
            Some("m12"),
            65_000,
            "retry",
            "Built in 14 s",
        ),
        msg(
            "m14",
            "coder",
            "lead",
            "PR #318 opened",
            Some("m9"),
            67_000,
            "",
            "2 files, +9 −3, tests pass",
        ),
        msg(
            "m15",
            "lead",
            "you",
            "Fixed – PR #318 ready to merge",
            Some("m1"),
            69_000,
            "",
            "Needs one review before merging",
        ),
    ];
    v[10].status = SeqStatus::Failed;
    v
}

/// One pass of the demo (real ms).
pub const SEQ_LOOP_MS: u64 = 16_000;
/// Real ms between messages in the demo.
pub const SEQ_STEP_MS: u64 = 750;

/// How many demo messages are in `t` real ms into a pass (all fifteen after about 11 s).
pub fn seq_shown(t: u64) -> usize {
    ((t / SEQ_STEP_MS) as usize).min(seq_demo_messages().len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_frames_bars_and_counts() {
        let m = seq_demo_messages();
        let f = seq_frames(&m);
        assert_eq!(f, vec![("par".into(), 2, 7), ("retry".into(), 9, 12)]);
        let y = seq_rows(&m);
        assert!((y[2] - y[1] - (SEQ_ROW + SEQ_FRAME_GAP)).abs() < 0.01);
        let bars = seq_bars(&m);
        assert!(bars.contains(&("research".into(), 2, Some(7))));
        assert_eq!(seq_status(&m[..5]), (4, 0));
        assert_eq!(seq_status(&m), (0, 1));
        assert_eq!(seq_answer(&m, 4), "Answered in 1s");
        assert_eq!(seq_answer(&m[..5], 4), "Waiting for the reply");
        assert_eq!(seq_time(72_000), "1:12");
        assert_eq!(seq_time(3_725_000), "1:02:05");
        assert_eq!(seq_stats(&m, "research", 69_000), (2, 2, 1, 8_000));
        assert_eq!(
            seq_label(&m[4], "Researcher", "GitHub"),
            "Researcher to GitHub: search_code(\"isValidEmail\"), 0:05"
        );
        assert_eq!(seq_shown(0), 0);
        assert_eq!(seq_shown(15_000), 15);
    }
}
