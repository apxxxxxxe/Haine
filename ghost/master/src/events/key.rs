use crate::events::aitalk::{on_ai_talk, random_talks_analysis};
use crate::events::bootend::on_first_boot;
use crate::events::common::*;
use crate::events::periodic::on_hour_time_signal;
use shiorust::message::{Request, Response};

pub fn on_key_press(req: &Request) -> Response {
  let refs = get_references(req);
  match refs[0] {
    "r" => new_response_with_value(
      "unload:10秒後にリロード\\![unload,shiori]\\_w[10000]\\![reload,ghost]".to_string(),
      TranslateOption::balloon_surface_only(),
    ),
    "t" => on_ai_talk(req),
    "a" => on_hour_time_signal(req),
    "b" => on_first_boot(req),
    "c" => new_response_with_value(
      random_talks_analysis(),
      TranslateOption::balloon_surface_only(),
    ),
    _ => new_response_nocontent(),
  }
}
