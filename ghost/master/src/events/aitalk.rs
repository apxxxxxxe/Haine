use crate::events::common::*;
use shiorust::message::{Response, Request};

pub fn version(_req: &Request) -> Response {
    new_response_with_value(&String::from(env!("CARGO_PKG_VERSION")))
}

pub fn on_ai_talk(_req: &Request) -> Response {
    let talks = [
        "\\0\\s[1111201]おはようございます",
        "\\0\\s[1111207]こんにちは",
        "\\0\\s[1111204]こんばんは",
        "\\0\\s[1111209]おやすみなさい",
    ];

    let talk = choose_one(&talks).unwrap();

    new_response_with_value(&talk)
}
