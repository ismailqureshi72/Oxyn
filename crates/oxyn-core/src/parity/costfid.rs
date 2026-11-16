//! affectUI fidelity pass (plan #32, overnight batch, family B part 6): the model behind the exact
//! **Cost Waterfall** – where the money and tokens of an agent run go: every LLM call, tool call and
//! sub-agent as a step in a waterfall with model, tokens, cost and time, the most expensive step
//! on top and a switch between cost and tokens. Behaviour only.

/// What a step is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostKind {
    /// A sub-agent (adds its children up).
    Agent,
    /// An LLM call.
    Llm,
    /// A tool call.
    Tool,
}

impl CostKind {
    /// `agent` · `llm` · `tool`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Llm => "llm",
            Self::Tool => "tool",
        }
    }
    /// What its details are called: (in, out).
    pub fn io(self) -> (&'static str, &'static str) {
        match self {
            Self::Agent => ("Task", "Result"),
            Self::Llm => ("Prompt", "Response"),
            Self::Tool => ("Input", "Output"),
        }
    }
}

/// Where a step is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostStatus {
    /// Still running.
    Running,
    /// Done.
    Done,
    /// Failed.
    Failed,
}

/// One step of the run.
#[derive(Clone, Debug, PartialEq)]
pub struct CostStep {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Kind.
    pub kind: CostKind,
    /// Status.
    pub status: CostStatus,
    /// Model or a short detail ("31 links").
    pub detail: String,
    /// Brand mark for the icon tile (empty: the kind's glyph).
    pub brand: String,
    /// Its own tokens in.
    pub tokens_in: u32,
    /// Its own tokens out.
    pub tokens_out: u32,
    /// Its own cost in dollars (negative = a credit); `None` shows as –.
    pub cost: Option<f64>,
    /// Milliseconds.
    pub duration: u32,
    /// Prompt / task / input.
    pub input: String,
    /// Response / result / output.
    pub output: String,
    /// The error of a failed step.
    pub error: String,
    /// A sub-agent's steps.
    pub children: Vec<CostStep>,
}

/// What the bars measure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostMode {
    /// Dollars.
    Cost,
    /// Tokens in + out.
    Tokens,
}

/// A step's total for a mode: its own plus its children's.
pub fn cost_total(s: &CostStep, mode: CostMode) -> f64 {
    let own = match mode {
        CostMode::Cost => s.cost.filter(|c| c.is_finite()).unwrap_or(0.0),
        CostMode::Tokens => (s.tokens_in + s.tokens_out) as f64,
    };
    own + s.children.iter().map(|c| cost_total(c, mode)).sum::<f64>()
}

/// (tokens in, tokens out) of a step with its children.
pub fn cost_tokens(s: &CostStep) -> (u32, u32) {
    s.children
        .iter()
        .fold((s.tokens_in, s.tokens_out), |(a, b), c| {
            let (x, y) = cost_tokens(c);
            (a + x, b + y)
        })
}

/// A cost with its children (None when neither has one).
pub fn cost_sum(s: &CostStep) -> Option<f64> {
    let kids: Vec<f64> = s.children.iter().filter_map(cost_sum).collect();
    match (s.cost.filter(|c| c.is_finite()), kids.is_empty()) {
        (None, true) => None,
        (c, _) => Some(c.unwrap_or(0.0) + kids.iter().sum::<f64>()),
    }
}

/// One visible row.
#[derive(Clone, Debug, PartialEq)]
pub struct CostRow {
    /// The step (children kept).
    pub step: CostStep,
    /// Depth (0 top level).
    pub depth: usize,
    /// Parent id (empty at the top).
    pub parent: String,
    /// Bar start (in the mode's units).
    pub start: f64,
    /// Bar length.
    pub len: f64,
    /// How many steps hide under a folded sub-agent.
    pub hidden: usize,
}

fn count(s: &CostStep) -> usize {
    s.children.iter().map(|c| 1 + count(c)).sum()
}

/// The visible rows in order: each bar starts where the one before it ended; a sub-agent's
/// steps start where it starts; folded sub-agents hide theirs.
pub fn cost_rows(steps: &[CostStep], mode: CostMode, folded: &[String]) -> Vec<CostRow> {
    fn walk(
        steps: &[CostStep],
        mode: CostMode,
        folded: &[String],
        depth: usize,
        parent: &str,
        start: &mut f64,
        out: &mut Vec<CostRow>,
    ) {
        for s in steps {
            let len = cost_total(s, mode);
            let fold = folded.contains(&s.id);
            out.push(CostRow {
                step: s.clone(),
                depth,
                parent: parent.to_string(),
                start: *start,
                len,
                hidden: if fold { count(s) } else { 0 },
            });
            if !fold && !s.children.is_empty() {
                let mut inner = *start;
                walk(&s.children, mode, folded, depth + 1, &s.id, &mut inner, out);
            }
            *start += len;
        }
    }
    let mut out = vec![];
    let mut start = 0.0;
    walk(steps, mode, folded, 0, "", &mut start, &mut out);
    out
}

/// The most expensive leaf step (an LLM or tool call) by cost.
pub fn cost_top(steps: &[CostStep]) -> Option<CostStep> {
    fn leaves(steps: &[CostStep], out: &mut Vec<CostStep>) {
        for s in steps {
            if s.kind == CostKind::Agent {
                leaves(&s.children, out);
            } else {
                out.push(s.clone());
            }
        }
    }
    let mut v = vec![];
    leaves(steps, &mut v);
    v.into_iter()
        .filter(|s| s.cost.is_some_and(|c| c.is_finite() && c > 0.0))
        .max_by(|a, b| a.cost.unwrap_or(0.0).total_cmp(&b.cost.unwrap_or(0.0)))
}

/// The ids on the way down to a step (its ancestors).
pub fn cost_path(steps: &[CostStep], id: &str) -> Vec<String> {
    for s in steps {
        if s.id == id {
            return vec![];
        }
        let p = cost_path(&s.children, id);
        if !p.is_empty() || s.children.iter().any(|c| c.id == id) {
            let mut v = vec![s.id.clone()];
            v.extend(p);
            return v;
        }
    }
    vec![]
}

/// The axis: (max, tick step) – ticks at nice 1·2·5 steps, the max a little past the total.
pub fn cost_axis(total: f64) -> (f64, f64) {
    if total <= 0.0 {
        return (1.0, 0.5);
    }
    let raw = total / 2.5;
    let p = 10f64.powi(raw.log10().floor() as i32);
    let m = raw / p;
    let nice = if m <= 1.0 {
        1.0
    } else if m <= 2.0 {
        2.0
    } else if m <= 5.0 {
        5.0
    } else {
        10.0
    };
    (total * 1.06, nice * p)
}

/// "$0.04" style: two decimals, three under half a cent ("$0.004"); "–" for none; a credit is "−$0.10".
pub fn cost_money(c: Option<f64>) -> String {
    match c.filter(|c| c.is_finite()) {
        None => "–".into(),
        Some(v) => {
            let a = v.abs();
            let s = if a > 0.0 && a < 0.005 {
                format!("${a:.3}")
            } else {
                format!("${a:.2}")
            };
            if v < 0.0 { format!("−{s}") } else { s }
        }
    }
}

/// "38.4k", "960", "–" for 0.
pub fn cost_tok(t: u32) -> String {
    if t == 0 {
        return "–".into();
    }
    if t >= 1000 {
        let v = t as f64 / 1000.0;
        if v >= 100.0 {
            format!("{}k", v.round() as u32)
        } else {
            format!("{v:.1}k")
        }
    } else {
        t.to_string()
    }
}

/// "3.0s" or "875ms".
pub fn cost_time(ms: u32) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else {
        format!("{:.1}s", ms as f64 / 1000.0)
    }
}

/// The demo run (our own copy): a release notes run.
pub fn cost_demo() -> Vec<CostStep> {
    #[allow(clippy::too_many_arguments)]
    fn st(
        id: &str,
        name: &str,
        kind: CostKind,
        detail: &str,
        brand: &str,
        tin: u32,
        tout: u32,
        cost: Option<f64>,
        ms: u32,
    ) -> CostStep {
        CostStep {
            id: id.into(),
            name: name.into(),
            kind,
            status: CostStatus::Done,
            detail: detail.into(),
            brand: brand.into(),
            tokens_in: tin,
            tokens_out: tout,
            cost,
            duration: ms,
            input: String::new(),
            output: String::new(),
            error: String::new(),
            children: vec![],
        }
    }
    use CostKind::*;
    let mut plan = st(
        "plan",
        "Outline the notes",
        Llm,
        "Opus 5.5",
        "claude",
        3_800,
        880,
        Some(0.04),
        2_800,
    );
    plan.input =
        "Draft release notes for v2.4 – group the changes, call out the two breaking ones.".into();
    plan.output = "1. Collect merged PRs  2. Write the notes  3. Check links  4. Publish".into();
    let mut scout = st(
        "scout",
        "Collector",
        Agent,
        "Gemini 3.8 Flash",
        "gemini",
        0,
        0,
        None,
        7_900,
    );
    scout.input = "Find every PR merged since v2.3 and what it changed.".into();
    scout.output = "41 PRs in 6 groups".into();
    scout.children = vec![
        st(
            "s1",
            "search_prs",
            Tool,
            "since v2.3",
            "github",
            0,
            0,
            Some(0.01),
            1_200,
        ),
        st(
            "s2",
            "search_prs",
            Tool,
            "labels: breaking",
            "github",
            0,
            0,
            Some(0.01),
            900,
        ),
        st("s3", "read_diffs", Tool, "12 files", "", 0, 0, None, 2_300),
        st(
            "s4",
            "Group the changes",
            Llm,
            "Gemini 3.8 Flash",
            "gemini",
            34_900,
            1_100,
            Some(0.01),
            3_500,
        ),
    ];
    let mut write = st(
        "write",
        "Write the notes",
        Llm,
        "Opus 5.5",
        "claude",
        20_600,
        5_200,
        Some(0.23),
        7_100,
    );
    write.input =
        "Write the v2.4 notes from the six groups; lead with the breaking changes.".into();
    write.output = "1,240 words · 6 sections".into();
    let mut review = st(
        "review",
        "Checker",
        Agent,
        "Haiku 5.5",
        "claude",
        0,
        0,
        None,
        3_100,
    );
    let mut links = st(
        "links",
        "check_links",
        Tool,
        "27 links",
        "",
        0,
        0,
        None,
        1_000,
    );
    links.status = CostStatus::Failed;
    links.error = "2 links time out (docs preview is down)".into();
    review.children = vec![
        st(
            "facts",
            "Check claims",
            Llm,
            "Haiku 5.5",
            "claude",
            24_200,
            640,
            Some(0.03),
            1_900,
        ),
        links,
    ];
    let mut fix = st(
        "fix",
        "Apply fixes",
        Llm,
        "Sonnet 5.5",
        "claude",
        22_900,
        1_700,
        Some(0.09),
        2_600,
    );
    fix.input = "Swap the two dead links for the stable docs and keep everything else.".into();
    fix.output = "2 links swapped – notes ready".into();
    let save = st(
        "save",
        "Publish notes",
        Tool,
        "Notion",
        "notion",
        0,
        0,
        None,
        820,
    );
    vec![plan, scout, write, review, fix, save]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_totals_and_formats() {
        let d = cost_demo();
        let total: f64 = d.iter().map(|s| cost_total(s, CostMode::Cost)).sum();
        assert!((total - 0.42).abs() < 1e-9);
        let rows = cost_rows(&d, CostMode::Tokens, &[]);
        assert_eq!(rows.len(), 12);
        // A child starts where its sub-agent starts; the next top step where the sub-agent ended.
        assert_eq!(rows[2].start, rows[1].start);
        assert_eq!(rows[6].start, rows[1].start + rows[1].len);
        let folded = cost_rows(&d, CostMode::Tokens, &["scout".into()]);
        assert_eq!(folded.len(), 8);
        assert_eq!(folded[1].hidden, 4);
        assert_eq!(cost_top(&d).map(|s| s.id), Some("write".into()));
        assert_eq!(cost_path(&d, "facts"), vec!["review".to_string()]);
        assert_eq!(cost_sum(&d[1]), Some(0.03));
        assert_eq!(cost_sum(&d[5]), None);
        assert_eq!(cost_tokens(&d[1]), (34_900, 1_100));
        assert_eq!(cost_money(Some(0.04)), "$0.04");
        assert_eq!(cost_money(Some(0.004)), "$0.004");
        assert_eq!(cost_money(Some(0.01)), "$0.01");
        assert_eq!(cost_money(Some(-0.1)), "−$0.10");
        assert_eq!(cost_money(None), "–");
        assert_eq!(cost_tok(38_400), "38.4k");
        assert_eq!(cost_tok(0), "–");
        assert_eq!(cost_time(875), "875ms");
        assert_eq!(cost_time(3_000), "3.0s");
        let (mx, st) = cost_axis(125_000.0);
        assert_eq!(st, 50_000.0);
        assert!(mx > 125_000.0);
        assert_eq!(cost_axis(0.46).1, 0.2);
    }
}
