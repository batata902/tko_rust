use std::path::{Path, PathBuf};
use regex::Regex;

use crate::game::task::{Task};
use crate::game::task_config::{TaskEdit, TaskLoss, TaskTest};

pub struct TaskParser {
    index_path: PathBuf,
    task: Option<Task>,
}

impl TaskParser {
    pub fn new(index_path: &Path, source_alias: &str) -> Self {
        let mut task = Task::new();
        task.identity.set_remote_name(source_alias);
        Self {
            index_path: index_path.to_path_buf(),
            task: Some(task),
        }
    }

    pub fn filter_task_key(key: String) -> String {
        let allowed = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_+";
        let mut new_key = String::new();

        for c in key.chars() {
            if allowed.contains(c) {
                new_key.push(c);
            } else {
                break;
            }
        }
        new_key
    }

    pub fn match_full_pattern(&self, line: &str) -> (bool, String, String, String) {
        let pattern = r"\s*?- \[ \](.*?)\[([^\]]+)\]\(([^)]+)\)?";
        let re = Regex::new(pattern).unwrap();

        if let Some(captures) = re.captures(line) {
            let tags = captures
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or("")
                .trim()
                .replace('`', " ")
                .replace("<!--", " ")
                .replace("-->", " ");

            let title = captures
                .get(2)
                .map(|m| m.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            let link = captures
                .get(3)
                .map(|m| m.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            (true, tags, title, link)
        } else {
            (false, String::new(), String::new(), String::new())
        }
    }

    /// Decodes a type tag (the part after ":") and updates task fields accordingly.
    fn decode_task_types(&mut self, type_str: &str) {
        if let Some(task) = self.task.as_mut() {
            match type_str {
                "free" => task.config.loss = TaskLoss::FREE,
                "part" => task.config.loss = TaskLoss::PART,
                "zero" => task.config.loss = TaskLoss::ZERO,
                "test" => task.config.test = TaskTest::TEST,
                "self" => task.config.test = TaskTest::SELF,
                "view" => task.config.mode = TaskEdit::VIEW,
                "edit" => task.config.mode = TaskEdit::EDIT,
                _ => {}
            }
        }
    }

    /// Parses the raw tag/title string, setting the task key, type flags,
    /// and returning the remainder as the clean title.
    fn parse_key_task_types(&mut self, tags: &str) -> String {
        if self.task.is_none() {
            return String::new();
        }

        // Reset task_loss to NULL before re-parsing.
        if let Some(task) = self.task.as_mut() {
            task.config.loss = TaskLoss::NULL;
        }

        let words: Vec<String> = tags
            .split(' ')
            .filter(|w| !w.is_empty())
            .map(|w| w.to_string())
            .collect();

        let mut new_title: Vec<String> = Vec::new();

        for item in &words {
            if item.starts_with('@') {
                let key = TaskParser::filter_task_key(item[1..].to_string());
                if let Some(task) = self.task.as_mut() {
                    task.identity.set_key(key);
                }
            } else if item.starts_with(':') {
                // decode_task_types borrows &mut self, so we clone the slice first.
                let type_str = item[1..].to_string();
                self.decode_task_types(&type_str);
            } else {
                new_title.push(item.clone());
            }
        }

        // Determine task_mode from whether the key starts with "+".
        if let Some(task) = self.task.as_mut() {
            if task.identity.get_key().starts_with('+') {
                task.config.mode = TaskEdit::VIEW;
            } else {
                task.config.mode = TaskEdit::EDIT;
            }

            // Apply default loss/test values that depend on the mode.
            if task.config.mode == TaskEdit::VIEW {
                if task.config.loss == TaskLoss::NULL {
                    task.config.loss = TaskLoss::FREE;
                }
                if task.config.test == TaskTest::NULL {
                    task.config.test = TaskTest::SELF;
                }
            } else {
                // EDIT
                if task.config.loss == TaskLoss::NULL {
                    task.config.loss = TaskLoss::PART;
                }
                if task.config.test == TaskTest::NULL {
                    task.config.test = TaskTest::TEST;
                }
            }
        }

        new_title.join(" ")
    }

    /// Resolves a relative link against the directory that contains `index_path`.
    /// Absolute links are returned unchanged.
    pub fn redirect_from_readme(&self, link: &str) -> String {
        let link_path = Path::new(link);
        if !link_path.is_absolute() {
            let basedir = self.index_path.parent().unwrap_or(Path::new(""));
            return basedir.join(link).to_string_lossy().into_owned();
        }
        link.to_string()
    }

    /// Parses a single markdown line and, if it matches the task pattern,
    /// populates `self.task` with all derived metadata.
    pub fn parse_line(&mut self, line: &str, line_num: usize) -> &mut Self {
        let (found, tags, title, link) = self.match_full_pattern(line);
        if !found {
            self.task = None;
        }
        if self.task.is_none() {
            return self;
        }

        // Store raw line info on the task.
        if let Some(task) = self.task.as_mut() {
            task.location.line_number = line_num;
            task.location.line = line.to_string();
        }

        // Parse combined tag + title string to extract key, flags and clean title.
        let combined = format!("{} {}", tags, title);
        let new_title = self.parse_key_task_types(&combined);

        if let Some(task) = self.task.as_mut() {
            task.identity.set_title(new_title);
        }

        // A task without a key is invalid — discard it.
        let key_empty = self
            .task
            .as_ref()
            .map(|t| t.identity.get_key().is_empty())
            .unwrap_or(true);

        if key_empty {
            self.task = None;
            return self;
        }

        // Remote (HTTP/HTTPS) links need no local path resolution.
        if link.starts_with("http://") || link.starts_with("https://") {
            if let Some(task) = self.task.as_mut() {
                task.location.set_remote_view_type();
                task.location.target = link;
            }
            return self;
        }

        // Resolve the link relative to the readme directory, then update the task.
        let resolved = self.redirect_from_readme(&link);
        let origin_folder = PathBuf::from(&resolved)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();

        if let Some(task) = self.task.as_mut() {
            task.location.set_origin_folder(origin_folder);
            if task.config.mode == TaskEdit::VIEW {
                task.location.target = resolved;
            } else {
                task.location.target = link;
            }
        }

        self
    }

    /// Returns a reference to the parsed task, or `None` if the last
    /// `parse_line` call did not produce a valid task.
    pub fn get_task(&mut self) -> Option<&mut Task> {
        self.task.as_mut()
    }

    /// Verifies that import-type tasks point to an existing file.
    /// Returns `Err` with a descriptive message when the file is missing.
    pub fn check_path_try(&mut self) -> Result<&mut Self, String> {
        if let Some(task) = &self.task {
            if task.is_import_type() {
                let relative_path = self
                    .index_path
                    .parent()
                    .unwrap_or(Path::new(""))
                    .join(&task.location.target);

                if !relative_path.exists() {
                    return Err(format!(
                        "Parsing {:?}, Arquivo de tarefa não encontrado: {}",
                        self.index_path, task.location.target
                    ));
                }
            }
        }
        Ok(self)
    }
}