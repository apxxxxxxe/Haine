use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::randomtalk::{
  changing_place_talks, finishing_aroused_talks, RANDOMTALK_COMMENTS,
};
use crate::events::talk::anchor::anchor_talks;
use crate::events::talk::randomtalk::random_talks;
use crate::events::talk::{register_talk_collection, TalkType, TalkingPlace};
use crate::get_touch_info;
use crate::variables::{get_global_vars, EventFlag, IDLE_THRESHOLD};
use shiorust::message::{parts::HeaderName, Request, Response};

// トーク1回あたりに上昇する没入度
const IMMERSIVE_RATE: u32 = 8;

pub fn on_ai_talk(_req: &Request) -> Response {
  let vars = get_global_vars();
  let if_consume_talk_bias = vars.volatility.idle_seconds() < IDLE_THRESHOLD;

  vars
    .volatility
    .set_last_random_talk_time(vars.volatility.ghost_up_time());

  let text_count = FIRST_RANDOMTALKS.len();
  for (i, text) in FIRST_RANDOMTALKS.iter().enumerate() {
    if !vars
      .flags()
      .check(&EventFlag::FirstRandomTalkDone(i as u32))
    {
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
      let mut res = new_response_with_value(m, TranslateOption::with_shadow_completion());
      res.headers.insert_by_header_name(
        HeaderName::from("Marker"),
        format!("邂逅({}/{})", i + 2, text_count + 1),
      );
      return res;
    }
  }

  if vars.volatility.aroused() {
    vars.volatility.set_aroused(false);
    get_touch_info!("0headdoubleclick").reset();
    let talks = finishing_aroused_talks();
    let choosed_talk = talks[choose_one(&talks, if_consume_talk_bias).unwrap()].to_string();
    return new_response_with_value(choosed_talk, TranslateOption::with_shadow_completion());
  }

  // 没入度を上げる
  let immersive_degrees = std::cmp::min(vars.volatility.immersive_degrees() + IMMERSIVE_RATE, 100);

  vars.set_cumulative_talk_count(vars.cumulative_talk_count() + 1);

  if immersive_degrees >= 100 {
    let (previous_talking_place, current_talking_place) = match vars.volatility.talking_place() {
      TalkingPlace::LivingRoom => (TalkingPlace::LivingRoom, TalkingPlace::Library),
      TalkingPlace::Library => (TalkingPlace::Library, TalkingPlace::LivingRoom),
    };

    let messages = changing_place_talks(&previous_talking_place, &current_talking_place);

    vars.volatility.set_talking_place(current_talking_place);
    vars.volatility.set_immersive_degrees(0);

    return new_response_with_value(
      messages[choose_one(&messages, true).unwrap()].to_owned(),
      TranslateOption::with_shadow_completion(),
    );
  } else {
    vars.volatility.set_immersive_degrees(immersive_degrees);
  }

  let talks = vars
    .volatility
    .talking_place()
    .talk_types()
    .iter()
    .filter(|t| vars.flags().check(&EventFlag::TalkTypeUnlock(**t)))
    .flat_map(|t| random_talks(*t))
    .collect::<Vec<_>>();

  let choosed_talk = talks[choose_one(&talks, if_consume_talk_bias).unwrap()].clone();

  if if_consume_talk_bias {
    // ユーザが見ているときのみトークを消費する
    register_talk_collection(&choosed_talk);
  }

  let comment = RANDOMTALK_COMMENTS[choose_one(&RANDOMTALK_COMMENTS, false).unwrap()];
  new_response_with_value(
    format!(
      "\\0\\![set,balloonnum,{}]{}",
      comment,
      choosed_talk.consume()
    ),
    TranslateOption::with_shadow_completion(),
  )
}

pub fn on_anchor_select_ex(req: &Request) -> Response {
  let refs = get_references(req);
  let id = refs[1];
  let user_dialog = refs.get(2).unwrap_or(&"").to_string();

  let mut m = String::from("\\C");
  m += "\\0\\n\\f[align,center]\\_q─\\w1──\\w1───\\w1─────\\w1────\\w1──\\w1──\\w1─\\w1─\\n\\_w[750]\\_q\\_l[@0,]";
  if !user_dialog.is_empty() {
    m += &format!("\\1『{}』\\_w[500]", user_dialog);
  }
  match anchor_talks(id) {
    Some(t) => new_response_with_value(m + &t, TranslateOption::with_shadow_completion()),
    None => new_response_nocontent(),
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::variables::get_global_vars;
  use shiorust::message::parts::*;
  use shiorust::message::Request;
  use std::collections::HashMap;

  #[test]
  fn test_aitalk() -> Result<(), Box<dyn std::error::Error>> {
    let vars = get_global_vars();
    vars.load()?;
    vars.volatility.set_idle_seconds(1);

    let req = Request {
      method: Method::GET,
      version: Version::V20,
      headers: Headers::new(),
    };
    let mut results = HashMap::new();
    for _i in 0..100 {
      let res = on_ai_talk(&req);
      let value = res.headers.get(&HeaderName::from("Value")).unwrap();
      let md5 = format!("{:x}", md5::compute(value.as_bytes()));
      let n = results.get(&md5).unwrap_or(&0);
      results.insert(md5, n + 1);
    }
    for (k, v) in results.iter() {
      println!("{}: {}", k, v);
    }
    Ok(())
  }
}
