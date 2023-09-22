mod aitalk;
mod bootend;
mod common;
mod menu;
mod mouse;
mod periodic;
mod translate;

use crate::events::aitalk::*;
use crate::events::bootend::*;
use crate::events::common::*;
use crate::events::menu::*;
use crate::events::mouse::*;
use crate::events::periodic::*;

use serde::{Deserialize, Serialize};
use shiorust::message::{parts::*, traits::*, Request, Response};

const VAR_PATH: &str = "vars.json";

#[derive(Serialize, Deserialize)]
pub struct GlobalVariables {
    // ゴーストの累計起動時間(秒数)
    pub total_time: u64,

    // ゴーストが起動してからの秒数
    #[serde(skip)]
    pub ghost_up_time: u64,

    // ランダムトークの間隔(秒数)
    pub random_talk_interval: u64,
}

impl GlobalVariables {
    pub fn new() -> Self {
        Self {
            ghost_up_time: 0,
            total_time: 0,
            random_talk_interval: 180,
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
        self.total_time = vars.total_time;
        self.random_talk_interval = vars.random_talk_interval;
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

pub fn handle_request(req: &Request, vars: &mut GlobalVariables) -> Response {
    match req.method {
        Method::GET => (),
        _ => return new_response_nocontent(),
    };

    let event_id;
    match req.headers.get("ID") {
        Some(id) => {
            event_id = id;
        }
        None => return new_response_nocontent(),
    };

    debug!("event: {}", event_id);

    let event = match event_id.as_str() {
        "version" => version,
        "OnBoot" => on_boot,
        "OnAiTalk" => on_ai_talk,
        "OnSecondChange" => on_second_change,
        "OnMenuExec" => on_menu_exec,
        "OnMouseDoubleClick" => on_mouse_double_click,
        _ => return new_response_nocontent(),
    };

    let res = event(req, vars);
    debug!("response: {:?}", res);
    res
}
