use crate::events::common::*;
use shiorust::message::{Response, Request};

pub fn on_second_change(_req: &Request) -> Response {
    new_response_nocontent()
}
