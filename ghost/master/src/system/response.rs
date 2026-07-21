use crate::events::aitalk::IMMERSIVE_ICON_COUNT;
use crate::events::aitalk::IMMERSIVE_RATE_MAX;
use crate::events::talk::TalkType;
use crate::events::talk::TalkingPlace;
use crate::events::translate::on_translate;
use crate::system::error::ShioriError;
use crate::system::roulette::RouletteCell;
use crate::system::variables::*;
use core::fmt::{Display, Formatter};
use std::collections::HashSet;

use shiorust::message::{parts::HeaderName, parts::*, traits::*, Request, Response};

pub(crate) const REMOVE_BALLOON_NUM: &str = "\\0\\![set,balloonnum,,,]";
pub(crate) const RESET_BINDS: &str = "\
  \\![bind,シルエット,,0]\
  \\![bind,ex,,0]\
  \\![bind,目,こっち目,1]\
  \\![bind,口,通常口,1]\
  \\![bind,眉,通常眉,1]\
  \\![bind,顔色,通常,1]\
  \\![bind,腕,前手,1]\
  \\![bind,スカート状態,通常,1]";
pub(crate) const STICK_SURFACE: &str = "\
  \\C\
  \\1\
  \\![reset,sticky-window]\
  \\![set,alignmenttodesktop,free]\
  \\![move,--X=0,--Y=0,--time=0,--base=0]\
  \\![set,sticky-window,1,0]\
  \\0\
  ";

pub(crate) fn on_stick_surface(_req: &Request) -> Response {
  // \1のサーフェスを\0に重ねて固定する
  new_response_with_value_with_notranslate(STICK_SURFACE.to_string(), TranslateOption::none())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum TranslateOption {
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

pub(crate) fn add_notice_description(res: &mut Response, error: &str) {
  res
    .headers
    .insert(HeaderName::from("ErrorDescription"), error.to_string());
  res
    .headers
    .insert(HeaderName::from("ErrorLevel"), "notice".to_string());
}

pub(crate) fn add_error_description(res: &mut Response, error: &str) {
  res
    .headers
    .insert(HeaderName::from("ErrorDescription"), error.to_string());
  res
    .headers
    .insert(HeaderName::from("ErrorLevel"), "error".to_string());
}

pub(crate) fn new_response() -> Response {
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

pub(crate) fn new_response_nocontent() -> Response {
  let mut r = new_response();
  r.status = Status::NoContent;
  r
}

pub(crate) fn new_response_with_value_with_notranslate(
  value: String,
  option: HashSet<TranslateOption>,
) -> Response {
  let balloon_completion = if option.contains(&TranslateOption::CompleteBalloonSurface) {
    format!("\\b[{}]", get_read(&TALKING_PLACE).balloon_surface())
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

pub(crate) fn new_response_with_value_with_translate(
  value: String,
  option: HashSet<TranslateOption>,
) -> Result<Response, ShioriError> {
  let balloon_completion = if option.contains(&TranslateOption::CompleteBalloonSurface) {
    format!("\\b[{}]", get_read(&TALKING_PLACE).balloon_surface())
  } else {
    String::new()
  };

  let v = if option.contains(&TranslateOption::DoTranslate) {
    on_translate(value, option.contains(&TranslateOption::CompleteShadow))?
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

pub(crate) fn choose_one(values: &[impl RouletteCell], update_weight: bool) -> Option<usize> {
  if values.is_empty() {
    return None;
  }
  let u = get_write(&TALK_BIAS).roulette(values, update_weight);
  u
}

// return all combinations of values
// e.g. [a, b], [c, d], [e, f] => "ace", "acf", "ade", "adf", "bce", "bcf", "bde", "bdf"
pub(crate) fn all_combo(values: &Vec<Vec<String>>) -> Vec<String> {
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

pub(crate) fn get_references(req: &Request) -> Vec<&str> {
  let mut references: Vec<&str> = Vec::new();
  const MAX_REF: usize = 10; // とりあえず10個まで取得
  for i in 0..MAX_REF {
    if let Some(value) = req
      .headers
      .get(&HeaderName::from(&format!("Reference{}", i)))
    {
      references.push(value);
    } else {
      references.push("");
    }
  }
  // 最後の空でない参照のインデックスを取得し、それ以降の要素を削除
  let last_valid_index = references.iter().rposition(|&s| !s.is_empty()).unwrap_or(0);
  references.truncate(last_valid_index + 1);
  references
}

pub(crate) fn render_shadow(is_complete: bool) -> String {
  const DEFAULT_Y: i32 = -700;
  const MAX_Y: i32 = -200;
  if is_complete {
    let degree = *get_read(&IMMERSIVE_DEGREES);
    format!(
      "\\0\\![bind,ex,没入度用,1]\\![anim,offset,904000,0,{}]",
      ((MAX_Y - DEFAULT_Y) as f32 * (degree as f32 / (IMMERSIVE_RATE_MAX as f32))) as i32
        + DEFAULT_Y,
    )
  } else {
    "\\0\\![bind,ex,没入度用,0]".to_string()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BlinkDirection {
  Here,
  Down,
  There,
  None,
}

pub(crate) struct BlinkTransition {
  pub base: i32,
  pub is_closed: bool,
  pub direction: BlinkDirection,
  pub to_close: Vec<i32>,
}

impl BlinkTransition {
  pub(crate) fn all() -> Vec<Self> {
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

pub(crate) fn eye_name(code: i32) -> &'static str {
  match code {
    1 => "こっち目",
    2 => "上の空",
    3 => "あっち目",
    4 => "こっち半目",
    5 => "上の空半目",
    6 => "あっち半目",
    7 => "こっち半半目",
    8 => "上の空半半目",
    9 => "あっち半半目",
    10 => "閉じ目",
    11 => "にこ目",
    12 => "驚き目",
    13 => "皮肉目",
    14 => "びっくり目",
    15 => "あっち皮肉目",
    _ => "こっち目",
  }
}

pub(crate) fn mouth_name(code: i32) -> &'static str {
  match code {
    1 => "通常口",
    2 => "笑い口",
    3 => "微笑口",
    4 => "ぽかん口",
    5 => "いひひ口",
    6 => "あ口",
    7 => "キス口",
    8 => "にひ口",
    9 => "わはは口",
    _ => "通常口",
  }
}

pub(crate) fn arm_name(code: i32) -> &'static str {
  match code {
    1 => "前手",
    2 => "胸に手",
    3 => "考える手",
    4 => "後ろ手",
    _ => "前手",
  }
}

pub(crate) fn eyebrow_name(code: i32) -> &'static str {
  match code {
    1 => "通常眉",
    2 => "困り眉",
    3 => "怒り眉",
    4 => "驚き眉",
    _ => "通常眉",
  }
}

pub(crate) fn face_color_name(code: i32) -> &'static str {
  match code {
    1 => "通常",
    2 => "照れ1",
    3 => "照れ2",
    4 => "焦り",
    5 => "涙",
    _ => "通常",
  }
}

/// 7桁サーフェスコードをデコードし、bind命令群を生成する
pub(crate) fn generate_bind_script(
  from_surface: i32,
  dest_surface: i32,
  shadow_script: &str,
  ignore_upper_completion: bool,
) -> String {
  const EYE_INDEX_DIGIT: u32 = 2;
  let eye_index_digit_pow = 10_i32.pow(EYE_INDEX_DIGIT);

  let dest_eyes = dest_surface % eye_index_digit_pow;
  let from_eyes = from_surface % eye_index_digit_pow;
  let dest_mouth = (dest_surface / 100) % 10;
  let dest_arm = (dest_surface / 1000) % 10;
  let dest_eyebrow = (dest_surface / 10000) % 10;
  let dest_face = (dest_surface / 100000) % 10;

  // 非目パーツのbind命令を構築
  let non_eye_binds = format!(
    "\\![bind,眉,{},1]\\![bind,顔色,{},1]\\![bind,腕,{},1]\\![bind,口,{},1]",
    eyebrow_name(dest_eyebrow),
    face_color_name(dest_face),
    arm_name(dest_arm),
    mouth_name(dest_mouth),
  );

  // 同一コードの場合は話者0への切り替えのみ
  if from_surface == dest_surface {
    return "\\0".to_string();
  }

  // 目の遷移スクリプトを生成
  let eye_script = generate_eye_transition(from_eyes, dest_eyes, ignore_upper_completion);

  // 全体を組み立て
  format!(
    "\\0\\![lock,repaint]\\s[1000100]{}{}\\![unlock,repaint]{}",
    non_eye_binds, shadow_script, eye_script
  )
}

/// 目の遷移アニメーション（まばたき補完）をbind方式で生成
fn generate_eye_transition(
  from_eyes: i32,
  dest_eyes: i32,
  ignore_upper_completion: bool,
) -> String {
  let transitions = BlinkTransition::all();
  const DELAY: i32 = 100;
  const CLOSE_EYES_INDEX: i32 = 10;

  // 直接遷移スクリプト
  let direct = format!(
    "\\0\\![lock,repaint]\\![bind,目,{},1]\\![unlock,repaint]",
    eye_name(dest_eyes)
  );

  // 目コードが0の場合は直接遷移
  if from_eyes == 0 || dest_eyes == 0 {
    return direct;
  }
  // 同じ目の場合は何もしない
  if from_eyes == dest_eyes {
    return String::new();
  }

  let mut cuts: Vec<i32> = vec![];
  if let Some(from) = transitions.iter().find(|t| t.base == from_eyes) {
    if let Some(dest) = transitions.iter().find(|t| t.base == dest_eyes) {
      // 同じ視線方向なら直接遷移
      if from.direction == dest.direction {
        return direct;
      }
      if !ignore_upper_completion {
        cuts.push(from_eyes);
        cuts.extend(from.to_close.iter());
        if !from.is_closed && !dest.is_closed {
          cuts.push(CLOSE_EYES_INDEX);
        }
      }
      cuts.extend(dest.to_close.iter().rev());
    }
  }

  cuts.push(dest_eyes);

  let delay = format!("\\_w[{}]", DELAY);
  cuts
    .iter()
    .map(|e| {
      format!(
        "\\0\\![lock,repaint]\\![bind,目,{},1]\\![unlock,repaint]",
        eye_name(*e)
      )
    })
    .collect::<Vec<String>>()
    .join(&delay)
}

#[allow(dead_code)]
pub(crate) enum Icon {
  Cog,
  Cross,
  ArrowRight,
  ArrowLeft,
  Bubble,
  Info,
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
      Icon::Cog => 902,
      Icon::Cross => 903,
      Icon::ArrowRight => 904,
      Icon::ArrowLeft => 905,
      Icon::Bubble => 906,
      Icon::Info => 907,
    }
  }
}

pub(crate) fn render_achievement_message(talk_type: TalkType) -> String {
  format!(
    "\\1\\![quicksection,1]\
    \\f[align,center]\\f[valign,center]\\f[bold,1]\
    トークカテゴリ「{}」が解放された。\
    \\f[default]",
    talk_type
  )
}

pub(crate) fn shake_with_notext() -> String {
  let shakes = [(10, 10), (-14, -14), (4, 4)];
  shakes
    .iter()
    .map(|(x, y)| format!("\\![move,--X={},--Y={},--time=50,--base=me]", x, y))
    .collect::<Vec<String>>()
    .join("")
}

pub(crate) fn render_immersive_icon() -> String {
  let immersive_degrees = *get_read(&IMMERSIVE_DEGREES);
  let icon_count_float =
    immersive_degrees as f32 * IMMERSIVE_ICON_COUNT as f32 / IMMERSIVE_RATE_MAX as f32;
  let current_icon_count = if *get_read(&TALKING_PLACE) == TalkingPlace::Library {
    // 繰り上げ
    icon_count_float.ceil() as u32
  } else {
    // 切り捨て
    icon_count_float.floor() as u32
  };
  let mut candles = *get_write(&CANDLES);
  let mut v = String::new();
  for i in 1..=IMMERSIVE_ICON_COUNT {
    let blowed = i <= current_icon_count;
    v.push_str(&format!(
      "\\![bind,icon,没入度{},{}]\\![bind,icon,消え{},{}]",
      i,
      if blowed { 0 } else { 1 },
      i,
      if blowed { 1 } else { 0 }
    ));
    candles[i as usize - 1] = blowed;
  }
  format!("\\p[2]{}\\0", v)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_generate_bind_script_same_surface() {
    // 同一コードの場合は話者0への切り替えのみ
    let result = generate_bind_script(1111201, 1111201, "", false);
    assert_eq!(result, "\\0");
  }

  #[test]
  fn test_generate_bind_script_zero_eye_code() {
    // 目コードが0の場合は直接遷移
    let result = generate_bind_script(1111200, 1111201, "", false);
    assert!(result.contains("\\s[1000100]"));
    assert!(result.contains("\\![bind,目,こっち目,1]"));
  }

  #[test]
  fn test_generate_bind_script_same_direction() {
    // 同じ視線方向（Here: 1, 4）の場合は直接遷移
    let result = generate_bind_script(1111201, 1111204, "", false);
    assert!(result.contains("\\![bind,目,こっち半目,1]"));
    assert!(!result.contains("\\_w[")); // 遅延なし
  }

  #[test]
  fn test_generate_bind_script_different_direction() {
    // 異なる視線方向の場合はアニメーション補完
    // 1（Here）→ 3（There）は異方向
    let result = generate_bind_script(1111201, 1111203, "", false);

    // 遅延があること（アニメーション）
    assert!(result.contains("\\_w[100]"));

    // 最終的に目的の目に到達
    assert!(result.contains("\\![bind,目,あっち目,1]"));

    // 中間フレームが含まれる（閉じ目を経由）
    assert!(result.contains("\\![bind,目,閉じ目,1]"));
  }

  #[test]
  fn test_generate_bind_script_with_half_blink() {
    // ignore_upper_completion=true の場合は閉じる過程をスキップ
    let result_full = generate_bind_script(1111201, 1111203, "", false);
    let result_half = generate_bind_script(1111201, 1111203, "", true);

    // 半まばたきの方がフレーム数が少ない
    let full_frames = result_full.matches("\\![bind,目,").count();
    let half_frames = result_half.matches("\\![bind,目,").count();
    assert!(half_frames < full_frames);
  }

  #[test]
  fn test_generate_bind_script_with_shadow_script() {
    // 影スクリプトが含まれる
    let shadow = "\\![bind,ex,没入度用,1]";
    let result = generate_bind_script(1111201, 1111203, shadow, false);
    assert!(result.contains(shadow));
  }

  #[test]
  fn test_generate_bind_script_here_to_down() {
    // Here（1）→ Down（2）の遷移
    let result = generate_bind_script(1111201, 1111202, "", false);

    // アニメーションが生成される
    assert!(result.contains("\\_w[100]"));
    assert!(result.contains("\\![bind,目,上の空,1]"));
  }

  #[test]
  fn test_generate_bind_script_closed_eyes() {
    // 閉じ目（10）からの遷移
    let result = generate_bind_script(1111210, 1111201, "", false);

    // 最終的にこっち目に到達
    assert!(result.contains("\\![bind,目,こっち目,1]"));
  }

  #[test]
  fn test_generate_bind_script_non_eye_binds() {
    // 非目パーツのbindが正しく生成されるか
    // h1223201 = 特殊1, 照れ1(2), 困り眉(2), 考える手(3), 笑い口(2), こっち目(01)
    let result = generate_bind_script(1111201, 1223201, "", false);
    assert!(result.contains("\\![bind,眉,困り眉,1]"));
    assert!(result.contains("\\![bind,顔色,照れ1,1]"));
    assert!(result.contains("\\![bind,口,笑い口,1]"));
  }

  #[test]
  fn test_eye_name_mapping() {
    assert_eq!(eye_name(1), "こっち目");
    assert_eq!(eye_name(10), "閉じ目");
    assert_eq!(eye_name(15), "あっち皮肉目");
  }

  #[test]
  fn test_mouth_name_mapping() {
    assert_eq!(mouth_name(1), "通常口");
    assert_eq!(mouth_name(9), "わはは口");
  }
}
