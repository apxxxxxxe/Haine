pub(crate) mod anchor;
pub(crate) mod first_boot;
pub(crate) mod randomtalk;

use crate::check_error;
use crate::error::ShioriError;
use crate::events::common::*;
use crate::events::talk::randomtalk::random_talks;
use crate::roulette::RouletteCell;
use crate::variables::TALK_COLLECTION;
use core::fmt::{Display, Formatter};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use shiorust::message::{Request, Response};
use std::collections::HashSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use self::randomtalk::{derivative_talks_per_talk_type, get_parent_talk};

use super::aitalk::render_talk;

#[derive(Clone)]
pub(crate) struct Talk {
  pub talk_type: Option<TalkType>,
  pub text: String,
  pub id: String,
  pub callback: Option<fn()>,
}

impl RouletteCell for Talk {
  fn key(&self) -> String {
    self.id.to_string()
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
  pub fn new(
    talk_type: Option<TalkType>,
    id: String,
    text: String,
    callback: Option<fn()>,
  ) -> Self {
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
  SelfIntroduce,
  WithYou,
  Servant,
  Lore,
  Past,
  Abstract,
}

impl Display for TalkType {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    let s = match self {
      Self::SelfIntroduce => "ハイネ自身の話題",
      Self::Lore => "ロア/オカルト",
      Self::Servant => "従者について",
      Self::Past => "ハイネの過去についての話題",
      Self::Abstract => "抽象的な話題",
      Self::WithYou => "あなたについての話題",
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TalkingPlace {
  LivingRoom,
  Library,
}

impl Display for TalkingPlace {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    let s = match self {
      Self::LivingRoom => "客間",
      Self::Library => "書斎",
    };
    write!(f, "{}", s)
  }
}

impl TalkingPlace {
  pub fn balloon_surface(&self) -> u32 {
    match self {
      Self::LivingRoom => 0,
      Self::Library => 6,
    }
  }

  pub fn talk_types(&self) -> Vec<TalkType> {
    match self {
      Self::LivingRoom => vec![TalkType::SelfIntroduce, TalkType::Lore, TalkType::WithYou],
      Self::Library => vec![TalkType::Abstract],
    }
  }
}

pub(crate) fn on_check_unseen_talks(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let talk_type_num = check_error!(refs[0].parse::<u32>(), ShioriError::ParseIntError);
  let talk_type = TalkType::from_u32(talk_type_num).unwrap();
  let choosed_talk;
  {
    let talk_collection = TALK_COLLECTION.read().unwrap();
    let empty_hashset = HashSet::new();
    let seen_talks = talk_collection.get(&talk_type).unwrap_or(&empty_hashset);
    let talks = Talk::get_unseen_talks(talk_type, seen_talks).ok_or(ShioriError::TalkNotFound)?;
    let derivative_talks = DerivaliveTalk::get_unseen_talks(talk_type, seen_talks)
      .unwrap_or_default()
      .iter()
      .map(get_parent_talk)
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
  let mut talk_collection = TALK_COLLECTION.write().unwrap();
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

  pub fn get_unseen_talks(
    talk_type: TalkType,
    seen: &HashSet<String>,
  ) -> Option<Vec<DerivaliveTalk>> {
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

#[cfg(test)]
mod tests {
  use crate::events::menu::QUESTIONS;
  use crate::events::talk::first_boot::{FIRST_BOOT_TALK, FIRST_CLOSE_TALK, FIRST_RANDOMTALKS};
  use crate::events::talk::{random_talks, TalkType};
  use std::fs::File;
  use std::io::Write;

  use super::randomtalk::{derivative_talks, get_parent_talk, RANDOMTALK_COMMENTS_LIVING_ROOM};

  #[test]
  fn write_all_talks() {
    let mut all_talks_file = File::create("all_talks.txt").unwrap();
    let mut write = |text: String| {
      writeln!(all_talks_file, "{}", text).unwrap();
    };
    write(FIRST_BOOT_TALK.to_string());
    for t in FIRST_RANDOMTALKS.iter() {
      write(t.to_string());
    }
    write(FIRST_CLOSE_TALK.to_string());
    for q in QUESTIONS.iter() {
      write(q.talk());
    }
    for talk_type in TalkType::all() {
      let talks = random_talks(talk_type).unwrap();
      for t in talks {
        write(t.text);
      }
    }
    for derivative_talk in derivative_talks().iter() {
      let parent_talk = get_parent_talk(derivative_talk);
      write(format!(
        "{}\\1{}{}",
        parent_talk.text, derivative_talk.summary, derivative_talk.text
      ));
    }
    for t in RANDOMTALK_COMMENTS_LIVING_ROOM.iter() {
      write(t.to_string());
    }
  }
}
