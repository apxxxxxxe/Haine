use crate::events::common::*;
use crate::variables::get_global_vars;
use shiorust::message::{Request, Response};

pub fn on_user_input(req: &Request) -> Response {
  let refs = get_references(req);
  let input_id = refs[0].to_string();
  let text = refs[1].to_string();
  let responser = match input_id.as_str() {
    "user_name" => input_user_name,
    _ => |_text: String| new_response_nocontent(),
  };
  responser(text)
}

fn input_user_name(text: String) -> Response {
  let vars = get_global_vars();
  vars.set_user_name(Some(text.clone()));
  let m = format!(
    "\
    h1111204そう、h1111210ならばそう呼ぶことにしましょう。\
    \\1\\_q(ユーザ名を{}に設定しました)\
    ",
    text
  );
  new_response_with_value(m, TranslateOption::none())
}

pub fn on_window_state_restore(_req: &Request) -> Response {
  let vars = get_global_vars();
  // トーク間隔をリセット
  vars
    .volatility
    .set_last_random_talk_time(vars.volatility.ghost_up_time());
  new_response_nocontent()
}
