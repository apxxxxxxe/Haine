use crate::events::common::*;
use crate::events::GlobalVariables;
use shiorust::message::{Request, Response};

pub fn on_second_change(_req: &Request, vars: &mut GlobalVariables) -> Response {
    vars.total_time += 1;
    vars.ghost_up_time += 1;
    new_response_nocontent()
}
