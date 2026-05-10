use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::borrow::Cow;


use crate::game::tree_item::TreeItem;
use crate::game::task_info::TaskInfo;
use crate::utils::symbols;
use crate::utils::text::{AddValue, Text};

#[derive(Clone, Copy, PartialEq)]
pub enum TaskTest {
    NULL,
    TEST,
    SELF
}

#[derive(Clone, Copy, PartialEq)]
pub enum TaskMain {
    MAIN,
    PERK,
    SIDE
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum TaskLoss {
    NULL,
    FREE,
    PART,
    ZERO
}

#[derive(Clone, Copy, PartialEq)]
pub enum TaskEdit {
    VIEW,
    EDIT
}


impl TaskTest {
    pub fn value(&self) -> &'static str {
        match self {
            TaskTest::NULL => "null",
            TaskTest::TEST => "test",
            TaskTest::SELF => "self"
        }
    }
}

impl TaskMain {
    pub fn value(&self) -> &'static str {
        match self {
            TaskMain::MAIN => "main",
            TaskMain::PERK => "perk",
            TaskMain::SIDE => "side"
        }
    }
}

impl TaskLoss {
    pub fn value(&self) -> &'static str {
        match self {
            TaskLoss::NULL => "null",
            TaskLoss::FREE => "free",
            TaskLoss::PART => "part",
            TaskLoss::ZERO => "zero"
        }
    }
}

impl TaskEdit {
    pub fn value(&self) -> &'static str {
        match self {
            TaskEdit::EDIT => "edit",
            TaskEdit::VIEW => "view"
        }
    }
}

#[derive(Clone)]
pub struct TaskGrader{
    info: Rc<RefCell<TaskInfo>>,
    loss: Rc<TaskLoss>,
    grades: HashMap<TaskLoss, HashMap<String, i32>>
}

impl TaskGrader {
    pub fn new(task_loss: Rc<TaskLoss>, task_info: Rc<RefCell<TaskInfo>>) -> Self {
        let mut free_value: HashMap<String, i32> = HashMap::new();
        free_value.insert("guided".to_string(), 100);
        free_value.insert("code".to_string(), 100);
        free_value.insert("debug".to_string(), 100);
        free_value.insert("problem".to_string(), 100);

        let mut part_value: HashMap<String, i32> = HashMap::new();
        part_value.insert("guided".to_string(), 80);
        part_value.insert("code".to_string(), 40);
        part_value.insert("debug".to_string(), 80);
        part_value.insert("problem".to_string(), 90);

        let mut zero_value: HashMap<String, i32> = HashMap::new();
        zero_value.insert("guided".to_string(), 0);
        zero_value.insert("code".to_string(), 0);
        zero_value.insert("debug".to_string(), 0);
        zero_value.insert("problem".to_string(), 0);

        let mut grades: HashMap<TaskLoss, HashMap<String, i32>> = HashMap::new();
        grades.insert(TaskLoss::FREE, free_value);
        grades.insert(TaskLoss::PART, part_value);
        grades.insert(TaskLoss::ZERO, zero_value);

        Self { 
            info: task_info, 
            loss: task_loss, 
            grades
        }
    }

    pub fn get_rate_percent(&self) -> f64 {
        let rate = self.info.borrow().rate as f64;
        rate
    }

    pub fn get_quality_percent(&self) -> f64 {
        if !self.info.borrow().feedback {
            return 0.0;
        }
        let mut rate = 100.0;
        if self.info.borrow().guided {
            let value = self.grades
                .get(&self.loss).unwrap()
                .get("guided").unwrap();
            rate *= *value as f64 / 100.0;
        }
        if self.info.borrow().ia_code {
            let value = self.grades
                .get(&self.loss).unwrap()
                .get("code").unwrap();
            rate *= *value as f64 / 100.0;
        }
        if self.info.borrow().ia_debug {
            let value = self.grades
                .get(&self.loss).unwrap()
                .get("debug").unwrap();
            rate *= *value as f64 / 100.0;
        }
        if self.info.borrow().ia_problem {
            let value = self.grades
                .get(&self.loss).unwrap()
                .get("problem").unwrap();
            rate *= *value as f64 / 100.0;
        }

        rate
    }

    pub fn get_ratio(&self) -> f64 {
        self.get_rate_percent() / 100.0
    }
}

#[derive(Clone)]
pub struct Task {
    pub task: TreeItem,

    pub line_number: usize,
    pub line: String,
    pub info: Rc<RefCell<TaskInfo>>,
    pub main_idx: i32,

    pub task_test: TaskTest,
    pub task_path: TaskMain,
    pub task_loss: Rc<TaskLoss>,
    pub task_mode: TaskEdit,

    pub grader: TaskGrader,

    pub skills: HashMap<String, i32>,

    pub xp: i32,

    pub target: String,
    pub quest_key: String,
    pub remote_name: String,
    pub __origin_folder: Option<PathBuf>,
    pub __workspace_folder: Option<PathBuf>,
    pub __is_recheable: bool,
    pub default_min_value: i32
}

static STR_INDEX: &str = "idx";

impl Task {
    pub fn new() -> Self {
        let task: TreeItem = TreeItem::new();

        let line_number: usize = 0;
        let line: String = String::from("");
        let info: Rc<RefCell<TaskInfo>> = Rc::new(RefCell::new(TaskInfo::new()));
        let main_idx: i32 = 0;

        let task_test: TaskTest = TaskTest::TEST;
        let task_path: TaskMain = TaskMain::MAIN;
        let task_loss: Rc<TaskLoss> = Rc::new(TaskLoss::PART);
        let task_mode: TaskEdit = TaskEdit::EDIT;

        let grader: TaskGrader = TaskGrader::new(task_loss.clone(), Rc::clone(&info));
        let skills: HashMap<String, i32> = HashMap::new();

        let xp: i32 = 1;

        let target: String = String::from("");
        let quest_key: String = String::from("");
        let remote_name: String = String::from("");
        let __origin_folder: Option<PathBuf> = None;
        let __workspace_folder: Option<PathBuf> = None;
        let __is_recheable: bool = false;
        let default_min_value: i32 = 5;

        Self { task, line_number, line, info, main_idx, task_test, task_path, task_loss, task_mode, grader, skills, xp, target, quest_key, remote_name, __origin_folder, __workspace_folder, __is_recheable, default_min_value }
    }

    pub fn clone(&self) -> Task {
        let mut new_task: Task = Task::new();
        new_task.line_number = self.line_number;
        new_task.line = self.line.clone();
        new_task.info = Rc::new(self.info.as_ref().clone());
        new_task.main_idx = self.main_idx;
        new_task.task_test = self.task_test;
        new_task.task_path = self.task_path;
        new_task.task_loss = Rc::new(self.task_loss.as_ref().clone());
        new_task.task_mode = self.task_mode.clone();
        new_task.grader = TaskGrader::new(new_task.task_loss.clone(), Rc::clone(&new_task.info));
        new_task.skills = self.skills.clone();
        new_task.xp = self.xp;
        new_task.target = self.target.clone();
        new_task.quest_key = self.quest_key.clone();
        new_task.remote_name = self.remote_name.clone();
        new_task.__origin_folder = self.__origin_folder.clone();
        new_task.__workspace_folder = self.__workspace_folder.clone();
        new_task.__is_recheable = self.__is_recheable;

        new_task
    }

    fn ljust(s: &str, width: usize, pad_char: char) -> String {
        let mut result = String::from(s);
        let s_len = result.chars().count();

        if s_len < width {
            result.extend(std::iter::repeat(pad_char).take(width - s_len));
        }

        result
    }

    pub fn get_full_title(&self, mut key_pad: Option<usize>, pad_char: char) -> String {
        let key: &str = self.task.get_key();
        if key_pad.is_none() {
            key_pad = Some(key.chars().count());
        }
        if !self.task.get_title().contains(&format!("@{key}")) {
            return format!("@{} {}", 
            Self::ljust(key, key_pad.unwrap(), pad_char).as_str(), 
            self.task.get_title()
            );
        }

        self.task.get_title().to_string()
    }

    pub fn set_reachable(&mut self, reachable: bool) -> &mut Self {
        self.__is_recheable = reachable;

        self
    }

    pub fn get_origin_folder(&self) -> &Option<PathBuf> {
        &self.__origin_folder
    }

    pub fn get_workspace_folder(&self) -> &Option<PathBuf> {
        &self.__workspace_folder
    }


    pub fn is_optional(&self) -> bool {
        self.task_path == TaskMain::SIDE
    }

    pub fn is_auto(&self) -> bool {
        self.task_test == TaskTest::TEST
    }

    pub fn is_reachable(&self) -> bool {
        self.__is_recheable
    }

    pub fn is_link(&self) -> bool {
        if self.task_mode == TaskEdit::VIEW {
            return true;
        }
        self.__origin_folder.is_none() && self.__workspace_folder.is_none()
    }

    pub fn set_remote_view_type(&mut self) -> &mut Self {
        self.__origin_folder = None;
        self.__workspace_folder = None;

        self
    }

    pub fn is_import_type(&self) -> bool {
        self.task_mode == TaskEdit::EDIT && self.__origin_folder != None && self.__workspace_folder != None
    }

    pub fn is_static_type(&self) -> bool {
        if self.is_link() {
            return false;
        }
        self.get_origin_folder() == self.get_workspace_folder()
    }

    pub fn set_origin_folder(&mut self, folder: PathBuf) -> &mut Self {
        self.__origin_folder = Some(folder);
        self
    }

    pub fn get_origin_readme(&self) -> PathBuf {
        let origin_folder = self.get_origin_folder();
        match origin_folder {
            Some(folder) => folder.join("README.md"),
            None => PathBuf::new()
        }
    }

    pub fn set_workspace_folder(&mut self, folder: PathBuf) -> std::io::Result<&mut Self> {
        self.__workspace_folder = Some(folder.canonicalize()?);
        Ok(self)
    }

    pub fn decode_from_dict(&mut self, value: &str) {
        // saturating sub extrai 1 com segurança, sem deixar ficar negativo
        let value_list = &value[1..value.len().saturating_sub(1)];
        let mut kv_dict: HashMap<String, String> = HashMap::new();

        for kv in value_list.split(",") {
            let Some((k, val)) = kv.split_once(":") else {
                continue;
            };
            kv_dict.insert(k.trim().to_string(), val.trim().to_string());
        }
        self.info.borrow_mut().load_from_kv(&kv_dict);

        if kv_dict.contains_key(STR_INDEX) {
            match kv_dict.get(STR_INDEX) {
                Some(key) => {
                    match key.parse::<i32>() {
                        Ok(integer) => self.main_idx = integer,
                        Err(e) => panic!("Error: {}", e)
                    }
                },
                None => ()
            }
        }
    }

    pub fn is_db_empty(&self) -> bool {
        self.info.borrow().get_kv().len() == 0
    }

    pub fn get_rate_color(&self, value: i32, min_value: Option<i32>) -> String {
        let prog = value;
        if prog == 0 {
            return String::from("c");
        }
        else if prog < min_value.unwrap_or(self.default_min_value) {
            return String::from("r");
        }
        else if prog < 10 {
            return String::from("y");
        }
        else if prog == 10 {
            return String::from("g");
        }
        "w".to_string()
    }

    pub fn get_rate_symbol(&self, value: i32, min_value: Option<i32>) -> Text {
        let min_value = min_value.unwrap_or(self.default_min_value);
        let color = self.get_rate_color(value, Some(min_value));
        let prog = value;
        let mut text = Text::new(None, None);
        if prog == 0 {
            text.add(Some(AddValue::Str(Cow::Owned("x".to_string()))));
            return text;
        }
        else if prog < min_value {
            text.addf(color, Some(AddValue::Str(Cow::Owned(prog.to_string()))));
            return text;
        }
        else if prog < 10 {
            text.addf(color, Some(AddValue::Str(Cow::Owned(prog.to_string()))));
            return text;
        }
        else if prog == 10 {
            text.addf(color, Some(AddValue::Str(Cow::Borrowed(&symbols::CHECK.to_string()))));
            return text;
        }
        text.add(Some(AddValue::Str(Cow::Owned("0".to_string()))));
        text
    }

    pub fn get_xp(&self) -> i32 {
        if self.xp == 0 {
            return 1;
        }
        self.xp
    }

    pub fn get_rate_percent(&self) -> f64 {
        let value = self.grader.get_rate_percent();
        if value < 0.1 {
            return 0.0;
        }
        value
    }

    pub fn get_quality_percent(&self) -> f64 {
        if *self.task_loss == TaskLoss::FREE {
            return 100.0;
        }
        let value = self.grader.get_quality_percent();
        if value < 0.1 {
            return 0.0;
        }
        value
    }

    pub fn get_ratio(&self) -> f64 {
        self.grader.get_ratio()
    }

    pub fn is_complete(&self) -> bool {
        self.grader.get_rate_percent() >= 70.0
    }

    pub fn not_started(&self) -> bool {
        self.grader.get_rate_percent() == 0.0
    }

    pub fn in_progress(&self) -> bool {
        self.grader.get_rate_percent() < 100.0
    }

    pub fn has_at_symbol(&self) -> bool {
        self.task.get_title()
        .split_whitespace()
        .any(|s| s.starts_with('@'))
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lnum = format!("{:>3}", self.line_number);

        let full_key = self.task.get_key();
        let title = self.task.get_title();

        let key = if full_key == title {
            String::new()
        } else {
            format!("{} ", full_key)
        };

        write!(
            f,
            "{} key:{} title:{} skills:{:?} remote:{}",
            lnum,
            key,
            title,
            self.skills,
            self.target
        )
    }
}