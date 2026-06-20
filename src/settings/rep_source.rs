use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fmt;
use serde::{Deserialize, Serialize};

use crate::settings::git_cache::GitCache;

// ─── SourceType ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    #[serde(rename = "local")]
    LocalFile,
    #[serde(rename = "git")]
    GitSource,
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::LocalFile => write!(f, "local"),
            SourceType::GitSource => write!(f, "git"),
        }
    }
}


pub const STUDENT_SANDBOX_NAME: &str = "sandbox";

pub struct Keys;

impl Keys {
    pub const NAME: &'static str = "name";
    pub const TARGET: &'static str = "target";
    pub const TYPE: &'static str = "type";
    pub const INDEX: &'static str = "index";
    pub const WRITEABLE: &'static str = "writeable";
    pub const QUESTS: &'static str = "quests";
    pub const TASKS: &'static str = "tasks";
    pub const BRANCH: &'static str = "branch";
}

pub struct RepSource {
    pub name: String,
    pub target: String,
    pub source_type: SourceType,
    pub writeable: bool,
    pub index: String,
    pub branch: Option<String>,
    pub quests: Option<HashMap<String, String>>,
    pub tasks: Option<HashMap<String, String>>,
    pub rep_local_workspace: Option<PathBuf>,
    pub rep_cache_folder: Option<PathBuf>,
    pub git_cache: Option<GitCache>,
}

impl RepSource {
    pub fn new(alias: &str, git_cache: Option<GitCache>) -> Self {
        Self {
            name: alias.to_string(),
            target: String::new(),
            source_type: SourceType::LocalFile,
            writeable: false,
            index: "README.md".to_string(),
            branch: None,
            quests: None,
            tasks: None,
            rep_local_workspace: None,
            rep_cache_folder: None,
            git_cache,
        }
    }

    pub fn set_git_cache(mut self, git_cache: GitCache) -> Self {
        self.git_cache = Some(git_cache);
        self
    }

    pub fn set_local_source(mut self, target: &Path, writeable: bool) -> Self {
        self.source_type = SourceType::LocalFile;
        self.target = target.to_string_lossy().to_string();
        self.writeable = writeable;
        self
    }

    pub fn is_sandbox_source(&self) -> bool {
        self.name == STUDENT_SANDBOX_NAME
    }

    pub fn set_student_sandbox(mut self) -> Self {
        self.name = STUDENT_SANDBOX_NAME.to_string();
        self = self.set_local_source(Path::new(STUDENT_SANDBOX_NAME), true);
        self
    }

    pub fn get_filters(&mut self) -> (Option<&HashMap<String, String>>, Option<&HashMap<String, String>>) {
        (self.quests.as_ref(), self.tasks.as_ref())
    }

    pub fn set_git_source(mut self, target: &str, branch: Option<String>) -> Self {
        self.source_type = SourceType::GitSource;
        self.target = target.to_string();
        self.branch = branch;
        self
    }

    pub fn is_local(&self) -> bool {
        self.source_type == SourceType::LocalFile
    }

    pub fn set_writeable(mut self, writeable: bool) -> Self {
        self.writeable = writeable;
        self
    }

    pub fn get_writeable(&self) -> bool {
        self.writeable
    }

    pub fn is_read_only(&self) -> bool {
        match self.source_type {
            SourceType::LocalFile => !self.writeable,
            SourceType::GitSource => true,
        }
    }

    pub fn get_url_link(&mut self) -> &str {
        &self.target
    }

    pub fn is_git_source(&self) -> bool {
        self.source_type == SourceType::GitSource
    }

    pub fn is_local_source(&self) -> bool {
        self.source_type == SourceType::LocalFile
    }

    pub fn get_source_readme(&mut self, verbose: bool) -> Result<PathBuf, String> {
        match self.get_source_folder(verbose) {
            Ok(folder) => Ok(folder.join(self.index.clone())),
            Err(e) => Err(e)
        }
    }

    pub fn get_source_folder(&mut self, verbose: bool) -> Result<PathBuf, String> {
        if self.is_sandbox_source() {
            return self.get_workspace().cloned();
        }
        if self.source_type == SourceType::LocalFile {
            return Ok(PathBuf::from(self.target.clone()));
        }
        if self.source_type == SourceType::GitSource {
            if self.git_cache.is_none() {
                return Err("Git cache is not set for git source".to_string());
            }
            let url_link = self.get_url_link().to_string();
            let repodir = self.git_cache.as_mut().unwrap().get_repo_dir(&url_link, verbose);
            if repodir.is_none() {
                return Err("Failed to get repository directory".to_string());
            }
            return Ok(repodir.unwrap().to_path_buf());
        }

        Err("Unknown source type".to_string())
    }

    pub fn set_filters(mut self, quests: Option<HashMap<String, String>>, tasks: Option<HashMap<String, String>>) -> Self {
        self.quests = quests;
        self.tasks = tasks;
        self
    }

    pub fn set_repo_globals(&mut self, local_workspace: PathBuf, cache_folder: PathBuf) {
        self.rep_local_workspace = Some(local_workspace);
        self.rep_cache_folder = Some(cache_folder);
    }

    pub fn get_repo_cache_folder(&self) -> Result<&PathBuf, String> {
        self.rep_cache_folder
            .as_ref()
            .ok_or_else(|| "Local cache folder is not set".to_string())
    }

    pub fn get_workspace(&self) -> Result<&PathBuf, String> {
        self.rep_local_workspace
            .as_ref()
            .ok_or_else(|| "Local workspace is not set".to_string())
    }

    pub fn get_task_workspace(&self, task_key: &str) -> Result<PathBuf, String> {
        if !self.is_read_only() {
            return Err("Source is not read-only, task workspace is the same as source workspace".to_string());
        }
        
        Ok(self.get_workspace().unwrap().join(task_key))
    }

    pub fn load_from_dict(mut self, data: &serde_json::Value) -> Self {
        for key in &["name", "alias", "database"] {
            if let Some(v) = data[key].as_str() {
                self.name = v.to_string();
                break;
            }
        }

        // target / backward-compat "link"
        if let Some(v) = data[Keys::TARGET].as_str() {
            self.target = v.to_string();
        } else if let Some(v) = data["link"].as_str() {
            self.target = if v.ends_with("README.md") {
                Path::new(v)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default()
            } else {
                v.to_string()
            };
        }

        // branch
        self.branch = data[Keys::BRANCH]
            .as_str()
            .map(|s| s.to_string())
            .or(Some("master".to_string()));

        // source_type
        self.source_type = match data[Keys::TYPE].as_str() {
            Some("local") => SourceType::LocalFile,
            _ => SourceType::GitSource,
        };

        // quests (list or dict) + backward-compat "filters"
        self.quests = Self::parse_str_map(&data[Keys::QUESTS])
            .or_else(|| Self::parse_str_map(&data["filters"]));

        // tasks (list or dict)
        self.tasks = Self::parse_str_map(&data[Keys::TASKS]);

        // writeable
        if let Some(v) = data[Keys::WRITEABLE].as_bool() {
            self.writeable = v;
        }
        if self.name == STUDENT_SANDBOX_NAME {
            self.writeable = true;
        }

        // index
        self.index = data[Keys::INDEX]
            .as_str()
            .unwrap_or("README.md")
            .to_string();

        self
    }

    pub fn save_to_dict(&self) -> serde_json::Value {
        let mut output = serde_json::json!({
            Keys::NAME:     self.name,
            Keys::TARGET:   self.target,
            Keys::INDEX:    self.index,
            Keys::TYPE:     self.source_type.to_string(),
            Keys::WRITEABLE: self.writeable,
        });

        if let Some(branch) = &self.branch {
            if branch != "master" {
                output[Keys::BRANCH] = serde_json::Value::String(branch.clone());
            }
        }

        output[Keys::QUESTS] = match &self.quests {
            None => serde_json::Value::Null,
            Some(m) => serde_json::json!(m),
        };
        output[Keys::TASKS] = match &self.tasks {
            None => serde_json::Value::Null,
            Some(m) => serde_json::json!(m),
        };

        output
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn parse_str_map(value: &serde_json::Value) -> Option<HashMap<String, String>> {
        if let Some(arr) = value.as_array() {
            let map = arr
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| (s.to_string(), String::new()))
                .collect();
            return Some(map);
        }
        if let Some(obj) = value.as_object() {
            let map = obj
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect();
            return Some(map);
        }
        None
    }
}