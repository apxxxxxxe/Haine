use crate::events::common::*;
use crate::events::GlobalVariables;
use shiorust::message::{Response, *};

pub fn on_boot(_req: &Request, _vars: &mut GlobalVariables) -> Response {
    new_response_with_value("\\0\\s[1111201]Hello")
}
