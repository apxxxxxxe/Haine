use crate::check_error;
use crate::error::ShioriError;
use crate::events::aitalk::IMMERSIVE_RATE_MAX;
use crate::events::talk::{TalkType, TalkingPlace};
use crate::events::translate::on_translate;
use crate::roulette::RouletteCell;
use crate::variables::get_global_vars;
use core::fmt::{Display, Formatter};
use std::collections::HashSet;

use shiorust::message::{parts::HeaderName, parts::*, traits::*, Request, Response};

pub const REMOVE_BALLOON_NUM: &str = "\\0\\![set,balloonnum,,,]";
pub const RESET_BINDS: &str = "\\![bind,シルエット,,0]\\![bind,ex,,0]";
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
  new_response_with_value_with_notranslate(STICK_SURFACE.to_string(), TranslateOption::none())
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

pub fn new_response_with_value_with_notranslate(
  value: String,
  option: HashSet<TranslateOption>,
) -> Response {
  let vars = get_global_vars();

  let balloon_completion = if option.contains(&TranslateOption::CompleteBalloonSurface) {
    format!("\\b[{}]", vars.volatility.talking_place().balloon_surface(),)
  } else {
    String::new()
  };

  let mut v = balloon_completion + value.as_str();
  // \\Cが含まれているなら文頭に\\Cを補完
  if v.contains("\\C") {
    v = format!("\\C{}", v.replace("\\C", ""));
  }

  let mut r = new_response();
  r.headers.insert(HeaderName::from("Value"), v);
  r
}

pub fn new_response_with_value_with_translate(
  value: String,
  option: HashSet<TranslateOption>,
) -> Result<Response, ShioriError> {
  let vars = get_global_vars();

  let balloon_completion = if option.contains(&TranslateOption::CompleteBalloonSurface) {
    format!("\\b[{}]", vars.volatility.talking_place().balloon_surface(),)
  } else {
    String::new()
  };

  let v = if option.contains(&TranslateOption::DoTranslate) {
    if vars.volatility.inserter_mut().is_ready() {
      on_translate(value, option.contains(&TranslateOption::CompleteShadow))?
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
  Ok(r)
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
  u
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

pub fn render_shadow(is_complete: bool) -> String {
  const DEFAULT_Y: i32 = -700;
  const MAX_Y: i32 = -200;
  let vars = get_global_vars();
  if is_complete {
    let degree = if vars.volatility.talking_place() == TalkingPlace::Library {
      100 - vars.volatility.immersive_degrees()
    } else {
      vars.volatility.immersive_degrees()
    };
    format!(
      "\\0\\![bind,ex,没入度用,1]\\![anim,offset,800100,0,{}]",
      ((MAX_Y - DEFAULT_Y) as f32 * (degree as f32 / (IMMERSIVE_RATE_MAX as f32))) as i32
        + DEFAULT_Y,
    )
  } else {
    "\\0\\![bind,ex,没入度用,0]".to_string()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BlinkDirection {
  Here,
  Down,
  There,
  None,
}

struct BlinkTransition {
  base: i32,
  is_closed: bool,
  direction: BlinkDirection,
  to_close: Vec<i32>,
}

impl BlinkTransition {
  fn all() -> Vec<Self> {
    vec![
      BlinkTransition {
        base: 1,
        is_closed: false,
        direction: BlinkDirection::Here,
        to_close: vec![4, 7],
      },
      BlinkTransition {
        base: 2,
        is_closed: false,
        direction: BlinkDirection::Down,
        to_close: vec![5, 8],
      },
      BlinkTransition {
        base: 3,
        is_closed: false,
        direction: BlinkDirection::There,
        to_close: vec![6, 9],
      },
      BlinkTransition {
        base: 4,
        is_closed: false,
        direction: BlinkDirection::Here,
        to_close: vec![7],
      },
      BlinkTransition {
        base: 5,
        is_closed: false,
        direction: BlinkDirection::Down,
        to_close: vec![8],
      },
      BlinkTransition {
        base: 6,
        is_closed: false,
        direction: BlinkDirection::There,
        to_close: vec![9],
      },
      BlinkTransition {
        base: 7,
        is_closed: false,
        direction: BlinkDirection::Here,
        to_close: vec![],
      },
      BlinkTransition {
        base: 8,
        is_closed: false,
        direction: BlinkDirection::Down,
        to_close: vec![],
      },
      BlinkTransition {
        base: 9,
        is_closed: false,
        direction: BlinkDirection::There,
        to_close: vec![],
      },
      BlinkTransition {
        base: 10,
        is_closed: true,
        direction: BlinkDirection::None,
        to_close: vec![],
      },
      BlinkTransition {
        base: 11,
        is_closed: true,
        direction: BlinkDirection::None,
        to_close: vec![],
      },
      BlinkTransition {
        base: 12,
        is_closed: false,
        direction: BlinkDirection::None,
        to_close: vec![],
      },
      BlinkTransition {
        base: 13,
        is_closed: false,
        direction: BlinkDirection::Here,
        to_close: vec![4, 7],
      },
      BlinkTransition {
        base: 14,
        is_closed: false,
        direction: BlinkDirection::Here,
        to_close: vec![11],
      },
      BlinkTransition {
        base: 15,
        is_closed: false,
        direction: BlinkDirection::There,
        to_close: vec![6, 9],
      },
    ]
  }
}

// サーフェス変更の際に目線が動くとき、なめらかに見えるようにまばたきのサーフェスを補完する関数
pub fn on_smooth_blink(req: &Request) -> Result<Response, ShioriError> {
  let transitions = BlinkTransition::all();
  const DELAY: i32 = 100;
  const CLOSE_EYES_INDEX: i32 = 10;
  const EYE_INDEX_DIGIT: u32 = 2;
  let eye_index_digit_pow = 10_i32.pow(EYE_INDEX_DIGIT);

  let mut err = String::new();
  let refs = get_references(req);
  let dest_surface = check_error!(refs[0].parse::<i32>(), ShioriError::ParseIntError);
  let is_complete = check_error!(refs[1].parse::<i32>(), ShioriError::ParseIntError) == 1;
  let ignore_upper_completion =
    check_error!(refs[2].parse::<i32>(), ShioriError::ParseIntError) == 1;
  let dest_eyes = dest_surface % eye_index_digit_pow;
  let dest_remain = dest_surface - dest_eyes;
  let from_surface = get_global_vars().volatility.current_surface();
  let from_eyes = from_surface % eye_index_digit_pow;
  let direct_res = new_response_with_value_with_notranslate(
    format!("\\s[{}]{}", dest_surface, render_shadow(is_complete)),
    TranslateOption::none(),
  );

  if from_eyes == 0 || dest_eyes == 0 {
    return Ok(direct_res);
  }
  if from_surface == dest_surface {
    return Ok(new_response_nocontent());
  }

  let mut cuts = vec![];
  if let Some(from) = transitions.iter().find(|t| t.base == from_eyes) {
    if let Some(dest) = transitions.iter().find(|t| t.base == dest_eyes) {
      if from.direction == dest.direction {
        return Ok(direct_res);
      }
      if !ignore_upper_completion {
        cuts.push(from_surface);
        cuts.extend(from.to_close.clone().iter().map(|i| dest_remain + i));
        if !from.is_closed && !dest.is_closed {
          cuts.push(dest_remain + CLOSE_EYES_INDEX);
        }
      }
      cuts.extend(dest.to_close.clone().iter().rev().map(|i| dest_remain + i));
    } else {
      err = format!("目線の変更先が不正です: {}", dest_eyes);
    }
  } else {
    err = format!("目線の変更元が不正です: {}", from_eyes);
  }

  cuts.push(dest_surface);

  let delay = format!("\\_w[{}]", DELAY);
  let animation = cuts
    .iter()
    .map(|s| format!("\\0\\s[{}]{}", s, render_shadow(is_complete)))
    .collect::<Vec<String>>()
    .join(delay.as_str());

  let mut res = new_response_with_value_with_notranslate(animation, TranslateOption::none());
  if !err.is_empty() {
    add_error_description(
      &mut res,
      format!("まばたき補完中にエラーが発生しました: {}", err).as_str(),
    );
  }
  Ok(res)
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
    \\f[height,14]\\f[name,icomoon.ttf]\
    \\_u[0xE{}]\
    \\f[name,default]\\f[height,default]\
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

pub fn render_achievement_message(talk_type: TalkType) -> String {
  format!(
    "\\1\\![quicksection,1]\
    \\f[align,center]\\f[valign,center]\\f[bold,1]\
    トークカテゴリ「{}」が解放された。\
    \\f[default]",
    talk_type
  )
}

pub fn add_immsersive_degree(degree: u32) {
  let vars = get_global_vars();
  let new_degree = std::cmp::min(
    vars.volatility.immersive_degrees() + degree,
    IMMERSIVE_RATE_MAX,
  );
  vars.volatility.set_immersive_degrees(new_degree);
}

pub fn sub_immsersive_degree(degree: u32) {
  let vars = get_global_vars();
  let new_degree = vars.volatility.immersive_degrees().saturating_sub(degree);
  vars.volatility.set_immersive_degrees(new_degree);
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
