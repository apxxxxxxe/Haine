use crate::error::ShioriError;
use crate::events::aitalk::on_ai_talk;
use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::talk::random_talks_analysis;
use crate::variables::{get_global_vars, EventFlag, GlobalVariables, GHOST_NAME};
use shiorust::message::{Request, Response};

pub(crate) fn on_key_press(req: &Request) -> Result<Response, ShioriError> {
  let vars = get_global_vars();
  let refs = get_references(req);
  match refs[0] {
    "t" => {
      if !vars.flags().check(&EventFlag::FirstRandomTalkDone(
        FIRST_RANDOMTALKS.len() as u32 - 1,
      )) {
        Ok(new_response_nocontent())
      } else {
        on_ai_talk(req)
      }
    }
    "c" => {
      if vars.volatility.debug_mode() {
        Ok(new_response_with_value_with_notranslate(
          random_talks_analysis(),
          TranslateOption::balloon_surface_only(),
        ))
      } else {
        Ok(new_response_nocontent())
      }
    }

    "d" => {
      if vars.volatility.debug_mode() {
        *vars = GlobalVariables::new();
        Ok(new_response_with_value_with_notranslate(
          format!("\\![change,ghost,{}]", GHOST_NAME),
          TranslateOption::none(),
        ))
      } else {
        Ok(new_response_nocontent())
      }
    }
    _ => Ok(new_response_nocontent()),
  }
}
