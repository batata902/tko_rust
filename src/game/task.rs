use std::collections::HashMap;
use std::fmt;
use std::borrow::Cow;


use crate::game::task_config::{TaskConfig, TaskMain, TaskTest, TaskLoss, TaskGrader};
use crate::game::tree_item::{TreeItem, TreeUi};
use crate::game::task_info::TaskSelfInfo;
use crate::game::task_location::TaskLocation;
use crate::utils::symbols;
use crate::utils::text::{AddValue, Text};


#[derive(Clone)]
pub struct Task {
    pub identity: TreeItem,
    pub ui: TreeUi,

    pub info: TaskSelfInfo,
    pub config: TaskConfig,
    pub main_idx: i32,

    pub skills: HashMap<String, i32>,

    pub xp: i32,

    pub quest_key: String,
    pub __is_recheable: bool,
    pub default_min_value: i32,

    pub location: TaskLocation
}

static STR_INDEX: &str = "idx";

impl Task {
    pub fn new() -> Self {
        let identity: TreeItem = TreeItem::new();
        let ui = TreeUi::new();
        let config = TaskConfig::default();

        let info: TaskSelfInfo = TaskSelfInfo::new();
        let main_idx: i32 = 0;

        let skills: HashMap<String, i32> = HashMap::new();

        let xp: i32 = 1;

        let quest_key: String = String::from("");
        let __is_recheable: bool = false;
        let default_min_value: i32 = 5;
        let location = TaskLocation::new();

        Self {
            identity,
            ui,
            config,
            info, 
            main_idx, 
            skills, 
            xp, 
            quest_key, 
            __is_recheable, 
            default_min_value,
            location
        }
    }

    fn grader(&self) -> TaskGrader {
        self.config.build_grader(&self.info)
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
        let key: &str = self.identity.get_key();
        if key_pad.is_none() {
            key_pad = Some(key.chars().count());
        }
        if !self.identity.get_title().contains(&format!("@{key}")) {
            return format!("@{} {}", 
            Self::ljust(key, key_pad.unwrap(), pad_char).as_str(), 
            self.identity.get_title()
            );
        }

        self.identity.get_title().to_string()
    }

    pub fn set_reachable(&mut self, reachable: bool) -> &mut Self {
        self.__is_recheable = reachable;

        self
    }


    pub fn is_optional(&self) -> bool {
        self.config.path == TaskMain::SIDE
    }

    pub fn is_auto(&self) -> bool {
        self.config.test == TaskTest::TEST
    }

    pub fn is_reachable(&self) -> bool {
        self.__is_recheable
    }

    pub fn is_link(&self) -> bool {
        self.location.is_link(self.config.mode)
    }

    pub fn is_import_type(&self) -> bool {
        self.location.is_import_type(self.config.mode)
    }

    pub fn is_static_type(&self) -> bool {
        self.location.is_static_type(self.config.mode)
    }

    pub fn is_db_empty(&self) -> bool {
        self.info.get_kv().len() == 0
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
        self.info.load_from_kv(&kv_dict);

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
        let value = self.grader().get_rate_percent();
        if value < 0.1 {
            return 0.0;
        }
        value
    }

    pub fn get_quality_percent(&self) -> f64 {
        if self.config.loss == TaskLoss::FREE {
            return 100.0;
        }
        let value = self.grader().get_quality_percent();
        if value < 0.1 {
            return 0.0;
        }
        value
    }

    pub fn get_ratio(&self) -> f64 {
        self.grader().get_ratio()
    }

    pub fn is_complete(&self) -> bool {
        self.grader().get_rate_percent() >= 70.0
    }

    pub fn not_started(&self) -> bool {
        self.grader().get_rate_percent() == 0.0
    }

    pub fn in_progress(&self) -> bool {
        self.grader().get_rate_percent() < 100.0
    }

    pub fn has_at_symbol(&self) -> bool {
        self.identity.get_title()
        .split_whitespace()
        .any(|s| s.starts_with('@'))
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lnum = format!("{:>3}", self.location.line_number);

        let full_key = self.identity.get_key();
        let title = self.identity.get_title();

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
            self.location.target
        )
    }
}