use crate::error::ShioriError;
use crate::events::common::*;
use crate::events::first_boot::{
  FIRST_BOOT_MARKER, FIRST_BOOT_TALK, FIRST_CLOSE_TALK, FIRST_RANDOMTALKS,
};
use crate::events::TalkType;
use crate::events::TalkingPlace;
use crate::events::IMMERSIVE_RATE_MAX;
use crate::variables::{get_global_vars, EventFlag, TRANSPARENT_SURFACE};
use chrono::Timelike;
use rand::seq::SliceRandom;
use shiorust::message::{parts::HeaderName, Response, *};

pub const UNLOCK_PAST_BOOT_COUNT: u64 = 3;

pub fn on_boot(_req: &Request) -> Result<Response, ShioriError> {
  let vars = get_global_vars();
  vars.set_total_boot_count(vars.total_boot_count() + 1);

  // 初回起動
  if !vars.flags().check(&EventFlag::FirstBoot) {
    vars.flags_mut().done(EventFlag::FirstBoot);
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

  // 情報解禁&過去トーク開放
  if !vars
    .flags()
    .check(&EventFlag::TalkTypeUnlock(super::TalkType::Past))
    && vars.total_boot_count() >= UNLOCK_PAST_BOOT_COUNT
    && vars.flags().check(&EventFlag::FirstPlaceChange)
  {
    vars.volatility.set_immersive_degrees(IMMERSIVE_RATE_MAX);
    vars.volatility.set_talking_place(TalkingPlace::Library);
    vars
      .flags_mut()
      .done(EventFlag::TalkTypeUnlock(TalkType::Past));
    let achievements_message = render_achievement_message(TalkType::Past);
    let m = format!(
      "\
      h1111105\\b[{}]……。\\1ハイネ……？\\n\
      ……いつもの出迎えがないのを不思議に思っていたのだが、\\n\
      彼女が思索に耽っているときに来てしまったようだ。\\n\
      ……しばらくそっとしておこう……。\
      \\0\\c\\1\\b[-1]h1000000───────────────\\_w[1200]\\c\
      h1111110\\b[{}]…………幽霊にとって、自身の死の記憶はある種のタブー。\\n\
      誰もが持つがゆえの共通認識。自身の死は恥部。\\n\
      私も、彼らのそれには深く踏み込まない。\\n\
      けれど、あの子は生者だから。\\n\
      \\n\
      ……私の死因が自殺であると。\\n\
      この家で死に、そしてここに縛り付けられたと、\\n\
      打ち明けてもよいのかもしれない。\\n\
      {}",
      TalkingPlace::LivingRoom.balloon_surface(),
      TalkingPlace::Library.balloon_surface(),
      achievements_message
    );
    return new_response_with_value_with_translate(m, TranslateOption::simple_translate());
  }

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

pub fn on_close(_req: &Request) -> Result<Response, ShioriError> {
  let vars = get_global_vars();
  let mut parts = vec![vec![RESET_BINDS.to_string()]];
  let is_immersing = vars.volatility.talking_place() == TalkingPlace::Library;

  if is_immersing {
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
  if !vars.flags().check(&EventFlag::FirstClose) {
    vars.flags_mut().done(EventFlag::FirstClose);
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

// FIXME: 実装予定
pub fn on_vanish_selected(_req: &Request) -> Response {
  new_response_nocontent()
}

fn randomize_underwear() -> String {
  let mut rng = rand::thread_rng();
  let candidates = ["A", "B"];
  format!(
    "\\0\\![bind,下着,{},1]",
    candidates.choose(&mut rng).unwrap()
  )
}
