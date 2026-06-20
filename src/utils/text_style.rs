use std::{collections::{HashMap, HashSet}, sync::LazyLock};
use std::fmt;
use cached::proc_macro::cached;

static ANSI_RESET: &str = "\x1b[0m";

static FG_CODES: LazyLock<HashMap<char, &str>> = LazyLock::new(|| {
    let mut codes: HashMap<char, &str> = HashMap::new();

    codes.insert('d', "");
    codes.insert('k', "30");
    codes.insert('r', "31");
    codes.insert('g', "32");
    codes.insert('y', "33");
    codes.insert('b', "34");
    codes.insert('m', "35");
    codes.insert('c', "36");
    codes.insert('w', "37");

    codes
});

static BG_CODES: LazyLock<HashMap<char, &str>> = LazyLock::new(|| {
    let mut codes: HashMap<char, &str> = HashMap::new();

    codes.insert('D', "");
    codes.insert('K', "40");
    codes.insert('R', "41");
    codes.insert('G', "42");
    codes.insert('Y', "43");
    codes.insert('B', "44");
    codes.insert('M', "45");
    codes.insert('C', "46");
    codes.insert('W', "47");

    codes
});

static ATTR_CODES: LazyLock<HashMap<char, &str>> = LazyLock::new(|| {
    let mut codes: HashMap<char, &str> = HashMap::new();

    codes.insert('*', "1");
    codes.insert('_', "4");
    codes.insert('/', "3");
    codes.insert('X', "7");
    codes.insert('!', "9");

    codes

});


// Keys
static FG_KEYS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    FG_CODES.keys().copied().collect()
});
static BG_KEYS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    BG_CODES.keys().copied().collect()
});
static ATTRS_KEYS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    ATTR_CODES.keys().copied().collect()
});


static CODES: LazyLock<HashMap<char, &str>> = LazyLock::new(|| {
    let mut codes: HashMap<char, &str> = HashMap::new();

    codes.extend(FG_CODES.iter().map(|(k, v)| (*k, *v)));
    codes.extend(BG_CODES.iter().map(|(k, v)| (*k, *v)));
    codes.extend(ATTR_CODES.iter().map(|(k, v)| (*k, *v)));

    codes
});

static ANSI_TO_FG: LazyLock<HashMap<&str, char>> = LazyLock::new(|| {
    FG_CODES.iter().map(|(k, v)| (*v, *k)).collect()
});

static ANSI_TO_BG: LazyLock<HashMap<&str, char>> = LazyLock::new(|| {
    BG_CODES.iter().map(|(k, v)| (*v, *k)).collect()
});

static ANSI_TO_ATTR: LazyLock<HashMap<&str, char>> = LazyLock::new(|| {
    ATTR_CODES.iter().map(|(k, v)| (*v, *k)).collect()
});

// Static Methods, Optei por deixa-los fora da struct
// pois o cached não funciona dentro de um impl

#[cached]
pub fn parse(style: String) -> TextStyle {
    let mut fg: Option<char> = None;
    let mut bg: Option<char> = None;
    let mut attrs: HashSet<char> = HashSet::new();

    for ch in style.chars() {
        if FG_KEYS.contains(&ch) {
            fg = Some(ch);
        } else if BG_KEYS.contains(&ch) {
            bg = Some(ch);
        } else if ATTRS_KEYS.contains(&ch) {
            attrs.insert(ch);
        }
    }

    if fg == Some('d') {
        fg = None;
    }
    if bg == Some('D') {
        bg = None;
    }

    TextStyle { 
        fg, 
        bg, 
        attrs
    }
}

pub fn from_ansi_codes<I>(codes: I, base: Option<&TextStyle>) -> TextStyle
where 
I: IntoIterator, // O tipo I pode ser qualquer iterador
// Os itens do iterador podem ser lidos como string 
I::Item: AsRef<str> {
    let mut fg: Option<char> = base.and_then(|b: &TextStyle| b.fg);
    let mut bg: Option<char> = base.and_then(|b: &TextStyle| b.bg);
    let mut attrs: HashSet<char> = base.map_or(
        HashSet::new(), 
                |tstyle: &TextStyle| tstyle.attrs.clone()
            );

    for code in codes.into_iter() {
        let code_str = code.as_ref();
        // full reset
        if code_str == "0" {
            fg = None;
            bg = None;
            attrs.clear();
        }
        // foreground
        else if ANSI_TO_FG.contains_key(code_str) {
            fg = ANSI_TO_FG.get(code_str).copied();
        }
        // background
        else if ANSI_TO_BG.contains_key(code_str) {
            bg = ANSI_TO_BG.get(code_str).copied();
        }
        // attrs
        else if ANSI_TO_ATTR.contains_key(code_str) {
            attrs.insert(ANSI_TO_ATTR.get(code_str).copied().unwrap());
        }
    }

    TextStyle { 
        fg, 
        bg, 
        attrs 
    }

}


// TextStyle em si

#[derive(Debug, PartialEq, Clone)]
struct TextStyle {
    fg: Option<char>,
    bg: Option<char>,
    attrs: HashSet<char>
}

// Como o Rust não permite muito facilmente isso-> other: TextStyle | str
// no método overlay, optei por usar um enum junto do trait Into e From
enum TextStyleSource {
    Style(TextStyle),
    Text(String)
}

impl From<TextStyle> for TextStyleSource {
    fn from(value: TextStyle) -> Self {
        TextStyleSource::Style(value)
    }
}

impl From<String> for TextStyleSource {
    fn from(value: String) -> Self {
        TextStyleSource::Text(value)
    }
}

impl TextStyle {
    pub fn new() -> Self {
        Self { 
            fg: None, 
            bg: None, 
            attrs: HashSet::new() 
        }
    }

    pub fn overlay<T>(&self, other: T) -> TextStyle where T: Into<TextStyleSource> {
        let source: TextStyleSource = other.into();

        let other_style: TextStyle = match source {
            TextStyleSource::Style(style) => style,
            TextStyleSource::Text(text) => parse(text)
        };

        let mut combined_attrs: HashSet<char> = self.attrs.clone();
        combined_attrs.extend(other_style.attrs);

        TextStyle { 
            fg: other_style.fg.or(self.fg.clone()), 
            bg: other_style.bg.or(self.bg.clone()), 
            attrs: combined_attrs 
        }
    }

    pub fn clear_attrs(&self) -> TextStyle {
        TextStyle { 
            fg: self.fg.clone(), 
            bg: self.bg.clone(), 
            attrs: HashSet::new() 
        }
    }

    pub fn is_plain(&self) -> bool {
        self.fg.is_none() && self.bg.is_none() && self.attrs.is_empty()
    }

    pub fn to_tag(&self) -> String {
        let mut parts: Vec<char> = Vec::new();

        if !self.fg.is_none() {
            parts.push(self.fg.unwrap());
        }
        if !self.bg.is_none() {
            parts.push(self.bg.unwrap());
        }

        let mut sorted_attrs: Vec<char> = Vec::new();
        sorted_attrs.extend(&self.attrs);
        sorted_attrs.sort();
        
        parts.extend(sorted_attrs);

        parts.iter().collect()
    }

    // Não encontrei uma forma de implementar o cached nesse caso
    pub fn ansi(&self) -> String {
        let mut codes: Vec<&str> = Vec::new();

        if !self.fg.is_none() {
            codes.push(CODES.get(&self.fg.unwrap()).unwrap());
        }
        if !self.bg.is_none() {
            codes.push(CODES.get(&self.bg.unwrap()).unwrap());
        }

        let mut sorted_attrs: Vec<char> = Vec::new();
        sorted_attrs.extend(&self.attrs);
        sorted_attrs.sort();

        for attr in &sorted_attrs {
            codes.push(CODES.get(attr).unwrap());
        }

        if codes.is_empty() {
            return String::new();
        }

        format!("\x1b[{}m", codes.join(";"))
    }
}

impl fmt::Display for TextStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_tag())
    }
}