use crate::error::ShioriError;
use crate::events::aitalk::on_ai_talk;
use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::talk::random_talks_analysis;
use crate::variables::{get_global_vars, EventFlag, GlobalVariables, GHOST_NAME};
use shiorust::message::{Request, Response};

pub fn on_key_press(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  match refs[0] {
    "r" => Ok(new_response_with_value_with_notranslate(
      "unload:10秒後にリロード\\![unload,shiori]\\_w[10000]\\![reload,ghost]".to_string(),
      TranslateOption::balloon_surface_only(),
    )),
    "t" => {
      let vars = get_global_vars();
      if !vars.flags().check(&EventFlag::FirstRandomTalkDone(
        FIRST_RANDOMTALKS.len() as u32 - 1,
      )) {
        Ok(new_response_nocontent())
      } else if vars.volatility.aroused() {
        Ok(new_response_with_value_with_notranslate(
          "\\![raise,OnMouseDoubleClick,dummy,dummy,dummy,0,head]".to_string(),
          TranslateOption::none(),
        ))
      } else {
        on_ai_talk(req)
      }
    }
    "c" => Ok(new_response_with_value_with_notranslate(
      random_talks_analysis(),
      TranslateOption::balloon_surface_only(),
    )),
    "d" => {
      let vars = get_global_vars();
      *vars = GlobalVariables::new();
      Ok(new_response_with_value_with_notranslate(
        format!("\\![change,ghost,{}]", GHOST_NAME),
        TranslateOption::none(),
      ))
    }
    _ => Ok(new_response_nocontent()),
  }
}
