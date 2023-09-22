use crate::events::common::*;
use crate::events::GlobalVariables;
use shiorust::message::{Response, *};

pub fn on_boot(_req: &Request, _vars: &mut GlobalVariables) -> Response {
    new_response_with_value("h1111201Hello".to_string(), true)
}
