pub mod aitalk;
mod bootend;
pub mod common;
mod input;
mod key;
mod menu;
mod mouse;
pub mod mouse_core;
mod periodic;
pub mod talk;
mod tooltip;
pub mod translate;
mod webclap;

use crate::events::aitalk::*;
use crate::events::bootend::*;
use crate::events::common::*;
use crate::events::input::*;
use crate::events::key::*;
use crate::events::menu::*;
use crate::events::mouse::*;
use crate::events::mouse_core::*;
use crate::events::periodic::*;
use crate::events::talk::*;
use crate::events::tooltip::*;
use crate::events::translate::*;
use crate::events::webclap::*;
use crate::variables::get_global_vars;
use shiorust::message::{parts::*, traits::*, Request, Response};

pub fn handle_request(req: &Request) -> Response {
  match req.method {
    Method::GET => (),
    Method::NOTIFY => (),
    _ => return new_response_nocontent(),
  };

  let event_id = match req.headers.get("ID") {
    Some(id) => id,
    None => return new_response_nocontent(),
  };
  debug!("event: {}", event_id);

  let event = match get_event(event_id.as_str()) {
    Some(e) => e,
    None => {
      let base_id = match req.headers.get("BaseID") {
        Some(id) => id,
        None => return new_response_nocontent(),
      };
      match get_event(base_id.as_str()) {
        Some(e) => e,
        None => return new_response_nocontent(),
      }
    }
  };

  let res = event(req);
  debug!("response: {:?}", res);
  res
}

fn version(_req: &Request) -> Response {
  new_response_with_value(
    String::from(env!("CARGO_PKG_VERSION")),
    TranslateOption::none(),
  )
}

fn craftman(_req: &Request) -> Response {
  new_response_with_value(String::from("HinoTsumi"), TranslateOption::none())
}

fn craftmanw(_req: &Request) -> Response {
  new_response_with_value(String::from("日野つみ"), TranslateOption::none())
}

fn name(_req: &Request) -> Response {
  new_response_with_value(String::from("haine"), TranslateOption::none())
}

fn log_path(_req: &Request) -> Response {
  let log_path = get_global_vars().volatility.log_path();
  new_response_with_value(log_path, TranslateOption::none())
}

fn get_event(id: &str) -> Option<fn(&Request) -> Response> {
  match id {
    "version" => Some(version),
    "craftman" => Some(craftman),
    "craftmanw" => Some(craftmanw),
    "name" => Some(name),
    "log_path" => Some(log_path),
    "OnBoot" => Some(on_boot),
    "OnClose" => Some(on_close),
    "OnVanishSelected" => Some(on_vanish_selected),
    "OnAiTalk" => Some(on_ai_talk),
    "OnAnchorSelectEx" => Some(on_anchor_select_ex),
    "OnNotifyUserInfo" => Some(on_notify_user_info),
    "OnSecondChange" => Some(on_second_change),
    "OnSurfaceChange" => Some(on_surface_change),
    "OnSmoothBlink" => Some(on_smooth_blink),
    "OnMenuExec" => Some(on_menu_exec),
    "OnBreakTime" => Some(on_break_time),
    "OnImmersiveRateReduced" => Some(on_immersive_rate_reduced),
    "OnTalkIntervalChanged" => Some(on_talk_interval_changed),
    "OnMouseClickEx" => Some(on_mouse_click_ex),
    "OnMouseDoubleClick" => Some(on_mouse_double_click),
    "OnMouseMove" => Some(on_mouse_move),
    "OnMouseWheel" => Some(on_mouse_wheel),
    "OnKeyPress" => Some(on_key_press),
    "OnTalk" => Some(on_talk),
    "OnTalkAnswer" => Some(on_talk_answer),
    "OnWebClapOpen" => Some(on_web_clap_open),
    "OnWebClapInput" => Some(on_web_clap_input),
    "OnExecuteHTTPComplete" => Some(on_execute_http_complete),
    "OnExecuteHTTPFailure" => Some(on_execute_http_failure),
    "balloon_tooltip" => Some(balloon_tooltip),
    "OnBalloonTooltip" => Some(on_balloon_tooltip),
    "OnStickSurface" => Some(on_stick_surface),
    "OnWaitTranslater" => Some(on_wait_translater),
    "OnCheckTalkCollection" => Some(on_check_talk_collection),
    "OnCheckUnseenTalks" => Some(on_check_unseen_talks),
    "OnHeadHit" => Some(on_head_hit),
    "OnHeadHitCancel" => Some(on_head_hit_cancel),
    "OnWindowStateRestore" => Some(on_window_state_restore),
    "OnUserInput" => Some(on_user_input),
    "OnChangingUserName" => Some(on_changing_user_name),
    _ => None,
  }
}
