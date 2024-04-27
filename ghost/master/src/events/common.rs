use crate::events::aitalk::TalkingPlace;
use crate::events::translate::on_translate;
use crate::roulette::RouletteCell;
use crate::variables::get_global_vars;
use core::fmt::{Display, Formatter};
use std::collections::HashSet;

use shiorust::message::{parts::HeaderName, parts::*, traits::*, Request, Response};

pub const REMOVE_BALLOON_NUM: &str = "\\0\\![set,balloonnum,,,]";

pub const STICK_SURFACE: &str = "\
  \\C\
  \\1\
  \\![reset,sticky-window]\
  \\![set,alignmenttodesktop,free]\
  \\![move,--X=0,--Y=0,--time=0,--base=0]\
  \\![set,sticky-window,1,0]\
  \\0\
  ";

pub fn on_stick_surface(_req: &Request) -> Response {
  // \1のサーフェスを\0に重ねて固定する
  new_response_with_value(STICK_SURFACE.to_string(), TranslateOption::none())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TranslateOption {
  DoTranslate,
  CompleteShadow,
  CompleteBalloonSurface,
}

impl TranslateOption {
  fn new(options: Vec<TranslateOption>) -> HashSet<TranslateOption> {
    options.into_iter().collect()
  }

  pub fn none() -> HashSet<TranslateOption> {
    TranslateOption::new(vec![])
  }

  pub fn balloon_surface_only() -> HashSet<TranslateOption> {
    TranslateOption::new(vec![TranslateOption::CompleteBalloonSurface])
  }

  pub fn simple_translate() -> HashSet<TranslateOption> {
    TranslateOption::new(vec![
      TranslateOption::DoTranslate,
      TranslateOption::CompleteBalloonSurface,
    ])
  }

  pub fn with_shadow_completion() -> HashSet<TranslateOption> {
    TranslateOption::new(vec![
      TranslateOption::DoTranslate,
      TranslateOption::CompleteShadow,
      TranslateOption::CompleteBalloonSurface,
    ])
  }
}

pub fn add_error_description(res: &mut Response, error: &str) {
  res
    .headers
    .insert(HeaderName::from("ErrorDescription"), error.to_string());
  res
    .headers
    .insert(HeaderName::from("ErrorLevel"), "error".to_string());
}

pub fn new_response() -> Response {
  let mut headers = Headers::new();
  headers.insert(
    HeaderName::Standard(StandardHeaderName::Charset),
    String::from("UTF-8"),
  );
  Response {
    version: Version::V30,
    status: Status::OK,
    headers,
  }
}

pub fn new_response_nocontent() -> Response {
  let mut r = new_response();
  r.status = Status::NoContent;
  r
}

pub fn new_response_with_value(value: String, option: HashSet<TranslateOption>) -> Response {
  let vars = get_global_vars();

  let balloon_completion = if option.contains(&TranslateOption::CompleteBalloonSurface) {
    format!("\\b[{}]", vars.volatility.talking_place().balloon_surface(),)
  } else {
    String::new()
  };

  let v = if option.contains(&TranslateOption::DoTranslate) {
    if vars.volatility.inserter_mut().is_ready() {
      on_translate(value, option.contains(&TranslateOption::CompleteShadow))
    } else {
      vars.volatility.set_waiting_talk(Some((value, option)));
      "\\1Loading...\\_w[1000]\\![raise,OnWaitTranslater]".to_string()
    }
  } else {
    value
  };

  let mut v = balloon_completion + v.as_str();
  // \\Cが含まれているなら文頭に\\Cを補完
  if v.contains("\\C") {
    v = format!("\\C{}", v.replace("\\C", ""));
  }

  let mut r = new_response();
  r.headers.insert(HeaderName::from("Value"), v);
  r
}

pub fn choose_one(values: &[impl RouletteCell], update_weight: bool) -> Option<usize> {
  if values.is_empty() {
    return None;
  }
  let vars = get_global_vars();
  let u = vars
    .volatility
    .talk_bias_mut()
    .roulette(values, update_weight);
  Some(u)
}

// return all combinations of values
// e.g. [a, b], [c, d], [e, f] => "ace", "acf", "ade", "adf", "bce", "bcf", "bde", "bdf"
pub fn all_combo(values: &Vec<Vec<String>>) -> Vec<String> {
  let mut result = Vec::new();
  let mut current = Vec::new();
  all_combo_inner(values, &mut result, &mut current, 0);
  result.iter().map(|v| v.join("")).collect()
}

fn all_combo_inner(
  values: &Vec<Vec<String>>,
  result: &mut Vec<Vec<String>>,
  current: &mut Vec<String>,
  index: usize,
) {
  if index == values.len() {
    result.push(current.clone());
    return;
  }
  for v in values[index].iter() {
    current.push(v.to_string());
    all_combo_inner(values, result, current, index + 1);
    current.pop();
  }
}

pub fn get_references(req: &Request) -> Vec<&str> {
  let mut references: Vec<&str> = Vec::new();
  let mut i = 0;
  while let Some(value) = req
    .headers
    .get(&HeaderName::from(&format!("Reference{}", i)))
  {
    references.push(value);
    i += 1;
  }
  references
}

pub fn user_talk(dialog: &str, text: &str, text_first: bool) -> String {
  let mut d = String::new();
  if !dialog.is_empty() {
    d = format!("『{}』", dialog);
  }
  let mut t = String::new();
  if !text.is_empty() {
    t = text.to_string();
  }

  let mut v: Vec<String>;
  if text_first {
    v = vec![t, d];
  } else {
    v = vec![d, t];
  }
  v = v
    .iter()
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string())
    .collect();

  format!("\\1{}\\n", v.join("\\n"))
}

fn complete_shadow(is_complete: bool) -> String {
  const DEFAULT_Y: i32 = -700;
  const MAX_Y: i32 = -350;
  let vars = get_global_vars();
  if is_complete {
    let degree = if vars.volatility.talking_place() == TalkingPlace::Library {
      100 - vars.volatility.immersive_degrees()
    } else {
      vars.volatility.immersive_degrees()
    };
    format!(
      "\\0\\![bind,ex,没入度用,1]\\![anim,offset,800100,0,{}]",
      ((MAX_Y - DEFAULT_Y) as f32 * (degree as f32 / 100.0)) as i32 + DEFAULT_Y,
    )
  } else {
    "\\0\\![bind,ex,没入度用,0]".to_string()
  }
}

// サーフェス変更の際に目線が動くとき、なめらかに見えるようにまばたきのサーフェスを補完する関数
pub fn on_smooth_blink(req: &Request) -> Response {
  const DELAY: i32 = 100;
  const CLOSE_EYES_INDEX: i32 = 10;
  const SMILE_EYES_INDEX: i32 = 11;
  const EYE_DIRECTION_NUM: i32 = 3; //こっち、下、あっち
  const EYE_INDEX_DIGIT: u32 = 2;
  let eye_index_digit_pow = 10_i32.pow(EYE_INDEX_DIGIT);

  let refs = get_references(req);
  let dest_surface = refs[0].parse::<i32>().unwrap();
  let is_complete = refs[1].parse::<i32>().unwrap() == 1;
  let dest_eyes = dest_surface % eye_index_digit_pow;
  let dest_remain = dest_surface - dest_eyes;
  let from_surface = get_global_vars().volatility.current_surface();
  let from_eyes = from_surface % eye_index_digit_pow;
  let is_close_eyes = |i: i32| -> bool { i == SMILE_EYES_INDEX || i == CLOSE_EYES_INDEX };

  if from_surface == dest_surface {
    return new_response_nocontent();
  }

  if !(1..=SMILE_EYES_INDEX).contains(&from_eyes) || !(1..=SMILE_EYES_INDEX).contains(&dest_eyes) {
    return new_response_with_value(
      format!("\\s[{}]{}", dest_surface, complete_shadow(is_complete)),
      TranslateOption::none(),
    );
  }

  let mut cuts = vec![from_surface];
  if is_close_eyes(from_eyes) && !is_close_eyes(dest_eyes) {
    //直前が目閉じかつ目標が開き目の場合
    cuts.extend(from_close(
      dest_eyes,
      dest_remain,
      EYE_DIRECTION_NUM,
      CLOSE_EYES_INDEX,
    ));
  } else if !is_close_eyes(from_eyes) && is_close_eyes(dest_eyes) {
    // 直前が開き目かつ目標が目閉じの場合
    cuts.extend(to_close(
      from_eyes,
      dest_remain,
      EYE_DIRECTION_NUM,
      CLOSE_EYES_INDEX,
    ));
  } else if from_eyes % EYE_DIRECTION_NUM != dest_eyes % EYE_DIRECTION_NUM {
    // 直前と目標の目線方向が違う場合）
    cuts.extend(to_close(
      from_eyes,
      dest_remain,
      EYE_DIRECTION_NUM,
      CLOSE_EYES_INDEX,
    ));
    cuts.push(dest_remain + CLOSE_EYES_INDEX);
    cuts.extend(from_close(
      dest_eyes,
      dest_remain,
      EYE_DIRECTION_NUM,
      CLOSE_EYES_INDEX,
    ));
  }
  cuts.push(dest_surface);

  let delay = format!("\\_w[{}]", DELAY);
  let animation = cuts
    .iter()
    .map(|s| format!("\\s[{}]{}", s, complete_shadow(is_complete)))
    .collect::<Vec<String>>()
    .join(delay.as_str());

  new_response_with_value(animation, TranslateOption::none())
}

fn to_close(
  from_eyes: i32,
  dest_remain: i32,
  eye_direction_num: i32,
  close_eyes_index: i32,
) -> Vec<i32> {
  let mut res = vec![];
  let mut i = from_eyes + eye_direction_num;
  while i < close_eyes_index {
    res.push(dest_remain + i);
    i += eye_direction_num;
  }
  res
}

fn from_close(
  dest_eyes: i32,
  dest_remain: i32,
  eye_direction_num: i32,
  close_eyes_index: i32,
) -> Vec<i32> {
  let mut res = vec![];
  let mut i = dest_eyes + eye_direction_num;
  while i < close_eyes_index {
    res.insert(0, dest_remain + i);
    i += eye_direction_num;
  }
  res
}

pub fn to_aroused() {
  let vars = get_global_vars();
  vars.volatility.set_aroused(true);
  vars
    .volatility
    .set_last_random_talk_time(vars.volatility.ghost_up_time());
}

pub enum Icon {
  Info,
  Cross,
}

impl Display for Icon {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "\
    \\f[height,14]\\f[name,icomoon.ttf]\\_l[,@-1]\
    \\_u[0xE{}]\
    \\f[name,default]\\f[height,default]\\_l[,@1]\
    ",
      self.to_code()
    )
  }
}

impl Icon {
  fn to_code(&self) -> u32 {
    match self {
      Icon::Info => 900,
      Icon::Cross => 901,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_all_combo() {
    let values = vec![
      vec!["a".to_string(), "b".to_string()],
      vec!["c".to_string(), "d".to_string()],
      vec!["e".to_string(), "f".to_string()],
    ];
    let result = all_combo(&values);
    println!("{:?}", result);
  }
}
