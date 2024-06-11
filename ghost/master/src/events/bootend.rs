use crate::events::common::*;
use crate::events::first_boot::{
  FIRST_BOOT_MARKER, FIRST_BOOT_TALK, FIRST_CLOSE_TALK, FIRST_RANDOMTALKS,
};
use crate::variables::{get_global_vars, EventFlag, TRANSPARENT_SURFACE};
use rand::seq::SliceRandom;
use shiorust::message::{parts::HeaderName, Response, *};

pub fn on_boot(_req: &Request) -> Response {
  let vars = get_global_vars();
  vars.set_total_boot_count(vars.total_boot_count() + 1);
  if !vars.flags().check(&EventFlag::FirstBoot) {
    vars.flags_mut().done(EventFlag::FirstBoot);
    let mut res = new_response_with_value(
      FIRST_BOOT_TALK.to_string(),
      TranslateOption::simple_translate(),
    );
    res.headers.insert_by_header_name(
      HeaderName::from("Marker"),
      format!("{}(1/{})", FIRST_BOOT_MARKER, FIRST_RANDOMTALKS.len() + 1),
    );
    res
  } else {
    let talks = all_combo(&vec![
      vec!["h1113105\\1今日も、霧が濃い。".to_string()],
      vec!["\
      h1113105……h1113101\\_w[300]h1113201あら。\\n\
      h1111204いらっしゃい、{user_name}。\
      "
      .to_string()],
    ]);
    let v = format!(
      "\\0\\s[{}]{}\\![embed,OnStickSurface]{}{}",
      TRANSPARENT_SURFACE,
      RESET_BINDS,
      randomize_underwear(),
      talks[choose_one(&talks, false).unwrap()],
    );
    new_response_with_value(v, TranslateOption::simple_translate())
  }
}

pub fn on_close(_req: &Request) -> Response {
  let vars = get_global_vars();
  if !vars.flags().check(&EventFlag::FirstClose) {
    vars.flags_mut().done(EventFlag::FirstClose);
    return new_response_with_value(
      FIRST_CLOSE_TALK.to_string(),
      TranslateOption::simple_translate(),
    );
  }
  let talks = all_combo(&vec![
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
  new_response_with_value(
    format!(
      "{}{}\\-",
      RESET_BINDS,
      talks[choose_one(&talks, true).unwrap()].clone()
    ),
    TranslateOption::simple_translate(),
  )
}

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
