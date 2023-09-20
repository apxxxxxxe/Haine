mod aitalk;
mod bootend;
mod common;
mod menu;
mod mouse;
mod periodic;

use crate::events::aitalk::*;
use crate::events::bootend::*;
use crate::events::common::*;
use crate::events::menu::*;
use crate::events::mouse::*;
use crate::events::periodic::*;

use shiorust::message::{parts::*, traits::*, Request, Response};

const VAR_PATH: &str = "vars.json";

pub struct GlobalVariables {
    pub hoge: i32,
}

impl GlobalVariables {
    pub fn new() -> Self {
        Self { hoge: 0 }
    }

    pub fn load(&mut self) {
        let json = match std::fs::read_to_string(VAR_PATH) {
            Ok(json) => json,
            Err(_) => return,
        };
        let json: serde_json::Value = match serde_json::from_str(&json) {
            Ok(json) => json,
            Err(_) => return,
        };
        match json["hoge"].as_i64() {
            Some(hoge) => {
                self.hoge = hoge as i32;
            },
            None => (),
        };
    }

    pub fn save(&self) {
        let json = serde_json::json!({
            "hoge": self.hoge,
        });
        match std::fs::write(VAR_PATH, json.to_string()) {
            Ok(_) => (),
            Err(_) => (),
        };
    }
}

pub fn handle_request(req: &Request) -> Response {
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

    if event_id != "OnSecondChange" {
        debug!("event: {}", event_id);
    }

    let event = match event_id.as_str() {
        "version" => version,
        "OnBoot" => on_boot,
        "OnAiTalk" => on_ai_talk,
        "OnSecondChange" => on_second_change,
        "OnMenuExec" => on_menu_exec,
        "OnMouseDoubleClick" => on_mouse_double_click,
        _ => return new_response_nocontent(),
    };

    let res = event(req);
    if event_id != "OnSecondChange" {
        debug!("response: {:?}", res);
    }
    res
}
