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
use crate::variables::GlobalVariables;

use shiorust::message::{parts::*, traits::*, Request, Response};

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
        "OnMouseClickEx" => on_mouse_click_ex,
        "OnMouseDoubleClick" => on_mouse_double_click,
        "OnMouseMove" => on_mouse_move,
        "OnMouseWheel" => on_mouse_wheel,
        "OnKeyPress" => on_key_press,
        _ => return new_response_nocontent(),
    };

    let res = event(req, vars);
    debug!("response: {:?}", res);
    res
}
