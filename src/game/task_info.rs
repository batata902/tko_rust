use std::collections::HashMap;

pub mod Keys {
    pub static rate_str: &str = "rate";
    pub static ia_concept_str: &str = "concept";
    pub static ia_problem_str: &str = "problem";
    pub static ia_coding_str: &str = "code";
    pub static ia_debug_str: &str = "debug";
    pub static ia_refactor_str: &str = "refactor";
    pub static guided_str: &str = "guided";

    pub static study_str: &str = "study";
    pub static friend_str: &str = "friend";
    pub static feedback_str: &str = "self";
}

pub struct TaskInfo {
    rate: i32,

    study: i32,

    feedback: bool,
    friend: String,

    guided: bool,
    ia_concept: bool,
    ia_problem: bool,
    ia_code: bool,
    ia_debug: bool,
    ia_refactor: bool
}

impl TaskInfo {
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

    pub fn copy_quality_from(&mut self, other: TaskInfo) {
        self.feedback = other.feedback;
        self.friend = other.friend;

        self.guided = other.guided;
        self.ia_concept = other.ia_problem;
        self.ia_code = other.ia_code;
        self.ia_debug = other.ia_debug;
        self.ia_refactor = other.ia_refactor;
    }

    pub fn clone(&mut self) -> &mut TaskInfo {
        TaskInfo::load_from_kv(self, self.get_kv())
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

    pub fn load_from_kv(&mut self, kv: HashMap<&'static str, String>) -> &mut Self {
        if kv.contains_key(Keys::rate_str) {
            self.set_rate(kv.get(Keys::rate_str).unwrap());
        }
        if kv.contains_key(Keys::study_str) {
            self.set_study(kv.get(Keys::study_str).unwrap());
        }

        self.friend = kv.get(Keys::friend_str)
            .cloned()
            .unwrap_or("".to_string());
        self.feedback = kv.get(Keys::feedback_str)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.guided = kv.get(Keys::guided_str)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.ia_concept = kv.get(Keys::ia_concept_str)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.ia_problem = kv.get(Keys::ia_problem_str)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.ia_code = kv.get(Keys::ia_coding_str)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.ia_debug = kv.get(Keys::ia_debug_str)
            .cloned()
            .unwrap_or("0".to_string()) == "1";
        self.ia_refactor = kv.get(Keys::ia_refactor_str)
            .cloned()
            .unwrap_or("0".to_string()) == "1";

        self
    }

    pub fn get_kv(&self) -> HashMap<&'static str, String> {
        let mut kv: HashMap<&'static str, String> = HashMap::new();
        
        if self.feedback {
            kv.insert(Keys::feedback_str, "1".to_string());
        }
        if self.rate != 0 {
            kv.insert(Keys::rate_str, self.rate.to_string());
        }
        if self.study != 0 {
            kv.insert(Keys::study_str, self.study.to_string());
        }
        if self.friend != "".to_string() {
            kv.insert(Keys::friend_str, self.friend.clone());
        }
        if self.guided {
            kv.insert(Keys::guided_str, "1".to_string());
        }
        if self.ia_concept {
            kv.insert(Keys::ia_concept_str, "1".to_string());
        }
        if self.ia_problem {
            kv.insert(Keys::ia_problem_str, "1".to_string());
        }
        if self.ia_code {
            kv.insert(Keys::ia_coding_str, "1".to_string());
        }
        if self.ia_debug {
            kv.insert(Keys::ia_debug_str, "1".to_string());
        }
        if self.ia_refactor {
            kv.insert(Keys::ia_refactor_str, "1".to_string());
        }

        kv
    }
}