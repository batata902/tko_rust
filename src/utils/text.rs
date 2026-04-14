use std::sync::atomic::{AtomicBool, Ordering};


static ENABLED: AtomicBool = AtomicBool::new(true);
pub struct AnsiColor;

impl AnsiColor {
    pub fn get(key: char) -> &'static str {
        match key {
            '.' => "\033[0m",
            '*' => "\033[1m",
            '/' => "\033[3m",
            '_' => "\033[4m",

            'k' => "\033[30m",
            'r' => "\033[31m",
            'g' => "\033[32m",
            'y' => "\033[33m",
            'b' => "\033[34m",
            'm' => "\033[35m",
            'c' => "\033[36m",
            'w' => "\033[37m",

            'K' => "\033[40m",
            'W' => "\033[47m",
            'R' => "\033[41m",
            'G' => "\033[42m",
            'Y' => "\033[43m",
            'B' => "\033[44m",
            'M' => "\033[45m",
            'C' => "\033[46m",
            _ => ""
        }
    }

    pub fn colour(modifiers: &str, text: &str) -> String {
        if !ENABLED.load(Ordering::Relaxed) {
            return text.to_string();
        }
        let mut output = String::new();
        for m in modifiers.chars() {
            let val = AnsiColor::get(m);
            if val != "".to_string() {
                output += &val;
            }
        }
        output += text;
        if modifiers.len() > 0 {
            output += AnsiColor::get('.');
        }
        output
    }
}

pub struct Token {
    text: String,
    fmt: String
}


impl Token {
    pub fn new(text: String, fmt: String) -> Self {
        Self { text, fmt }
    }

    pub fn set_text(&mut self, text: String) -> &mut Self {
        self.text = text;

        self
    }

    pub fn set_fmt(&mut self, fmt: String) -> &mut Self {
        self.fmt = fmt;

        self
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.fmt == other.fmt
    }    
}

pub struct Text {
    value: String,
    data: Vec<Token>,
    fmt: String
}

impl Text {
    pub fn new(value: Option<String>, fmt: Option<String>) -> Self {
        Self { 
            data: Vec::new(),
            value: value.unwrap_or("".to_string()), 
            fmt: fmt.unwrap_or("".to_string()) 
        }
    }
}