pub struct Elem {
    opt: bool,
    value: i32,
    percent: f64
}

impl Elem {
    pub fn new(opt: bool, value: i32, percent: f64) -> Self {
        Self { opt, value, percent }
    }
}

pub struct QuestGrader;

impl QuestGrader {
    pub fn calc_xp_earned_total(tasks: Vec<Elem>) -> (f64, f64) {
        let mut total_xp = 0.0;
        let mut earned_xp = 0.0;

        for elem in tasks {
            total_xp += elem.value as f64;
            if elem.percent > 1.0 {
                earned_xp += elem.value as f64 * (elem.percent / 100.0);
            }
        }
        (earned_xp, total_xp)
    }

    pub fn get_percent(earned_xp: f64, total_xp: f64) -> Option<f64> {
        if total_xp == 0.0 {
            return None;
        }
        Some((earned_xp * 100.0) / total_xp)
    }
}