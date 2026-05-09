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
            '.' => "\x1b[0m",
            '*' => "\x1b[1m",
            '/' => "\x1b[3m",
            '_' => "\x1b[4m",

            'k' => "\x1b[30m",
            'r' => "\x1b[31m",
            'g' => "\x1b[32m",
            'y' => "\x1b[33m",
            'b' => "\x1b[34m",
            'm' => "\x1b[35m",
            'c' => "\x1b[36m",
            'w' => "\x1b[37m",

            'K' => "\x1b[40m",
            'W' => "\x1b[47m",
            'R' => "\x1b[41m",
            'G' => "\x1b[42m",
            'Y' => "\x1b[43m",
            'B' => "\x1b[44m",
            'M' => "\x1b[45m",
            'C' => "\x1b[46m",

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
    pub text: String,
    pub fmt: String
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
    pub value: String,
    pub data: Vec<Token>,
    pub fmt: String
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