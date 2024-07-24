use crate::error::ShioriError;
use crate::events::common::*;
use crate::events::randomtalk::{
  finishing_aroused_talks, moving_to_library_talk, moving_to_living_room_talk,
  RANDOMTALK_COMMENTS_LIVING_ROOM,
};
use crate::events::talk::anchor::anchor_talks;
use crate::events::talk::randomtalk::random_talks;
use crate::events::talk::{register_talk_collection, TalkType, TalkingPlace};
use crate::events::{
  first_boot::{FIRST_BOOT_MARKER, FIRST_RANDOMTALKS},
  randomtalk::RANDOMTALK_COMMENTS_LIBRARY_ACTIVE,
  randomtalk::RANDOMTALK_COMMENTS_LIBRARY_INACTIVE,
};
use crate::get_touch_info;
use crate::variables::{get_global_vars, EventFlag, IDLE_THRESHOLD};
use shiorust::message::{parts::*, traits::*, Request, Response};

// トーク1回あたりに上昇する没入度の割合(%)
pub const IMMERSIVE_RATE: u32 = 5;
pub const IMMERSIVE_RATE_MAX: u32 = 100;

pub fn on_ai_talk(req: &Request) -> Result<Response, ShioriError> {
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

  // 興奮状態の終了
  if vars.volatility.aroused() {
    vars.volatility.set_aroused(false);
    get_touch_info!("0headdoubleclick").reset();
    let talks = finishing_aroused_talks();
    let index = choose_one(&talks, if_consume_talk_bias).ok_or(ShioriError::TalkNotFound)?;
    let choosed_talk = talks[index].to_string();
    return new_response_with_value_with_translate(
      choosed_talk,
      TranslateOption::with_shadow_completion(),
    );
  }

  if vars.volatility.talking_place() == TalkingPlace::LivingRoom {
    // 居間にいるときは没入度を上げる
    add_immsersive_degree(IMMERSIVE_RATE);
    // 没入度が最大に達したら書斎に移動
    if vars.volatility.immersive_degrees() == IMMERSIVE_RATE_MAX {
      vars.volatility.set_talking_place(TalkingPlace::Library);

      let messages = moving_to_library_talk()?;
      let index = choose_one(&messages, true).ok_or(ShioriError::TalkNotFound)?;
      return new_response_with_value_with_translate(
        messages[index].to_owned(),
        TranslateOption::with_shadow_completion(),
      );
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

  let comment = if vars.volatility.talking_place() == TalkingPlace::Library {
    // 書斎では能動的に話しかけた場合のみ没入度低下&コメントを表示
    if req.headers.get("ID").is_some_and(|v| v == "OnSecondChange") {
      let index = choose_one(&RANDOMTALK_COMMENTS_LIBRARY_INACTIVE, false)
        .ok_or(ShioriError::TalkNotFound)?;
      RANDOMTALK_COMMENTS_LIBRARY_INACTIVE[index]
    } else {
      sub_immsersive_degree(IMMERSIVE_RATE);

      // 書斎で没入度が0になったら居間に移動
      if vars.volatility.immersive_degrees() == 0 {
        vars.volatility.set_talking_place(TalkingPlace::LivingRoom);

        let messages = moving_to_living_room_talk()?;
        let index = choose_one(&messages, true).ok_or(ShioriError::TalkNotFound)?;
        return new_response_with_value_with_translate(
          messages[index].to_owned(),
          TranslateOption::with_shadow_completion(),
        );
      }

      let index =
        choose_one(&RANDOMTALK_COMMENTS_LIBRARY_ACTIVE, false).ok_or(ShioriError::TalkNotFound)?;
      RANDOMTALK_COMMENTS_LIBRARY_ACTIVE[index]
    }
  } else if vars
    .flags()
    .check(&EventFlag::TalkTypeUnlock(TalkType::Servant))
  {
    // 居間では従者トーク解禁済みの場合コメントを表示
    let index =
      choose_one(&RANDOMTALK_COMMENTS_LIVING_ROOM, false).ok_or(ShioriError::TalkNotFound)?;
    RANDOMTALK_COMMENTS_LIVING_ROOM[index]
  } else {
    ""
  };

  new_response_with_value_with_translate(
    format!(
      "\\0\\![set,balloonnum,{}]{}",
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

pub fn on_anchor_select_ex(req: &Request) -> Result<Response, ShioriError> {
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
  use crate::events::talk::randomtalk::{
    random_talks, TALK_ID_LORE_INTRO, TALK_ID_SERVANT_INTRO, TALK_UNLOCK_COUNT_LORE,
    TALK_UNLOCK_COUNT_SERVANT,
  };
  use crate::variables::{get_global_vars, GlobalVariables};
  use shiorust::message::Request;

  #[test]
  fn test_firstboot_flags() -> Result<(), Box<dyn std::error::Error>> {
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
    headers.insert("ID", "OnKeyPress".to_string());

    let on_key_press_req = Request {
      method: Method::GET,
      version: Version::V20,
      headers,
    };

    // テスト中は常に非アイドル状態
    vars.volatility.set_idle_seconds(IDLE_THRESHOLD - 1);

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
    let required_talk_count = IMMERSIVE_RATE_MAX / IMMERSIVE_RATE;
    assert!(!vars.flags().check(&EventFlag::FirstPlaceChange));
    for _i in 0..required_talk_count {
      on_ai_talk(&on_second_change_req)?;
    }
    assert!(vars.flags().check(&EventFlag::FirstPlaceChange));

    // 書斎から正しく戻れるかのテスト
    assert_eq!(vars.volatility.talking_place(), TalkingPlace::Library);
    for _i in 0..required_talk_count {
      on_ai_talk(&on_second_change_req)?; // 自動ランダムトークでは没入度が下がらない
    }
    assert_eq!(vars.volatility.talking_place(), TalkingPlace::Library);
    for _i in 0..required_talk_count {
      on_ai_talk(&on_key_press_req)?; // 手動ランダムトークでは没入度が下がる
    }
    assert_eq!(vars.volatility.talking_place(), TalkingPlace::LivingRoom);

    // 従者関連トークの開放確認
    while vars.cumulative_talk_count() < TALK_UNLOCK_COUNT_SERVANT {
      on_ai_talk(&on_second_change_req)?;
    }
    let r = random_talks(TalkType::SelfIntroduce).ok_or(Box::new(ShioriError::TalkNotFound))?;
    let unlock_talk_servant = r.iter().find(|t| t.id == TALK_ID_SERVANT_INTRO);
    if unlock_talk_servant.is_none() {
      return Err("Failed to find unlock talk for servant".into());
    }
    unlock_talk_servant.unwrap().consume();
    assert!(vars
      .flags()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Servant)));

    // ロア関連トークの開放確認
    while vars.cumulative_talk_count() < TALK_UNLOCK_COUNT_LORE {
      on_ai_talk(&on_second_change_req)?;
    }
    let r = random_talks(TalkType::SelfIntroduce).ok_or(Box::new(ShioriError::TalkNotFound))?;
    let unlock_talk_lore = r.iter().find(|t| t.id == TALK_ID_LORE_INTRO);
    if unlock_talk_lore.is_none() {
      return Err("Failed to find unlock talk for lore".into());
    }
    unlock_talk_lore.unwrap().consume();
    assert!(vars
      .flags()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Lore)));

    Ok(())
  }
}
