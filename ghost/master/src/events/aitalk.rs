use crate::error::ShioriError;
use crate::events::common::*;
use crate::events::randomtalk::RANDOMTALK_COMMENTS_LIVING_ROOM;
use crate::events::talk::anchor::anchor_talks;
use crate::events::talk::randomtalk::random_talks;
use crate::events::talk::{register_talk_collection, TalkType, TalkingPlace};
use crate::events::{
  first_boot::{FIRST_BOOT_MARKER, FIRST_RANDOMTALKS},
  randomtalk::RANDOMTALK_COMMENTS_LIBRARY_INACTIVE,
};
use crate::variables::*;
use shiorust::message::{parts::*, Request, Response};
use vibrato::errors::Result;

use super::talk::randomtalk::{derivative_talk_by_id, derivative_talks};
use super::talk::Talk;
use super::webclap::derivative_talk_request_open;

pub(crate) const IMMERSIVE_RATE_MAX: u32 = 100;
// トーク1回あたりに増減する没入度の割合(%)
pub(crate) const IMMERSIVE_RATE: u32 = 5;

pub(crate) const IMMERSIVE_ICON_COUNT: u32 = 5;

pub(crate) fn on_ai_talk(_req: &Request) -> Result<Response, ShioriError> {
  let if_consume_talk_bias = *IDLE_SECONDS.read().unwrap() < IDLE_THRESHOLD;
  *LAST_RANDOM_TALK_TIME.write().unwrap() = *GHOST_UP_TIME.read().unwrap();

  // 初回ランダムトーク
  let text_count = FIRST_RANDOMTALKS.len();
  for (i, text) in FIRST_RANDOMTALKS.iter().enumerate() {
    if !FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::FirstRandomTalkDone(i as u32))
    {
      return first_random_talk_response(text.to_string(), i, text_count);
    }
  }

  // 通常ランダムトーク
  let talk_types = TALKING_PLACE.read().unwrap().talk_types();
  let talk_lists = talk_types
    .iter()
    .filter(|t| FLAGS.read().unwrap().check(&EventFlag::TalkTypeUnlock(**t)))
    .map(|t| random_talks(*t));
  if talk_lists.clone().any(|t| t.is_none()) {
    return Err(ShioriError::TalkNotFound);
  };
  let talks = talk_lists.flat_map(|t| t.unwrap()).collect::<Vec<_>>();
  let len_after_flatten = talks.len();
  let index = if let Some(v) = choose_one(&talks, if_consume_talk_bias) {
    v
  } else {
    let mut res = new_response_nocontent();
    add_error_description(
      &mut res,
      format!("No talk found: , {}", len_after_flatten).as_str(),
    );
    return Ok(res);
  };
  let choosed_talk = talks[index].clone();
  if if_consume_talk_bias {
    // ユーザが見ているときのみトークを消費&トークカウントを加算
    if let Some(talk_type) = choosed_talk.talk_type {
      register_talk_collection(&choosed_talk.id, talk_type)?;
    }
    *CUMULATIVE_TALK_COUNT.write().unwrap() += 1;
  }

  // バルーン右下に表示するコメントを取得
  let comment = if *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
    // 書斎では能動的に話しかけたかどうかで異なるコメントを表示
    let index =
      choose_one(&RANDOMTALK_COMMENTS_LIBRARY_INACTIVE, false).ok_or(ShioriError::TalkNotFound)?;
    RANDOMTALK_COMMENTS_LIBRARY_INACTIVE[index].to_string()
  } else {
    // 居間では従者トーク解禁済みの場合コメントを表示
    if FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Servant))
    {
      let index =
        choose_one(&RANDOMTALK_COMMENTS_LIVING_ROOM, false).ok_or(ShioriError::TalkNotFound)?;
      RANDOMTALK_COMMENTS_LIVING_ROOM[index].to_string()
    } else {
      "".to_string()
    }
  };

  // 没入度を増減
  // トークのたび燭台への干渉を修復する方へ没入度が増減する
  if *TALKING_PLACE.read().unwrap() == TalkingPlace::LivingRoom {
    let new_rate;
    {
      new_rate = IMMERSIVE_DEGREES
        .read()
        .unwrap()
        .saturating_sub(IMMERSIVE_RATE);
    }
    *IMMERSIVE_DEGREES.write().unwrap() = new_rate;
  } else {
    let new_rate;
    {
      new_rate = *IMMERSIVE_DEGREES.read().unwrap() + IMMERSIVE_RATE;
    }
    *IMMERSIVE_DEGREES.write().unwrap() = new_rate.min(IMMERSIVE_RATE_MAX);
  }

  new_response_with_value_with_translate(
    format!(
      "\\0{}\\![set,balloonnum,{}]{}",
      render_immersive_icon(),
      comment,
      render_talk(&choosed_talk),
    ),
    TranslateOption::with_shadow_completion(),
  )
}

pub fn render_talk(talk: &Talk) -> String {
  let derivative_talk_request_button = if *DERIVATIVE_TALK_REQUESTABLE.read().unwrap()
    && *TALKING_PLACE.read().unwrap() == TalkingPlace::LivingRoom
  {
    format!(
      "\\0\\f[default]\\f[anchornotselectfontcolor,default.plain]\\_a[DerivativeTalkRequest,{}]{}\\_a\\f[anchornotselectfontcolor,default]\\_l[0,@1.5em]",
      talk.id,
      Icon::Bubble,
    )
  } else {
    String::new()
  };

  let derivative_talk_anchors = if let Some(dtalks) = derivative_talk_by_id(&talk.id) {
    let mut anchors = "\\1\\_q".to_string();
    for (i, dtalk) in dtalks.iter().enumerate() {
      if i > 0 {
        anchors += "\\_w[1em]";
      }
      anchors += &format!(
        "\\![*]\\_a[DerivativeTalk,{}]{}\\_a\\n",
        dtalk.id, dtalk.summary,
      );
    }
    anchors
  } else {
    String::new()
  };

  format!(
    "{}{}{}",
    derivative_talk_request_button,
    talk.consume(),
    derivative_talk_anchors,
  )
}

fn first_random_talk_response(
  text: String,
  i: usize,
  text_count: usize,
) -> Result<Response, ShioriError> {
  FLAGS
    .write()
    .unwrap()
    .done(EventFlag::FirstRandomTalkDone(i as u32));
  let m = if i == text_count - 1 {
    let achieved_talk_types = [TalkType::AboutMe, TalkType::WithYou];
    achieved_talk_types.iter().for_each(|t| {
      FLAGS.write().unwrap().done(EventFlag::TalkTypeUnlock(*t));
    });
    let achievements_messages = achieved_talk_types
      .iter()
      .map(|t| render_achievement_message(*t))
      .collect::<Vec<_>>();
    format!(
      "{}\\1\\c{}",
      text.clone(),
      &achievements_messages.join("\\n")
    )
  } else {
    text.clone()
  };
  let mut res =
    new_response_with_value_with_translate(m, TranslateOption::with_shadow_completion())?;
  res.headers.insert_by_header_name(
    HeaderName::from("Marker"),
    format!("{}({}/{})", FIRST_BOOT_MARKER, i + 2, text_count + 1),
  );
  Ok(res)
}

pub(crate) fn on_anchor_select_ex(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let anchor_type = refs[1]; // AnchorTalk || DerivativeTalk // DerivativeTalkRequest
  let id = refs[2];
  let user_dialog = refs.get(3).unwrap_or(&"").to_string();

  if *LAST_ANCHOR_ID.read().unwrap() == Some(id.to_string()) {
    return Ok(new_response_nocontent());
  }

  match anchor_type {
    "AnchorTalk" => anchor_talk_dialog(id, &user_dialog),
    "DerivativeTalk" => derivative_talk_dialog(id),
    "DerivativeTalkRequest" => derivative_talk_request_open(id),
    _ => Err(ShioriError::BadRequest),
  }
}

fn derivative_talk_dialog(id: &str) -> Result<Response, ShioriError> {
  match derivative_talks().iter().find(|t| t.id == id) {
    Some(talk) => {
      let mut m = String::from("\\C");
      m += &format!("\\1\\c\\_q{}\\n\\_q", talk.summary);
      m += "\\0\\n\\f[align,center]\\_q─\\w1──\\w1───\\w1─────\\w1────\\w1──\\w1──\\w1─\\w1─\\n";
      m += "\\_w[750]\\_q\\_l[@0,]";
      m += &talk.consume();
      let parent_talk = TalkType::all()
        .iter()
        .map(|t| {
          if let Some(talks) = random_talks(*t) {
            talks.iter().find(|t| t.id == talk.parent_id).cloned()
          } else {
            None
          }
        })
        .find(|t| t.is_some())
        .and_then(|t| t);
      if let Some(parent) = parent_talk {
        register_talk_collection(id, parent.talk_type.unwrap())?;
      }
      new_response_with_value_with_translate(m, TranslateOption::with_shadow_completion())
    }
    None => Ok(new_response_nocontent()),
  }
}

fn anchor_talk_dialog(id: &str, user_dialog: &str) -> Result<Response, ShioriError> {
  let mut m = String::from("\\C");
  m += "\\0\\n\\f[align,center]\\_q─\\w1──\\w1───\\w1─────\\w1────\\w1──\\w1──\\w1─\\w1─\\n\\_w[750]\\_q\\_l[@0,]";
  if !user_dialog.is_empty() {
    m += &format!("\\1『{}』\\_w[500]", user_dialog);
  }
  match anchor_talks(id) {
    Some(t) => {
      *LAST_ANCHOR_ID.write().unwrap() = Some(id.to_string());
      new_response_with_value_with_translate(m + &t, TranslateOption::with_shadow_completion())
    }
    None => Ok(new_response_nocontent()),
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::events::on_boot;
  use crate::events::on_close;
  use crate::events::on_minute_change;
  use crate::events::on_mouse_double_click;
  use crate::events::on_story_event;
  use crate::events::TALK_UNLOCK_COUNT_LORE;
  use crate::events::TALK_UNLOCK_COUNT_SERVANT;
  use crate::variables::PendingEvent;
  use crate::variables::INSERTER;
  use crate::variables::USER_NAME;
  use shiorust::message::Request;

  #[test]
  fn test_firstboot_flags() -> Result<(), Box<dyn std::error::Error>> {
    const FIRST_CLOSE_TALK_PART: &str = "生きたあなたと話していたい";
    const SECOND_CLOSE_TALK_PART: &str = "がありますように";
    const CLOSE_TALK_IN_LIBRARY_PART: &str = "ハイネはお茶を一口飲んだ";

    *USER_NAME.write().unwrap() = "test".to_string(); // 実際はOnNotifyUserInfoで設定される

    let mut headers = Headers::new();
    headers.insert_by_header_name(HeaderName::from("ID"), "OnSecondChange".to_string());

    let on_second_change_req = Request {
      method: Method::GET,
      version: Version::V20,
      headers,
    };

    let mut headers = Headers::new();
    headers.insert_by_header_name(HeaderName::from("ID"), "OnMouseDoubleClick".to_string());
    headers.insert_by_header_name(HeaderName::from("Reference0"), "0".to_string());
    headers.insert_by_header_name(HeaderName::from("Reference1"), "0".to_string());
    headers.insert_by_header_name(HeaderName::from("Reference2"), "0".to_string());
    headers.insert_by_header_name(HeaderName::from("Reference3"), "2".to_string());
    headers.insert_by_header_name(HeaderName::from("Reference4"), "candle".to_string());

    let on_mouse_double_click_req = Request {
      method: Method::GET,
      version: Version::V20,
      headers,
    };

    // テスト中は常に非アイドル状態
    *IDLE_SECONDS.write().unwrap() = IDLE_THRESHOLD - 1;

    // Inserterの初期化を別スレッドで開始(本来はloadで行われる)
    INSERTER.write().unwrap().start_init();
    // translatorの初期化を待つ
    while !INSERTER.read().unwrap().is_ready() {
      std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 初回起動時のフラグチェック
    assert!(!FLAGS.read().unwrap().check(&EventFlag::FirstBoot));
    on_boot(&on_second_change_req)?;
    assert!(FLAGS.read().unwrap().check(&EventFlag::FirstBoot));

    // 初回ランダムトークのフラグチェック
    assert!(!FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::TalkTypeUnlock(TalkType::AboutMe)));
    assert!(!FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::TalkTypeUnlock(TalkType::WithYou)));
    for i in 0..FIRST_RANDOMTALKS.len() {
      on_ai_talk(&on_second_change_req)?;
      assert!(FLAGS
        .read()
        .unwrap()
        .check(&EventFlag::FirstRandomTalkDone(i as u32)));
    }
    assert!(FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::TalkTypeUnlock(TalkType::AboutMe)));
    assert!(FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::TalkTypeUnlock(TalkType::WithYou)));

    // 初回没入度マックス時の場所変更
    assert!(!FLAGS.read().unwrap().check(&EventFlag::FirstPlaceChange));
    for _i in 0..IMMERSIVE_ICON_COUNT {
      on_mouse_double_click(&on_mouse_double_click_req)?;
    }
    assert!(FLAGS.read().unwrap().check(&EventFlag::FirstPlaceChange));

    // 初回終了時に独白モードだったときトークが特別なものになるかのテスト
    assert!(!FLAGS.read().unwrap().check(&EventFlag::FirstClose));
    let res = on_close(&on_second_change_req)?;
    let value = res
      .headers
      .get_by_header_name(&HeaderName::from("Value"))
      .ok_or("Failed to get value")?;
    assert!(value.contains(CLOSE_TALK_IN_LIBRARY_PART)); // 独白モード終了トークが含まれていることの確認
    assert!(value.contains(FIRST_CLOSE_TALK_PART)); // 初回終了トークが含まれていることの確認
    assert!(FLAGS.read().unwrap().check(&EventFlag::FirstClose));

    // 書斎から正しく戻れるかのテスト
    assert!(*TALKING_PLACE.read().unwrap() == TalkingPlace::Library);
    for _i in 0..IMMERSIVE_ICON_COUNT {
      on_mouse_double_click(&on_mouse_double_click_req)?;
    }
    assert!(*TALKING_PLACE.read().unwrap() == TalkingPlace::LivingRoom);

    // 従者関連トークの開放確認
    while *CUMULATIVE_TALK_COUNT.read().unwrap() < TALK_UNLOCK_COUNT_SERVANT {
      on_ai_talk(&on_second_change_req)?;
    }
    on_minute_change(&on_second_change_req);
    let story_event = if PENDING_EVENT_TALK.read().unwrap().is_some() {
      PENDING_EVENT_TALK.read().unwrap().clone().unwrap()
    } else {
      return Err("Failed to get story event".into());
    };
    assert_eq!(story_event, PendingEvent::UnlockingServantsComments);
    on_story_event(&make_story_event_request(story_event))?;
    assert!(FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Servant)));
    assert!(PENDING_EVENT_TALK.read().unwrap().is_none());

    // ロア関連トークの開放確認
    while *CUMULATIVE_TALK_COUNT.read().unwrap() < TALK_UNLOCK_COUNT_LORE {
      on_ai_talk(&on_second_change_req)?;
    }
    on_minute_change(&on_second_change_req);
    let story_event = if PENDING_EVENT_TALK.read().unwrap().is_some() {
      PENDING_EVENT_TALK.read().unwrap().clone().unwrap()
    } else {
      return Err("Failed to get story event".into());
    };
    assert_eq!(story_event, PendingEvent::UnlockingLoreTalks);
    on_story_event(&make_story_event_request(story_event))?;
    assert!(FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Lore)));
    assert!(PENDING_EVENT_TALK.read().unwrap().is_none());

    // 初回終了時に通常モードだったときのトークが再生されるかのテスト
    FLAGS.write().unwrap().delete(EventFlag::FirstClose);
    assert!(!FLAGS.read().unwrap().check(&EventFlag::FirstClose));
    let res = on_close(&on_second_change_req)?;
    let value = res
      .headers
      .get_by_header_name(&HeaderName::from("Value"))
      .ok_or("Failed to get value")?;
    assert!(value.contains(FIRST_CLOSE_TALK_PART)); // 初回終了トークが含まれていることの確認

    // 2回目以降の終了時トークが再生されることの確認
    let res = on_close(&on_second_change_req)?;
    let value = res
      .headers
      .get_by_header_name(&HeaderName::from("Value"))
      .ok_or("Failed to get value")?;
    assert!(value.contains(SECOND_CLOSE_TALK_PART)); // 2回目以降の終了トークが含まれていることの確認

    Ok(())
  }

  fn make_story_event_request(event: PendingEvent) -> Request {
    let mut headers = Headers::new();
    headers.insert_by_header_name(HeaderName::from("ID"), "OnStoryEvent".to_string());
    headers.insert_by_header_name(HeaderName::from("Reference0"), event.title().to_string());
    Request {
      method: Method::GET,
      version: Version::V20,
      headers,
    }
  }
}
