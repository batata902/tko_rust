use std::collections::HashMap;


use crate::game::tree_item::TreeItem;
use crate::game::task_info::TaskInfo;
use crate::utils::symbols;

pub enum TaskTest {
    NULL,
    TEST,
    SELF
}

pub enum TaskMain {
    MAIN,
    SIDE
}

#[derive(PartialEq, Eq, Hash)]
pub enum TaskLoss {
    NULL,
    FREE,
    PART,
    ZERO
}

pub enum TaskEdit {
    VIEW,
    EDIT
}

pub struct TaskGrader {
    info: TaskInfo,
    loss: TaskLoss,
    grades: HashMap<TaskLoss, HashMap<String, i32>>
}

impl TaskGrader {
    pub fn new(task_loss: TaskLoss, task_info: TaskInfo) -> Self {
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


pub struct Task {
    // Implementar struct
}