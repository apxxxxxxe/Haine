use crate::events::aitalk::on_ai_talk;
use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::talk::random_talks_analysis;
use crate::variables::{get_global_vars, EventFlag, GlobalVariables, GHOST_NAME};
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
      *vars = GlobalVariables::new();
      new_response_with_value(
        format!("\\![change,ghost,{}]", GHOST_NAME),
        TranslateOption::none(),
      )
    }
    _ => new_response_nocontent(),
  }
}
