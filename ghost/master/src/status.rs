use std::sync::Mutex;

pub struct Status {
    talking: Mutex<bool>,
    choosing: Mutex<bool>,
    minimizing: Mutex<bool>,
    induction: Mutex<bool>,
    passive: Mutex<bool>,
    timecritical: Mutex<bool>,
    nouserbreak: Mutex<bool>,
    online: Mutex<bool>,
    // opening
    // balloon
}

impl Status {
    pub fn new() -> Self {
        Self {
            talking: Mutex::new(false),
            choosing: Mutex::new(false),
            minimizing: Mutex::new(false),
            induction: Mutex::new(false),
            passive: Mutex::new(false),
            timecritical: Mutex::new(false),
            nouserbreak: Mutex::new(false),
            online: Mutex::new(false),
        }
    }

    pub fn set(&mut self, status: String) {
        *self.talking.lock().unwrap() = status.contains("talking");
        *self.choosing.lock().unwrap() = status.contains("choosing");
        *self.minimizing.lock().unwrap() = status.contains("minimizing");
        *self.induction.lock().unwrap() = status.contains("induction");
        *self.passive.lock().unwrap() = status.contains("passive");
        *self.timecritical.lock().unwrap() = status.contains("timecritical");
        *self.nouserbreak.lock().unwrap() = status.contains("nouserbreak");
        *self.online.lock().unwrap() = status.contains("online");
    }

    pub fn get(&mut self, status: &str) -> Option<bool> {
        match status {
            "talking" => Some(*self.talking.lock().unwrap()),
            "choosing" => Some(*self.choosing.lock().unwrap()),
            "minimizing" => Some(*self.minimizing.lock().unwrap()),
            "induction" => Some(*self.induction.lock().unwrap()),
            "passive" => Some(*self.passive.lock().unwrap()),
            "timecritical" => Some(*self.timecritical.lock().unwrap()),
            "nouserbreak" => Some(*self.nouserbreak.lock().unwrap()),
            "online" => Some(*self.online.lock().unwrap()),
            _ => None,
        }
    }
}
