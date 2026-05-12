use std::{collections::HashMap, path::PathBuf};


pub struct Settings {
    cfg_file: String,
    lang_file: String,
    alias_git: HashMap<String, String>
}

impl Settings {
    pub fn new(_path_dir: Option<PathBuf>) -> Self {
        let mut alias_git: HashMap<String, String> = HashMap::new();
        alias_git.insert("poo".to_string(), "https://github.com/qxcodepoo/arcade.git".to_string());
        alias_git.insert("fup".to_string(), "https://github.com/qxcodefup/arcade.git".to_string());
        alias_git.insert("ed".to_string(), "https://github.com/qxcodeed/arcade.git".to_string());


        Self { 
            cfg_file: "settings.yaml".to_string(), 
            lang_file: "languages.toml".to_string(),
            alias_git,  
        }
    }
}