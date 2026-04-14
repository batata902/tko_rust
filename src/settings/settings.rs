use std::{collections::HashMap, path::PathBuf};


pub struct Settings {
    CFG_FILE: String,
    LANG_FILE: String,
    alias_git: HashMap<String, String>
}

impl Settings {
    pub fn new(path_dir: Option<PathBuf>) -> Self {
        let mut alias_git: HashMap<String, String> = HashMap::new();
        alias_git.insert("poo".to_string(), "https://github.com/qxcodepoo/arcade.git".to_string());
        alias_git.insert("fup".to_string(), "https://github.com/qxcodefup/arcade.git".to_string());
        alias_git.insert("ed".to_string(), "https://github.com/qxcodeed/arcade.git".to_string());


        Self { 
            CFG_FILE: "settings.yaml".to_string(), 
            LANG_FILE: "languages.toml".to_string(),
            alias_git,  
        }
    }
}