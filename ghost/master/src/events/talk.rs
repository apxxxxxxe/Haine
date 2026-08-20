pub(crate) mod anchor;
pub(crate) mod first_boot;
pub(crate) mod randomtalk;

use crate::check_error;
use crate::events::talk::randomtalk::random_talks;
use crate::system::error::ShioriError;
use crate::system::response::*;
use crate::system::roulette::RouletteCell;
use crate::system::variables::{get_read, get_write, PENDING_BRANCHES, TALK_COLLECTION};
use core::fmt::{Display, Formatter};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use shiorust::message::{Request, Response};
use std::collections::HashSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use self::randomtalk::{derivative_talks_per_talk_type, get_parent_talk};

use super::aitalk::render_talk;

#[allow(unused)]
#[derive(Clone)]
pub(crate) struct Talk {
  pub talk_type: Option<TalkType>,
  pub text: String,
  pub id: String,
  pub callback: Option<fn()>,
}

impl RouletteCell for Talk {
  fn key(&self) -> &str {
    &self.id
  }
}

impl Display for Talk {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.text)
  }
}

impl Talk {
  pub fn consume(&self) -> String {
    if let Some(callback) = self.callback {
      callback();
    }
    self.text.clone()
  }
}

#[allow(dead_code)]
impl Talk {
  pub fn new(talk_type: Option<TalkType>, id: String, text: String, callback: Option<fn()>) -> Self {
    Self {
      talk_type,
      text,
      id,
      callback,
    }
  }

  pub fn all_talks() -> Option<Vec<Talk>> {
    let mut v = Vec::new();
    for t in TalkType::all() {
      let talks = random_talks(t)?;
      v.extend(talks);
    }
    Some(v)
  }

  pub fn get_unseen_talks(talk_type: TalkType, seen: &HashSet<String>) -> Option<Vec<Talk>> {
    let talks = random_talks(talk_type)?;
    Some(
      talks
        .into_iter()
        .filter(|t| !seen.contains(&t.id))
        .collect(),
    )
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, EnumIter)]
pub(crate) enum TalkType {
  #[serde(alias = "SelfIntroduce")]
  AboutMe,
  WithYou,
  Servant,
  Lore,
  Past,
  Abstract,
  GuestRoom,
}

impl Display for TalkType {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    let s = match self {
      Self::AboutMe => "ハイネ自身の話題",
      Self::Lore => "ロア/オカルト",
      Self::Servant => "従者について",
      Self::Past => "ハイネの過去についての話題",
      Self::Abstract => "抽象的な話題",
      Self::WithYou => "あなたについての話題",
      Self::GuestRoom => "ゲストルームでの出来事",
    };
    write!(f, "{}", s)
  }
}

impl TalkType {
  pub fn from_u32(n: u32) -> Option<Self> {
    Self::all().into_iter().find(|t| *t as u32 == n)
  }

  pub fn all() -> Vec<Self> {
    Self::iter().collect()
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TalkingPlace {
  LivingRoom(bool), // boolは没入モードか否か
  Conservatory,
  GuestRoom,
  Kitchen,
}

impl Display for TalkingPlace {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    let s = match self {
      Self::LivingRoom(is_immersed) => {
        if *is_immersed {
          "書斎"
        } else {
          "居間"
        }
      }
      Self::Conservatory => "温室",
      Self::GuestRoom => "ゲストルーム",
      Self::Kitchen => "キッチン",
    };
    write!(f, "{}", s)
  }
}

impl std::str::FromStr for TalkingPlace {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "居間" => Ok(Self::DEFAULT_LIVING_ROOM),
      "書斎" => Ok(Self::IMMERSED_LIVING_ROOM),
      "温室" => Ok(Self::Conservatory),
      "ゲストルーム" => Ok(Self::GuestRoom),
      "キッチン" => Ok(Self::Kitchen),
      _ => Err(format!("不明な部屋名: {}", s)),
    }
  }
}

impl TalkingPlace {
  pub(crate) const DEFAULT_LIVING_ROOM: Self = Self::LivingRoom(false);
  pub(crate) const IMMERSED_LIVING_ROOM: Self = Self::LivingRoom(true);

  pub fn all() -> Vec<Self> {
    vec![
      Self::DEFAULT_LIVING_ROOM,
      Self::IMMERSED_LIVING_ROOM,
      Self::Conservatory,
      Self::GuestRoom,
      Self::Kitchen,
    ]
  }

  pub fn balloon_surface_sakura(&self) -> u32 {
    match self {
      Self::LivingRoom(is_immersed) => {
        if *is_immersed {
          6
        } else {
          0
        }
      }
      Self::Conservatory => 4,
      Self::GuestRoom => 8,
      Self::Kitchen => 8,
    }
  }

  pub fn balloon_surface_kero(&self) -> u32 {
    match self {
      Self::LivingRoom(is_immersed) => {
        if *is_immersed {
          8
        } else {
          4
        }
      }
      Self::Conservatory => 6,
      Self::GuestRoom => 4,
      Self::Kitchen => 10,
    }
  }

  pub fn talk_types(&self) -> Vec<TalkType> {
    match self {
      Self::LivingRoom(is_immersed) => {
        if *is_immersed {
          vec![TalkType::Abstract]
        } else {
          vec![
            TalkType::AboutMe,
            TalkType::Lore,
            TalkType::WithYou,
            TalkType::Servant,
          ]
        }
      }
      Self::Conservatory => vec![],
      Self::GuestRoom => vec![TalkType::GuestRoom],
      Self::Kitchen => vec![],
    }
  }
}

pub(crate) fn on_check_unseen_talks(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let talk_type_num = check_error!(refs[0].parse::<u32>(), ShioriError::ParseIntError);
  let talk_type = TalkType::from_u32(talk_type_num).ok_or(ShioriError::BadRequest)?;
  let choosed_talk;
  {
    let talk_collection = get_read(&TALK_COLLECTION);
    let empty_hashset = HashSet::new();
    let seen_talks = talk_collection.get(&talk_type).unwrap_or(&empty_hashset);
    let talks = Talk::get_unseen_talks(talk_type, seen_talks).ok_or(ShioriError::TalkNotFound)?;
    let derivative_talks = DerivaliveTalk::get_unseen_talks(talk_type, seen_talks)
      .unwrap_or_default()
      .iter()
      .filter_map(get_parent_talk)
      .collect::<Vec<Talk>>();
    let combined_talks = talks
      .into_iter()
      .chain(derivative_talks)
      .collect::<Vec<Talk>>();
    choosed_talk = combined_talks
      .choose(&mut rand::thread_rng())
      .ok_or(ShioriError::TalkNotFound)?
      .clone();
  }
  register_talk_collection(&choosed_talk.id, talk_type)?;

  new_response_with_value_with_translate(
    render_talk(&choosed_talk),
    TranslateOption::with_shadow_completion(),
  )
}

pub(crate) fn register_talk_collection(id: &str, talk_type: TalkType) -> Result<(), ShioriError> {
  let mut talk_collection = get_write(&TALK_COLLECTION);
  match talk_collection.get_mut(&talk_type) {
    Some(t) => {
      let key = id.to_string();
      if !t.contains(&key) {
        t.insert(key);
      }
    }
    None => {
      talk_collection.insert(talk_type, HashSet::from_iter(vec![id.to_string()]));
    }
  }
  Ok(())
}

pub(crate) fn random_talks_analysis() -> String {
  let mut s = String::new();
  let mut sum = 0;
  for talk_type in TalkType::all() {
    let len = if let Some(v) = random_talks(talk_type) {
      v.len()
    } else {
      0
    };
    s.push_str(&format!("{:?}: {}\\n", talk_type, len,));
    sum += len;
  }

  format!(
    "\\_q{}
    ---\\n\
    TOTAL: {}",
    s, sum
  )
}

#[derive(Clone)]
pub(crate) struct DerivaliveTalk {
  pub(crate) parent_id: String,
  pub(crate) id: String,
  pub(crate) summary: String,
  pub(crate) text: String,
  pub(crate) required_condition: Option<fn() -> bool>,
  pub(crate) callback: Option<fn()>,
}

impl DerivaliveTalk {
  pub fn consume(&self) -> String {
    if let Some(callback) = self.callback {
      callback();
    }
    self.text.clone()
  }

  pub fn get_unseen_talks(talk_type: TalkType, seen: &HashSet<String>) -> Option<Vec<DerivaliveTalk>> {
    let talks = derivative_talks_per_talk_type()
      .get(&talk_type)
      .cloned()
      .unwrap_or_default();
    let mut result = Vec::new();
    for talk in talks {
      if !seen.contains(&talk.id) {
        if let Some(condition) = talk.required_condition {
          if condition() {
            result.push(talk);
          }
        } else {
          result.push(talk);
        }
      }
    }
    if result.is_empty() {
      None
    } else {
      Some(result)
    }
  }
}

/// 「トーク→選択肢→トーク」の分岐の一節。
/// render() が本文と選択肢列のスクリプトを生成し、選択待ちの枝を PENDING_BRANCHES に積む。
/// 選択されると OnTalkBranch が対応する next を同じ手順で表示する（多段ネストはこの再帰で成立する）。
///
/// 選択肢はバルーンが生きている間だけ有効。バルーン消滅後のクリックや
/// 別トークによる上書きは黙って無視され、既読管理もしない。
/// 「読み終えた後の任意の寄り道」には DerivativeTalk を使う。こちらは分岐そのものが本体のトーク用。
#[derive(Clone)]
pub(crate) struct BranchTalk {
  pub(crate) text: String,
  /// 空なら終端
  pub(crate) choices: Vec<BranchChoice>,
  pub(crate) callback: Option<fn()>,
}

/// BranchTalk の選択肢1つ。
#[derive(Clone)]
pub(crate) struct BranchChoice {
  /// 選択肢として表示する文言
  pub(crate) label: String,
  pub(crate) next: BranchTalk,
  /// Some(f) のとき f() が false なら選択肢に出さない
  pub(crate) required_condition: Option<fn() -> bool>,
}

impl BranchChoice {
  #[allow(dead_code)]
  pub(crate) fn new(label: impl Into<String>, next: BranchTalk) -> Self {
    Self {
      label: label.into(),
      next,
      required_condition: None,
    }
  }
}

impl BranchTalk {
  /// 選択肢を持たない終端ノードを作る
  pub(crate) fn leaf(text: impl Into<String>) -> Self {
    Self {
      text: text.into(),
      choices: vec![],
      callback: None,
    }
  }

  pub(crate) fn choice(label: impl Into<String>, next: BranchTalk) -> BranchChoice {
    BranchChoice {
      label: label.into(),
      next,
      required_condition: None,
    }
  }

  /// 選択肢付きのノードを作る
  #[allow(dead_code)]
  pub(crate) fn node(text: impl Into<String>, choices: Vec<BranchChoice>) -> Self {
    Self {
      text: text.into(),
      choices,
      callback: None,
    }
  }

  /// 自身と全子孫の text を深さ優先で集める。dump_talks 用
  pub(crate) fn all_texts(&self) -> Vec<String> {
    let mut texts = vec![self.text.clone()];
    for choice in &self.choices {
      texts.extend(choice.next.all_texts());
    }
    texts
  }

  /// 本文+選択肢列のスクリプトを返し、PENDING_BRANCHES を表示中の枝で上書きする。
  /// 選択肢のインデックスは required_condition によるフィルタ後の並びで振る。
  pub(crate) fn render(&self) -> String {
    if let Some(callback) = self.callback {
      callback();
    }
    let available: Vec<BranchChoice> = self
      .choices
      .iter()
      .filter(|c| c.required_condition.is_none_or(|f| f()))
      .cloned()
      .collect();
    let mut script = self.text.clone();
    if !available.is_empty() {
      script.push_str("\\1");
      for (i, choice) in available.iter().enumerate() {
        if i > 0 {
          script.push_str("\\n");
        }
        script.push_str(&format!(
          "\\![*]\\__q[OnTalkBranch,{}]{}\\__q",
          i, choice.label
        ));
      }
    }
    *get_write(&PENDING_BRANCHES) = available;
    script
  }
}

pub(crate) fn on_talk_branch(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let index_str = refs.first().ok_or(ShioriError::BadRequest)?;
  let index = check_error!(index_str.parse::<usize>(), ShioriError::ParseIntError);
  // 失効した選択肢（リロード後に残ったバルーン等）のクリックは正常系として黙って無視する
  let next = {
    let pending = get_read(&PENDING_BRANCHES);
    match pending.get(index) {
      Some(choice) => choice.next.clone(),
      None => return Ok(new_response_nocontent()),
    }
  };
  new_response_with_value_with_translate(next.render(), TranslateOption::with_shadow_completion())
}

#[cfg(test)]
mod branch_talk_tests {
  use super::*;
  use shiorust::message::parts::*;
  use std::sync::atomic::{AtomicBool, Ordering};

  static CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);

  fn leaf(text: &str) -> BranchTalk {
    BranchTalk {
      text: text.to_string(),
      choices: vec![],
      callback: None,
    }
  }

  fn make_branch_request(index: &str) -> Request {
    let mut headers = Headers::new();
    headers.insert_by_header_name(HeaderName::from("ID"), "OnTalkBranch".to_string());
    headers.insert_by_header_name(HeaderName::from("Reference0"), index.to_string());
    Request {
      method: Method::GET,
      version: Version::V20,
      headers,
    }
  }

  fn response_value(res: &Response) -> String {
    res
      .headers
      .get_by_header_name(&HeaderName::from("Value"))
      .cloned()
      .unwrap_or_default()
  }

  // グローバル状態(PENDING_BRANCHES)を共有するため、順序保証のある単一テストにまとめる
  #[test]
  fn test_branch_talk_flow() {
    let tree = BranchTalk {
      text: "根の本文".to_string(),
      choices: vec![
        BranchChoice {
          label: "条件で消える枝".to_string(),
          next: leaf("到達しない"),
          required_condition: Some(|| false),
        },
        BranchChoice {
          label: "掘り下げる".to_string(),
          next: BranchTalk {
            text: "二段目の本文".to_string(),
            choices: vec![BranchChoice {
              label: "終端へ".to_string(),
              next: leaf("終端の本文"),
              required_condition: None,
            }],
            callback: Some(|| CALLBACK_CALLED.store(true, Ordering::SeqCst)),
          },
          required_condition: Some(|| true),
        },
      ],
      callback: None,
    };

    // 1. render: 条件フィルタ後の枝だけが表示され、インデックスはフィルタ後の並びで振られる
    let script = tree.render();
    assert!(script.contains("根の本文"));
    assert!(!script.contains("条件で消える枝"));
    assert!(script.contains("\\__q[OnTalkBranch,0]掘り下げる\\__q"));
    assert_eq!(get_read(&PENDING_BRANCHES).len(), 1);

    // 2. 選択: next が表示され、その choices が積み直される(多段ネスト)
    let res = on_talk_branch(&make_branch_request("0")).unwrap();
    let value = response_value(&res);
    assert!(value.contains("二段目の本文"));
    assert!(value.contains("\\__q[OnTalkBranch,0]終端へ\\__q"));
    assert!(
      CALLBACK_CALLED.load(Ordering::SeqCst),
      "表示時に callback が発火するべき"
    );
    assert_eq!(get_read(&PENDING_BRANCHES).len(), 1);

    // 3. 範囲外インデックス(失効クリック)は握りつぶし、状態も壊さない
    let res = on_talk_branch(&make_branch_request("5")).unwrap();
    assert!(response_value(&res).is_empty());
    assert_eq!(get_read(&PENDING_BRANCHES).len(), 1);

    // 4. 終端を選ぶと選択肢なしで表示され、状態がクリアされる
    let res = on_talk_branch(&make_branch_request("0")).unwrap();
    let value = response_value(&res);
    assert!(value.contains("終端の本文"));
    assert!(!value.contains("OnTalkBranch"));
    assert!(get_read(&PENDING_BRANCHES).is_empty());

    // 5. クリア後のクリック(バルーン残骸)も握りつぶす
    let res = on_talk_branch(&make_branch_request("0")).unwrap();
    assert!(response_value(&res).is_empty());
  }
}

/// 全トークを一つのテキストにまとめて返す。
/// dump_talks バイナリ（`cargo run --bin dump_talks`）から使う開発用ユーティリティ。
pub fn render_all_talks() -> String {
  use crate::events::menu::questions::QUESTIONS;
  use crate::events::talk::first_boot::{FIRST_BOOT_TALK, FIRST_CLOSE_TALK, FIRST_RANDOMTALKS};
  use randomtalk::{derivative_talks, get_parent_talk, RANDOMTALK_COMMENTS_LIVING_ROOM};

  let mut lines: Vec<String> = Vec::new();
  lines.push(FIRST_BOOT_TALK.to_string());
  for t in FIRST_RANDOMTALKS.iter() {
    lines.push(t.to_string());
  }
  lines.push(FIRST_CLOSE_TALK.to_string());
  for q in QUESTIONS.iter() {
    lines.extend(q.branch_talk().all_texts());
  }
  for talk_type in TalkType::all() {
    if let Some(talks) = random_talks(talk_type) {
      for t in talks {
        lines.push(t.text);
      }
    }
  }
  for derivative_talk in derivative_talks().iter() {
    if let Some(parent_talk) = get_parent_talk(derivative_talk) {
      lines.push(format!(
        "{}\\1{}{}",
        parent_talk.text, derivative_talk.summary, derivative_talk.text
      ));
    }
  }
  for t in RANDOMTALK_COMMENTS_LIVING_ROOM.iter() {
    lines.push(t.to_string());
  }
  let mut result = lines.join("\n");
  result.push('\n');
  result
}
