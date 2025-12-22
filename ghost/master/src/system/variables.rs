use crate::system::autobreakline::Inserter;
use crate::check_error;
use crate::system::error::ShioriError;
use crate::events::aitalk::IMMERSIVE_ICON_COUNT;
use crate::system::response::TranslateOption;
use crate::events::mouse_core::Direction;
use crate::events::talk::randomtalk::{derivative_talks, random_talks};
use crate::events::talk::{TalkType, TalkingPlace};
use crate::system::roulette::TalkBias;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::LazyLock;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) const GHOST_NAME: &str = "Crave The Grave";
const VAR_PATH: &str = "vars.json";
const VAR_BACKUP_PATH: &str = "vars.json.bak";
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
pub(crate) static DERIVATIVE_TALK_REQUESTABLE: LazyLock<RwLock<bool>> =
  LazyLock::new(|| RwLock::new(false));
pub(crate) static LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX: LazyLock<RwLock<u32>> =
  LazyLock::new(|| RwLock::new(1000));
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub(crate) enum LoadStatus {
  #[default]
  NotLoaded, // まだロードしていない（初期状態）
  FirstBoot,                   // 初回起動（セーブファイルなし）
  Success,                     // 正常ロード（バックアップあり）
  SuccessNoBackup,             // 正常ロード（バックアップなし）
  RestoredFromBackup,          // メインが失敗、バックアップから復元
  FailedNoBackup,              // ロード失敗、バックアップもなし
  PartialSuccess(Vec<String>), // 部分成功、失敗フィールド名を保持
}

impl LoadStatus {
  pub fn is_failed(&self) -> bool {
    matches!(self, LoadStatus::FailedNoBackup)
  }

  pub fn should_save(&self) -> bool {
    !self.is_failed()
  }
}

pub(crate) static LOAD_STATUS: LazyLock<RwLock<LoadStatus>> =
  LazyLock::new(|| RwLock::new(LoadStatus::NotLoaded));

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

/// RawVariables 構造体と load_partial_from メソッドを生成するマクロ
///
/// フィールドを追加する場合はこのマクロ呼び出しを編集してください。
/// 構造体定義と部分パース処理が自動的に同期されます。
macro_rules! define_raw_variables {
  (
    primitives: { $($prim_field:ident : $prim_type:ty),* $(,)? },
    options: { $($opt_field:ident : $opt_inner:ty),* $(,)? },
    custom: { $($custom_field:ident : $custom_type:ty => $parser:expr),* $(,)? }
  ) => {
    #[derive(Serialize, Deserialize, Default)]
    pub(crate) struct RawVariables {
      $($prim_field: $prim_type,)*
      $($opt_field: Option<$opt_inner>,)*
      $($custom_field: $custom_type,)*
    }

    impl RawVariables {
      /// 部分的にロード。失敗したフィールドはデフォルト値を使用
      /// 戻り値: (RawVariables, 失敗したフィールド名のリスト)
      pub fn load_partial_from(path: &str) -> Result<(Self, Vec<String>), Box<dyn std::error::Error>> {
        if !std::path::Path::new(path).exists() {
          return Ok((Self::default(), vec![]));
        }

        let json_str = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json_str)?;

        let obj = value
          .as_object()
          .ok_or("JSON root is not an object")?;

        let mut result = Self::default();
        let mut failed_fields = Vec::new();

        // プリミティブフィールドのパース
        $(
          if let Some(v) = obj.get(stringify!($prim_field)) {
            match serde_json::from_value::<$prim_type>(v.clone()) {
              Ok(val) => result.$prim_field = val,
              Err(e) => {
                warn!("{} のパースに失敗: {}", stringify!($prim_field), e);
                failed_fields.push(stringify!($prim_field).to_string());
              }
            }
          }
        )*

        // Option フィールドのパース
        $(
          if let Some(v) = obj.get(stringify!($opt_field)) {
            if !v.is_null() {
              match serde_json::from_value::<$opt_inner>(v.clone()) {
                Ok(val) => result.$opt_field = Some(val),
                Err(e) => {
                  warn!("{} のパースに失敗: {}", stringify!($opt_field), e);
                  failed_fields.push(stringify!($opt_field).to_string());
                }
              }
            }
          }
        )*

        // カスタムパーサーフィールドのパース
        $(
          if let Some(v) = obj.get(stringify!($custom_field)) {
            match $parser(v) {
              Ok((val, warnings)) => {
                result.$custom_field = val;
                // スキップした項目があれば警告として記録
                for w in warnings {
                  failed_fields.push(format!("{}: {}", stringify!($custom_field), w));
                }
              }
              Err(e) => {
                warn!("{} のパースに失敗: {}", stringify!($custom_field), e);
                failed_fields.push(stringify!($custom_field).to_string());
              }
            }
          }
        )*

        Ok((result, failed_fields))
      }
    }
  };
}

// RawVariables 構造体の定義
// フィールドを追加・変更する場合はここを編集
define_raw_variables! {
  primitives: {
    total_boot_count: u64,
    cumulative_talk_count: u64,
  },
  options: {
    total_time: u64,
    random_talk_interval: u64,
    user_name: String,
    pending_event_talk: PendingEvent,
    derivative_talk_requestable: bool,
    library_transition_sequense_dialog_index: u32,
  },
  custom: {
    talk_collection: HashMap<TalkType, HashSet<String>> => parse_talk_collection_lenient,
    flags: EventFlags => parse_event_flags_lenient,
  }
}

impl RawVariables {
  pub fn save(&self) -> Result<(), Box<dyn Error>> {
    self.save_to(VAR_PATH, VAR_BACKUP_PATH)
  }

  pub fn save_to(&self, path: &str, backup_path: &str) -> Result<(), Box<dyn Error>> {
    // 既存ファイルをバックアップ
    if std::path::Path::new(path).exists() {
      std::fs::copy(path, backup_path)?;
    }
    let json_str_indent = serde_json::to_string_pretty(&self)?;
    std::fs::write(path, json_str_indent)?;
    Ok(())
  }
}

/// TalkType のキーが一部無効でも、有効なものだけを読み込む
/// 戻り値: (パース結果, スキップした項目の警告リスト)
fn parse_talk_collection_lenient(
  value: &serde_json::Value,
) -> Result<(HashMap<TalkType, HashSet<String>>, Vec<String>), String> {
  use crate::events::talk::TalkType;

  let obj = value
    .as_object()
    .ok_or("talk_collection is not an object")?;

  let mut result = HashMap::new();
  let mut warnings = Vec::new();

  for (key, val) in obj {
    // TalkType のパースを試みる
    match serde_json::from_value::<TalkType>(serde_json::Value::String(key.clone())) {
      Ok(talk_type) => {
        // HashSet<String> のパース
        match serde_json::from_value::<HashSet<String>>(val.clone()) {
          Ok(ids) => {
            result.insert(talk_type, ids);
          }
          Err(e) => {
            let msg = format!("talk_collection[{}] の値のパースに失敗: {}", key, e);
            warn!("{}", msg);
            warnings.push(msg);
          }
        }
      }
      Err(_) => {
        // 不明なキーはスキップするが、データは失われる
        let msg = format!("不明なTalkType '{}' をスキップ", key);
        warn!("{}", msg);
        warnings.push(msg);
      }
    }
  }

  Ok((result, warnings))
}

/// EventFlag のバリアントが一部無効でも、有効なものだけを読み込む
/// 旧形式 {"flags": [...]} と新形式 [...] の両方に対応
/// 戻り値: (パース結果, スキップした項目の警告リスト)
fn parse_event_flags_lenient(
  value: &serde_json::Value,
) -> Result<(EventFlags, Vec<String>), String> {
  // 新形式: 配列そのまま
  // 旧形式: {"flags": [...]} からflagsを取り出す
  let arr = if let Some(obj) = value.as_object() {
    obj
      .get("flags")
      .and_then(|v| v.as_array())
      .ok_or("flags.flags is not an array (old format)")?
  } else if let Some(arr) = value.as_array() {
    arr
  } else {
    return Err("flags is neither an object nor an array".to_string());
  };

  let mut valid_flags = HashSet::new();
  let mut warnings = Vec::new();

  for flag_value in arr {
    match serde_json::from_value::<EventFlag>(flag_value.clone()) {
      Ok(flag) => {
        valid_flags.insert(flag);
      }
      Err(_) => {
        let msg = format!("不明なEventFlag をスキップ: {:?}", flag_value);
        warn!("{}", msg);
        warnings.push(msg);
      }
    }
  }

  Ok((EventFlags { flags: valid_flags }, warnings))
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
  /// 季節イベント: (年, 月, 日)
  SeasonEvent(u32, u32, u32),
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(transparent)]
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

  /// 指定した年月日の季節イベントが既に閲覧済みかチェック
  pub fn check_season_event(&self, year: u32, month: u32, day: u32) -> bool {
    self
      .flags
      .contains(&EventFlag::SeasonEvent(year, month, day))
  }

  /// 季節イベントを閲覧済みとしてマーク
  pub fn mark_season_event(&mut self, year: u32, month: u32, day: u32) {
    self.flags.insert(EventFlag::SeasonEvent(year, month, day));
  }

  /// 指定した月日の季節イベントを過去に何回見たか取得
  #[allow(dead_code)]
  pub fn count_season_event(&self, month: u32, day: u32) -> usize {
    self
      .flags
      .iter()
      .filter(|f| {
        if let EventFlag::SeasonEvent(_, m, d) = f {
          *m == month && *d == day
        } else {
          false
        }
      })
      .count()
  }
}

pub(crate) const TRANSPARENT_SURFACE: i32 = 1000000;

pub fn load_global_variables() -> Result<(), Box<dyn Error>> {
  let main_exists = std::path::Path::new(VAR_PATH).exists();
  let backup_exists = std::path::Path::new(VAR_BACKUP_PATH).exists();

  // メインファイルから部分パースを試行
  let raw_vars = match RawVariables::load_partial_from(VAR_PATH) {
    Ok((vars, failed_fields)) => {
      if !main_exists {
        // ファイルが存在しない場合は FirstBoot
        *LOAD_STATUS.write().unwrap() = LoadStatus::FirstBoot;
      } else if failed_fields.is_empty() {
        // 全フィールド成功
        if backup_exists {
          *LOAD_STATUS.write().unwrap() = LoadStatus::Success;
        } else {
          *LOAD_STATUS.write().unwrap() = LoadStatus::SuccessNoBackup;
        }
      } else {
        // 一部フィールドが失敗
        warn!("部分パースで失敗したフィールド: {:?}", failed_fields);
        *LOAD_STATUS.write().unwrap() = LoadStatus::PartialSuccess(failed_fields);
      }
      vars
    }
    Err(e) => {
      // メイン部分パースも失敗 → バックアップから部分パースを試行
      warn!("メインファイルのパースに失敗: {}", e);

      match RawVariables::load_partial_from(VAR_BACKUP_PATH) {
        Ok((backup_vars, backup_failed_fields)) => {
          if backup_failed_fields.is_empty() {
            warn!("バックアップから復元");
            *LOAD_STATUS.write().unwrap() = LoadStatus::RestoredFromBackup;
          } else {
            warn!(
              "バックアップから部分復元、失敗フィールド: {:?}",
              backup_failed_fields
            );
            *LOAD_STATUS.write().unwrap() = LoadStatus::PartialSuccess(backup_failed_fields);
          }
          backup_vars
        }
        Err(_) => {
          *LOAD_STATUS.write().unwrap() = LoadStatus::FailedNoBackup;
          return Err(e);
        }
      }
    }
  };
  debug!("load status: {:?}", *LOAD_STATUS.read().unwrap());

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
  let mut all_talk_ids = TalkType::all()
    .into_iter()
    .filter_map(random_talks)
    .flatten()
    .map(|t| t.id)
    .collect::<Vec<_>>();
  let derivalive_talk_ids = derivative_talks()
    .into_iter()
    .map(|t| t.id)
    .collect::<Vec<_>>();
  all_talk_ids.extend(derivalive_talk_ids);

  // 各TalkTypeごとに有効なトークIDのマップを作成
  let mut valid_talk_ids_per_type: HashMap<TalkType, HashSet<String>> = HashMap::new();
  for talk_type in TalkType::all() {
    let mut valid_ids = HashSet::new();

    // 通常トークのIDを追加
    if let Some(talks) = random_talks(talk_type) {
      for talk in talks {
        valid_ids.insert(talk.id);
      }
    }

    // 派生トークのIDを追加
    for derivative_talk in derivative_talks() {
      // 派生トークの親トークがこのTalkTypeに属するかチェック
      if let Some(parent_talks) = random_talks(talk_type) {
        if parent_talks
          .iter()
          .any(|t| t.id == derivative_talk.parent_id)
        {
          valid_ids.insert(derivative_talk.id);
        }
      }
    }

    valid_talk_ids_per_type.insert(talk_type, valid_ids);
  }

  for (talk_type, ids) in raw_vars.talk_collection {
    // そのTalkTypeに属するトークIDのみを残す
    let valid_ids = valid_talk_ids_per_type
      .get(&talk_type)
      .cloned()
      .unwrap_or_default();
    let existing_and_seen_talk_ids: HashSet<String> = ids
      .into_iter()
      .filter(|id| valid_ids.contains(id))
      .collect::<HashSet<String>>();
    raw_talk_collection.insert(talk_type, existing_and_seen_talk_ids);
  }
  *TALK_COLLECTION.write().unwrap() = raw_talk_collection;
  if let Some(derivative_talk_requestable) = raw_vars.derivative_talk_requestable {
    *DERIVATIVE_TALK_REQUESTABLE.write().unwrap() = derivative_talk_requestable;
  } else {
    *DERIVATIVE_TALK_REQUESTABLE.write().unwrap() = false;
  }

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
    derivative_talk_requestable: Some(*DERIVATIVE_TALK_REQUESTABLE.read().unwrap()),
    library_transition_sequense_dialog_index: Some(
      *LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX.read().unwrap(),
    ),
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

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  #[test]
  fn test_save_creates_backup() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("vars.json");
    let backup_path = dir.path().join("vars.json.bak");

    // 初回保存（バックアップなし）
    let vars = RawVariables::default();
    vars
      .save_to(main_path.to_str().unwrap(), backup_path.to_str().unwrap())
      .unwrap();
    assert!(main_path.exists());
    assert!(!backup_path.exists()); // 初回はバックアップなし

    // 2回目保存（バックアップ作成）
    vars
      .save_to(main_path.to_str().unwrap(), backup_path.to_str().unwrap())
      .unwrap();
    assert!(backup_path.exists()); // バックアップが作成される
  }

  #[test]
  fn test_load_backup_on_main_file_corruption() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("vars.json");
    let backup_path = dir.path().join("vars.json.bak");

    // 正常なデータを保存（2回保存してバックアップを作成）
    let vars = RawVariables {
      total_boot_count: 42,
      ..Default::default()
    };
    vars
      .save_to(main_path.to_str().unwrap(), backup_path.to_str().unwrap())
      .unwrap();
    vars
      .save_to(main_path.to_str().unwrap(), backup_path.to_str().unwrap())
      .unwrap();

    // メインファイルを破損
    fs::write(&main_path, "invalid json").unwrap();

    // メインからの部分パースは失敗（不正なJSON）
    let result = RawVariables::load_partial_from(main_path.to_str().unwrap());
    assert!(result.is_err());

    // バックアップから復元
    let (loaded, failed_fields) =
      RawVariables::load_partial_from(backup_path.to_str().unwrap()).unwrap();
    assert_eq!(loaded.total_boot_count, 42);
    assert!(failed_fields.is_empty());
  }

  #[test]
  fn test_load_fails_when_both_files_corrupted() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("vars.json");
    let backup_path = dir.path().join("vars.json.bak");

    // 両方を破損
    fs::write(&main_path, "invalid").unwrap();
    fs::write(&backup_path, "also invalid").unwrap();

    // 両方とも部分パースに失敗
    let main_result = RawVariables::load_partial_from(main_path.to_str().unwrap());
    assert!(main_result.is_err());

    let backup_result = RawVariables::load_partial_from(backup_path.to_str().unwrap());
    assert!(backup_result.is_err());
  }

  #[test]
  fn test_backup_preserves_previous_data() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("vars.json");
    let backup_path = dir.path().join("vars.json.bak");

    // 最初のデータを保存
    let vars1 = RawVariables {
      total_boot_count: 10,
      ..Default::default()
    };
    vars1
      .save_to(main_path.to_str().unwrap(), backup_path.to_str().unwrap())
      .unwrap();

    // 2回目のデータを保存（vars1がバックアップされる）
    let vars2 = RawVariables {
      total_boot_count: 20,
      ..Default::default()
    };
    vars2
      .save_to(main_path.to_str().unwrap(), backup_path.to_str().unwrap())
      .unwrap();

    // バックアップには前回のデータ（10）が残っている
    let (backup_loaded, _) =
      RawVariables::load_partial_from(backup_path.to_str().unwrap()).unwrap();
    assert_eq!(backup_loaded.total_boot_count, 10);

    // メインには新しいデータ（20）がある
    let (main_loaded, _) = RawVariables::load_partial_from(main_path.to_str().unwrap()).unwrap();
    assert_eq!(main_loaded.total_boot_count, 20);
  }

  #[test]
  fn test_load_from_nonexistent_file_returns_default() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("nonexistent.json");

    let (loaded, failed_fields) =
      RawVariables::load_partial_from(main_path.to_str().unwrap()).unwrap();
    assert_eq!(loaded.total_boot_count, 0); // default値
    assert!(failed_fields.is_empty());
  }

  #[test]
  fn test_partial_load_with_unknown_talk_type() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("vars.json");

    // 不明なTalkType "UnknownType" を含むJSON
    let json = r#"{
      "total_boot_count": 100,
      "cumulative_talk_count": 50,
      "talk_collection": {
        "AboutMe": ["talk1", "talk2"],
        "UnknownType": ["talk3"],
        "WithYou": ["talk4"]
      },
      "flags": {"flags": []}
    }"#;
    fs::write(&main_path, json).unwrap();

    // 部分パースは成功し、UnknownTypeだけスキップ
    let (vars, failed_fields) =
      RawVariables::load_partial_from(main_path.to_str().unwrap()).unwrap();
    assert_eq!(vars.total_boot_count, 100);
    assert_eq!(vars.cumulative_talk_count, 50);
    assert!(vars.talk_collection.contains_key(&TalkType::AboutMe));
    assert!(vars.talk_collection.contains_key(&TalkType::WithYou));
    // スキップした項目が警告として記録されている
    assert!(
      failed_fields
        .iter()
        .any(|f| f.contains("talk_collection") && f.contains("UnknownType")),
      "failed_fields should contain warning about UnknownType: {:?}",
      failed_fields
    );
  }

  #[test]
  fn test_partial_load_with_unknown_event_flag() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("vars.json");

    // 不明なEventFlag を含むJSON
    let json = r#"{
      "total_boot_count": 200,
      "cumulative_talk_count": 0,
      "talk_collection": {},
      "flags": {
        "flags": [
          "FirstBoot",
          {"UnknownFlag": "some_value"},
          "FirstClose"
        ]
      }
    }"#;
    fs::write(&main_path, json).unwrap();

    // 部分パースは成功し、不明なフラグだけスキップ
    let (vars, failed_fields) =
      RawVariables::load_partial_from(main_path.to_str().unwrap()).unwrap();
    assert_eq!(vars.total_boot_count, 200);
    assert!(vars.flags.check(&EventFlag::FirstBoot));
    assert!(vars.flags.check(&EventFlag::FirstClose));
    // スキップした項目が警告として記録されている
    assert!(
      failed_fields
        .iter()
        .any(|f| f.contains("flags") && f.contains("UnknownFlag")),
      "failed_fields should contain warning about UnknownFlag: {:?}",
      failed_fields
    );
  }

  #[test]
  fn test_partial_load_with_corrupted_field() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("vars.json");

    // total_time が不正な型（文字列）になっているJSON
    let json = r#"{
      "total_boot_count": 300,
      "total_time": "not_a_number",
      "cumulative_talk_count": 25,
      "talk_collection": {},
      "flags": {"flags": []}
    }"#;
    fs::write(&main_path, json).unwrap();

    // 部分パースは成功し、total_time だけデフォルト値
    let (vars, failed_fields) =
      RawVariables::load_partial_from(main_path.to_str().unwrap()).unwrap();
    assert_eq!(vars.total_boot_count, 300);
    assert_eq!(vars.cumulative_talk_count, 25);
    assert!(vars.total_time.is_none()); // デフォルト値
    assert!(failed_fields.contains(&"total_time".to_string()));
  }

  #[test]
  fn test_partial_load_returns_empty_failed_fields_on_valid_json() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("vars.json");

    // 正常なJSONを保存
    let vars = RawVariables {
      total_boot_count: 500,
      cumulative_talk_count: 100,
      ..Default::default()
    };
    vars
      .save_to(
        main_path.to_str().unwrap(),
        dir.path().join("backup.json").to_str().unwrap(),
      )
      .unwrap();

    // 部分パースでも全フィールド成功
    let (loaded, failed_fields) =
      RawVariables::load_partial_from(main_path.to_str().unwrap()).unwrap();
    assert_eq!(loaded.total_boot_count, 500);
    assert_eq!(loaded.cumulative_talk_count, 100);
    assert!(failed_fields.is_empty());
  }

  #[test]
  fn test_partial_load_nonexistent_file_returns_default() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("nonexistent.json");

    let (vars, failed_fields) =
      RawVariables::load_partial_from(main_path.to_str().unwrap()).unwrap();
    assert_eq!(vars.total_boot_count, 0);
    assert!(failed_fields.is_empty());
  }
}
