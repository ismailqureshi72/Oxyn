# Regenerates crates/oxyn-core/src/brand.rs from Simple Icons (CC0) and Lobe Icons (MIT).
# Usage: BRAND_SRC=<dir with si/ and lobe/> python3 scripts/brand/gen_brand.py
import json, re, os
# $BRAND_SRC holds si/ (simple-icons) and lobe/ (@lobehub/icons-static-svg), unpacked from npm.
B = os.environ.get("BRAND_SRC", "brand-src")
si_data = json.load(open(f"{B}/si/data/simple-icons.json")) if os.path.exists(f"{B}/si/data/simple-icons.json") else None
if si_data is None:
    import glob
    for f in glob.glob(f"{B}/si/**/*.json", recursive=True):
        d = json.load(open(f))
        if isinstance(d, list) and d and "hex" in d[0]: si_data = d; break
        if isinstance(d, dict) and "icons" in d: si_data = d["icons"]; break
def si_slug(t):
    s = t.lower().replace("+", "plus").replace(".", "dot").replace("&", "and")
    return re.sub(r"[^a-z0-9]", "", s)
si_hex = {}
for e in si_data:
    si_hex[e.get("slug") or si_slug(e["title"])] = e["hex"]

# id, title, source, file, hex, aliases, kind
L = "lobe"; S = "si"; O = "oxyn"
# Marks drawn by Oxyn (MIT) from basic shapes: Slack has no permissive file (removed from Simple Icons).
def _slack(fill):
    import math
    cols = ["#36C5F0", "#2EB67D", "#ECB22E", "#E01E5A"]
    out = []
    for k in range(4):
        f = f' fill="{cols[k]}"' if fill else ""
        out.append(f'<g transform="rotate({90*k} 12 12)"><rect x="7.6" y="1" width="4.2" height="10.4" rx="2.1"{f}></rect><path d="M12.8 1h2.1a2.1 2.1 0 012.1 2.1 2.1 2.1 0 01-2.1 2.1h-2.1z"{f}></path></g>')
    return "".join(out)
OWN = {"slack": _slack(False)}
OWN_COLOURED = {
  "slack": _slack(True),
  "gmail": '<path d="M1.5 6.6 6 9.9V20H3a1.5 1.5 0 01-1.5-1.5z" fill="#4285F4"></path><path d="M22.5 6.6 18 9.9V20h3a1.5 1.5 0 001.5-1.5z" fill="#34A853"></path><path d="M6 9.9 12 14.3l6-4.4V5l-6 4.4L6 5z" fill="#EA4335"></path><path d="M1.5 6.6v-1c0-1.7 1.9-2.6 3.3-1.6L6 5v4.9z" fill="#C5221F"></path><path d="M22.5 6.6v-1c0-1.7-1.9-2.6-3.3-1.6L18 5v4.9z" fill="#FBBC04"></path>',
  "googledrive": '<path d="M8 3 1 15l3.5 6 7-12z" fill="#00AC47"></path><path d="M8 3h8l7 12h-7z" fill="#FFBA00"></path><path d="M4.5 21h15l3.5-6H8z" fill="#2684FC"></path>',
  "clickup": '<defs><linearGradient id="oxyn-cu-a" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#8930FD"></stop><stop offset="1" stop-color="#49CCF9"></stop></linearGradient><linearGradient id="oxyn-cu-b" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#FF02F0"></stop><stop offset="1" stop-color="#FFC800"></stop></linearGradient></defs><path d="M2 18.439l3.69-2.828c1.961 2.56 4.044 3.739 6.363 3.739 2.307 0 4.33-1.166 6.203-3.704L22 18.405C19.298 22.065 15.941 24 12.053 24 8.178 24 4.788 22.078 2 18.439z" fill="url(#oxyn-cu-b)"></path><path d="M12.04 6.15l-6.568 5.66-3.036-3.52L12.055 0l9.543 8.296-3.05 3.509z" fill="url(#oxyn-cu-a)"></path>',
}
ICONS = [
  ("openai", "OpenAI", L, "openai", "000000", ["gpt", "chatgpt", "o1", "o3", "o4"], "ai"),
  ("anthropic", "Anthropic", L, "anthropic", "191919", [], "ai"),
  ("claude", "Claude", L, "claude", "D97757", ["sonnet", "opus", "haiku"], "ai"),
  ("gemini", "Gemini", L, "gemini", "3186FF", ["bard"], "ai"),
  ("google", "Google", S, "google", None, [], "company"),
  ("meta", "Meta", L, "meta", "0866FF", ["llama"], "ai"),
  ("mistral", "Mistral AI", L, "mistral", "FA520F", ["mistral", "mixtral", "codestral", "devstral"], "ai"),
  ("deepseek", "DeepSeek", L, "deepseek", "4D6BFE", [], "ai"),
  ("xai", "xAI", L, "xai", "000000", [], "ai"),
  ("grok", "Grok", L, "grok", "000000", [], "ai"),
  ("cohere", "Cohere", L, "cohere", "39594D", ["command-r"], "ai"),
  ("perplexity", "Perplexity", L, "perplexity", "1FB8CD", ["sonar"], "ai"),
  ("ollama", "Ollama", L, "ollama", "000000", [], "ai"),
  ("huggingface", "Hugging Face", L, "huggingface", "FFD21E", ["hf"], "ai"),
  ("qwen", "Qwen", L, "qwen", "615CED", [], "ai"),
  ("groq", "Groq", L, "groq", "F55036", [], "ai"),
  ("kimi", "Kimi", L, "kimi", "000000", ["moonshot"], "ai"),
  ("cursor", "Cursor", L, "cursor", "000000", [], "ai"),
  ("aws", "AWS", L, "aws", "FF9900", ["amazon web services", "bedrock"], "company"),
  ("zapier", "Zapier", L, "zapier", "FF4F00", [], "integration"),
  ("github", "GitHub", S, "github", None, ["gh"], "integration"),
  ("gitlab", "GitLab", S, "gitlab", None, [], "integration"),
  ("linear", "Linear", S, "linear", None, [], "integration"),
  ("figma", "Figma", S, "figma", None, [], "integration"),
  ("vercel", "Vercel", S, "vercel", None, [], "integration"),
  ("notion", "Notion", S, "notion", None, [], "integration"),
  ("stripe", "Stripe", S, "stripe", None, [], "integration"),
  ("postgresql", "PostgreSQL", S, "postgresql", None, ["postgres", "pg"], "integration"),
  ("supabase", "Supabase", S, "supabase", None, [], "integration"),
  ("jira", "Jira", S, "jira", None, [], "integration"),
  ("gmail", "Gmail", S, "gmail", None, [], "integration"),
  ("googledrive", "Google Drive", S, "googledrive", None, [], "integration"),
  ("googlecalendar", "Google Calendar", S, "googlecalendar", None, ["gcal"], "integration"),
  ("discord", "Discord", S, "discord", None, [], "integration"),
  ("sentry", "Sentry", S, "sentry", None, [], "integration"),
  ("cloudflare", "Cloudflare", S, "cloudflare", None, [], "integration"),
  ("docker", "Docker", S, "docker", None, [], "integration"),
  ("zoom", "Zoom", S, "zoom", None, [], "integration"),
  ("shopify", "Shopify", S, "shopify", None, [], "integration"),
  ("intercom", "Intercom", S, "intercom", None, [], "integration"),
  ("hubspot", "HubSpot", S, "hubspot", None, [], "integration"),
  ("resend", "Resend", S, "resend", None, [], "integration"),
  ("redis", "Redis", S, "redis", None, [], "integration"),
  ("airtable", "Airtable", S, "airtable", None, [], "integration"),
  ("netlify", "Netlify", S, "netlify", None, [], "integration"),
  ("npm", "npm", S, "npm", None, [], "integration"),
  ("rust", "Rust", S, "rust", None, [], "integration"),
  ("python", "Python", S, "python", None, [], "integration"),
  ("typescript", "TypeScript", S, "typescript", None, ["ts"], "integration"),
  ("react", "React", S, "react", None, [], "integration"),
  ("nextdotjs", "Next.js", S, "nextdotjs", None, ["next.js", "nextjs"], "integration"),
  ("v0", "v0", L, "v0", "000000", ["v0.dev"], "ai"),
  ("clickup", "ClickUp", S, "clickup", None, [], "integration"),
  ("dropbox", "Dropbox", S, "dropbox", None, [], "integration"),
  ("slack", "Slack", O, "slack", "4A154B", [], "integration"),
]

# Full-colour versions (`COLOURED`): Lobe Icons' own -color files, and marks drawn by Oxyn from
# basic shapes in the official colours where no permissive multi-colour file exists.
COLOUR_LOBE = {"claude": "claude-color", "gemini": "gemini-color", "google": "google-color", "figma": "figma-color"}

def body(src, f):
    if src == O:
        return "0 0 24 24", OWN[f], "nonzero"
    t = open(f"{B}/{'lobe/icons' if src == L else 'si/icons'}/{f}.svg").read()
    vb = re.search(r'viewBox="([^"]+)"', t).group(1)
    inner = re.search(r"<svg[^>]*>(.*)</svg>", t, re.S).group(1)
    inner = re.sub(r"<title>.*?</title>", "", inner, flags=re.S)
    assert "fill=\"#" not in inner and "<defs" not in inner and "url(" not in inner, f
    rule = "evenodd" if src == L else "nonzero"
    return vb, inner.strip(), rule

rows = []
for (i, title, src, f, hx, al, kind) in ICONS:
    vb, inner, rule = body(src, f)
    if hx is None: hx = si_hex[f]
    rows.append((i, title, src, vb, inner, rule, hx.upper(), al, kind))

def rs(s): return "r##\"" + s + "\"##"
out = ['//! Brand and model marks, inlined from permissively licensed icon sets (generated by',
       '//! `scripts/brand/gen_brand.py`; do not edit by hand). AI model and provider marks come from Lobe Icons',
       '//! (`@lobehub/icons-static-svg`, MIT); company and integration marks from Simple Icons (CC0).',
       '//! Credits are in `THIRD_PARTY_NOTICES.md`. Trademarks belong to their owners.',
       '//!',
       '//! Marks are monochrome by default (`currentColor`, per the colour rule); [`BrandIcon::svg`]',
       '//! with `coloured = true` fills them with the brand colour instead. Renderers draw them',
       '//! (Dioxus `BrandIcon`, GPUI `brand_icon`); [`brand`] resolves an id, title, alias or a model',
       '//! name ("claude-sonnet-4", "gpt-4o") to a mark, and callers fall back to a monogram on `None`.',
       '',
       '/// Where a mark comes from (its licence).',
       '#[derive(Clone, Copy, Debug, PartialEq, Eq)]',
       'pub enum BrandSource {',
       '    /// Simple Icons, CC0 1.0.',
       '    SimpleIcons,',
       '    /// Lobe Icons (`@lobehub/icons-static-svg`), MIT.',
       '    LobeIcons,',
       '    /// Drawn by Oxyn from basic shapes in the official colours (MIT).',
       '    Oxyn,',
       '}',
       '',
       '/// What a mark stands for.',
       '#[derive(Clone, Copy, Debug, PartialEq, Eq)]',
       'pub enum BrandKind {',
       '    /// An AI model family or provider.',
       '    Ai,',
       '    /// A company.',
       '    Company,',
       '    /// A tool or integration.',
       '    Integration,',
       '}',
       '',
       '/// One brand mark.',
       '#[derive(Clone, Copy, Debug, PartialEq, Eq)]',
       'pub struct BrandIcon {',
       '    /// Stable id (`openai`, `github`).',
       '    pub id: &\'static str,',
       '    /// Display name.',
       '    pub title: &\'static str,',
       '    /// Source set.',
       '    pub source: BrandSource,',
       '    /// What it stands for.',
       '    pub kind: BrandKind,',
       '    /// SVG `viewBox`.',
       '    pub view_box: &\'static str,',
       '    /// Inner SVG markup (paths without fills).',
       '    pub body: &\'static str,',
       '    /// `evenodd` or `nonzero`.',
       '    pub fill_rule: &\'static str,',
       '    /// Brand colour, `RRGGBB`.',
       '    pub hex: &\'static str,',
       '    /// Other names that resolve to this mark (lowercase).',
       '    pub aliases: &\'static [&\'static str],',
       '}',
       '',
       'impl BrandIcon {',
       '    /// A standalone SVG document: `currentColor` fill, or the brand colour when `coloured`.',
       '    pub fn svg(&self, coloured: bool) -> String {',
       '        let fill = if coloured { format!("#{}", self.hex) } else { "currentColor".to_string() };',
       '        format!(',
       '            "<svg xmlns=\\"http://www.w3.org/2000/svg\\" viewBox=\\"{}\\" fill=\\"{}\\" fill-rule=\\"{}\\" clip-rule=\\"{}\\">{}</svg>",',
       '            self.view_box, fill, self.fill_rule, self.fill_rule, self.body',
       '        )',
       '    }',
       '    /// A brand colour that reads on both themes: near-black marks (OpenAI, xAI, Vercel)',
       '    /// return `None` so the caller keeps the text colour.',
       '    pub fn display_hex(&self) -> Option<&\'static str> {',
       '        let v = u32::from_str_radix(self.hex, 16).unwrap_or(0);',
       '        let (r, g, b) = ((v >> 16) & 255, (v >> 8) & 255, v & 255);',
       '        if r + g + b < 120 || r + g + b > 700 { None } else { Some(self.hex) }',
       '    }',
       '}',
       '',
       '/// Every mark, in a stable order.',
       'pub const BRANDS: &[BrandIcon] = &[']
for (i, title, src, vb, inner, rule, hx, al, kind) in rows:
    out.append("    BrandIcon {")
    out.append(f'        id: "{i}",')
    out.append(f'        title: "{title}",')
    out.append(f'        source: BrandSource::{ {L: "LobeIcons", S: "SimpleIcons", O: "Oxyn"}[src] },')
    out.append(f'        kind: BrandKind::{ {"ai":"Ai","company":"Company","integration":"Integration"}[kind] },')
    out.append(f'        view_box: "{vb}",')
    out.append(f'        body: {rs(inner)},')
    out.append(f'        fill_rule: "{rule}",')
    out.append(f'        hex: "{hx}",')
    out.append("        aliases: &[" + ", ".join(f'"{a}"' for a in al) + "],")
    out.append("    },")
out.append("];")
# Full-colour bodies.
def colour_body(i):
    if i in OWN_COLOURED: return OWN_COLOURED[i]
    t = open(f"{B}/lobe/icons/{COLOUR_LOBE[i]}.svg").read()
    inner = re.search(r"<svg[^>]*>(.*)</svg>", t, re.S).group(1)
    return re.sub(r"<title>.*?</title>", "", inner, flags=re.S).strip()
out.append("")
out.append("/// Full-colour marks (id, inner SVG with its own fills) for brands whose logo has more than one")
out.append("/// colour: Lobe Icons' `-color` files, and Gmail, Google Drive, Slack and ClickUp drawn by Oxyn")
out.append("/// from basic shapes in the official colours. [`BrandIcon::svg_full`] uses them.")
out.append("pub const COLOURED: &[(&str, &str)] = &[")
for i in list(COLOUR_LOBE) + list(OWN_COLOURED):
    out.append(f'    ("{i}", {rs(colour_body(i))}),')
out.append("];")
out.append('''
impl BrandIcon {
    /// The full-colour inner SVG when the mark has more than one colour.
    pub fn coloured_body(&self) -> Option<&'static str> {
        COLOURED.iter().find(|(i, _)| *i == self.id).map(|(_, b)| *b)
    }
    /// The mark as it should look in colour: the multi-colour version when there is one, else the
    /// brand colour (near-black marks keep `currentColor` so they read on dark themes).
    pub fn svg_full(&self) -> String {
        match self.coloured_body() {
            Some(b) => format!(
                "<svg xmlns=\\"http://www.w3.org/2000/svg\\" viewBox=\\"{}\\">{}</svg>",
                self.view_box, b
            ),
            None if self.display_hex().is_some() => self.svg(true),
            None => self.svg(false),
        }
    }
}
''')
out.append('''
fn norm(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == ' ' || *c == '-')
        .collect()
}

/// Resolves an id, title or alias exactly, then a model or product name by its leading word
/// (`claude-sonnet-4.5` → Claude, `gpt-4o mini` → OpenAI, `llama 3.1` → Meta). `None` when
/// nothing matches, so the caller keeps its monogram.
pub fn brand(name: &str) -> Option<&'static BrandIcon> {
    let n = norm(name);
    if n.is_empty() {
        return None;
    }
    let squash: String = n.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let exact = BRANDS.iter().find(|b| {
        b.id == squash
            || norm(b.title) == n
            || b.aliases.iter().any(|a| *a == n || *a == squash)
    });
    if exact.is_some() {
        return exact;
    }
    // A domain ("github.com", "docs.stripe.com", "https://linear.app/x") resolves by its site label.
    if !n.contains(' ') && n.contains('.') {
        let host = n.split("//").last().unwrap_or("").split('/').next().unwrap_or("");
        let labels: Vec<&str> = host.split('.').filter(|l| !l.is_empty() && *l != "www").collect();
        if labels.len() >= 2 {
            let site = labels[labels.len() - 2];
            if let Some(b) = BRANDS.iter().find(|b| b.id == site || b.aliases.contains(&site)) {
                return Some(b);
            }
        }
    }
    let head = n
        .split([' ', '-', '/', ':', '_'])
        .next()
        .unwrap_or("");
    let head: String = head.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if head.is_empty() {
        return None;
    }
    BRANDS.iter().find(|b| {
        b.id == head
            || b.aliases.iter().any(|a| *a == head)
            || (head.len() >= 3 && b.aliases.iter().any(|a| a.len() >= 2 && head.starts_with(a) && head[a.len()..].chars().next().is_some_and(|c| c.is_ascii_digit())))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_ids_titles_aliases_and_models() {
        assert_eq!(brand("openai").map(|b| b.id), Some("openai"));
        assert_eq!(brand("GitHub").map(|b| b.id), Some("github"));
        assert_eq!(brand("GH").map(|b| b.id), Some("github"));
        assert_eq!(brand("Claude Sonnet 4.5").map(|b| b.id), Some("claude"));
        assert_eq!(brand("sonnet-4").map(|b| b.id), Some("claude"));
        assert_eq!(brand("gpt-4o mini").map(|b| b.id), Some("openai"));
        assert_eq!(brand("o3").map(|b| b.id), Some("openai"));
        assert_eq!(brand("Llama 3.1 70B").map(|b| b.id), Some("meta"));
        assert_eq!(brand("Mistral AI").map(|b| b.id), Some("mistral"));
        assert_eq!(brand("Google Drive").map(|b| b.id), Some("googledrive"));
        assert_eq!(brand("Postgres").map(|b| b.id), Some("postgresql"));
        assert_eq!(brand("github.com").map(|b| b.id), Some("github"));
        assert_eq!(brand("https://docs.stripe.com/api").map(|b| b.id), Some("stripe"));
        assert_eq!(brand("rfc-editor.org"), None);
        assert_eq!(brand("Acme Corp"), None);
        assert_eq!(brand(""), None);
        assert_eq!(brand("L"), None);
    }

    #[test]
    fn svg_is_monochrome_unless_coloured() {
        let b = brand("claude").unwrap();
        assert!(b.svg(false).contains("fill=\\"currentColor\\""));
        assert!(b.svg(true).contains("#D97757"));
        assert!(BRANDS.iter().all(|b| !b.body.contains("fill=\\"#")));
        assert_eq!(brand("openai").unwrap().display_hex(), None);
        assert_eq!(brand("claude").unwrap().display_hex(), Some("D97757"));
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<_> = BRANDS.iter().map(|b| b.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), BRANDS.len());
    }
}''')
open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../crates/oxyn-core/src/brand.rs"), "w").write("\n".join(out) + "\n")
print(len(rows), sum(len(r[4]) for r in rows))
