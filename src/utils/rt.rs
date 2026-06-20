use std::sync::LazyLock;
use std::collections::HashMap;

type Run = (String, String);


static FG: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m: HashMap<&'static str, &'static str> = HashMap::new();

    m.insert("k", "\x1b[30m");
    m.insert("r", "\x1b[31m");
    m.insert("g", "\x1b[32m");
    m.insert("y", "\x1b[33m");
    m.insert("b", "\x1b[34m");
    m.insert("m", "\x1b[35m");
    m.insert("c", "\x1b[36m");
    m.insert("w", "\x1b[37m");

    m
});

enum ATTR {
    Reset,
    Bold,
    Underline,
    Italic,
    Reverse,
    Strikethroungh
}

enum FG {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White
}

impl ATTR {
    fn ansi(&self) -> &'static str {
        match self {
            ATTR::Reset => "\x1b[0m",
            ATTR::Bold => "\x1b[1m",
            ATTR::Underline => "\x1b[4m",
            ATTR::Italic => "\x1b[3m",
            ATTR::Reverse => "\x1b[7m",
            ATTR::Strikethroungh => "\x1b[9m"
        }
    }
}

impl FG {
    fn ansi(&self) -> &'static str {
        match self {
            FG::Black => "\x1b[30m",
            FG::Red => "\x1b[31m",
            FG::Green => "\x1b[32m",
            FG::Yellow => "\x1b[33m",
            FG::Blue => "\x1b[34m",
            FG::Magenta => "\x1b[35m",
            FG::Cyan => "\x1b[36m",
            FG::White => "\x1b[37m"
        }
    }
}


pub fn normalize_style(style: &str) -> String {
    let mut fg: Vec<String> = Vec::new();
    let mut bg: Vec<String> = Vec::new();
    let mut attr: Vec<String> = Vec::new();

    for c in style.chars() {
        if FG
    }
}

pub struct RT {
    runs: Vec<Run>
}

impl RT {
    pub fn new(text: Option<&str>, style: &str, runs: Option<Vec<Run>>) -> Self {
        let mut runs: Vec<Run> = Vec::new();
        if !runs.is_none() {
            runs = self._merge(runs);
        }
    }

    pub fn _merge(&self, runs: &Vec<Run>) -> Vec<Run> {
        let merged: Vec<Run> = Vec::new();
        for (style, text) in runs {
            if text.is_empty() {
                continue;;
            }
            let style = normalize_style(style);
        }
    }
}