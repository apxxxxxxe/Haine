pub mod anchor;
pub mod first_boot;
pub mod randomtalk;

use crate::events::common::*;
use crate::events::talk::randomtalk::random_talks;
use crate::get_global_vars;
use crate::roulette::RouletteCell;
use core::fmt::{Display, Formatter};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use shiorust::message::{Request, Response};
use std::collections::HashSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone)]
pub struct Talk {
  pub talk_type: Option<TalkType>,
  text: String,
  pub id: &'static str,
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
    id: &'static str,
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

  pub fn all_talks() -> Vec<Talk> {
    let mut v = Vec::new();
    for t in TalkType::all() {
      v.extend(random_talks(t));
    }
    v
  }

  pub fn get_unseen_talks(talk_type: TalkType, seen: &HashSet<String>) -> Vec<Talk> {
    random_talks(talk_type)
      .into_iter()
      .filter(|t| !seen.contains(t.id))
      .collect()
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, EnumIter)]
pub enum TalkType {
  SelfIntroduce,
  Lore,
  Servant,
  Past,
  Abstract,
  WithYou,
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
pub enum TalkingPlace {
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
      Self::Library => vec![TalkType::Abstract, TalkType::Past],
    }
  }
}

pub fn on_check_unseen_talks(req: &Request) -> Response {
  let refs = get_references(req);
  let talk_type = TalkType::from_u32(refs[0].parse::<u32>().unwrap()).unwrap();
  let talk_collection = get_global_vars().talk_collection();
  let seen_talks = talk_collection.get(&talk_type).unwrap();

  let talks = Talk::get_unseen_talks(talk_type, seen_talks);
  let choosed_talk = talks.choose(&mut rand::thread_rng()).unwrap().to_owned();

  register_talk_collection(&choosed_talk);

  new_response_with_value(
    choosed_talk.consume(),
    TranslateOption::with_shadow_completion(),
  )
}

pub fn register_talk_collection(talk: &Talk) {
  let mut talk_collection = get_global_vars().talk_collection_mut();
  match talk_collection.get_mut(&talk.talk_type.unwrap()) {
    Some(t) => {
      let key = talk.id.to_string();
      if !t.contains(&key) {
        t.insert(key);
      }
    }
    None => {
      talk_collection.insert(
        talk.talk_type.unwrap(),
        HashSet::from_iter(vec![talk.id.to_string()]),
      );
    }
  }
}

pub fn random_talks_analysis() -> String {
  let mut s = String::new();
  let mut sum = 0;
  for talk_type in TalkType::all() {
    let len = random_talks(talk_type).len();
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
