use crate::error::ShioriError;
use crate::events::aitalk::on_ai_talk;
use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::talk::random_talks_analysis;
use crate::variables::*;
use shiorust::message::{Request, Response};
use std::collections::HashMap;

use super::bootend::halloween_boot_talk;

pub(crate) fn on_key_press(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  match refs[0] {
    "t" => {
      if !FLAGS.read().unwrap().check(&EventFlag::FirstRandomTalkDone(
        FIRST_RANDOMTALKS.len() as u32 - 1,
      )) {
        Ok(new_response_nocontent())
      } else {
        on_ai_talk(req)
      }
    }
    "c" => {
      if *DEBUG_MODE.read().unwrap() {
        Ok(new_response_with_value_with_notranslate(
          random_talks_analysis(),
          TranslateOption::balloon_surface_only(),
        ))
      } else {
        Ok(new_response_nocontent())
      }
    }
    "h" => {
      if *DEBUG_MODE.read().unwrap() {
        let v = format!(
          "\\0\\s[{}]{}\\![embed,OnStickSurface]{}",
          TRANSPARENT_SURFACE,
          RESET_BINDS,
          halloween_boot_talk(),
        );
        new_response_with_value_with_translate(v, TranslateOption::simple_translate())
      } else {
        Ok(new_response_nocontent())
      }
    }
    "d" => {
      if *DEBUG_MODE.read().unwrap() {
        // 全変数をリセット
        *TOTAL_BOOT_COUNT.write().unwrap() = 0;
        *TOTAL_TIME.write().unwrap() = 0;
        *RANDOM_TALK_INTERVAL.write().unwrap() = 0;
        *USER_NAME.write().unwrap() = "".to_string();
        *TALK_COLLECTION.write().unwrap() = HashMap::new();
        *CUMULATIVE_TALK_COUNT.write().unwrap() = 0;
        *FLAGS.write().unwrap() = EventFlags::default();
        *PENDING_EVENT_TALK.write().unwrap() = None;
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
