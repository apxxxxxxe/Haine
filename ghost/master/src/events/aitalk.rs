use crate::error::ShioriError;
use crate::events::common::*;
use crate::events::randomtalk::RANDOMTALK_COMMENTS_LIVING_ROOM;
use crate::events::talk::anchor::anchor_talks;
use crate::events::talk::randomtalk::random_talks;
use crate::events::talk::{register_talk_collection, TalkType, TalkingPlace};
use crate::events::BodyPart;
use crate::events::{
  first_boot::{FIRST_BOOT_MARKER, FIRST_RANDOMTALKS},
  randomtalk::RANDOMTALK_COMMENTS_LIBRARY_ACTIVE,
  randomtalk::RANDOMTALK_COMMENTS_LIBRARY_INACTIVE,
};
use crate::variables::{get_global_vars, EventFlag, IDLE_THRESHOLD};
use shiorust::message::{parts::*, traits::*, Request, Response};

pub(crate) const IMMERSIVE_RATE_MAX: u32 = 100;
// トーク1回あたりに増減する没入度の割合(%)
pub(crate) const IMMERSIVE_RATE: u32 = 5;

pub(crate) const IMMERSIVE_ICON_COUNT: u32 = 5;

pub(crate) fn on_ai_talk(req: &Request) -> Result<Response, ShioriError> {
  let vars = get_global_vars();
  let if_consume_talk_bias = vars.volatility.idle_seconds() < IDLE_THRESHOLD;
  vars
    .volatility
    .set_last_random_talk_time(vars.volatility.ghost_up_time());

  // 初回ランダムトーク
  let text_count = FIRST_RANDOMTALKS.len();
  for (i, text) in FIRST_RANDOMTALKS.iter().enumerate() {
    if !vars
      .flags()
      .check(&EventFlag::FirstRandomTalkDone(i as u32))
    {
      return first_random_talk_response(text.to_string(), i, text_count);
    }
  }

  // 通常ランダムトーク
  let talk_types = vars.volatility.talking_place().talk_types();
  let talk_lists = talk_types
    .iter()
    .filter(|t| vars.flags().check(&EventFlag::TalkTypeUnlock(**t)))
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
    register_talk_collection(&choosed_talk)?;
    vars.set_cumulative_talk_count(vars.cumulative_talk_count() + 1);
  }

  // バルーン右下に表示するコメントを取得
  let comment = if vars.volatility.talking_place() == TalkingPlace::Library {
    // 書斎では能動的に話しかけたかどうかで異なるコメントを表示
    if let Some(id) = req.headers.get("ID") {
      match id.as_str() {
        "OnSecondChange" => {
          let index = choose_one(&RANDOMTALK_COMMENTS_LIBRARY_INACTIVE, false)
            .ok_or(ShioriError::TalkNotFound)?;
          RANDOMTALK_COMMENTS_LIBRARY_INACTIVE[index].to_string()
        }
        "OnMouseDoubleClick" | "OnMouseMove" | "OnMouseWheel" | "OnMouseClickEx" => {
          let refs = get_references(req);
          if let Some(r) = refs.get(4) {
            let part = BodyPart::from_str(r).ok_or(ShioriError::TalkNotFound)?;
            format!("{}に触れても、反応はない。", part)
          } else {
            "".to_string()
          }
        }
        "OnKeyPress" => {
          let index = choose_one(&RANDOMTALK_COMMENTS_LIBRARY_ACTIVE, false)
            .ok_or(ShioriError::TalkNotFound)?;
          RANDOMTALK_COMMENTS_LIBRARY_ACTIVE[index].to_string()
        }
        _ => "".to_string(),
      }
    } else {
      "".to_string()
    }
  } else {
    // 居間では従者トーク解禁済みの場合コメントを表示
    if vars
      .flags()
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
  if vars.volatility.talking_place() == TalkingPlace::LivingRoom {
    vars.volatility.set_immersive_degrees(
      vars
        .volatility
        .immersive_degrees()
        .saturating_sub(IMMERSIVE_RATE),
    );
  } else {
    let new_rate = vars.volatility.immersive_degrees() + IMMERSIVE_RATE;
    vars
      .volatility
      .set_immersive_degrees(new_rate.min(IMMERSIVE_RATE_MAX));
  }

  new_response_with_value_with_translate(
    format!(
      "\\0{}\\![set,balloonnum,{}]{}",
      render_immersive_icon(),
      comment,
      choosed_talk.consume()
    ),
    TranslateOption::with_shadow_completion(),
  )
}

fn first_random_talk_response(
  text: String,
  i: usize,
  text_count: usize,
) -> Result<Response, ShioriError> {
  let vars = get_global_vars();
  vars
    .flags_mut()
    .done(EventFlag::FirstRandomTalkDone(i as u32));
  let m = if i == text_count - 1 {
    let achieved_talk_types = [TalkType::SelfIntroduce, TalkType::WithYou];
    achieved_talk_types.iter().for_each(|t| {
      vars.flags_mut().done(EventFlag::TalkTypeUnlock(*t));
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
  let vars = get_global_vars();
  let refs = get_references(req);
  let id = refs[1];
  let user_dialog = refs.get(2).unwrap_or(&"").to_string();

  if vars.volatility.last_anchor_id() == Some(id.to_string()) {
    return Ok(new_response_nocontent());
  }

  let mut m = String::from("\\C");
  m += "\\0\\n\\f[align,center]\\_q─\\w1──\\w1───\\w1─────\\w1────\\w1──\\w1──\\w1─\\w1─\\n\\_w[750]\\_q\\_l[@0,]";
  if !user_dialog.is_empty() {
    m += &format!("\\1『{}』\\_w[500]", user_dialog);
  }
  match anchor_talks(id) {
    Some(t) => {
      vars.volatility.set_last_anchor_id(Some(id.to_string()));
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
  use crate::variables::{get_global_vars, GlobalVariables};
  use shiorust::message::Request;

  #[test]
  fn test_firstboot_flags() -> Result<(), Box<dyn std::error::Error>> {
    const FIRST_CLOSE_TALK_PART: &str = "生きたあなたと話していたい";
    const SECOND_CLOSE_TALK_PART: &str = "がありますように";
    const CLOSE_TALK_IN_LIBRARY_PART: &str = "戻ってきたようだ。";

    let vars = get_global_vars();
    *vars = GlobalVariables::new();
    vars.set_user_name(Some("test".to_string())); // 実際はOnNotifyUserInfoで設定される

    let mut headers = Headers::new();
    headers.insert("ID", "OnSecondChange".to_string());

    let on_second_change_req = Request {
      method: Method::GET,
      version: Version::V20,
      headers,
    };

    let mut headers = Headers::new();
    headers.insert("ID", "OnMouseDoubleClick".to_string());
    headers.insert("Reference0", "0".to_string());
    headers.insert("Reference1", "0".to_string());
    headers.insert("Reference2", "0".to_string());
    headers.insert("Reference3", "2".to_string());
    headers.insert("Reference4", "candle".to_string());

    let on_mouse_double_click_req = Request {
      method: Method::GET,
      version: Version::V20,
      headers,
    };

    // テスト中は常に非アイドル状態
    vars.volatility.set_idle_seconds(IDLE_THRESHOLD - 1);

    // translatorの初期化を待つ
    while !vars.volatility.inserter_mut().is_ready() {
      std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 初回起動時のフラグチェック
    assert!(!vars.flags().check(&EventFlag::FirstBoot));
    on_boot(&on_second_change_req)?;
    assert!(vars.flags().check(&EventFlag::FirstBoot));

    // 初回ランダムトークのフラグチェック
    assert!(!vars
      .flags()
      .check(&EventFlag::TalkTypeUnlock(TalkType::SelfIntroduce)));
    assert!(!vars
      .flags()
      .check(&EventFlag::TalkTypeUnlock(TalkType::WithYou)));
    for i in 0..FIRST_RANDOMTALKS.len() {
      on_ai_talk(&on_second_change_req)?;
      assert!(vars
        .flags()
        .check(&EventFlag::FirstRandomTalkDone(i as u32)));
    }
    assert!(vars
      .flags()
      .check(&EventFlag::TalkTypeUnlock(TalkType::SelfIntroduce)));
    assert!(vars
      .flags()
      .check(&EventFlag::TalkTypeUnlock(TalkType::WithYou)));

    // 初回没入度マックス時の場所変更
    assert!(!vars.flags().check(&EventFlag::FirstPlaceChange));
    for _i in 0..IMMERSIVE_ICON_COUNT {
      on_mouse_double_click(&on_mouse_double_click_req)?;
    }
    assert!(vars.flags().check(&EventFlag::FirstPlaceChange));

    // 初回終了時に独白モードだったときトークが特別なものになるかのテスト
    assert!(!vars.flags().check(&EventFlag::FirstClose));
    let res = on_close(&on_second_change_req)?;
    let value = res.headers.get("Value").ok_or("Failed to get value")?;
    assert!(value.contains(CLOSE_TALK_IN_LIBRARY_PART)); // 独白モード終了トークが含まれていることの確認
    assert!(value.contains(FIRST_CLOSE_TALK_PART)); // 初回終了トークが含まれていることの確認
    assert!(vars.flags().check(&EventFlag::FirstClose));

    // 書斎から正しく戻れるかのテスト
    assert_eq!(vars.volatility.talking_place(), TalkingPlace::Library);
    for _i in 0..IMMERSIVE_ICON_COUNT {
      on_mouse_double_click(&on_mouse_double_click_req)?;
    }
    assert_eq!(vars.volatility.talking_place(), TalkingPlace::LivingRoom);

    // 従者関連トークの開放確認
    while vars.cumulative_talk_count() < TALK_UNLOCK_COUNT_SERVANT {
      on_ai_talk(&on_second_change_req)?;
    }
    on_minute_change(&on_second_change_req);
    let story_event = vars
      .pending_event_talk()
      .ok_or("Failed to get story event")?;
    assert_eq!(story_event, PendingEvent::UnlockingServantsComments);
    on_story_event(&make_story_event_request(story_event))?;
    assert!(vars
      .flags()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Servant)));
    let story_event = vars.pending_event_talk();
    assert!(story_event.is_none());

    // ロア関連トークの開放確認
    while vars.cumulative_talk_count() < TALK_UNLOCK_COUNT_LORE {
      on_ai_talk(&on_second_change_req)?;
    }
    on_minute_change(&on_second_change_req);
    let story_event = vars
      .pending_event_talk()
      .ok_or("Failed to get story event")?;
    assert_eq!(story_event, PendingEvent::UnlockingLoreTalks);
    on_story_event(&make_story_event_request(story_event))?;
    assert!(vars
      .flags()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Lore)));
    let story_event = vars.pending_event_talk();
    assert!(story_event.is_none());

    // 初回終了時に通常モードだったときのトークが再生されるかのテスト
    vars.flags_mut().delete(EventFlag::FirstClose);
    assert!(!vars.flags().check(&EventFlag::FirstClose));
    let res = on_close(&on_second_change_req)?;
    let value = res.headers.get("Value").ok_or("Failed to get value")?;
    assert!(value.contains(FIRST_CLOSE_TALK_PART)); // 初回終了トークが含まれていることの確認

    // 2回目以降の終了時トークが再生されることの確認
    let res = on_close(&on_second_change_req)?;
    let value = res.headers.get("Value").ok_or("Failed to get value")?;
    assert!(value.contains(SECOND_CLOSE_TALK_PART)); // 2回目以降の終了トークが含まれていることの確認

    Ok(())
  }

  fn make_story_event_request(event: PendingEvent) -> Request {
    let mut headers = Headers::new();
    headers.insert("ID", "OnStoryEvent".to_string());
    headers.insert("Reference0", event.title().to_string());
    Request {
      method: Method::GET,
      version: Version::V20,
      headers,
    }
  }
}
