use crate::events::common::*;
use shiorust::message::{Response, *};

pub fn on_boot(_req: &Request) -> Response {
    new_response_with_value("h1111201Hello".to_string(), true)
}

pub fn on_close(_req: &Request) -> Response {
    let talks = all_combo(&vec![
        vec!["h1111209".to_string(), "h1111207".to_string()],
        vec!["あなたに".to_string()],
        vec![
            "すばらしき朝".to_string(),
            "蜜のようなまどろみ".to_string(),
            "暗くて静かな安らぎ".to_string(),
            "良き終わり".to_string(),
            "孤独と救い".to_string(),
        ],
        vec!["がありますように。\\nh1111204またね、{user_name}。\\_w[1200]".to_string()],
    ]);
    new_response_with_value(choose_one(&talks, true).unwrap(), true)
}
