use crate::events::common::*;
use shiorust::message::{Request, Response};

pub(crate) fn show_tooltip(id: &str) -> String {
  format!("\\__q[OnBalloonTooltip,{}]{}\\__q", id, Icon::Info)
}

pub(crate) fn on_balloon_tooltip(_req: &Request) -> Response {
  new_response_with_value_with_notranslate("\\C\\_l[0,0] ".to_string(), TranslateOption::none())
}

pub(crate) fn balloon_tooltip(req: &Request) -> Response {
  let refs = get_references(req);
  if refs[1] != "OnBalloonTooltip" {
    return new_response_nocontent();
  }
  match refs[2] {
    "WhatIsImersiveDegree" => new_response_with_value_with_notranslate(
      "ハイネがどれだけ思索に没頭しているかを表す指標です。\\n\
      最大になったとき、彼女は極度の集中状態に陥るでしょう。"
        .to_string(),
      TranslateOption::none(),
    ),
    _ => new_response_nocontent(),
  }
}
