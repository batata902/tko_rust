use std::{ops::Add, sync::atomic::{AtomicBool, Ordering}};
use std::borrow::Cow;
use unicode_normalization::UnicodeNormalization;

static ENABLED: AtomicBool = AtomicBool::new(true);

#[derive(PartialEq)]
pub enum AddValue <'a> {
    Str(Cow<'a, String>),
    Token(Token),
    Text(Text),
    Tuple((String, String))
}


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

#[derive(Clone)]
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

#[derive(PartialEq, Clone)]
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

    pub fn get_str(&self) -> String {
        self.data.iter().map(|t| t.text.as_str()).collect()
    }

    pub fn add(&mut self, mut value: Option<AddValue>) -> &mut Self {
        let Some(value) = value else {
            return self;
        };

        match value {
            AddValue::Str(valor) => {
                let norm_value: String = valor.nfc().collect();
                if !norm_value.is_empty() {
                    for c in norm_value.chars() {
                        self.data.push(Token { text: c.to_string(), fmt: "".to_string() });
                    }
                }
            },
            AddValue::Token(mut token) => {
                token.text = token.text.nfc().collect();
                if token.text != "" {
                    let fmt = token.fmt;
                    for c in token.text.chars() {
                        self.data.push(Token { text: c.to_string(), fmt: fmt.clone() });
                    }
                }
            },
            AddValue::Tuple(tupl) => {
                let (fmt, text) = tupl;
                let text: String = text.nfc().collect();
                if text != "" {
                    for c in text.chars() {
                        self.data.push(Token::new(c.to_string(), fmt.clone()));
                    }
                }
            },
            AddValue::Text(val) => {
                self.data.extend(val.data);
            }
        }

        self
    }

    pub fn addf(&mut self, fmt: String, value: Option<AddValue>) -> &mut Self {
        let Some(value) = value else {
            return self;
        };

        match value {
            AddValue::Str(val) => {
                self.add(Some(AddValue::Token(Token::new(val.to_string(), fmt))));
            },
            AddValue::Token(token) => {
                self.add(Some(AddValue::Token(Token::new(token.text, fmt))));
            },
            AddValue::Text(text) => {
                self.add(Some(AddValue::Token(Token::new(text.get_str(), fmt))));
            },
            AddValue::Tuple(tupl) => {
                self.add(Some(AddValue::Token(Token::new(tupl.1, fmt))));
            }
        }

        self
    }
}