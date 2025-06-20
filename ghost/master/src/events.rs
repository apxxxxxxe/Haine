pub(crate) mod aitalk;
mod bootend;
pub(crate) mod common;
mod input;
mod key;
mod menu;
mod mouse;
pub(crate) mod mouse_core;
mod periodic;
pub(crate) mod talk;
mod tooltip;
pub(crate) mod translate;
mod update;
mod webclap;

use crate::error::ShioriError;
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
use crate::events::update::*;
use crate::events::webclap::*;
use crate::variables::*;
use shiorust::message::{parts::*, traits::*, Request, Response};
use std::fs;

pub(crate) fn handle_request(req: &Request) -> Result<Response, ShioriError> {
  match req.method {
    Method::GET => (),
    Method::NOTIFY => (),
    _ => return Ok(new_response_nocontent()),
  };

  let event_id = match req.headers.get("ID") {
    Some(id) => id,
    None => return Ok(new_response_nocontent()),
  };
  debug!("event: {}", event_id);

  let event = match get_event(event_id.as_str()) {
    Some(e) => e,
    None => {
      let base_id = match req.headers.get("BaseID") {
        Some(id) => id,
        None => return Ok(new_response_nocontent()),
      };
      match get_event(base_id.as_str()) {
        Some(e) => e,
        None => return Ok(new_response_nocontent()),
      }
    }
  };

  match event {
    EventHandler::AlwaysSuccess(e) => Ok(e(req)),
    EventHandler::MayFailure(e) => e(req),
  }
}

fn version(_req: &Request) -> Response {
  new_response_with_value_with_notranslate(
    String::from(env!("CARGO_PKG_VERSION")),
    TranslateOption::none(),
  )
}

fn craftman(_req: &Request) -> Response {
  new_response_with_value_with_notranslate(String::from("HinoTsumi"), TranslateOption::none())
}

fn craftmanw(_req: &Request) -> Response {
  new_response_with_value_with_notranslate(String::from("日野つみ"), TranslateOption::none())
}

fn name(_req: &Request) -> Response {
  new_response_with_value_with_notranslate(String::from("haine"), TranslateOption::none())
}

fn log_path(_req: &Request) -> Response {
  let log_path = LOG_PATH.read().unwrap().clone();
  new_response_with_value_with_notranslate(log_path, TranslateOption::none())
}

fn uniqueid(req: &Request) -> Result<Response, ShioriError> {
  let id = req
    .headers
    .get("Reference0")
    .ok_or(ShioriError::BadRequest)?;
  // ローカルファイル`./debug`が存在しているなら上書きする
  if fs::metadata("./debug").is_ok() {
    fs::write("./debug", id).map_err(|_| ShioriError::FileWriteError)?;
  }
  Ok(new_response_nocontent())
}

pub(crate) enum EventHandler {
  AlwaysSuccess(fn(&Request) -> Response),
  MayFailure(fn(&Request) -> Result<Response, ShioriError>),
}

fn get_event(id: &str) -> Option<EventHandler> {
  match id {
    "version" => Some(EventHandler::AlwaysSuccess(version)),
    "craftman" => Some(EventHandler::AlwaysSuccess(craftman)),
    "craftmanw" => Some(EventHandler::AlwaysSuccess(craftmanw)),
    "name" => Some(EventHandler::AlwaysSuccess(name)),
    "log_path" => Some(EventHandler::AlwaysSuccess(log_path)),
    "uniqueid" => Some(EventHandler::MayFailure(uniqueid)),
    "OnBoot" => Some(EventHandler::MayFailure(on_boot)),
    "OnClose" => Some(EventHandler::MayFailure(on_close)),
    "OnVanishSelecting" => Some(EventHandler::AlwaysSuccess(on_vanish_selecting)),
    "OnAiTalk" => Some(EventHandler::MayFailure(on_ai_talk)),
    "OnAnchorSelectEx" => Some(EventHandler::MayFailure(on_anchor_select_ex)),
    "OnNotifyUserInfo" => Some(EventHandler::AlwaysSuccess(on_notify_user_info)),
    "OnMinuteChange" => Some(EventHandler::AlwaysSuccess(on_minute_change)),
    "OnSecondChange" => Some(EventHandler::MayFailure(on_second_change)),
    "OnSurfaceChange" => Some(EventHandler::MayFailure(on_surface_change)),
    "OnSmoothBlink" => Some(EventHandler::MayFailure(on_smooth_blink)),
    "OnMenuExec" => Some(EventHandler::AlwaysSuccess(on_menu_exec)),
    "OnConfigMenuExec" => Some(EventHandler::AlwaysSuccess(on_config_menu_exec)),
    "OnTalkIntervalChanged" => Some(EventHandler::MayFailure(on_talk_interval_changed)),
    "OnMouseClickEx" => Some(EventHandler::MayFailure(on_mouse_click_ex)),
    "OnMouseDoubleClick" => Some(EventHandler::MayFailure(on_mouse_double_click)),
    "OnMouseMove" => Some(EventHandler::MayFailure(on_mouse_move)),
    "OnMouseWheel" => Some(EventHandler::MayFailure(on_mouse_wheel)),
    "OnKeyPress" => Some(EventHandler::MayFailure(on_key_press)),
    "OnTalk" => Some(EventHandler::MayFailure(on_talk)),
    "OnTalkAnswer" => Some(EventHandler::MayFailure(on_talk_answer)),
    "OnWebClapOpen" => Some(EventHandler::MayFailure(on_web_clap_open)),
    "OnWebClapInput" => Some(EventHandler::MayFailure(on_web_clap_input)),
    "OnExecuteHTTPComplete" => Some(EventHandler::MayFailure(on_execute_http_complete)),
    "OnExecuteHTTPFailure" => Some(EventHandler::MayFailure(on_execute_http_failure)),
    "balloon_tooltip" => Some(EventHandler::AlwaysSuccess(balloon_tooltip)),
    "OnBalloonTooltip" => Some(EventHandler::AlwaysSuccess(on_balloon_tooltip)),
    "OnStickSurface" => Some(EventHandler::AlwaysSuccess(on_stick_surface)),
    "OnWaitTranslater" => Some(EventHandler::MayFailure(on_wait_translater)),
    "OnCheckTalkCollection" => Some(EventHandler::AlwaysSuccess(on_check_talk_collection)),
    "OnCheckUnseenTalks" => Some(EventHandler::MayFailure(on_check_unseen_talks)),
    "OnWindowStateRestore" => Some(EventHandler::AlwaysSuccess(on_window_state_restore)),
    "OnUserInput" => Some(EventHandler::MayFailure(on_user_input)),
    "OnChangingUserName" => Some(EventHandler::MayFailure(on_changing_user_name)),
    "OnImmersiveDegreeToggled" => Some(EventHandler::AlwaysSuccess(on_immersive_degree_toggled)),
    "OnStoryEvent" => Some(EventHandler::MayFailure(on_story_event)),
    "OnUpdateBegin" => Some(EventHandler::AlwaysSuccess(on_update_begin)),
    "OnUpdateResultEx" => Some(EventHandler::AlwaysSuccess(on_update_result_ex)),
    "OnStoryHistoryMenu" => Some(EventHandler::AlwaysSuccess(on_story_history_menu)),
    "OnStoryHistoryExec" => Some(EventHandler::MayFailure(on_story_history_exec)),
    "OnDerivativeTalkRequestButtonToggled" => Some(EventHandler::AlwaysSuccess(
      on_derivative_talk_request_button_toggled,
    )),
    "OnDerivativeTalkRequestOpen" => {
      Some(EventHandler::MayFailure(on_derivative_talk_request_open))
    }
    "OnDerivativeTalkRequestInput" => {
      Some(EventHandler::MayFailure(on_derivative_talk_request_input))
    }
    _ => None,
  }
}
