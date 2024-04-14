use crate::events::aitalk::{on_ai_talk, random_talks_analysis};
use crate::events::common::*;
use crate::variables::{get_global_vars, EventFlag};
use rand::prelude::*;
use shiorust::message::{Request, Response};

pub fn on_key_press(req: &Request) -> Response {
  let refs = get_references(req);
  match refs[0] {
    "r" => new_response_with_value(
      "unload:10秒後にリロード\\![unload,shiori]\\_w[10000]\\![reload,ghost]".to_string(),
      TranslateOption::balloon_surface_only(),
    ),
    "t" => on_ai_talk(req),
    "c" => new_response_with_value(
      random_talks_analysis(),
      TranslateOption::balloon_surface_only(),
    ),
    "d" => {
      let vars = get_global_vars();
      vars.flags_mut().delete(EventFlag::FirstRandomTalkDone(0));
      vars.flags_mut().delete(EventFlag::FirstRandomTalkDone(1));
      on_ai_talk(req)
    }
    "e" => {
      let mut rng = rand::thread_rng();
      let a = 1;
      let b = rng.gen_range(1..=3);
      let c = rng.gen_range(1..=4);
      let d = rng.gen_range(1..=4);
      let e = rng.gen_range(1..=8);
      let f = rng.gen_range(1..=12);
      let m = format!("{}{}{}{}{}{:02}", a, b, c, d, e, f);
      new_response_with_value(
        format!("\\_qh{}{}", m, m),
        TranslateOption::simple_translate(),
      )
    }
    _ => new_response_nocontent(),
  }
}
