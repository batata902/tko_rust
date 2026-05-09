use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::game::quest::Quest;
use crate::utils::get_md_link::get_md_link;


pub struct QuestParser {
    source_alias: String,
    quest: Quest,
    line: String,
    line_num: usize,
    filename: PathBuf
}

impl QuestParser {
    pub fn new(source_alias: String) -> Self {
        let mut quest = Quest::new(None, None);
        quest.tree.set_remote_name(&source_alias);

        Self { 
            source_alias, 
            quest,
            line: String::new(), 
            line_num: 0, 
            filename: PathBuf::new() 
        }
    }

    pub fn finish_quest(&self) -> Quest {
        let mut quest = self.quest.clone();

        if quest.tree.get_key().is_empty() {
            quest.tree.set_key(
                get_md_link(quest.tree.get_title().to_string())
            );
        }

        quest
    }

    pub fn match_full_pattern(&mut self) -> bool {
        let mut _line: String;
        if self.line.starts_with("## ") {
            _line = self.line.get(3..).unwrap_or("").to_string();
        } else if self.line.starts_with("### ") {
            _line = self.line.get(4..).unwrap_or("").to_string();
        } else {
            return false;
        }
        _line = _line
                .replace("<!--", " ")
                .replace("-->", " ")
                .replace("`", " ");
        let title = self.process_words(&_line);

        self.quest.tree.set_title(title);

        true
    }

    pub fn process_words(&mut self, line: &str) -> String {
        let mut words: Vec<&str> = Vec::new();
        for tag in line.split(" ") {
            if !tag.is_empty() {
                words.push(tag.trim());
            }
        }

        let mut keys: Vec<&str> = Vec::new();
        for tag in &words {
            if tag.starts_with('@') {
                keys.push(tag);
            }
        }

        if !keys.is_empty() {
            self.quest.tree.set_key(keys.get(0).unwrap().to_string());
        }

        let mut skills: Vec<&str> = Vec::new();
        for t in &words {
            if t.starts_with('+') {
                skills.push(t.get(1..).unwrap());
            }
        }
        if skills.len() > 0 {
            self.quest.skills = HashMap::new();
            for s in skills {
                match s.split_once(":") {
                    Some((k, v)) => {
                        self.quest.skills.insert(k.to_string(), v.parse::<i32>().unwrap());
                    },
                    None => {
                        self.quest.skills.insert(s.to_string(), 1);
                    }
                }
            }
        }
        if self.quest.skills.len() == 0 {
            self.quest.skills = HashMap::from([
                (self.quest.tree.get_key().to_string(), 1)
            ]);
            
        }

        let mut languages: Vec<&str> = Vec::new();
        for t in &words {
            if t.starts_with('=') {
                languages.push(t.get(1..).unwrap());
            }
        }
        if languages.len() > 0 {
            self.quest.languages = Vec::new();
            for l in languages {
                self.quest.languages.push(l.to_string());
            }
        }

        let mut qmin: Vec<&str> = Vec::new();
        for t in &words {
            if t.starts_with('%') {
                qmin.push(t.get(1..).unwrap());
            }
        }
        if qmin.len() > 0 {
            match qmin.get(0).unwrap().parse::<i32>() {
                Ok(value) => {
                    self.quest.min_percent_completion = value;
                },
                Err(_) => {
                    self.quest.min_percent_completion = 50;
                }
            }
        }

        let mut required: Vec<&str> = Vec::new();
        for t in &words {
            if t.starts_with('!') {
                required.push(t.get(1..).unwrap());
            }
        }

        for req_key in required {
            self.quest.add_require_key(req_key.to_string());
        }

        words = words.into_iter()
            .filter(|w| !w.starts_with(['@', '%', '=', '+', '!']))
            .collect();

        words.join(" ")
    }

    pub fn parse_quest(&mut self, filename: &Path, line: String, line_num: usize) -> Option<Quest> {
        self.line = line;
        self.line_num = line_num;
        self.filename = filename.to_path_buf();

        self.quest.line = self.line.clone();
        self.quest.line_number = self.line_num;
        self.quest.remote_name = String::new();

        if self.match_full_pattern() {
            return Some(self.finish_quest());
        }

        None
    }
}