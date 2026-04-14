pub struct Text {
    text: String,
    fmt: String
}

impl Text {
    pub fn new(text: Option<String>, fmt: Option<String>) -> Self {
        Self { 
            text: text.unwrap_or("".to_string()), 
            fmt: fmt.unwrap_or("".to_string()) 
        }
    }
}