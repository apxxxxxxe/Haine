use crate::events::aitalk::on_ai_talk;
use crate::events::common::*;
use crate::events::GlobalVariables;
use shiorust::message::{Request, Response};

pub fn on_key_press(req: &Request, vars: &mut GlobalVariables) -> Response {
    let refs = get_references(req);
    match refs[0] {
        "t" => on_ai_talk(req, vars),
        _ => new_response_nocontent(),
    }
}
