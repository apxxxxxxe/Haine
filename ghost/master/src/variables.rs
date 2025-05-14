use crate::autobreakline::Inserter;
use crate::check_error;
use crate::error::ShioriError;
use crate::events::aitalk::IMMERSIVE_ICON_COUNT;
use crate::events::common::TranslateOption;
use crate::events::mouse_core::Direction;
use crate::events::talk::randomtalk::random_talks;
use crate::events::talk::{TalkType, TalkingPlace};
use crate::roulette::TalkBias;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::LazyLock;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) const GHOST_NAME: &str = "Crave The Grave";
const VAR_PATH: &str = "vars.json";
pub(crate) static TOTAL_BOOT_COUNT: LazyLock<RwLock<u64>> = LazyLock::new(|| RwLock::new(0));
pub(crate) static TOTAL_TIME: LazyLock<RwLock<u64>> = LazyLock::new(|| RwLock::new(0));
pub(crate) static RANDOM_TALK_INTERVAL: LazyLock<RwLock<u64>> = LazyLock::new(|| RwLock::new(180));
pub(crate) static USER_NAME: LazyLock<RwLock<String>> =
  LazyLock::new(|| RwLock::new("".to_string()));
pub(crate) static TALK_COLLECTION: LazyLock<RwLock<HashMap<TalkType, HashSet<String>>>> =
  LazyLock::new(|| RwLock::new(HashMap::new()));
pub(crate) static CUMULATIVE_TALK_COUNT: LazyLock<RwLock<u64>> = LazyLock::new(|| RwLock::new(0));
pub(crate) static FLAGS: LazyLock<RwLock<EventFlags>> =
  LazyLock::new(|| RwLock::new(EventFlags::default()));
pub(crate) static PENDING_EVENT_TALK: LazyLock<RwLock<Option<PendingEvent>>> =
  LazyLock::new(|| RwLock::new(None));

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub(crate) enum PendingEvent {
  ConfessionOfSuicide,
  UnlockingLoreTalks,
  UnlockingServantsComments,
  FirstBoot,
  FirstRandomTalk(u32),
  FirstClose,
  FirstPlaceChange,
}

impl std::fmt::Display for PendingEvent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.title())
  }
}

impl PendingEvent {
  const SUICIDE: &'static str = "ハイネの様子を観察する";
  const LORE_TALKS: &'static str = "新しい話";
  const SERVANTS: &'static str = "従者について";
  const FIRST_BOOT: &'static str = "初回起動";
  const FIRST_RANDOMTALK: &'static str = "初回ランダムトーク";
  const FIRST_PLACE_CHANGE: &'static str = "初回独白モード開始";
  const FIRST_CLOSE: &'static str = "初回終了";
  pub fn from_str(title: &str) -> Option<Self> {
    match title {
      Self::SUICIDE => Some(Self::ConfessionOfSuicide),
      Self::LORE_TALKS => Some(Self::UnlockingLoreTalks),
      Self::SERVANTS => Some(Self::UnlockingServantsComments),
      Self::FIRST_BOOT => Some(Self::FirstBoot),
      Self::FIRST_CLOSE => Some(Self::FirstClose),
      Self::FIRST_PLACE_CHANGE => Some(Self::FirstPlaceChange),
      _ => {
        if !title.starts_with(Self::FIRST_RANDOMTALK) {
          None
        } else if let Ok(a) = title.replace(Self::FIRST_RANDOMTALK, "").parse::<u32>() {
          Some(Self::FirstRandomTalk(a))
        } else {
          None
        }
      }
    }
  }
  pub fn title(&self) -> String {
    match self {
      Self::ConfessionOfSuicide => Self::SUICIDE.to_string(),
      Self::UnlockingLoreTalks => Self::LORE_TALKS.to_string(),
      Self::UnlockingServantsComments => Self::SERVANTS.to_string(),
      Self::FirstBoot => Self::FIRST_BOOT.to_string(),
      Self::FirstRandomTalk(i) => format!("{}{}", Self::FIRST_RANDOMTALK, i),
      Self::FirstClose => Self::FIRST_CLOSE.to_string(),
      Self::FirstPlaceChange => Self::FIRST_PLACE_CHANGE.to_string(),
    }
  }
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct RawVariables {
  total_boot_count: u64,
  total_time: Option<u64>,
  random_talk_interval: Option<u64>,
  user_name: Option<String>,
  talk_collection: HashMap<TalkType, HashSet<String>>,
  cumulative_talk_count: u64,
  flags: EventFlags,
  pending_event_talk: Option<PendingEvent>,
}

impl RawVariables {
  pub fn load() -> Result<Self, Box<dyn Error>> {
    if !std::path::Path::new(VAR_PATH).exists() {
      return Ok(Self::default());
    }

    let json_str = std::fs::read_to_string(VAR_PATH)?;
    let vars: RawVariables = serde_json::from_str(&json_str)?;
    Ok(vars)
  }

  pub fn save(&self) -> Result<(), Box<dyn Error>> {
    let json_str_indent = serde_json::to_string_pretty(&self)?;
    std::fs::write(VAR_PATH, json_str_indent)?;
    Ok(())
  }
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Debug)]
pub(crate) enum EventFlag {
  FirstBoot,
  FirstRandomTalkDone(u32),
  FirstPlaceChange,
  FirstClose,
  FirstHitTalkStart,
  FirstHitTalkDone,
  TalkTypeUnlock(TalkType),
  FirstLibraryEnd,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct EventFlags {
  flags: HashSet<EventFlag>,
}

impl EventFlags {
  pub fn done(&mut self, flag: EventFlag) {
    self.flags.insert(flag);
  }

  pub fn check(&self, flag: &EventFlag) -> bool {
    self.flags.contains(flag)
  }

  pub fn delete(&mut self, flag: EventFlag) {
    self.flags.remove(&flag);
  }
}

pub(crate) const TRANSPARENT_SURFACE: i32 = 1000000;

pub fn load_global_variables() -> Result<(), Box<dyn Error>> {
  let raw_vars = RawVariables::load()?;

  *TOTAL_BOOT_COUNT.write().unwrap() = raw_vars.total_boot_count;
  if let Some(time) = raw_vars.total_time {
    *TOTAL_TIME.write().unwrap() = time;
  }
  if let Some(interval) = raw_vars.random_talk_interval {
    *RANDOM_TALK_INTERVAL.write().unwrap() = interval;
  }
  if let Some(name) = raw_vars.user_name {
    *USER_NAME.write().unwrap() = name;
  }
  *CUMULATIVE_TALK_COUNT.write().unwrap() = raw_vars.cumulative_talk_count;
  *FLAGS.write().unwrap() = raw_vars.flags;
  *PENDING_EVENT_TALK.write().unwrap() = raw_vars.pending_event_talk;
  let mut raw_talk_collection: HashMap<TalkType, HashSet<String>> = HashMap::new();
  for (talk_type, ids) in raw_vars.talk_collection {
    // トークidがtalksに含まれている場合のみ追加
    // 更新で削除されたトークなどを除外するため
    let talks = match random_talks(talk_type) {
      Some(t) => t,
      None => {
        continue;
      }
    };
    let talk_ids = talks.into_iter().map(|t| t.id).collect::<Vec<String>>();
    let existing_and_seen_talk_ids: HashSet<String> = ids
      .into_iter()
      .filter(|id| talk_ids.contains(id))
      .collect::<HashSet<String>>();
    raw_talk_collection.insert(talk_type, existing_and_seen_talk_ids);
  }
  *TALK_COLLECTION.write().unwrap() = raw_talk_collection;

  Ok(())
}

pub fn save_global_variables() -> Result<(), Box<dyn Error>> {
  let raw_vars = RawVariables {
    total_boot_count: *TOTAL_BOOT_COUNT.read().unwrap(),
    total_time: Some(*TOTAL_TIME.read().unwrap()),
    random_talk_interval: Some(*RANDOM_TALK_INTERVAL.read().unwrap()),
    user_name: Some(USER_NAME.read().unwrap().clone()),
    talk_collection: TALK_COLLECTION.read().unwrap().clone(),
    cumulative_talk_count: *CUMULATIVE_TALK_COUNT.read().unwrap(),
    flags: FLAGS.read().unwrap().clone(),
    pending_event_talk: PENDING_EVENT_TALK.read().unwrap().clone(),
  };

  raw_vars.save()?;

  Ok(())
}

// ゴーストのグローバル変数のうち、揮発性(起動毎にリセットされる)のもの
pub(crate) static DEBUG_MODE: LazyLock<RwLock<bool>> = LazyLock::new(|| RwLock::new(false));
pub(crate) static LOG_PATH: LazyLock<RwLock<String>> =
  LazyLock::new(|| RwLock::new("".to_string()));
pub(crate) static GHOST_UP_TIME: LazyLock<RwLock<u64>> = LazyLock::new(|| RwLock::new(0));
pub(crate) static LAST_RANDOM_TALK_TIME: LazyLock<RwLock<u64>> = LazyLock::new(|| RwLock::new(0));
pub(crate) static NADE_COUNTER: LazyLock<RwLock<i32>> = LazyLock::new(|| RwLock::new(0));
pub(crate) static LAST_NADE_COUNT_UNIXTIME: LazyLock<RwLock<SystemTime>> =
  LazyLock::new(|| RwLock::new(UNIX_EPOCH));
pub(crate) static LAST_NADE_PART: LazyLock<RwLock<String>> =
  LazyLock::new(|| RwLock::new("".to_string()));
pub(crate) static WHEEL_DIRECTION: LazyLock<RwLock<Direction>> =
  LazyLock::new(|| RwLock::new(Direction::Up));
pub(crate) static WHEEL_COUNTER: LazyLock<RwLock<i32>> = LazyLock::new(|| RwLock::new(0));
pub(crate) static LAST_WHEEL_COUNT_UNIXTIME: LazyLock<RwLock<SystemTime>> =
  LazyLock::new(|| RwLock::new(UNIX_EPOCH));
pub(crate) static LAST_WHEEL_PART: LazyLock<RwLock<String>> =
  LazyLock::new(|| RwLock::new("".to_string()));
pub(crate) static FIRST_SEXIAL_TOUCH: LazyLock<RwLock<bool>> = LazyLock::new(|| RwLock::new(false));
pub(crate) static LAST_TOUCH_INFO: LazyLock<RwLock<String>> =
  LazyLock::new(|| RwLock::new("".to_string()));
pub(crate) static INSERTER: LazyLock<RwLock<Inserter>> =
  LazyLock::new(|| RwLock::new(Inserter::new(22.0)));
pub(crate) static TALK_BIAS: LazyLock<RwLock<TalkBias>> =
  LazyLock::new(|| RwLock::new(TalkBias::new()));
pub(crate) static CURRENT_SURFACE: LazyLock<RwLock<i32>> = LazyLock::new(|| RwLock::new(0));
pub(crate) static IDLE_SECONDS: LazyLock<RwLock<i32>> = LazyLock::new(|| RwLock::new(0));
pub(crate) static IMMERSIVE_DEGREES: LazyLock<RwLock<u32>> = LazyLock::new(|| RwLock::new(0));
pub(crate) static WAITING_TALK: LazyLock<RwLock<Option<(String, HashSet<TranslateOption>)>>> =
  LazyLock::new(|| RwLock::new(None));
pub(crate) static TOUCH_INFO: LazyLock<RwLock<HashMap<String, TouchInfo>>> =
  LazyLock::new(|| RwLock::new(HashMap::new()));
pub(crate) static TALKING_PLACE: LazyLock<RwLock<TalkingPlace>> =
  LazyLock::new(|| RwLock::new(TalkingPlace::LivingRoom));
pub(crate) static LAST_ANCHOR_ID: LazyLock<RwLock<Option<String>>> =
  LazyLock::new(|| RwLock::new(None));
pub(crate) static CANDLES: LazyLock<RwLock<[bool; IMMERSIVE_ICON_COUNT as usize]>> =
  LazyLock::new(|| RwLock::new([false; IMMERSIVE_ICON_COUNT as usize]));
pub(crate) static IS_IMMERSIVE_DEGREES_FIXED: LazyLock<RwLock<bool>> =
  LazyLock::new(|| RwLock::new(false));

pub(crate) const IDLE_THRESHOLD: i32 = 60 * 5;

#[derive(Clone)]
pub(crate) struct TouchInfo {
  count: u32,
  pub last_unixtime: SystemTime,
}

const TOUCH_RESET_DURATION: Duration = Duration::from_secs(60 * 3);

impl TouchInfo {
  pub fn new() -> Self {
    Self {
      count: 0,
      last_unixtime: UNIX_EPOCH,
    }
  }

  pub fn reset(&mut self) {
    self.count = 0;
    self.last_unixtime = SystemTime::UNIX_EPOCH;
  }

  pub fn reset_if_timeover(&mut self) -> Result<(), ShioriError> {
    if check_error!(self.last_unixtime.elapsed(), ShioriError::SystemTimeError)
      > TOUCH_RESET_DURATION
    {
      self.reset();
    }
    Ok(())
  }

  pub fn count(&mut self) -> Result<u32, ShioriError> {
    if check_error!(self.last_unixtime.elapsed(), ShioriError::SystemTimeError)
      > TOUCH_RESET_DURATION
    {
      self.count = 0;
      Ok(0)
    } else {
      Ok(self.count)
    }
  }

  pub fn add(&mut self) {
    self.count += 1;
    self.last_unixtime = SystemTime::now();
  }
}
