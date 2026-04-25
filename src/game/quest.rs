use std::collections::HashMap;
use std::borrow::Cow;
use std::fmt;

use crate::game::quest_grader::Elem;
use crate::game::task::TaskMain;
use crate::game::{tree_item::TreeItem, quest_grader::QuestGrader, task::Task};
use crate::utils::text::{AddValue, Text};

fn startswith(text: String, prefix: String) -> bool {
    let prefix_len = prefix.chars().count();
    if prefix_len > text.chars().count() {
        return false;
    }
    text.starts_with(&prefix)
}

pub struct Quest {
    tree: TreeItem,
    line_number: i32,
    line: String,
    __tasks: Vec<Task>,
    requires: Vec<String>,
    requires_ptr: Vec<Quest>,
    required_by_ptr: Vec<Quest>,
    skills: HashMap<String, i32>,
    languages: Vec<String>,
    min_percent_completion: i32,
    filename: String,
    remote_name: String,
    __is_reachable: bool
}

impl Quest {
    pub fn new(title: String, key: String) -> Self {
        let mut tree: TreeItem = TreeItem::new();
        tree.set_key(key);
        tree.set_title(title);

        Self { 
            tree, 
            line_number: 0, 
            line: "".to_string(), 
            __tasks: Vec::new(), 
            requires: Vec::new(), 
            requires_ptr: Vec::new(), 
            required_by_ptr: Vec::new(), 
            skills: HashMap::new(), 
            languages: Vec::new(), 
            min_percent_completion: 50, 
            filename: "".to_string(), 
            remote_name: "".to_string(), 
            __is_reachable: false 
        }
    }

    pub fn add_require_key(&mut self, mut key: String) {
        if key.starts_with('@') {
            key = key.chars().skip(0).collect();
        }
        self.requires.push(format!("{}@{}", self.tree.get_remote_name(), key));
    }

    pub fn get_full_title(&self, show_skills: bool) -> Text {
        let mut output = Text::new(None, None);
        output.addf("c".to_string(), Some(AddValue::Str(Cow::Borrowed(&self.remote_name)))).add(Some(AddValue::Str(Cow::Owned(":".to_string())))).add(Some(AddValue::Str(Cow::Borrowed(&self.tree.get_title().to_string()))));
        if show_skills {
            for (skill, value) in &self.skills {
                if *value > 1 {
                    output.addf("b".to_string(), Some(AddValue::Str(Cow::Owned(format!(" +{}*{}", skill, value)))));
                } else {
                    output.addf("b".to_string(), Some(AddValue::Str(Cow::Owned(format!(" +{}", skill)))));
                }
            }
        }
        output
    }

    pub fn is_reachable(&self) -> bool {
        self.__is_reachable
    }

    pub fn set_reachable(&mut self, value: bool) -> &mut Self{
        self.__is_reachable = value;
        self
    }

    pub fn update_tasks_reachable(&mut self) {
        for t in &mut self.__tasks {
            t.set_reachable(true);
        }
    }

    pub fn is_complete(&self) -> bool {
        let Some(value) = self.get_percent(true, true) else {
            return true;
        };
        value >= self.min_percent_completion as f64
    }

    pub fn add_task(&mut self, mut task: Task) {
        task.skills.extend(self.skills.clone());
        self.__tasks.push(task);
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        self.__tasks.clone()
    }

    pub fn sort_tasks_by_title(&mut self) {
        self.__tasks.sort_by_key(|t| t.task.get_title().to_string());
    }

    pub fn get_xp(&self, include_main_perk: bool, include_side: bool) -> (f64, f64) {
        let mut tasks_info: Vec<Elem> = Vec::new();
        for t in &self.__tasks {
            if [TaskMain::MAIN, TaskMain::PERK].contains(&t.task_path) && !include_main_perk {
                continue;
            }
            if t.task_path == TaskMain::SIDE && !include_side {
                continue;
            }
            let percent = (t.get_rate_percent() * t.get_quality_percent()) / 100.0;
            tasks_info.push(Elem::new(t.is_optional(), t.xp, percent));
        }

        QuestGrader::calc_xp_earned_total(tasks_info)
    }

    pub fn get_completion(&self) -> (i32, i32) {
        let mut total = 0;
        let mut done = 0;
        for t in &self.__tasks {
            total += 1;
            if t.is_complete() {
                done += 1;
            }
        }
        (done, total)
    }

    pub fn get_percent_main_and_all(&self) -> (Option<f64>, f64) {
        let mut percent_main: Option<f64> = Some(0.0);
        let mut percent_all: f64 = 0.0;

        let (obtainedm, totalm) = self.get_xp(true, false);
        let (obtaineds, totals) = self.get_xp(false, true);
        if totalm > 0.0 {
            percent_main = Some((obtainedm / totalm) * 100.0);
            percent_all = ((obtainedm + obtaineds) / totalm) * 100.0;
        } else if totals > 0.0 {
            percent_main = None;
            percent_all = (obtaineds / totals) * 100.0;
        } else {
            percent_all = 0.0;
        }

        (percent_main, percent_all)
    }

    pub fn get_percent(&self, include_main_perk: bool, include_side: bool) -> Option<f64> {
        if !include_main_perk && !include_side {
            return None;
        }
        let (main_obt, main_total) = self.get_xp(include_main_perk, false);
        let (side_obt, side_total) = self.get_xp(false, include_side);
        if include_main_perk && include_side {
            return QuestGrader::get_percent(main_obt + side_obt, main_total);
        }
        if include_main_perk {
            return QuestGrader::get_percent(main_obt, main_total);
        }

        return QuestGrader::get_percent(side_obt, side_total);
    }

    pub fn get_percent_main(&self) -> Option<f64> {
        self.get_percent(true, false)
    }

    pub fn get_percent_side(&self) -> Option<f64> {
        self.get_percent(false, true)
    }
}

impl fmt::Display for Quest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = if self.tree.get_full_key() == self.tree.get_title() {
            String::new()
        } else {
            format!("{} ", self.tree.get_full_key())
        };
        write!(f, "{:>3} {:0>2} {}{} {:?} {:?}", self.line_number.to_string(), self.__tasks.len(), key, self.tree.get_title(), self.skills, self.requires)
    }
}
