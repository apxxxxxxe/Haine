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
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) const GHOST_NAME: &str = "Crave The Grave";
const VAR_PATH: &str = "vars.json";
static mut GLOBALVARS: Option<GlobalVariables> = None;

macro_rules! generate_getter_setter {
  ($field_name:ident, $field_type:ty, cloneable) => {
    generate_getter!($field_name, $field_type, cloneable);
    generate_setter!($field_name, $field_type);
  };
  ($field_name:ident, $field_type:ty, non_cloneable) => {
    generate_getter!($field_name, $field_type, non_cloneable);
    generate_setter!($field_name, $field_type);
  };
}

macro_rules! generate_getter {
  ($field_name:ident, $field_type:ty, cloneable) => {
    pub fn $field_name(&self) -> $field_type {
      self.$field_name.lock().unwrap().clone()
    }
  };
  ($field_name:ident, $field_type:ty, non_cloneable) => {
    pub fn $field_name(&self) -> std::sync::MutexGuard<$field_type> {
      self.$field_name.lock().unwrap()
    }
  };
}

macro_rules! generate_setter {
  ($field_name:ident, $field_type:ty) => {
    paste::item! {
        pub fn [<set_ $field_name>](&mut self, value: $field_type) {
          *self.$field_name.lock().unwrap() = value;
        }
    }
  };
}

macro_rules! generate_mut_getter {
  ($field_name:ident, $field_type:ty, cloneable) => {
    paste::item! {
        pub fn [<$field_name _mut>](&mut self) -> &mut $field_type {
            &mut self.$field_name.lock().unwrap()
        }
    }
  };
  ($field_name:ident, $field_type:ty, non_cloneable) => {
    paste::item! {
        pub fn [<$field_name _mut>](&mut self) -> std::sync::MutexGuard<$field_type> {
            self.$field_name.lock().unwrap()
        }
    }
  };
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub(crate) enum PendingEvent {
  ConfessionOfSuicide,
  UnlockingLoreTalks,
  UnlockingServantsComments,
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
  pub fn from_str(title: &str) -> Option<Self> {
    match title {
      Self::SUICIDE => Some(Self::ConfessionOfSuicide),
      Self::LORE_TALKS => Some(Self::UnlockingLoreTalks),
      Self::SERVANTS => Some(Self::UnlockingServantsComments),
      _ => None,
    }
  }
  pub fn title(&self) -> &str {
    match self {
      Self::ConfessionOfSuicide => Self::SUICIDE,
      Self::UnlockingLoreTalks => Self::LORE_TALKS,
      Self::UnlockingServantsComments => Self::SERVANTS,
    }
  }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct GlobalVariables {
  // ゴーストの累計起動回数
  total_boot_count: Mutex<u64>,

  // ゴーストの累計起動時間(秒数)
  total_time: Mutex<Option<u64>>,

  // ランダムトークの間隔(秒数)
  random_talk_interval: Mutex<Option<u64>>,

  // ユーザ名
  user_name: Mutex<Option<String>>,

  talk_collection: Mutex<HashMap<TalkType, HashSet<String>>>,

  cumulative_talk_count: Mutex<u64>,

  flags: Mutex<EventFlags>,

  pending_event_talk: Mutex<Option<PendingEvent>>,

  // 起動ごとにリセットされる変数
  #[serde(skip)]
  pub volatility: VolatilityVariables,
}

impl GlobalVariables {
  pub fn new() -> Self {
    let mut s = Self {
      total_boot_count: Mutex::new(0),
      total_time: Mutex::new(Some(0)),
      random_talk_interval: Mutex::new(Some(180)),
      user_name: Mutex::new(None),
      talk_collection: Mutex::new(HashMap::new()),
      volatility: VolatilityVariables::default(),
      cumulative_talk_count: Mutex::new(0),
      flags: Mutex::new(EventFlags::default()),
      pending_event_talk: Mutex::new(None),
    };

    // 形態素解析器は時間がかかるので非同期的に初期化
    s.volatility.inserter_mut().start_init();

    s
  }

  // vars_20060102150405.json の形式にリネーム
  fn backup(&mut self) -> Result<(), Box<dyn Error>> {
    if !std::path::Path::new(VAR_PATH).exists() {
      return Ok(());
    }
    let now = chrono::Local::now();
    let backup_path = format!("vars_{}.json", now.format("%Y%m%d%H%M%S"));
    std::fs::copy(VAR_PATH, backup_path)?;
    Ok(())
  }

  pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
    if !std::path::Path::new(VAR_PATH).exists() {
      return Ok(());
    }

    let json_str = match std::fs::read_to_string(VAR_PATH) {
      Ok(v) => v,
      Err(e) => {
        self.backup()?;
        return Err(Box::new(e));
      }
    };
    let vars: GlobalVariables = match serde_json::from_str(&json_str) {
      Ok(v) => v,
      Err(e) => {
        self.backup()?;
        return Err(Box::new(e));
      }
    };

    // TODO: 変数追加時はここに追加することを忘れない
    if let Some(t) = vars.total_time() {
      self.set_total_time(Some(t));
    }
    if let Some(t) = vars.random_talk_interval() {
      self.set_random_talk_interval(Some(t));
    }
    if let Some(t) = vars.user_name() {
      self.set_user_name(Some(t));
    }
    if !vars.talk_collection().is_empty() {
      self.set_talk_collection(vars.talk_collection());
    }
    if !vars.flags().is_empty() {
      self.set_flags(vars.flags());
    }
    if vars.pending_event_talk().is_some() {
      self.set_pending_event_talk(vars.pending_event_talk());
    }

    self.delete_undefined_talks();

    Ok(())
  }

  pub fn save(&self) -> Result<(), Box<dyn Error>> {
    let json_str_indent = serde_json::to_string_pretty(&self)?;
    std::fs::write(VAR_PATH, json_str_indent)?;
    Ok(())
  }

  fn delete_undefined_talks(&mut self) {
    let mut talk_collection = self.talk_collection_mut();
    for (k, v) in talk_collection.iter_mut() {
      let talks = if let Some(v) = random_talks(*k) {
        v
      } else {
        continue;
      };
      let talk_ids = talks
        .iter()
        .map(|t| t.id.to_string())
        .collect::<HashSet<String>>();
      for talk_id in v.clone() {
        if !talk_ids.contains(&talk_id) {
          v.remove(&talk_id);
        }
      }
    }
  }

  generate_getter_setter!(total_boot_count, u64, cloneable);
  generate_getter_setter!(total_time, Option<u64>, cloneable);
  generate_getter_setter!(random_talk_interval, Option<u64>, cloneable);
  generate_getter_setter!(user_name, Option<String>, cloneable);
  generate_getter_setter!(talk_collection, HashMap<TalkType, HashSet<String>>, cloneable);
  generate_mut_getter!(talk_collection, HashMap<TalkType, HashSet<String>>, non_cloneable);
  generate_getter_setter!(cumulative_talk_count, u64, cloneable);
  generate_getter_setter!(flags, EventFlags, cloneable);
  generate_mut_getter!(flags, EventFlags, non_cloneable);
  generate_getter_setter!(pending_event_talk, Option<PendingEvent>, cloneable);
}

pub(crate) fn get_global_vars() -> &'static mut GlobalVariables {
  unsafe {
    if GLOBALVARS.is_none() {
      GLOBALVARS = Some(GlobalVariables::new());
    }
    GLOBALVARS.as_mut().unwrap()
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
  pub fn is_empty(&self) -> bool {
    self.flags.is_empty()
  }

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

// ゴーストのグローバル変数のうち、揮発性(起動毎にリセットされる)のもの
pub(crate) struct VolatilityVariables {
  pub debug_mode: Mutex<bool>,

  // ログファイルのフルパス
  pub log_path: Mutex<String>,

  // ゴーストが起動してからの秒数
  pub ghost_up_time: Mutex<u64>,

  pub last_random_talk_time: Mutex<u64>,

  pub nade_counter: Mutex<i32>,

  pub last_nade_count_unixtime: Mutex<SystemTime>,

  pub last_nade_part: Mutex<String>,

  pub wheel_direction: Mutex<Direction>,

  pub wheel_counter: Mutex<i32>,

  pub last_wheel_count_unixtime: Mutex<SystemTime>,

  pub last_wheel_part: Mutex<String>,

  pub first_sexial_touch: Mutex<bool>,

  pub last_touch_info: Mutex<String>,

  pub inserter: Mutex<Inserter>,

  pub talk_bias: Mutex<TalkBias>,

  pub current_surface: Mutex<i32>,

  pub idle_seconds: Mutex<i32>,

  pub immersive_degrees: Mutex<u32>,

  pub waiting_talk: Mutex<Option<(String, HashSet<TranslateOption>)>>,

  pub touch_info: Mutex<HashMap<String, TouchInfo>>,

  pub talking_place: Mutex<TalkingPlace>,

  pub last_anchor_id: Mutex<Option<String>>,

  pub candles: Mutex<[bool; IMMERSIVE_ICON_COUNT as usize]>,

  pub is_immersive_degrees_fixed: Mutex<bool>,
}

#[allow(dead_code)]
impl VolatilityVariables {
  generate_getter_setter!(debug_mode, bool, cloneable);
  generate_getter_setter!(log_path, String, cloneable);
  generate_getter_setter!(ghost_up_time, u64, cloneable);
  generate_getter_setter!(last_random_talk_time, u64, cloneable);
  generate_getter_setter!(nade_counter, i32, cloneable);
  generate_getter_setter!(last_nade_count_unixtime, SystemTime, cloneable);
  generate_getter_setter!(last_nade_part, String, cloneable);
  generate_getter_setter!(wheel_direction, Direction, cloneable);
  generate_getter_setter!(wheel_counter, i32, cloneable);
  generate_getter_setter!(last_wheel_count_unixtime, SystemTime, cloneable);
  generate_getter_setter!(last_wheel_part, String, cloneable);
  generate_getter_setter!(first_sexial_touch, bool, cloneable);
  generate_getter_setter!(last_touch_info, String, cloneable);
  generate_mut_getter!(inserter, Inserter, non_cloneable);
  generate_mut_getter!(talk_bias, TalkBias, non_cloneable);
  generate_getter_setter!(current_surface, i32, cloneable);
  generate_getter_setter!(idle_seconds, i32, cloneable);
  generate_getter_setter!(immersive_degrees, u32, cloneable);
  generate_getter_setter!(
    waiting_talk,
    Option<(String, HashSet<TranslateOption>)>,
    cloneable
  );
  generate_mut_getter!(touch_info, HashMap<String, TouchInfo>, non_cloneable);
  generate_getter_setter!(talking_place, TalkingPlace, cloneable);
  generate_getter_setter!(last_anchor_id, Option<String>, cloneable);
  generate_mut_getter!(
    candles,
    [bool; IMMERSIVE_ICON_COUNT as usize],
    non_cloneable
  );
  generate_getter_setter!(is_immersive_degrees_fixed, bool, cloneable);
}

impl Default for VolatilityVariables {
  fn default() -> Self {
    Self {
      debug_mode: Mutex::new(false),
      log_path: Mutex::new("".to_string()),
      ghost_up_time: Mutex::new(0),
      last_random_talk_time: Mutex::new(0),
      nade_counter: Mutex::new(0),
      last_nade_count_unixtime: Mutex::new(UNIX_EPOCH),
      last_nade_part: Mutex::new("".to_string()),
      wheel_direction: Mutex::new(Direction::Up),
      wheel_counter: Mutex::new(0),
      last_wheel_count_unixtime: Mutex::new(UNIX_EPOCH),
      last_wheel_part: Mutex::new("".to_string()),
      first_sexial_touch: Mutex::new(false),
      last_touch_info: Mutex::new("".to_string()),
      inserter: Mutex::new(Inserter::new(22.0)), // "霧の郊外にて"に合わせた値
      talk_bias: Mutex::new(TalkBias::new()),
      current_surface: Mutex::new(0),
      idle_seconds: Mutex::new(0),
      immersive_degrees: Mutex::new(0),
      waiting_talk: Mutex::new(None),
      touch_info: Mutex::new(HashMap::new()),
      talking_place: Mutex::new(TalkingPlace::LivingRoom),
      last_anchor_id: Mutex::new(None),
      candles: Mutex::new([false; IMMERSIVE_ICON_COUNT as usize]),
      is_immersive_degrees_fixed: Mutex::new(false),
    }
  }
}

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
