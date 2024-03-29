use crate::autobreakline::Inserter;
use crate::events::common::TranslateOption;
use crate::events::mouse_core::Direction;
use crate::roulette::TalkBias;
use crate::status::Status;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

#[derive(Serialize, Deserialize)]
pub struct GlobalVariables {
  // ゴーストの累計起動時間(秒数)
  total_time: Mutex<Option<u64>>,

  // ランダムトークの間隔(秒数)
  random_talk_interval: Mutex<Option<u64>>,

  // ユーザ名
  user_name: Mutex<Option<String>>,

  // 起動ごとにリセットされる変数
  #[serde(skip)]
  pub volatility: VolatilityVariables,
}

impl GlobalVariables {
  pub fn new() -> Self {
    let mut s = Self {
      total_time: Mutex::new(Some(0)),
      random_talk_interval: Mutex::new(Some(180)),
      user_name: Mutex::new(Some("ユーザ".to_string())),
      volatility: VolatilityVariables::default(),
    };

    // 形態素解析器は時間がかかるので非同期的に初期化
    s.volatility.inserter_mut().start_init();

    s
  }

  pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
    let json_str = std::fs::read_to_string(VAR_PATH)?;
    let vars: GlobalVariables = serde_json::from_str(&json_str)?;

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

    Ok(())
  }

  pub fn save(&self) -> Result<(), Box<dyn Error>> {
    let json_str = serde_json::to_string(self)?;
    std::fs::write(VAR_PATH, json_str)?;
    Ok(())
  }

  generate_getter_setter!(total_time, Option<u64>, cloneable);
  generate_getter_setter!(random_talk_interval, Option<u64>, cloneable);
  generate_getter_setter!(user_name, Option<String>, cloneable);
}

pub fn get_global_vars() -> &'static mut GlobalVariables {
  unsafe {
    if GLOBALVARS.is_none() {
      GLOBALVARS = Some(GlobalVariables::new());
    }
    GLOBALVARS.as_mut().unwrap()
  }
}

// ゴーストのグローバル変数のうち、揮発性(起動毎にリセットされる)のもの
pub struct VolatilityVariables {
  // ログファイルのフルパス
  pub log_path: Mutex<String>,

  // ゴーストが起動してからの秒数
  pub ghost_up_time: Mutex<u64>,

  pub last_random_talk_time: Mutex<u64>,

  // ゴーストの起動日時
  pub ghost_boot_time: Mutex<SystemTime>,

  pub nade_counter: Mutex<i32>,

  pub last_nade_count_unixtime: Mutex<SystemTime>,

  pub last_nade_part: Mutex<String>,

  pub wheel_direction: Mutex<Direction>,

  pub wheel_counter: Mutex<i32>,

  pub last_wheel_count_unixtime: Mutex<SystemTime>,

  pub last_wheel_part: Mutex<String>,

  pub first_sexial_touch: Mutex<bool>,

  pub touch_count: Mutex<i32>,

  pub last_touch_info: Mutex<String>,

  pub inserter: Mutex<Inserter>,

  pub talk_bias: Mutex<TalkBias>,

  pub status: Mutex<Status>,

  pub current_surface: Mutex<i32>,

  pub idle_seconds: Mutex<i32>,

  pub immersive_degrees: Mutex<u32>,

  pub waiting_talk: Mutex<Option<(String, TranslateOption)>>,

  pub touch_info: Mutex<TouchInfoMap>,
}

#[allow(dead_code)]
impl VolatilityVariables {
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
  generate_getter_setter!(touch_count, i32, cloneable);
  generate_getter_setter!(last_touch_info, String, cloneable);
  generate_mut_getter!(inserter, Inserter, non_cloneable);
  generate_mut_getter!(talk_bias, TalkBias, non_cloneable);
  generate_getter_setter!(status, Status, non_cloneable);
  generate_mut_getter!(status, Status, non_cloneable);
  generate_getter_setter!(current_surface, i32, cloneable);
  generate_getter_setter!(idle_seconds, i32, cloneable);
  generate_getter_setter!(immersive_degrees, u32, cloneable);
  generate_getter_setter!(waiting_talk, Option<(String, TranslateOption)>, cloneable);
  generate_mut_getter!(touch_info, TouchInfoMap, non_cloneable);
}

impl Default for VolatilityVariables {
  fn default() -> Self {
    Self {
      log_path: Mutex::new("".to_string()),
      ghost_up_time: Mutex::new(0),
      last_random_talk_time: Mutex::new(0),
      ghost_boot_time: Mutex::new(SystemTime::now()),
      nade_counter: Mutex::new(0),
      last_nade_count_unixtime: Mutex::new(UNIX_EPOCH),
      last_nade_part: Mutex::new("".to_string()),
      wheel_direction: Mutex::new(Direction::Up),
      wheel_counter: Mutex::new(0),
      last_wheel_count_unixtime: Mutex::new(UNIX_EPOCH),
      last_wheel_part: Mutex::new("".to_string()),
      first_sexial_touch: Mutex::new(false),
      touch_count: Mutex::new(0),
      last_touch_info: Mutex::new("".to_string()),
      inserter: Mutex::new(Inserter::new(22.0)), // "霧の郊外にて"に合わせた値
      talk_bias: Mutex::new(TalkBias::new()),
      current_surface: Mutex::new(0),
      status: Mutex::new(Status::new()),
      idle_seconds: Mutex::new(0),
      immersive_degrees: Mutex::new(0),
      waiting_talk: Mutex::new(None),
      touch_info: Mutex::new(TouchInfoMap::new()),
    }
  }
}

pub const IDLE_THRESHOLD: i32 = 60 * 5;

#[derive(Clone)]
pub struct TouchInfoMap {
  pub head: TouchInfo,
  pub shoulder: TouchInfo,
  pub bust: TouchInfo,
  pub hand: TouchInfo,
  pub skirt: TouchInfo,
}

impl TouchInfoMap {
  pub fn new() -> Self {
    Self {
      head: TouchInfo::new(),
      shoulder: TouchInfo::new(),
      bust: TouchInfo::new(),
      hand: TouchInfo::new(),
      skirt: TouchInfo::new(),
    }
  }
}

#[derive(Clone)]
pub struct TouchInfo {
  pub count: i32,
  pub last_unixtime: SystemTime,
}

const TOUCH_RESET_DURATION: Duration = Duration::from_secs(60 * 5);

impl TouchInfo {
  pub fn new() -> Self {
    Self {
      count: 0,
      last_unixtime: UNIX_EPOCH,
    }
  }

  pub fn is_reset(&self) -> bool {
    debug!("elapsed: {:?}", self.last_unixtime.elapsed().unwrap());
    self.last_unixtime.elapsed().unwrap() > TOUCH_RESET_DURATION
  }

  pub fn add(&mut self) {
    self.count += 1;
    self.last_unixtime = SystemTime::now();
  }
}
