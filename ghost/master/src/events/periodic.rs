use crate::error::ShioriError;
use crate::events::aitalk::on_ai_talk;
use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::status::Status;
use crate::variables::{
  EventFlag, CUMULATIVE_TALK_COUNT, CURRENT_SURFACE, FLAGS, GHOST_UP_TIME, IDLE_SECONDS,
  LAST_RANDOM_TALK_TIME, PENDING_EVENT_TALK, TALK_COLLECTION, TOTAL_TIME, USER_NAME,
};
use crate::variables::{PendingEvent, RANDOM_TALK_INTERVAL};
use chrono::Timelike;
use rand::prelude::SliceRandom;
use shiorust::message::{Request, Response};

pub(crate) const TALK_UNLOCK_COUNT_SERVANT: u64 = 5;
pub(crate) const TALK_UNLOCK_COUNT_LORE: u64 = 10;

pub(crate) fn on_notify_user_info(req: &Request) -> Response {
  let refs = get_references(req);
  *USER_NAME.write().unwrap() = refs[0].to_string();
  new_response_nocontent()
}

pub(crate) fn on_minute_change(_req: &Request) -> Response {
  check_story_events();
  new_response_nocontent()
}

pub(crate) fn on_second_change(req: &Request) -> Result<Response, ShioriError> {
  // 最小化中かどうかに関わらず実行する処理
  *TOTAL_TIME.write().unwrap() += 1;
  *GHOST_UP_TIME.write().unwrap() += 1;

  // 初回起動イベントが終わるまではランダムトークなし
  if !FLAGS.read().unwrap().check(&EventFlag::FirstRandomTalkDone(
    FIRST_RANDOMTALKS.len() as u32 - 1,
  )) {
    return Ok(new_response_nocontent());
  }

  let refs = get_references(req);
  let idle_secs = match refs[4].parse::<i32>() {
    Ok(v) => v,
    Err(_) => return Err(ShioriError::ParseIntError),
  };
  *IDLE_SECONDS.write().unwrap() = idle_secs;

  let status = Status::from_request(req);

  debug!("status: {}", status);
  {
    let random_talk_interval = *RANDOM_TALK_INTERVAL.read().unwrap();
    if random_talk_interval > 0
      && (*GHOST_UP_TIME.read().unwrap() - *LAST_RANDOM_TALK_TIME.read().unwrap())
        >= random_talk_interval
      && !status.minimizing
    {
      return on_ai_talk(req);
    }
  }

  let mut text = String::new();
  {
    if *GHOST_UP_TIME.read().unwrap() % 60 == 0 && !status.talking {
      // 1分ごとにサーフェスを重ね直す
      text += STICK_SURFACE;
    }
  }

  let now = chrono::Local::now();
  if now.minute() == 0 && now.second() == 0 {
    let tanka_list = [
      tanka(
        "もう二度と死ななくてよい安らぎに\\n見つめてゐたり祖母の寝顔を",
        "梶原さい子",
      ),
      tanka(
        "眼のまはり真紅(まあか)くなして泣きやめぬ\\n妻のうしろに吾子死にてあり",
        "木下利玄",
      ),
      tanka(
        "我が母よ死にたまひゆく我が母よ\\n我(わ)を生まし乳足(ちた)らひし母よ",
        "斎藤茂吉",
      ),
      tanka(
        "眠られぬ母のためわが誦む童話\\n母の寝入りし後王子死す",
        "岡井隆",
      ),
      tanka(
        "死せる犬またもわが眼にうかび来ぬ、\\nかの川ばたの夕ぐれの色",
        "金子薫園",
      ),
      tanka(
        "死に一歩踏み入りしとふ実感は\\nひるがへつて生の実感なりし",
        "後藤悦良",
      ),
      tanka(
        "蛍光灯のカヴァーの底を死場所としたる\\nこの世の虫のかずかず",
        "小池光",
      ),
      tanka(
        "死に向かふ生の底知れぬ虚無の淵を\\nのぞき見たりき彼の夜の君に",
        "柴生田稔",
      ),
      tanka(
        "やわらかく厚い果肉を掘りすすみ\\n核の付近で死んでいる虫",
        "北辻千展",
      ),
      tanka(
        "死にし子をまつたく忘れてゐる日あり\\n百日忌日(ひやくにちきじつ)にそれをしぞ嘆く",
        "吉野秀雄",
      ),
      tanka(
        "十トンの恐竜もゐしこの星に\\n四十八キロの妻生きて死す",
        "高野公彦",
      ),
      tanka(
        "生まれてはつひに死ぬてふことのみぞ\\n定めなき世に定めありける",
        "平維盛",
      ),
    ];

    let tanka = if let Some(v) = tanka_list.choose(&mut rand::thread_rng()) {
      v
    } else {
      return Err(ShioriError::ArrayAccessError);
    };

    text += &format!("\\1\\_q{}時\\n{}", now.hour(), tanka);
  }

  if text.is_empty() {
    Ok(new_response_nocontent())
  } else {
    new_response_with_value_with_translate(text, TranslateOption::simple_translate())
  }
}

fn tanka(text: &str, author: &str) -> String {
  format!("{}\\n\\f[align,right]({})", text, author)
}

pub(crate) fn on_surface_change(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let surface = match refs[0].parse::<i32>() {
    Ok(v) => v,
    Err(_) => return Err(ShioriError::ParseIntError),
  };

  *CURRENT_SURFACE.write().unwrap() = surface;

  Ok(new_response_nocontent())
}

pub(crate) fn check_story_events() {
  if !FLAGS
    .read()
    .unwrap()
    .check(&EventFlag::TalkTypeUnlock(super::TalkType::Servant))
    && *CUMULATIVE_TALK_COUNT.read().unwrap() >= TALK_UNLOCK_COUNT_SERVANT
  {
    // 従者コメント開放
    *PENDING_EVENT_TALK.write().unwrap() = Some(PendingEvent::UnlockingServantsComments);
  } else if !FLAGS
    .read()
    .unwrap()
    .check(&EventFlag::TalkTypeUnlock(super::TalkType::Lore))
    && *CUMULATIVE_TALK_COUNT.read().unwrap() >= TALK_UNLOCK_COUNT_LORE
  {
    // ロアトーク開放
    *PENDING_EVENT_TALK.write().unwrap() = Some(PendingEvent::UnlockingLoreTalks);
  } else if *PENDING_EVENT_TALK.read().unwrap() == Some(PendingEvent::ConfessionOfSuicide) {
    // 仕様変更のため解禁されないように
    // すでにPendingEventにConfessionOfSuicideがセットされている場合は消す
    *PENDING_EVENT_TALK.write().unwrap() = None;
  }

  // 過去トークの解禁がされている場合、再び閉じる
  {
    let mut flags = FLAGS.write().unwrap();
    if flags.check(&EventFlag::TalkTypeUnlock(super::TalkType::Past)) {
      flags.delete(EventFlag::TalkTypeUnlock(super::TalkType::Past));
    }
  }

  // 変数に過去トークの情報が入っている場合消去する
  if TALK_COLLECTION
    .write()
    .unwrap()
    .remove(&super::TalkType::Past)
    .is_some()
  {
    debug!("過去トークの情報を消去しました");
  }
}
