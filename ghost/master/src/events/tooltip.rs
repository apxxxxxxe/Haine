use crate::events::common::*;
use shiorust::message::{Request, Response};

pub fn show_tooltip(id: &str) -> String {
  format!(
    "\\__q[OnBalloonTooltip,{}]\
    {}
    \\__q",
    id,
    Icon::Info,
  )
}

pub fn on_balloon_tooltip(_req: &Request) -> Response {
  new_response_with_value("\\C\\_l[0,0] ".to_string(), false)
}

pub fn balloon_tooltip(req: &Request) -> Response {
  let refs = get_references(req);
  if refs[1] != "OnBalloonTooltip" {
    return new_response_nocontent();
  }
  match refs[2] {
    "WhatIsImersiveDegree" => new_response_with_value(
      "没入度は、ハイネとあなたがどれだけ深い内容の会話をしているかを表します。\\n\
                                  没入度が高いほど、より抽象的でクリティカルな話題を扱います。"
        .to_string(),
      false,
    ),
    _ => new_response_nocontent(),
  }
}
