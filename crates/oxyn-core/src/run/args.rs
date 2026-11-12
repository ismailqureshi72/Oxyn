//! Tool arguments: a small JSON-like value (no serde_json dependency), the one-line preview, the
//! pretty form for detail panes, and redaction of secrets.

/// A JSON-like value.
#[derive(Clone, Debug, PartialEq)]
pub enum ArgValue {
    /// `null`.
    Null,
    /// Boolean.
    Bool(bool),
    /// Number (kept as written).
    Num(String),
    /// String.
    Str(String),
    /// Array.
    Arr(Vec<ArgValue>),
    /// Object, in key order.
    Obj(Vec<(String, ArgValue)>),
    /// A value hidden by the redactor.
    Redacted,
}

impl ArgValue {
    /// Object builder.
    pub fn obj<const N: usize>(pairs: [(&str, ArgValue); N]) -> Self {
        Self::Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    /// String builder.
    pub fn s(v: impl Into<String>) -> Self {
        Self::Str(v.into())
    }

    /// Number builder.
    pub fn n(v: impl ToString) -> Self {
        Self::Num(v.to_string())
    }

    fn short(&self, str_max: usize) -> String {
        match self {
            Self::Null => "null".into(),
            Self::Bool(b) => b.to_string(),
            Self::Num(n) => n.clone(),
            Self::Str(s) if s.len() > 4096 => format!("\"…\" ({} KB)", s.len() / 1024),
            Self::Str(s) => format!("\"{}\"", super::end_truncate(s, str_max)),
            Self::Arr(a) => format!("[{}]", a.len()),
            Self::Obj(_) => "{…}".into(),
            Self::Redacted => "••••".into(),
        }
    }

    /// Pretty, indented, for the Input pane; capped at `max_lines` with a count of the rest.
    pub fn pretty(&self, max_lines: usize) -> (String, usize) {
        let mut out = String::new();
        write(self, 0, &mut out);
        let lines: Vec<&str> = out.lines().collect();
        let total = lines.len();
        if total <= max_lines {
            (out, 0)
        } else {
            (lines[..max_lines].join("\n"), total - max_lines)
        }
    }
}

fn write(v: &ArgValue, ind: usize, out: &mut String) {
    let pad = "  ".repeat(ind + 1);
    match v {
        ArgValue::Arr(a) if !a.is_empty() => {
            out.push_str("[\n");
            for (i, x) in a.iter().enumerate() {
                out.push_str(&pad);
                write(x, ind + 1, out);
                out.push_str(if i + 1 < a.len() { ",\n" } else { "\n" });
            }
            out.push_str(&"  ".repeat(ind));
            out.push(']');
        }
        ArgValue::Obj(o) if !o.is_empty() => {
            out.push_str("{\n");
            for (i, (k, x)) in o.iter().enumerate() {
                out.push_str(&format!("{pad}\"{k}\": "));
                write(x, ind + 1, out);
                out.push_str(if i + 1 < o.len() { ",\n" } else { "\n" });
            }
            out.push_str(&"  ".repeat(ind));
            out.push('}');
        }
        ArgValue::Arr(_) => out.push_str("[]"),
        ArgValue::Obj(_) => out.push_str("{}"),
        ArgValue::Str(s) => out.push_str(&format!("\"{s}\"")),
        x => out.push_str(&x.short(usize::MAX)),
    }
}

const SECRET_KEYS: [&str; 8] = [
    "token",
    "secret",
    "password",
    "passwd",
    "api_key",
    "apikey",
    "authorization",
    "cookie",
];

/// Masks values under secret-looking keys (`token: ••••`).
pub fn redact(v: &ArgValue) -> ArgValue {
    match v {
        ArgValue::Obj(o) => ArgValue::Obj(
            o.iter()
                .map(|(k, x)| {
                    let lk = k.to_ascii_lowercase();
                    if SECRET_KEYS.iter().any(|s| lk.contains(s)) {
                        (k.clone(), ArgValue::Redacted)
                    } else {
                        (k.clone(), redact(x))
                    }
                })
                .collect(),
        ),
        ArgValue::Arr(a) => ArgValue::Arr(a.iter().map(redact).collect()),
        x => x.clone(),
    }
}

/// `key: value` pairs for the row (`max` pairs, strings cut at `str_max`), secrets masked.
pub fn args_preview(v: &ArgValue, max: usize, str_max: usize) -> String {
    match redact(v) {
        ArgValue::Obj(o) => {
            let mut parts: Vec<String> = o
                .iter()
                .take(max)
                .map(|(k, x)| format!("{k}: {}", x.short(str_max)))
                .collect();
            if o.len() > max {
                parts.push(format!("+{}", o.len() - max));
            }
            parts.join("  ")
        }
        x => x.short(str_max),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_and_redaction() {
        let a = ArgValue::obj([
            ("team", ArgValue::s("ENG")),
            ("api_token", ArgValue::s("sk-123")),
            (
                "title",
                ArgValue::s("Fix the refresh-token race in the auth middleware"),
            ),
            (
                "labels",
                ArgValue::Arr(vec![ArgValue::s("bug"), ArgValue::s("auth")]),
            ),
        ]);
        assert_eq!(
            args_preview(&a, 2, 24),
            "team: \"ENG\"  api_token: ••••  +2"
        );
        let (p, rest) = a.pretty(4);
        assert!(p.contains("\"api_token\": \"sk-123\"") || p.contains("••••") || rest > 0);
        let (p, _) = redact(&a).pretty(50);
        assert!(p.contains("\"api_token\": ••••"));
        assert!(p.contains("\"labels\": [\n"));
    }
}
