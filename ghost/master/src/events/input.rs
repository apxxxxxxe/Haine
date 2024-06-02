use crate::events::common::*;
use crate::variables::get_global_vars;
use shiorust::message::{Request, Response};
use std::fmt;
use std::fmt::{Display, Formatter};

pub enum InputId {
  UserName,
}

impl Display for InputId {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::UserName => write!(f, "user_name"),
    }
  }
}

impl InputId {
  pub fn from_str(s: &str) -> Option<Self> {
    match s {
      "user_name" => Some(Self::UserName),
      _ => None,
    }
  }
}

pub fn on_user_input(req: &Request) -> Response {
  let refs = get_references(req);
  let input_id = if let Some(input_id) = InputId::from_str(refs[0]) {
    input_id
  } else {
    error!("Unknown input id: {}", refs[0]);
    return new_response_nocontent();
  };
  let text = refs[1].to_string();
  let responser = match input_id {
    InputId::UserName => input_user_name,
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
  new_response_with_value(m, TranslateOption::simple_translate())
}

pub fn on_window_state_restore(_req: &Request) -> Response {
  let vars = get_global_vars();
  // トーク間隔をリセット
  vars
    .volatility
    .set_last_random_talk_time(vars.volatility.ghost_up_time());
  new_response_nocontent()
}
