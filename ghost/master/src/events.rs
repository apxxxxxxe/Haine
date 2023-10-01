mod aitalk;
mod bootend;
mod common;
mod key;
mod menu;
mod mouse;
mod mouse_core;
mod periodic;
mod translate;

use crate::events::aitalk::*;
use crate::events::bootend::*;
use crate::events::common::*;
use crate::events::key::*;
use crate::events::menu::*;
use crate::events::mouse_core::*;
use crate::events::periodic::*;
use crate::variables::get_global_vars;

use shiorust::message::{parts::*, traits::*, Request, Response};

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

    debug!("event: {}", event_id);

    let vars = get_global_vars();

    if let Some(v) = req.headers.get("Status") {
        vars.volatility.status.set(v.to_string());
    }

    let event = match get_event(event_id.as_str()) {
        Some(e) => e,
        None => {
            let base_id = match req.headers.get("BaseID") {
                Some(id) => id,
                None => return new_response_nocontent(),
            };
            match get_event(base_id.as_str()) {
                Some(e) => e,
                None => return new_response_nocontent(),
            }
        }
    };

    let res = event(req);
    debug!("response: {:?}", res);
    res
}

fn get_event(id: &str) -> Option<fn(&Request) -> Response> {
    match id {
        "version" => Some(version),
        "OnBoot" => Some(on_boot),
        "OnAiTalk" => Some(on_ai_talk),
        "OnSecondChange" => Some(on_second_change),
        "OnMenuExec" => Some(on_menu_exec),
        "OnMouseClickEx" => Some(on_mouse_click_ex),
        "OnMouseDoubleClick" => Some(on_mouse_double_click),
        "OnMouseMove" => Some(on_mouse_move),
        "OnMouseWheel" => Some(on_mouse_wheel),
        "OnKeyPress" => Some(on_key_press),
        _ => None,
    }
}
