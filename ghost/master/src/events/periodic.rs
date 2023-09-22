use crate::events::aitalk::on_ai_talk;
use crate::events::common::*;
use crate::events::GlobalVariables;
use shiorust::message::{Request, Response};

pub fn on_second_change(req: &Request, vars: &mut GlobalVariables) -> Response {
    vars.total_time += 1;
    vars.ghost_up_time += 1;

    if vars.ghost_up_time % vars.random_talk_interval == 0 {
        on_ai_talk(req, vars)
    } else {
        new_response_nocontent()
    }
}
