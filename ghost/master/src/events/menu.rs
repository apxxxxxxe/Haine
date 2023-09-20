use crate::events::common::*;
use shiorust::message::{Response, *};

pub fn on_menu_exec(_req: &Request) -> Response {
    new_response_with_value("menu")
}
