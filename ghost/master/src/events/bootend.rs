use crate::events::common::*;
use shiorust::message::{Response, *};

pub fn on_boot(_req: &Request) -> Response {
    new_response_with_value("h1111201Hello".to_string(), true)
}
