pub fn get_md_link(mut title: String) -> String {
    title = title.to_lowercase();
    let mut out = String::new();
    for c in title.chars() {
        if c == ' ' || c == '-' {
            out.push('-');
        } else if c == '_' {
            out.push('_');
        } else if c.is_alphanumeric() {
            out.push(c);
        }
    }
    out
}