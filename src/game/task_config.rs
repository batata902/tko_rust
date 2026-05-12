use std::collections::HashMap;

use crate::game::task_info::TaskSelfInfo;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskTest {
    NULL,
    TEST,
    SELF
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskMain {
    MAIN,
    PERK,
    SIDE
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TaskLoss {
    NULL,
    FREE,
    PART,
    ZERO
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct TaskGrader <'a>{
    info: &'a TaskSelfInfo,
    loss: TaskLoss,
    grades: HashMap<TaskLoss, HashMap<String, i32>>
}

impl <'a> TaskGrader <'a> {
    pub fn new(task_loss: TaskLoss, task_info: &'a TaskSelfInfo) -> Self {
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
        let rate = self.info.rate as f64;
        rate
    }

    pub fn get_quality_percent(&self) -> f64 {
        if !self.info.feedback {
            return 0.0;
        }
        let mut rate = 100.0;
        if self.info.guided {
            let value = self.grades
                .get(&self.loss).unwrap()
                .get("guided").unwrap();
            rate *= *value as f64 / 100.0;
        }
        if self.info.ia_code {
            let value = self.grades
                .get(&self.loss).unwrap()
                .get("code").unwrap();
            rate *= *value as f64 / 100.0;
        }
        if self.info.ia_debug {
            let value = self.grades
                .get(&self.loss).unwrap()
                .get("debug").unwrap();
            rate *= *value as f64 / 100.0;
        }
        if self.info.ia_problem {
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

#[derive(Debug, Clone)]
pub struct TaskConfig  {
    pub test: TaskTest,
    pub path: TaskMain,
    pub loss: TaskLoss,
    pub mode: TaskEdit
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self { 
            test: TaskTest::TEST, 
            path: TaskMain::MAIN, 
            loss: TaskLoss::PART, 
            mode: TaskEdit::EDIT 
        }
    }
}

impl TaskConfig {
    pub fn build_grader<'a>(&self, info: &'a TaskSelfInfo) -> TaskGrader<'a> {
        TaskGrader::new(self.loss, info)
    }
}