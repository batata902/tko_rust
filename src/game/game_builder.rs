use std::collections::HashMap;
use std::io::Write;
use std::{fs, io};
use std::path::Path;

use crate::game::{
    quest_parser::QuestParser, 
    task_parser::TaskParser,
    quest::Quest,
    task::Task
};

use crate::settings::rep_source::RepSource;
use crate::utils::decoder::Decoder;
use crate::feno::indexer::fix_readme;

pub struct GameBuilder {
    source: RepSource,
    ordered_quests: Vec<String>,
    quests: HashMap<String, Quest>,
    active_quest: Option<Quest>,
    interactive: bool,
    verbose: bool
}

impl GameBuilder {
    pub fn new(source: RepSource, verbose: bool) -> Self {
        Self { 
            source, 
            ordered_quests: Vec::new(), 
            quests: HashMap::new(), 
            active_quest: None, 
            interactive: false, 
            verbose 
        }
    }

    pub fn set_interactive(&mut self, interactive: bool) -> &mut Self {
        self.interactive = interactive;
    
        self
    }

    pub fn build_from(&mut self, language: &str) -> &mut Self {
        let filename = self.source.get_source_readme();
        match filename {
            Ok(file) => {
                self.__ensure_sandbox_readme_fixed(&file);
                let content: String = self.load_content(&file).unwrap();
                
            },
            Err(e) => {
                if self.verbose {
                    eprintln!("Erro ao obter o arquivo README da fonte {}: {}", self.source.name, e);
                }
                return self;
            }
        }

        self
    }

    pub fn load_content(&self, filename: &Path) -> io::Result<String> {
        let mut content: String = String::new();
        if !filename.exists() {
            if !self.source.is_sandbox_source() {
                if self.verbose {
                    eprintln!("Aviso: fonte {} não encontrada no source {}", filename.display(), self.source.name);
                }
            }
        } else {
            content = Decoder::load(filename, true)?;
        }

        Ok(content)
    }

    pub fn __ensure_sandbox_readme_fixed(&self, filename: &Path) -> io::Result<()> {
        if !self.source.is_sandbox_source() {
            return Ok(());
        }
        if !filename.parent().is_some_and(|p| p.exists()) {
            return Ok(());
        }
        if !filename.exists() {
            if self.verbose {
                eprintln!("Aviso: fonte {} não encontrada no source {}, criando arquivo", filename.display(), self.source.name);
            }
            if let Some(parent) = filename.parent() {
                fs::create_dir_all(parent)?;
                let mut file: fs::File = fs::File::create(filename)?;
                file.write_all(format!("# {}\n\n", self.source.name).as_bytes());
            }
        }
        fix_readme(fs::canonicalize(filename)?, self.source.get_repo_workspace().unwrap(), &self.source.name, false, false, true);

        Ok(())
    }

    pub fn __parse_file_content(&mut self, content: &String) {
        let lines: Vec<&str> = content.lines().collect();
        let alias = self.source.name.clone();

        match self.source.get_source_readme() {
            Ok(filename) => {
                for (line_num, line) in lines.iter().enumerate() {
                    // obs.: Alterar QuestParser para receber referência ao invés de clone() (Otimização)
                    let mut quest_parser = QuestParser::new(&alias); 
                    let quest = quest_parser.parse_quest(&filename, line.to_string(), line_num);
                    if !quest.is_none() {
                        self.__add_quest(quest_parser.finish_quest());
                        continue;
                    }

                    let mut tp = TaskParser::new(&filename, &alias);
                    
                    if let Ok(task) = tp
                        .parse_line(line, line_num + 1)
                        .check_path_try() {
                            if let Some(task) = task.get_task() {
                                if self.source.is_read_only() && !task.is_link() {
                                    task.set_workspace_folder(
                                        self.source.get_task_workspace(task.task.get_key()).unwrap()
                                    ).ok();
                                }

                                self.__add_task(task.clone());
                            }
                        }
                    
                

                }
            },
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        }
    }

    pub fn __add_quest(&mut self, quest: Quest) -> &mut Quest {
        let key = quest.identity.get_full_key().to_string();

        if !self.quests.contains_key(&key) {
            self.quests.insert(key.clone(), quest);
        }

        if !self.ordered_quests.contains(&key) {
            self.ordered_quests.push(key.clone());
        }

        self.active_quest = self.quests.get(&key).cloned();

        self.quests.get_mut(&key).unwrap()
    }

    pub fn __get_active_quest(&mut self) -> Option<&mut Quest> {
        if self.active_quest.is_none() {
            let qkey = String::from("_sem_quest");
            return Some(self.__add_quest(Quest::new(Some("Sem Quest".to_string()), Some(qkey))));
        }
        self.active_quest.as_mut()
    }

    pub fn __add_task(&mut self, task: Task) {
        self.__get_active_quest()
            .expect("no active quests")
            .add_task(task);
    }

    pub fn add_filtered_quests(&mut self, quest_filters: Option<HashMap<String, String>>) {
        if self.source.is_sandbox_source() {
            return;
        }

        let Some(quest_filters) = quest_filters else {
            return;
        };

        if quest_filters.is_empty() {
            return;
        }

        let mut result: HashMap<String, Quest> = HashMap::new();

        for (pattern, destiny) in quest_filters {
            let pattern_l = pattern.to_lowercase();

            for q in self.quests.values() {
                let title = q.identity.get_title().to_lowercase();
                let key_match = format!("@{}", q.identity.get_key().to_lowercase());

                if title.contains(&pattern_l) || pattern_l == key_match {
                    if destiny.is_empty() {
                        result.insert(q.identity.get_full_key(), q.clone());
                    } else {
                        let key = format!("{}@{}", self.source.name, destiny);

                        let entry = result.entry(key.clone()).or_insert_with(|| {
                            let mut ques = Quest::new(
                                Some(destiny.clone()),
                                Some(destiny.clone()),
                            );
                            ques.identity.set_remote_name(&self.source.name);
                            ques
                        });

                        for t in q.get_tasks() {
                            entry.add_task(t.clone());
                        }
                    }
                }
            }
        }

        self.quests = result;
    }
}