use crate::events::common::*;
use crate::events::menu::on_menu_exec;
use shiorust::message::{Response, *};

pub fn on_mouse_double_click(req: &Request) -> Response {
    let refs = get_references(req);
    if refs[4] == "" {
        on_menu_exec(req)
    } else {
        new_response_with_value(refs[4])
    }
}
