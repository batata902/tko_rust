use std::cell::Cell;
use std::fmt;
use unicode_width::UnicodeWidthChar;

// ─────────────────────────────────────────────
// ANSI helpers
// ─────────────────────────────────────────────

fn ansi_code(c: char) -> Option<&'static str> {
    match c {
        '.' => Some("\x1b[0m"),
        '*' => Some("\x1b[1m"),
        '_' => Some("\x1b[4m"),
        '/' => Some("\x1b[3m"),
        '~' => Some("\x1b[7m"),
        '!' => Some("\x1b[9m"),
        'k' => Some("\x1b[30m"),
        'r' => Some("\x1b[31m"),
        'g' => Some("\x1b[32m"),
        'y' => Some("\x1b[33m"),
        'b' => Some("\x1b[34m"),
        'm' => Some("\x1b[35m"),
        'c' => Some("\x1b[36m"),
        'w' => Some("\x1b[37m"),
        'K' => Some("\x1b[40m"),
        'R' => Some("\x1b[41m"),
        'G' => Some("\x1b[42m"),
        'Y' => Some("\x1b[43m"),
        'B' => Some("\x1b[44m"),
        'M' => Some("\x1b[45m"),
        'C' => Some("\x1b[46m"),
        'W' => Some("\x1b[47m"),
        _ => None,
    }
}

fn is_fg(c: char) -> bool { matches!(c, 'k'|'r'|'g'|'y'|'b'|'m'|'c'|'w') }
fn is_bg(c: char) -> bool { matches!(c, 'K'|'R'|'G'|'Y'|'B'|'M'|'C'|'W') }
fn is_attr(c: char) -> bool { matches!(c, '*'|'_'|'/'|'~'|'!') }

fn ansi_string(style: &str) -> String {
    style.chars().filter_map(ansi_code).collect()
}

pub fn normalize_style(style: &str) -> String {
    let mut fg: Option<char> = None;
    let mut bg: Option<char> = None;
    let mut attr: Vec<char> = Vec::new();
    for c in style.chars() {
        if is_fg(c)        { fg = Some(c); }
        else if is_bg(c)   { bg = Some(c); }
        else if is_attr(c) && !attr.contains(&c) { attr.push(c); }
    }
    let mut out = String::new();
    out.extend(attr.iter());
    if let Some(c) = fg { out.push(c); }
    if let Some(c) = bg { out.push(c); }
    out
}

fn combine_styles(base: &str, overlay: &str) -> String {
    let mut fg: Option<char> = None;
    let mut bg: Option<char> = None;
    let mut attr: Vec<char> = Vec::new();
    for c in base.chars().chain(overlay.chars()) {
        if is_fg(c)        { fg = Some(c); }
        else if is_bg(c)   { bg = Some(c); }
        else if is_attr(c) && !attr.contains(&c) { attr.push(c); }
    }
    let mut out = String::new();
    if let Some(c) = fg { out.push(c); }
    if let Some(c) = bg { out.push(c); }
    out.extend(attr.iter());
    out
}

// ─────────────────────────────────────────────
// RenderMode / RenderConfig
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode { #[default] Ansi, Plain, Debug }

thread_local! {
    static RENDER_MODE:    Cell<RenderMode> = Cell::new(RenderMode::Ansi);
    static RENDER_ENABLED: Cell<bool>       = Cell::new(true);
}

pub struct RenderConfig;
impl RenderConfig {
    pub fn mode()            -> RenderMode { RENDER_MODE.with(|m| m.get()) }
    pub fn set_mode(m: RenderMode)         { RENDER_MODE.with(|c| c.set(m)); }
    pub fn enabled()         -> bool       { RENDER_ENABLED.with(|e| e.get()) }
    pub fn set_enabled(v: bool)            { RENDER_ENABLED.with(|c| c.set(v)); }
}

// ─────────────────────────────────────────────
// Run = (style, text)
// ─────────────────────────────────────────────

pub type Run = (String, String);

// ─────────────────────────────────────────────
// RText
// ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RText { runs: Vec<Run> }

impl RText {
    // ── Constructors ────────────────────────

    pub fn new() -> Self { RText { runs: Vec::new() } }

    pub fn styled(text: impl Into<String>, style: impl AsRef<str>) -> Self {
        let text = text.into();
        let style = normalize_style(style.as_ref());
        if text.is_empty() { return Self::new(); }
        RText { runs: vec![(style, text)] }
    }

    pub fn plain_text(text: impl Into<String>) -> Self { Self::styled(text, "") }

    pub fn run(style: impl AsRef<str>, text: impl Into<String>) -> Self {
        Self::styled(text, style)
    }

    pub fn from_runs(runs: impl IntoIterator<Item = Run>) -> Self {
        RText { runs: Self::merge_runs(runs) }
    }

    fn merge_runs(runs: impl IntoIterator<Item = Run>) -> Vec<Run> {
        let mut merged: Vec<Run> = Vec::new();
        for (style, text) in runs {
            if text.is_empty() { continue; }
            let style = normalize_style(&style);
            match merged.last_mut() {
                Some(last) if last.0 == style => last.1.push_str(&text),
                _ => merged.push((style, text)),
            }
        }
        merged
    }

    // ── Combination ─────────────────────────

    pub fn concat(texts: impl IntoIterator<Item = RText>) -> Self {
        RText::from_runs(texts.into_iter().flat_map(|t| t.runs))
    }

    pub fn join(texts: impl IntoIterator<Item = RText>, sep: &RText) -> Self {
        let mut iter = texts.into_iter();
        let Some(first) = iter.next() else { return Self::new() };
        let mut out = first;
        for t in iter { out = out.append(sep).append(&t); }
        out
    }

    pub fn append(&self, other: &RText) -> RText {
        RText::from_runs(self.runs.iter().chain(other.runs.iter()).cloned())
    }

    // ── Rendering ───────────────────────────

    pub fn render(&self, mode: Option<RenderMode>) -> String {
        let mode = mode.unwrap_or_else(RenderConfig::mode);
        if !RenderConfig::enabled() || mode == RenderMode::Plain {
            return self.plain();
        }
        if mode == RenderMode::Debug {
            return self.runs.iter().map(|(s,t)| format!("[{}]{}", s, t)).collect();
        }
        let mut out = String::new();
        for (style, text) in &self.runs {
            if style.is_empty() { out.push_str(text); }
            else { out.push_str(&ansi_string(style)); out.push_str(text); out.push_str("\x1b[0m"); }
        }
        out
    }

    pub fn plain(&self) -> String {
        self.runs.iter().map(|(_, t)| t.as_str()).collect()
    }

    // ── Metrics ─────────────────────────────

    /// Display width (handles CJK wide chars).
    pub fn width(&self) -> usize {
        self.runs.iter()
            .flat_map(|(_, t)| t.chars())
            .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
            .sum()
    }

    /// Character count (not display width).
    pub fn char_len(&self) -> usize {
        self.runs.iter().map(|(_, t)| t.chars().count()).sum()
    }

    pub fn is_empty(&self) -> bool { self.runs.is_empty() }

    // ── Slicing ─────────────────────────────

    pub fn slice(&self, start: usize, end: usize) -> RText {
        if end <= start { return Self::new(); }
        let mut result: Vec<Run> = Vec::new();
        let mut i = 0usize;
        for (style, text) in &self.runs {
            if i >= end { break; }
            let mut part = String::new();
            for ch in text.chars() {
                if i >= start && i < end { part.push(ch); }
                i += 1;
            }
            if !part.is_empty() {
                match result.last_mut() {
                    Some(last) if last.0 == *style => last.1.push_str(&part),
                    _ => result.push((style.clone(), part)),
                }
            }
        }
        RText::from_runs(result)
    }

    // ── Word-wrap / truncate ─────────────────

    pub fn wrap(&self, max_width: usize) -> Vec<RText> {
        let mut lines: Vec<RText> = Vec::new();
        let mut cur: Vec<Run>     = Vec::new();
        let mut cur_w = 0usize;

        let flush = |lines: &mut Vec<RText>, cur: &mut Vec<Run>| {
            if !cur.is_empty() {
                lines.push(RText::from_runs(cur.drain(..).collect::<Vec<_>>()));
            }
        };

        for (style, text) in &self.runs {
            for ch in text.chars() {
                if ch == '\n' { flush(&mut lines, &mut cur); cur_w = 0; continue; }
                let w = UnicodeWidthChar::width(ch).unwrap_or(0);
                if cur_w + w > max_width { flush(&mut lines, &mut cur); cur_w = 0; }
                match cur.last_mut() {
                    Some(last) if last.0 == *style => last.1.push(ch),
                    _ => cur.push((style.clone(), ch.to_string())),
                }
                cur_w += w;
            }
        }
        flush(&mut lines, &mut cur);
        lines
    }

    pub fn truncate(&self, max_width: usize) -> RText {
        if max_width == 0 { return Self::new(); }
        let mut result: Vec<Run> = Vec::new();
        let mut cur_w = 0usize;
        'outer: for (style, text) in &self.runs {
            let mut part = String::new();
            for ch in text.chars() {
                let w = UnicodeWidthChar::width(ch).unwrap_or(0);
                if cur_w + w > max_width {
                    // push whatever we collected before stopping
                    if !part.is_empty() {
                        match result.last_mut() {
                            Some(last) if last.0 == *style => last.1.push_str(&part),
                            _ => result.push((style.clone(), part)),
                        }
                    }
                    break 'outer;
                }
                part.push(ch); cur_w += w;
            }
            if !part.is_empty() {
                match result.last_mut() {
                    Some(last) if last.0 == *style => last.1.push_str(&part),
                    _ => result.push((style.clone(), part)),
                }
            }
        }
        RText::from_runs(result)
    }

    // ── String-like ops ─────────────────────

    pub fn to_upper(&self) -> RText {
        RText::from_runs(self.runs.iter().map(|(s, t)| (s.clone(), t.to_uppercase())))
    }

    pub fn to_lower(&self) -> RText {
        RText::from_runs(self.runs.iter().map(|(s, t)| (s.clone(), t.to_lowercase())))
    }

    pub fn replace_text(&self, old: &str, new: &RText, count: Option<usize>) -> RText {
        if old.is_empty() { return self.clone(); }
        let n = old.chars().count();
        let chars: Vec<(char, &str)> = self.runs.iter()
            .flat_map(|(s, t)| t.chars().map(move |c| (c, s.as_str())))
            .collect();

        let mut result: Vec<Run> = Vec::new();
        let mut i = 0usize;
        let mut replaced = 0usize;
        let max = count.unwrap_or(usize::MAX);

        let push = |result: &mut Vec<Run>, style: &str, text: &str| {
            if text.is_empty() { return; }
            match result.last_mut() {
                Some(last) if last.0 == style => last.1.push_str(text),
                _ => result.push((style.to_string(), text.to_string())),
            }
        };

        while i < chars.len() {
            let window: String = chars[i..].iter().take(n).map(|(c,_)| *c).collect();
            if replaced < max && window == old {
                for (s, t) in new.runs() { push(&mut result, s, t); }
                i += n; replaced += 1;
            } else {
                let (c, s) = chars[i];
                push(&mut result, s, &c.to_string());
                i += 1;
            }
        }
        RText::from_runs(result)
    }

    pub fn split_text(&self, sep: &str) -> Vec<RText> {
        let sep_len = sep.chars().count();
        let chars: Vec<(char, &str)> = self.runs.iter()
            .flat_map(|(s, t)| t.chars().map(move |c| (c, s.as_str())))
            .collect();

        let mut parts: Vec<Vec<Run>> = vec![Vec::new()];
        let mut i = 0usize;
        while i < chars.len() {
            let window: String = chars[i..].iter().take(sep_len).map(|(c,_)| *c).collect();
            if window == sep {
                parts.push(Vec::new()); i += sep_len;
            } else {
                let (c, s) = chars[i];
                let cur = parts.last_mut().unwrap();
                match cur.last_mut() {
                    Some(last) if last.0 == s => last.1.push(c),
                    _ => cur.push((s.to_string(), c.to_string())),
                }
                i += 1;
            }
        }
        parts.into_iter().map(RText::from_runs).collect()
    }

    pub fn repeat(&self, n: usize) -> RText {
        RText::concat(std::iter::repeat(self.clone()).take(n))
    }

    // ── Alignment ───────────────────────────

    pub fn center(&self, width: usize, fill: &RText) -> RText {
        let len = self.char_len();
        if len >= width { return self.clone(); }
        let missing = width - len;
        let left = missing / 2;
        fill.repeat(left).append(self).append(&fill.repeat(missing - left))
    }

    pub fn ljust(&self, width: usize, fill: &RText) -> RText {
        let len = self.char_len();
        if len >= width { return self.clone(); }
        self.append(&fill.repeat(width - len))
    }

    pub fn rjust(&self, width: usize, fill: &RText) -> RText {
        let len = self.char_len();
        if len >= width { return self.clone(); }
        fill.repeat(width - len).append(self)
    }

    // ── Style manipulation ───────────────────

    pub fn set_style(&self, style: &str) -> RText {
        let style = normalize_style(style);
        RText::from_runs(self.runs.iter().map(|(_, t)| (style.clone(), t.clone())))
    }

    pub fn add_style(&self, style: &str) -> RText {
        RText::from_runs(
            self.runs.iter().map(|(s, t)| (normalize_style(&(s.clone() + style)), t.clone()))
        )
    }

    pub fn clear_style(&self) -> RText {
        RText::from_runs(self.runs.iter().map(|(_, t)| (String::new(), t.clone())))
    }

    // ── parse() template mini-language ──────
    //
    // Syntax (same as Python):
    //   [style]   → set/combine style
    //   [.]       → reset style
    //   [.style]  → reset then set style
    //   []        → next positional argument
    //   [[  ]]    → escaped brackets
    //
    pub fn parse(template: &str, args: &[RText]) -> RText {
        let mut runs: Vec<Run> = Vec::new();
        let mut buf   = String::new();
        let mut style = String::new();
        let mut arg_i = 0usize;
        let chars: Vec<char> = template.chars().collect();
        let mut i = 0usize;

        macro_rules! flush {
            () => { if !buf.is_empty() { runs.push((style.clone(), buf.clone())); buf.clear(); } }
        }

        while i < chars.len() {
            let c = chars[i];
            if c == '[' {
                if i + 1 < chars.len() && chars[i+1] == '[' { buf.push('['); i += 2; continue; }
                let j = chars[i..].iter().position(|&x| x == ']').map(|p| i + p);
                let Some(j) = j else { buf.push(c); i += 1; continue; };
                let token: String = chars[i+1..j].iter().collect();
                flush!();
                if token.is_empty() {
                    if arg_i < args.len() {
                        for (s, t) in args[arg_i].runs() {
                            runs.push((combine_styles(&style, s), t.clone()));
                        }
                        arg_i += 1;
                    }
                } else if token == "." {
                    style.clear();
                } else if let Some(rest) = token.strip_prefix('.') {
                    style = normalize_style(rest);
                } else {
                    style = combine_styles(&style, &token);
                }
                i = j + 1;
                continue;
            }
            if c == ']' && i+1 < chars.len() && chars[i+1] == ']' { buf.push(']'); i += 2; continue; }
            buf.push(c); i += 1;
        }
        if !buf.is_empty() { runs.push((style, buf)); }
        RText::from_runs(runs)
    }

    pub fn runs(&self) -> &[(String, String)] { &self.runs }
}

// ─────────────────────────────────────────────
// Operator overloads
// ─────────────────────────────────────────────

impl std::ops::Add for RText {
    type Output = RText;
    fn add(self, rhs: RText) -> RText { self.append(&rhs) }
}
impl std::ops::Add<&RText> for RText {
    type Output = RText;
    fn add(self, rhs: &RText) -> RText { self.append(rhs) }
}
impl std::ops::Add<&str> for RText {
    type Output = RText;
    fn add(self, rhs: &str) -> RText { self.append(&RText::plain_text(rhs)) }
}

impl fmt::Display for RText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render(None))
    }
}

// ─────────────────────────────────────────────
// RBuffer  (mutable builder)
// ─────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct RBuffer { runs: Vec<Run> }

impl RBuffer {
    pub fn new() -> Self { Self::default() }

    pub fn add_str(&mut self, text: &str, style: &str) -> &mut Self {
        self.push_run(style, text); self
    }
    pub fn add_text(&mut self, text: &RText) -> &mut Self {
        for (s, t) in text.runs() { self.push_run(s, t); } self
    }
    pub fn add_buf(&mut self, other: &RBuffer) -> &mut Self {
        for (s, t) in &other.runs { self.push_run(s, t); } self
    }
    pub fn run(&mut self, style: &str, text: &str) -> &mut Self {
        self.push_run(style, text); self
    }
    pub fn clear(&mut self) -> &mut Self { self.runs.clear(); self }

    fn push_run(&mut self, style: &str, text: &str) {
        if text.is_empty() { return; }
        let style = normalize_style(style);
        match self.runs.last_mut() {
            Some(last) if last.0 == style => last.1.push_str(text),
            _ => self.runs.push((style, text.to_string())),
        }
    }

    pub fn to_text(&self) -> RText {
        RText::from_runs(self.runs.iter().cloned())
    }

    pub fn is_empty(&self) -> bool { self.runs.is_empty() }
}

impl fmt::Display for RBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text().render(None))
    }
}

// ─────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_keeps_last_fg_bg() {
        assert_eq!(normalize_style("rg"), "g");
        assert_eq!(normalize_style("rg*"), "*g");
    }

    #[test]
    fn plain_concatenation() {
        let a = RText::run("r", "Hello ");
        let b = RText::run("g", "World");
        assert_eq!((a + b).plain(), "Hello World");
    }

    #[test]
    fn wrap_basic() {
        let t = RText::plain_text("Hello World");
        let lines = t.wrap(7);
        assert_eq!(lines[0].plain(), "Hello W");
        assert_eq!(lines[1].plain(), "orld");
    }

    #[test]
    fn wrap_newline() {
        let t = RText::plain_text("foo\nbar");
        let lines = t.wrap(80);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].plain(), "foo");
        assert_eq!(lines[1].plain(), "bar");
    }

    #[test]
    fn truncate_basic() {
        let t = RText::plain_text("Hello World");
        assert_eq!(t.truncate(5).plain(), "Hello");
    }

    #[test]
    fn split_basic() {
        let t = RText::plain_text("a,b,c");
        let parts = t.split_text(",");
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[1].plain(), "b");
    }

    #[test]
    fn replace_basic() {
        let t = RText::plain_text("foo bar foo");
        let r = t.replace_text("foo", &RText::run("r", "baz"), None);
        assert_eq!(r.plain(), "baz bar baz");
    }

    #[test]
    fn replace_count() {
        let t = RText::plain_text("aaa");
        let r = t.replace_text("a", &RText::plain_text("b"), Some(2));
        assert_eq!(r.plain(), "bba");
    }

    #[test]
    fn slice_basic() {
        let t = RText::plain_text("Hello");
        assert_eq!(t.slice(1, 4).plain(), "ell");
    }

    #[test]
    fn alignment() {
        let t    = RText::plain_text("hi");
        let fill = RText::plain_text("-");
        assert_eq!(t.center(6, &fill).plain(), "--hi--");
        assert_eq!(t.ljust(5,  &fill).plain(), "hi---");
        assert_eq!(t.rjust(5,  &fill).plain(), "---hi");
    }

    #[test]
    fn parse_template() {
        let result = RText::parse("[r]Hello [] World", &[RText::run("b", "Blue")]);
        assert_eq!(result.plain(), "Hello Blue World");
    }

    #[test]
    fn parse_reset() {
        let result = RText::parse("[r]red[.]plain", &[]);
        assert_eq!(result.render(Some(RenderMode::Debug)), "[r]red[]plain");
    }

    #[test]
    fn rbuffer_builds() {
        let mut buf = RBuffer::new();
        buf.add_str("Hello ", "r").add_str("World", "g");
        assert_eq!(buf.to_text().plain(), "Hello World");
    }

    #[test]
    fn render_modes() {
        let t = RText::run("r", "hi");
        assert_eq!(t.render(Some(RenderMode::Debug)), "[r]hi");
        assert_eq!(t.render(Some(RenderMode::Plain)), "hi");
        assert!(t.render(Some(RenderMode::Ansi)).contains("\x1b[31m"));
    }

    #[test]
    fn style_ops() {
        let t = RText::run("r", "hi");
        assert_eq!(t.set_style("g").render(Some(RenderMode::Debug)), "[g]hi");
        assert_eq!(t.clear_style().render(Some(RenderMode::Debug)), "[]hi");
        assert_eq!(t.add_style("*").render(Some(RenderMode::Debug)), "[*r]hi");
    }

    #[test]
    fn repeat() {
        let t = RText::run("r", "ab");
        assert_eq!(t.repeat(3).plain(), "ababab");
    }

    #[test]
    fn upper_lower() {
        let t = RText::plain_text("Hello");
        assert_eq!(t.to_upper().plain(), "HELLO");
        assert_eq!(t.to_lower().plain(), "hello");
    }
}
