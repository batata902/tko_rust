pub mod Decoder {
    use std::path::Path;
    use unicode_normalization::UnicodeNormalization;
    use std::fs;
    use std::io;

    pub fn load<P: AsRef<Path>>(file_path: P, normalize_endl: bool) -> io::Result<String> {
        let bytes = fs::read(file_path)?;

        let mut value = match String::from_utf8(bytes.clone()) {
            Ok(s) => {
                s
            },
            Err(_) => {
                bytes.into_iter()
                    .map(|b| b as char)
                    .collect()
            }
        };

        value = value.nfc().collect();
        if normalize_endl && !value.is_empty() && value.chars().last() != Some('\n') {
            return Ok(format!("{}\n", value));
        }
        Ok(value)
    }

    pub fn save<P: AsRef<Path>>(file_path: P, content: String) -> io::Result<()> {
        let mut path = Path::new(file_path.as_ref());

        if let Some(base_path) = path.parent() {
            fs::create_dir_all(base_path)?;
        }
        fs::write(path, format!("{}\n", content))?;
        Ok(())
    }
}