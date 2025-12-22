use crate::system::error::ShioriError;
use crate::system::response::*;
use crate::system::variables::*;
use shiorust::message::{Request, Response};
use std::fmt;
use std::fmt::{Display, Formatter};

pub(crate) enum InputId {
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

pub(crate) fn on_user_input(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let input_id = if let Some(input_id) = InputId::from_str(refs[0]) {
    input_id
  } else {
    error!("Unknown input id: {}", refs[0]);
    return Ok(new_response_nocontent());
  };
  let text = refs[1].to_string();
  let responser = match input_id {
    InputId::UserName => input_user_name,
  };
  responser(text)
}

fn input_user_name(text: String) -> Result<Response, ShioriError> {
  *USER_NAME.write().unwrap() = text.clone();
  let m = format!(
    "\
    h1111204そう、h1111210ならばそう呼ぶことにしましょう。\
    \\1\\_q(ユーザ名を{}に設定しました)\
    ",
    text
  );
  new_response_with_value_with_translate(m, TranslateOption::simple_translate())
}

pub(crate) fn on_window_state_restore(_req: &Request) -> Result<Response, ShioriError> {
  // トーク間隔をリセット
  *LAST_RANDOM_TALK_TIME.write().unwrap() = *GHOST_UP_TIME.read().unwrap();

  new_response_with_value_with_translate(
    "\\p[2]\\s[10000000]\\0\\s[1111110]h1111204".to_string(),
    TranslateOption::simple_translate(),
  )
}
