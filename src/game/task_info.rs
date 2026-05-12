use std::collections::HashMap;

pub mod keys {
    pub static RATE_STR: &str = "rate";
    pub static IA_CONCEPT_STR: &str = "concept";
    pub static IA_PROBLEM_STR: &str = "problem";
    pub static IA_CODING_STR: &str = "code";
    pub static IA_DEBUG_STR: &str = "debug";
    pub static IA_REFACTOR_STR: &str = "refactor";
    pub static GUIDED_STR: &str = "guided";

    pub static STUDY_STR: &str = "study";
    pub static FRIEND_STR: &str = "friend";
    pub static FEEDBACK_STR: &str = "self";
}

#[derive(Debug, Clone)]
pub struct TaskSelfInfo {
    pub rate: i32,

    pub study: i32,

    pub feedback: bool,
    pub friend: String,

    pub guided: bool,
    pub ia_concept: bool,
    pub ia_problem: bool,
    pub ia_code: bool,
    pub ia_debug: bool,
    pub ia_refactor: bool
}

impl TaskSelfInfo {
    pub fn new() -> Self {
        Self { 
            rate: 0, 
            study: 0, 

            feedback: false, 
            friend: String::from(""), 

            guided: false, 
            ia_concept: false, 
            ia_problem: false, 
            ia_code: false, 
            ia_debug: false, 
            ia_refactor: false 
        }
    }

    pub fn copy_quality_from(&mut self, other: TaskSelfInfo) {
        self.feedback = other.feedback;
        self.friend = other.friend;

        self.guided = other.guided;
        self.ia_concept = other.ia_problem;
        self.ia_code = other.ia_code;
        self.ia_debug = other.ia_debug;
        self.ia_refactor = other.ia_refactor;
    }

    pub fn clone(&mut self) -> &mut TaskSelfInfo {
        TaskSelfInfo::load_from_kv(self, &self.get_kv())
    }

    pub fn set_study(&mut self, value: &String) -> &mut Self {
        match value.parse::<i32>() {
            Ok(minutes) => {
                if minutes >= 0 {
                    self.study = minutes;
                }
            },
            Err(_) => {
                self.study = 0;
            }
        }
        self
    }

    pub fn set_rate(&mut self, value: &String) -> &mut Self {
        match value.parse::<i32>() {
            Ok(rate) => {
                if 0 <= rate && rate <= 100 {
                    self.rate = rate;
                }
            },
            Err(_) => {
                self.rate = 0;
            }
        }
        self
    }

    pub fn load_from_kv(&mut self, kv: &HashMap<String, String>) -> &mut Self {
        if kv.contains_key(keys::RATE_STR) {
            self.set_rate(kv.get(keys::RATE_STR).unwrap());
        }
        if kv.contains_key(keys::STUDY_STR) {
            self.set_study(kv.get(keys::STUDY_STR).unwrap());
        }

        self.friend = kv.get(keys::FRIEND_STR)
            .cloned()
            .unwrap_or("".to_string());
        self.feedback = kv.get(keys::FEEDBACK_STR)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.guided = kv.get(keys::GUIDED_STR)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.ia_concept = kv.get(keys::IA_CONCEPT_STR)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.ia_problem = kv.get(keys::IA_PROBLEM_STR)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.ia_code = kv.get(keys::IA_CODING_STR)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.ia_debug = kv.get(keys::IA_DEBUG_STR)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.ia_refactor = kv.get(keys::IA_REFACTOR_STR)
            .cloned()
            .unwrap_or("0".to_string()) == "1";

        self
    }

    pub fn get_kv(&self) -> HashMap<String, String> {
        let mut kv: HashMap<String, String> = HashMap::new();
        
        if self.feedback {
            kv.insert(keys::FEEDBACK_STR.to_string(), "1".to_string());
        }
        if self.rate != 0 {
            kv.insert(keys::RATE_STR.to_string(), self.rate.to_string());
        }
        if self.study != 0 {
            kv.insert(keys::STUDY_STR.to_string(), self.study.to_string());
        }
        if self.friend != "".to_string() {
            kv.insert(keys::FRIEND_STR.to_string(), self.friend.clone());
        }
        if self.guided {
            kv.insert(keys::GUIDED_STR.to_string(), "1".to_string());
        }
        if self.ia_concept {
            kv.insert(keys::IA_CONCEPT_STR.to_string(), "1".to_string());
        }
        if self.ia_problem {
            kv.insert(keys::IA_PROBLEM_STR.to_string(), "1".to_string());
        }
        if self.ia_code {
            kv.insert(keys::IA_CODING_STR.to_string(), "1".to_string());
        }
        if self.ia_debug {
            kv.insert(keys::IA_DEBUG_STR.to_string(), "1".to_string());
        }
        if self.ia_refactor {
            kv.insert(keys::IA_REFACTOR_STR.to_string(), "1".to_string());
        }

        kv
    }
}