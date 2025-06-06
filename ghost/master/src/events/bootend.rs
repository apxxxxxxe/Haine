use crate::error::ShioriError;
use crate::events::check_story_events;
use crate::events::common::*;
use crate::events::first_boot::{
  FIRST_BOOT_MARKER, FIRST_BOOT_TALK, FIRST_CLOSE_TALK, FIRST_RANDOMTALKS,
};
use crate::events::TalkingPlace;
use crate::variables::*;
use chrono::Timelike;
use rand::seq::SliceRandom;
use shiorust::message::{parts::HeaderName, Response, *};

pub(crate) fn on_boot(_req: &Request) -> Result<Response, ShioriError> {
  *TOTAL_BOOT_COUNT.write().unwrap() += 1;

  // 初回起動
  if !FLAGS.read().unwrap().check(&EventFlag::FirstBoot) {
    FLAGS.write().unwrap().done(EventFlag::FirstBoot);
    let mut res = new_response_with_value_with_translate(
      FIRST_BOOT_TALK.to_string(),
      TranslateOption::simple_translate(),
    )?;
    res.headers.insert_by_header_name(
      HeaderName::from("Marker"),
      format!("{}(1/{})", FIRST_BOOT_MARKER, FIRST_RANDOMTALKS.len() + 1),
    );
    return Ok(res);
  }

  check_story_events();

  let talks = all_combo(&vec![
    vec![render_immersive_icon()],
    vec!["h1113105\\1今日も、霧が濃い。".to_string()],
    vec![format!(
      "\
      h1113105……h1113101\\_w[300]h1113201あら。\\n\
      h1111204{}、{{user_name}}。\
      ",
      {
        let hour = chrono::Local::now().hour();
        if hour <= 3 || hour >= 19 {
          "こんばんは"
        } else if hour < 11 {
          "おはよう"
        } else {
          "こんにちは"
        }
      }
    )],
  ]);
  let index = choose_one(&talks, false).ok_or(ShioriError::ArrayAccessError)?;
  let v = format!(
    "\\0\\s[{}]{}\\![embed,OnStickSurface]{}{}",
    TRANSPARENT_SURFACE,
    RESET_BINDS,
    randomize_underwear(),
    talks[index],
  );
  new_response_with_value_with_translate(v, TranslateOption::simple_translate())
}

pub(crate) fn on_close(_req: &Request) -> Result<Response, ShioriError> {
  let mut parts = vec![vec![RESET_BINDS.to_string()]];

  if *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
    parts.push(vec![format!(
      "\\0\\b[{}]h1111705……。\
      \\1ネ……\\n\
      イネ……。\
      \\0\\b[{}]hr1141112φ！\
      \\1\\nハイネ！\
      \\0…………\\n\\n\
      h1111101……h1111204あら、{{user_name}}。\\n\
      \\1\\n\\n……戻ってきたようだ。\\n\
      \\0h1111210……そう、今日はおしまいにするのね。\\n\\n\\1\\b[-1]",
      TalkingPlace::Library.balloon_surface(),
      TalkingPlace::LivingRoom.balloon_surface(),
    )]);
  }
  if !FLAGS.read().unwrap().check(&EventFlag::FirstClose) {
    FLAGS.write().unwrap().done(EventFlag::FirstClose);
    parts.push(vec![FIRST_CLOSE_TALK.to_string()]);
  } else {
    parts.extend(vec![
      vec!["h1111210".to_string(), "h1111211".to_string()],
      vec!["あなたに".to_string()],
      vec![
        "すばらしき朝".to_string(),
        "蜜のようなまどろみ".to_string(),
        "暗くて静かな安らぎ".to_string(),
        "良き終わり".to_string(),
        "孤独と救い".to_string(),
      ],
      vec!["がありますように。\\nh1111204またね、{user_name}。\\_w[1200]".to_string()],
    ]);
  }
  let talks = all_combo(&parts);
  let index = choose_one(&talks, true).ok_or(ShioriError::ArrayAccessError)?;
  new_response_with_value_with_translate(
    format!("{}{}\\-", RESET_BINDS, talks[index].clone()),
    TranslateOption::simple_translate(),
  )
}

pub(crate) fn on_vanish_selecting(_req: &Request) -> Response {
  let m = "\\1※Vanishイベントは未実装です。".to_string();
  new_response_with_value_with_notranslate(m, TranslateOption::none())
}

fn randomize_underwear() -> String {
  let mut rng = rand::thread_rng();
  let candidates = ["A", "B"];
  format!(
    "\\0\\![bind,下着,{},1]",
    candidates.choose(&mut rng).unwrap()
  )
}
