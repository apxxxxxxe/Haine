use crate::events::aitalk::on_ai_talk;
use crate::events::common::*;
use shiorust::message::{Request, Response};

pub fn on_key_press(req: &Request) -> Response {
    let refs = get_references(req);
    match refs[0] {
        "t" => on_ai_talk(req),
        _ => new_response_nocontent(),
    }
}
