use crate::events::aitalk::on_ai_talk;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::talk::random_talks_analysis;
use crate::system::error::ShioriError;
use crate::system::response::*;
use crate::system::variables::*;
use shiorust::message::{Request, Response};
use std::collections::HashMap;

use super::bootend::halloween_boot_talk;

pub(crate) fn on_key_press(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  match refs[0] {
    "a" => new_response_with_value_with_translate(
      "h1113205".to_string(),
      TranslateOption::simple_translate(),
    ),
    "t" => {
      if !get_read(&FLAGS).check(&EventFlag::FirstRandomTalkDone(
        FIRST_RANDOMTALKS.len() as u32 - 1,
      )) {
        Ok(new_response_nocontent())
      } else {
        on_ai_talk(req)
      }
    }
    "c" => {
      if *get_read(&DEBUG_MODE) {
        Ok(new_response_with_value_with_notranslate(
          random_talks_analysis(),
          TranslateOption::balloon_surface_only(),
        ))
      } else {
        Ok(new_response_nocontent())
      }
    }
    "h" => {
      if *get_read(&DEBUG_MODE) {
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
      if *get_read(&DEBUG_MODE) {
        // 全変数をリセット
        *get_write(&TOTAL_BOOT_COUNT) = 0;
        *get_write(&TOTAL_TIME) = 0;
        *get_write(&RANDOM_TALK_INTERVAL) = 0;
        *get_write(&USER_NAME) = "".to_string();
        *get_write(&TALK_COLLECTION) = HashMap::new();
        *get_write(&CUMULATIVE_TALK_COUNT) = 0;
        *get_write(&FLAGS) = EventFlags::default();
        *get_write(&PENDING_EVENT_TALK) = None;
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
