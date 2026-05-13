use std::collections::HashMap;
use std::io::Write;
use std::{fs, io};
use std::path::{Path, PathBuf};

use crate::game::quest;
use crate::game::{
    quest_parser::QuestParser, 
    task_parser::TaskParser,
    quest::Quest,
    task::Task
};

use crate::settings::rep_source::RepSource;
use crate::utils::decoder::decoder;
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

    pub fn build_from(&mut self, _language: &str) -> &mut Self {
        let filename = self.source.get_source_readme();
        match filename {
            Ok(file) => {
                self.__ensure_sandbox_readme_fixed(&file);
                let _content: String = self.load_content(&file).unwrap();
                self.__parse_file_content(&_content);

                let quest_filters = {
                    let (qf, _) = self.source.get_filters();
                    qf.cloned()
                };
                
                self.__remove_empty_and_other_language_and_filtered(_language, quest_filters);
                self.__create_
            
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
            content = decoder::load(filename, true)?;
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
                                    task.location.set_workspace_folder(
                                        self.source.get_task_workspace(task.identity.get_key()).unwrap()
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

    pub fn add_filtered_quests(&mut self, quest_filters: Option<&HashMap<String, String>>) {
        // Mesmo comportamento do Python
        if self.source.is_sandbox_source() {
            return;
        }

        let Some(quest_filters) = quest_filters else {
            return;
        };

        if quest_filters.is_empty() {
            return;
        }

        // Snapshot das quests atuais.
        // Isso preserva o comportamento do Python:
        //
        // available_quests = [q for q in self.quests.values()]
        //
        // Além disso evita problemas de borrow ao reconstruir self.quests.
        let available_quests: Vec<Quest> =
            self.quests.values().cloned().collect();

        // Resultado final das quests filtradas
        let mut result: HashMap<String, Quest> = HashMap::new();

        // Mantém a ordem de inserção equivalente ao __add_quest
        let mut ordered_quests: Vec<String> = Vec::new();

        // Última quest adicionada
        let mut active_quest: Option<Quest> = None;

        for (pattern, destiny) in quest_filters {
            let pattern_l = pattern.to_lowercase();

            for q in &available_quests {
                let title_l = q.identity.get_title().to_lowercase();

                let key_match =
                    format!("@{}", q.identity.get_key().to_lowercase());

                let matches =
                    title_l.contains(&pattern_l)
                    || pattern_l == key_match;

                if !matches {
                    continue;
                }

                // Caso:
                //
                // if destiny == "":
                //
                if destiny.is_empty() {
                    let key = q.identity.get_full_key();

                    if !result.contains_key(&key) {
                        result.insert(key.clone(), q.clone());
                        ordered_quests.push(key.clone());
                    }

                    active_quest = Some(q.clone());
                } else {
                    // Quest destino agregadora
                    let key =
                        format!("{}@{}", self.source.name, destiny);

                    // Cria somente uma vez
                    if !result.contains_key(&key) {
                        let mut qdestiny = Quest::new(
                            Some(destiny.clone()),
                            Some(destiny.clone()),
                        );

                        qdestiny
                            .identity
                            .set_remote_name(&self.source.name);

                        result.insert(key.clone(), qdestiny);

                        ordered_quests.push(key.clone());
                    }

                    let entry = result.get_mut(&key).unwrap();

                    // Copia todas as tasks
                    for t in q.get_tasks() {
                        entry.add_task(t.clone());
                    }

                    active_quest = Some(entry.clone());
                }
            }
        }

        // Reconstrói o estado final
        self.quests = result;
        self.ordered_quests = ordered_quests;
        self.active_quest = active_quest;
    }

    pub fn filter_by_langugage_and_empty(&mut self, language: String) {
        let mut quests: Vec<Quest> = Vec::new();
        for q in self.quests.values().cloned() {
            if q.get_tasks().len() == 0 {
                continue;
            }
            if q.languages.len() == 0 || q.languages.contains(&language) {
                quests.push(q);
            }
        }
        self.quests = quests.into_iter().map(|q| (q.identity.get_full_key(), q)).collect();
    }

    pub fn __remove_empty_and_other_language_and_filtered(&mut self, language: &str, quest_filters: Option<HashMap<String, String>>) -> &mut Self {
        if quest_filters.is_none() || quest_filters.as_ref().is_some_and(|qf| qf.len() == 0) {
            self.filter_by_langugage_and_empty(language.to_string());
        } else {
            self.add_filtered_quests(quest_filters.as_ref());
        }

        self
    }

    pub fn collect_quests(&self) -> HashMap<String, &Quest> {
        let mut quests: HashMap<String, &Quest> = HashMap::new();
        for quest in self.quests.values() {
            quests.insert(quest.identity.get_full_key(), quest);
        }
        quests
    }

    pub fn __create_requirement_pointers(&self) -> () {
        let (quests, tasks) = self.source.get_filters();
        if !quests.is_none() || !tasks.is_none() {
            return;
        }
        let filename: Result<PathBuf, String>  = self.source.get_source_readme(self.verbose);
        let quest = self.
    }

    pub fn __create_cross_references(&mut self) {
        for (_, quest) in &mut self.quests {
            quest.identity.set_remote_name(&self.source.name);

            let key = quest.identity.get_full_key();
            for task in quest.get_tasks_mut() {
                task.identity.set_remote_name(&self.source.name);
                task.quest_key = key.clone();
            }
        }
    }
}