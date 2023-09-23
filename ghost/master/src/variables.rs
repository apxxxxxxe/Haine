use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const VAR_PATH: &str = "vars.json";

#[derive(Serialize, Deserialize)]
pub struct GlobalVariables {
    // ゴーストの累計起動時間(秒数)
    pub total_time: Option<u64>,

    // ランダムトークの間隔(秒数)
    pub random_talk_interval: Option<u64>,

    // ユーザ名
    pub user_name: Option<String>,

    // 起動ごとにリセットされる変数
    #[serde(skip)]
    pub volatility: VolatilityVariables,
}

impl GlobalVariables {
    pub fn new() -> Self {
        Self {
            total_time: Some(0),
            random_talk_interval: Some(180),
            user_name: Some("ユーザ".to_string()),
            volatility: VolatilityVariables::default(),
        }
    }

    pub fn load(&mut self) {
        let json_str = match std::fs::read_to_string(VAR_PATH) {
            Ok(s) => s,
            Err(_) => {
                error!("Failed to load variables.");
                return;
            }
        };

        let vars: GlobalVariables = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(_) => {
                error!("Failed to parse variables.");
                return;
            }
        };

        // TODO: 変数追加時はここに追加することを忘れない
        if let Some(t) = vars.total_time {
            self.total_time = Some(t);
        }
        if let Some(t) = vars.random_talk_interval {
            self.random_talk_interval = Some(t);
        }
        if let Some(t) = vars.user_name {
            self.user_name = Some(t);
        }
    }

    pub fn save(&self) {
        let json_str = match serde_json::to_string(self) {
            Ok(s) => s,
            Err(_) => {
                error!("Failed to serialize variables");
                return;
            }
        };
        match std::fs::write(VAR_PATH, json_str) {
            Ok(_) => (),
            Err(_) => {
                error!("Failed to save variables");
                return;
            }
        };
    }
}

#[derive(PartialEq)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    pub fn to_str(&self) -> &str {
        match self {
            Direction::Up => "up",
            Direction::Down => "down",
        }
    }
}

// ゴーストのグローバル変数のうち、揮発性(起動毎にリセットされる)なもの
pub struct VolatilityVariables {
    // ゴーストが起動してからの秒数
    pub ghost_up_time: u64,

    // ゴーストの起動日時
    pub ghost_boot_time: SystemTime,

    pub nade_counter: i32,

    pub last_nade_count_unixtime: SystemTime,

    pub last_nade_part: String,

    pub wheel_direction: Direction,

    pub wheel_counter: i32,

    pub last_wheel_count_unixtime: SystemTime,

    pub last_wheel_part: String,
}

impl Default for VolatilityVariables {
    fn default() -> Self {
        Self {
            ghost_up_time: 0,
            ghost_boot_time: SystemTime::now(),
            nade_counter: 0,
            last_nade_count_unixtime: UNIX_EPOCH,
            last_nade_part: "".to_string(),
            wheel_direction: Direction::Up,
            wheel_counter: 0,
            last_wheel_count_unixtime: UNIX_EPOCH,
            last_wheel_part: "".to_string(),
        }
    }
}
