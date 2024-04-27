use crate::events::aitalk::{on_ai_talk, random_talks_analysis, FIRST_RANDOMTALKS};
use crate::events::common::*;
use crate::variables::{get_global_vars, EventFlag, GHOST_NAME};
use shiorust::message::{Request, Response};

pub fn on_key_press(req: &Request) -> Response {
  let refs = get_references(req);
  match refs[0] {
    "r" => new_response_with_value(
      "unload:10秒後にリロード\\![unload,shiori]\\_w[10000]\\![reload,ghost]".to_string(),
      TranslateOption::balloon_surface_only(),
    ),
    "t" => {
      let vars = get_global_vars();
      if !vars.flags().check(&EventFlag::FirstRandomTalkDone(
        FIRST_RANDOMTALKS.len() as u32 - 1,
      )) {
        new_response_nocontent()
      } else if vars.volatility.aroused() {
        new_response_with_value(
          "\\![raise,OnMouseDoubleClick,dummy,dummy,dummy,0,head]".to_string(),
          TranslateOption::none(),
        )
      } else {
        on_ai_talk(req)
      }
    }
    "c" => new_response_with_value(
      random_talks_analysis(),
      TranslateOption::balloon_surface_only(),
    ),
    "d" => {
      let vars = get_global_vars();
      vars.flags_mut().delete(EventFlag::FirstBoot);
      vars.flags_mut().delete(EventFlag::FirstRandomTalkDone(0));
      vars.flags_mut().delete(EventFlag::FirstRandomTalkDone(1));
      vars.flags_mut().delete(EventFlag::FirstClose);
      vars.flags_mut().delete(EventFlag::FirstHitTalkDone);
      new_response_with_value(
        format!("\\![change,ghost,{}]", GHOST_NAME),
        TranslateOption::none(),
      )
    }
    _ => new_response_nocontent(),
  }
}
