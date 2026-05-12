//! Indexer — port of the Python `Indexer` / `IndexLine` classes.
//!
//! Manages a Markdown task-index file of the form:
//!
//! ```markdown
//! # Title
//!
//! ## Quest name
//!
//! - [ ] `@label` [Task title](path/to/README.md)
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::utils::decoder;
use crate::utils::rtext::RText;

// ─────────────────────────────────────────────────────────────────
// helpers
// ─────────────────────────────────────────────────────────────────

/// Read the first `# Title` from a Markdown file.
pub fn load_title_from_markdown_file(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# ") {
            return Some(rest.trim().to_string());
        }
    }
    Some("NÃO TEM TÍTULO".to_string())
}

/// Characters allowed in a task label.
fn is_valid_label_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+')
}

// ─────────────────────────────────────────────────────────────────
// IndexLine
// ─────────────────────────────────────────────────────────────────

/// One line of the index file, optionally parsed as a task entry.
#[derive(Debug, Clone)]
pub struct IndexLine {
    /// Raw, unparsed text of the line (empty when built from a README).
    raw_line: String,

    /// True when the line represents a task checkbox entry.
    pub is_task: bool,

    /// Text before the `[title](link)` part (e.g. `` - [ ] `@label` ``).
    pub pre: String,

    /// Display title extracted from `[title](...)`.
    pub title: String,

    /// Resolved, absolute path to the task's README.md.
    pub readme_file: PathBuf,

    /// Text after the `[title](link)` part.
    pub pos: String,

    /// Resolved path of the index file itself.
    index_path: PathBuf,

    /// Resolved base directory.
    base_dir: PathBuf,
}

impl IndexLine {
    pub fn new(index_path: &Path, base_dir: &Path) -> Self {
        Self {
            raw_line: String::new(),
            is_task: false,
            pre: String::new(),
            title: String::new(),
            readme_file: PathBuf::new(),
            pos: String::new(),
            index_path: index_path.canonicalize().unwrap_or_else(|_| index_path.to_path_buf()),
            base_dir: base_dir.canonicalize().unwrap_or_else(|_| base_dir.to_path_buf()),
        }
    }

    /// Parse a raw index-file line. Returns self for chaining.
    pub fn init_by_line(mut self, line: &str) -> Self {
        self.raw_line = line.to_string();

        // Pattern:  - [ ] ...anything...[title](path)rest
        let re = Regex::new(r"^(- \[ \].*?)\[([^\]]*)\]\(([^()]*)\)(.*)$").unwrap();

        if let Some(caps) = re.captures(line) {
            self.is_task = true;
            self.pre = caps[1].to_string();
            self.title = caps[2].to_string();

            let file = Path::new(&caps[3]);
            self.readme_file = if file.is_absolute() {
                file.canonicalize().unwrap_or_else(|_| file.to_path_buf())
            } else {
                let joined = self.index_path.parent()
                    .unwrap_or(Path::new("."))
                    .join(file);
                joined.canonicalize().unwrap_or(joined)
            };

            self.pos = caps[4].to_string();
        }

        self
    }

    /// Build a synthetic entry from a discovered README path + title.
    pub fn init_by_readme_file(mut self, readme_file: PathBuf, title: String) -> Self {
        let key = readme_file
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        self.readme_file = readme_file;
        self.title = title;
        self.pre = format!("@{}", key);
        self.is_task = true;
        self
    }

    pub fn get_raw_line(&self) -> &str {
        &self.raw_line
    }

    /// Render the line back to its canonical Markdown representation.
    pub fn get_render_line(&self, key_pad: usize) -> String {
        if !self.is_task {
            return self.raw_line.clone();
        }

        // Make readme_file relative to the directory of index_path.
        let index_dir = self.index_path.parent().unwrap_or(Path::new("."));
        let readme_path = pathdiff::diff_paths(&self.readme_file, index_dir)
            .unwrap_or_else(|| self.readme_file.clone());

        format!(
            "- [ ]{}[{}]({}){}", 
            self.get_pre(key_pad),
            self.title,
            readme_path.display(),
            self.pos
        )
    }

    /// Build the formatted `` `@label  ` `` prefix.
    pub fn get_pre(&self, key_pad: usize) -> String {
        let label = self.get_label();

        // Strip the leading `@label`, backticks, `- [ ]` from self.pre,
        // then split remaining words into `:tag` and other tokens.
        let stripped = self.pre
            .replace(&format!("@{}", label), "")
            .replace('`', "")
            .replace("- [ ]", "");
        let stripped = stripped.trim();

        let words: Vec<&str> = stripped
            .split_whitespace()
            .filter(|w| !w.starts_with('@'))
            .collect();

        let tags: Vec<&str>   = words.iter().copied().filter(|w| w.starts_with(':')).collect();
        let others: Vec<&str> = words.iter().copied().filter(|w| !w.starts_with(':')).collect();

        let tag_str = tags.join(" ");
        let label_field = format!("`@{:<width$}{}`", label, 
            if tag_str.is_empty() { String::new() } else { format!(" {}", tag_str) },
            width = key_pad + 1);

        if others.is_empty() {
            label_field
        } else {
            format!("{} {} ", label_field, others.join(" "))
        }
    }

    /// Extract the task label from the readme folder name.
    pub fn get_label(&self) -> String {
        let base_dir = self.base_dir.canonicalize()
            .unwrap_or_else(|_| self.base_dir.clone());
        let readme = self.readme_file.canonicalize()
            .unwrap_or_else(|_| self.readme_file.clone());

        let raw = if readme.starts_with(&base_dir) {
            readme
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        } else {
            String::new()
        };

        // Validate characters
        let mut valid = String::new();
        for c in raw.chars() {
            if is_valid_label_char(c) {
                valid.push(c);
            } else {
                eprintln!("fail: error in {}", raw);
                panic!("Invalid label character '{}' in '{}'", c, raw);
            }
        }
        valid
    }
}

// ─────────────────────────────────────────────────────────────────
// Indexer
// ─────────────────────────────────────────────────────────────────

pub struct Indexer {
    pub index_path: PathBuf,
    pub base_dir: PathBuf,
    pub verbose: bool,

    pub index_lines: Vec<IndexLine>,
    pub path_title_dict: HashMap<PathBuf, String>,
    pub missing_entries: HashMap<PathBuf, IndexLine>,
}

impl Indexer {
    pub fn new(index_path: PathBuf, base_dir: PathBuf, verbose: bool) -> Self {
        Self {
            index_path,
            base_dir,
            verbose,
            index_lines: Vec::new(),
            path_title_dict: HashMap::new(),
            missing_entries: HashMap::new(),
        }
    }

    // ── Step 1 ───────────────────────────────────────────────────

    /// Scan `base_dir` for subdirectory README.md files and cache their titles.
    pub fn load_readme_title_dict_from_basedir(&mut self) -> std::io::Result<()> {
        let mut titles = HashMap::new();

        for entry in std::fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let readme = path.join("README.md");
                if readme.exists() {
                    let readme = readme.canonicalize().unwrap_or(readme);
                    if let Some(title) = load_title_from_markdown_file(&readme) {
                        titles.insert(readme, title);
                    }
                }
            }
        }

        if self.verbose {
            println!(
                "Found {} README.md files in base directory '{}'",
                titles.len(),
                self.base_dir.display()
            );
        }

        self.path_title_dict = titles;
        Ok(())
    }

    // ── Step 2 ───────────────────────────────────────────────────

    /// Parse the index file, silently dropping task lines whose README is missing.
    pub fn load_index_lines_removing_broken(&mut self) -> std::io::Result<()> {
        let content = decoder::decoder::load(&self.index_path, false)?;
        let mut lines = Vec::new();

        for raw in content.lines() {
            let line = IndexLine::new(&self.index_path, &self.base_dir)
                .init_by_line(raw);

            if line.is_task && !line.readme_file.exists() {
                if self.verbose {
                    println!(
                        "{}",
                        RText::parse(
                            "Warning: README file '[y]{}[.]' does not exist for task:[b]{}[.], removing from index",
                            &[
                                RText::plain_text(line.readme_file.display().to_string()),
                                RText::plain_text(line.get_label()),
                            ]
                        )
                        .render(None)
                    );
                }
                continue;
            }

            lines.push(line);
        }

        self.index_lines = lines;
        Ok(())
    }

    // ── Step 3 ───────────────────────────────────────────────────

    /// Detect README titles that differ from what's written in the index.
    ///
    /// * `save_titles` – overwrite the README `# Title` with the index title.  
    /// * `load_titles` – overwrite the index title with the README title.
    pub fn fix_titles(&mut self, save_titles: bool, load_titles: bool) {
        for line in &mut self.index_lines {
            if !line.is_task {
                continue;
            }
            let Some(folder_title) = self.path_title_dict.get(&line.readme_file) else {
                if self.verbose {
                    println!(
                        "{}",
                        RText::parse(
                            "Warning: README file '[y]{}[.]' not in title dict",
                            &[RText::plain_text(line.readme_file.display().to_string())]
                        )
                        .render(None)
                    );
                }
                continue;
            };

            if folder_title == &line.title {
                continue;
            }

            if self.verbose {
                println!(
                    "{}",
                    RText::parse(
                        "Mismatch title for task:[b]{}[.]\n\tREADME:'[y]{}[.]' != TASK:'[g]{}[.]'",
                        &[
                            RText::plain_text(line.readme_file.display().to_string()),
                            RText::plain_text(&line.title),
                            RText::plain_text(folder_title),
                        ]
                    )
                    .render(None)
                );
            }

            if save_titles {
                Self::replace_title_in_readme(&line.readme_file, &line.title, self.verbose);
            }
            if load_titles {
                line.title = folder_title.clone();
            }
        }
    }

    fn replace_title_in_readme(readme: &Path, new_title: &str, verbose: bool) {
        if !readme.exists() {
            eprintln!("Error: README file '{}' does not exist, cannot replace title.", readme.display());
            return;
        }
        let content = match std::fs::read_to_string(readme) {
            Ok(c) => c,
            Err(e) => { eprintln!("Error reading '{}': {}", readme.display(), e); return; }
        };

        let re = Regex::new(r"(?m)^(# .*)$").unwrap();
        let new_content = re.replacen(&content, 1, format!("# {}", new_title).as_str()).to_string();

        if let Err(e) = std::fs::write(readme, &new_content) {
            eprintln!("Error writing '{}': {}", readme.display(), e);
            return;
        }

        if verbose {
            println!("Replaced title in '{}' with '{}'", readme.display(), new_title);
        }
    }

    // ── Step 4 ───────────────────────────────────────────────────

    /// Collect task directories that exist on disk but are absent from the index.
    pub fn found_unused_task_dirs(&mut self) {
        let indexed: std::collections::HashSet<PathBuf> = self
            .index_lines
            .iter()
            .filter(|l| l.is_task)
            .map(|l| l.readme_file.clone())
            .collect();

        let mut missing = HashMap::new();
        for (readme, title) in &self.path_title_dict {
            if !indexed.contains(readme) {
                let entry =
                    IndexLine::new(&self.index_path, &self.base_dir)
                        .init_by_readme_file(readme.clone(), title.clone());
                missing.insert(readme.clone(), entry);
            }
        }
        self.missing_entries = missing;
    }

    // ── Step 5 ───────────────────────────────────────────────────

    /// Group `index_lines` into quests (sections starting with `## `).
    ///
    /// If `default_quest_name` doesn't exist yet, a new section is appended.
    /// Missing entries are appended to that section.
    pub fn insert_missing_tasks(
        &self,
        default_quest_name: &str,
    ) -> Vec<Vec<IndexLine>> {
        let mut quests: Vec<Vec<IndexLine>> = Vec::new();
        let mut found_index: Option<usize> = None;

        let mut iter = self.index_lines.iter();

        // First line goes into its own group (usually the `# Title` header).
        if let Some(first) = iter.next() {
            quests.push(vec![first.clone()]);
        }

        for line in iter {
            if line.get_raw_line().starts_with("## ") {
                quests.push(vec![line.clone()]);
                if line.get_raw_line().starts_with(&format!("## {}", default_quest_name)) {
                    found_index = Some(quests.len() - 1);
                }
            } else if !line.get_raw_line().trim().is_empty() {
                if let Some(last) = quests.last_mut() {
                    last.push(line.clone());
                }
            }
        }

        // Create the default quest section if it wasn't found.
        let target = match found_index {
            Some(idx) => idx,
            None => {
                let header = IndexLine::new(&self.index_path, &self.base_dir)
                    .init_by_line(&format!("## {}", default_quest_name));
                quests.push(vec![header]);
                quests.len() - 1
            }
        };

        if !self.missing_entries.is_empty() {
            if self.verbose {
                println!(
                    "Found {} missing hooks, adding to quest '{}':",
                    self.missing_entries.len(),
                    default_quest_name
                );
            }
            // Sort by readme path for deterministic output.
            let mut sorted: Vec<(&PathBuf, &IndexLine)> = self.missing_entries.iter().collect();
            sorted.sort_by_key(|(p, _)| p.as_path());
            for (_, line) in sorted {
                quests[target].push(line.clone());
            }
        }

        quests
    }

    // ── Step 6 ───────────────────────────────────────────────────

    /// Serialise `quest_lines` back to `self.index_path`.
    pub fn write_file(&self, quest_lines: &[Vec<IndexLine>]) -> std::io::Result<()> {
        // Compute padding width from all task labels.
        let key_pad = quest_lines
            .iter()
            .flatten()
            .filter(|l| l.is_task)
            .map(|l| l.get_label().len())
            .max()
            .unwrap_or(0);

        let mut out = String::new();
        for quest in quest_lines {
            // First element is the section header (or file header).
            if let Some(header) = quest.first() {
                out.push_str(&header.get_render_line(key_pad));
                out.push_str("\n\n");
            }
            for line in quest.iter().skip(1) {
                out.push_str(&line.get_render_line(key_pad));
                out.push('\n');
            }
            out.push('\n');
        }

        // Strip trailing newlines then add exactly one.
        let trimmed = out.trim_end_matches('\n');
        std::fs::write(&self.index_path, format!("{}\n", trimmed))
    }
}

// ─────────────────────────────────────────────────────────────────
// Public entry-point  (mirrors Python's `fix_readme`)
// ─────────────────────────────────────────────────────────────────

pub fn fix_readme(
    index: PathBuf,
    base_dir: &Path,
    default_quest_name: &str,
    verbose: bool,
    save_titles: bool,
    load_titles: bool,
) -> std::io::Result<()> {
    let index = index.canonicalize().unwrap_or_else(|_| index.to_path_buf());
    let base_dir = base_dir.canonicalize().unwrap_or_else(|_| base_dir.to_path_buf());

    let mut indexer = Indexer::new(index, base_dir, verbose);
    indexer.load_readme_title_dict_from_basedir()?;
    indexer.load_index_lines_removing_broken()?;
    indexer.fix_titles(save_titles, load_titles);
    indexer.found_unused_task_dirs();
    let quest_lines = indexer.insert_missing_tasks(default_quest_name);
    indexer.write_file(&quest_lines)
}

// ─────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────

