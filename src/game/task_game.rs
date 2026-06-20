use std::collections::HashSet;

#[derive(Clone)]
pub struct TaskGame {
    pub default_min_value: usize,
    xp: isize,
    tier: isize,
    pub skills: HashSet<String>,
    pub is_reachable: bool
}

impl TaskGame {
    pub fn new() -> Self {
        Self { 
            default_min_value: 5, 
            xp: 1, 
            tier: 1, 
            skills: HashSet::new(), 
            is_reachable: false 
        }
    }

    pub fn get_rate_color(&self, value: usize, min_value: Option<usize>) -> char {
        let min_value: usize = min_value.unwrap_or(self.default_min_value);

        if value == 0 {
            return 'c';
        }
        if value < min_value {
            return 'r';
        }
        if value < 10 {
            return 'y'
        }
        if value == 10 {
            return 'g';
        }

        'w'
    }

    pub fn xp(&self) -> isize {
        if self.xp == 0 {
            return 1;
        }

        self.xp
    }

    pub fn set_xp(&mut self, mut value: isize) {
        if value < 0 {
            value = 1;
        }
        self.xp = value
    }

    pub fn tier(&self) -> isize {
        if self.tier == 0 {
            return 1;
        }

        self.tier
    }

    pub fn set_tier(&mut self, mut value: isize) {
        if value < 0 {
            value = 1;
        }
        if value > 4 {
            value = 4;
        }

        self.tier = value
    }


}