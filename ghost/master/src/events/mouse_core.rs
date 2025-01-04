use crate::check_error;
use crate::error::ShioriError;
use crate::events::common::*;
use crate::events::mouse::*;
use crate::status::Status;
use crate::variables::get_global_vars;
use shiorust::message::{Request, Response};
use std::time::SystemTime;

// 撫での蓄積値の閾値: この値を超えたら撫でイベントを発生させる
const NADE_THRESHOLD: i32 = 12;

// 撫でをカウントする間隔(ms): 最後の撫でからこの時間が経つまで撫でをカウントしない
const NADE_DURATION: u128 = 100;

// 撫での蓄積値がリセットされるまでの時間(ms)
const NADE_LIFETIME: u128 = 3000;

// ホイールの蓄積値の閾値: この値を超えたらホイールイベントを発生させる
const WHEEL_THRESHOLD: i32 = 3;

// ホイールの蓄積値がリセットされるまでの時間(ms)
const WHEEL_LIFETIME: u128 = 3000;

#[derive(PartialEq, Clone)]
pub(crate) enum Direction {
  Up,
  Down,
}

impl Direction {
  pub fn to_str(&self) -> &str {
    match self {
      Direction::Up => "up",
      Direction::Down => "down",
    }
  }
}

pub(crate) fn on_mouse_wheel(req: &Request) -> Result<Response, ShioriError> {
  let vars = get_global_vars();
  let refs = get_references(req);
  let now = SystemTime::now();
  let dur = check_error!(
    now.duration_since(vars.volatility.last_wheel_count_unixtime()),
    ShioriError::SystemTimeError
  )
  .as_millis();

  let d: Direction = if check_error!(refs[2].parse::<i32>(), ShioriError::ParseIntError) > 0 {
    Direction::Up
  } else {
    Direction::Down
  };

  if vars.volatility.last_wheel_part() != refs[4]
    || vars.volatility.wheel_direction() != d
    || dur > WHEEL_LIFETIME
  {
    vars.volatility.set_wheel_counter(1);
  } else {
    vars
      .volatility
      .set_wheel_counter(vars.volatility.wheel_counter() + 1);
  }

  if vars.volatility.wheel_counter() >= WHEEL_THRESHOLD {
    vars.volatility.set_wheel_counter(0);
    new_mouse_response(req, format!("{}{}{}", refs[3], refs[4], d.to_str()))
  } else {
    vars.volatility.set_last_wheel_count_unixtime(now);
    vars.volatility.set_last_wheel_part(refs[4].to_string());
    vars.volatility.set_wheel_direction(d);
    Ok(new_response_nocontent())
  }
}

pub(crate) fn on_mouse_double_click(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  new_mouse_response(req, format!("{}{}doubleclick", refs[3], refs[4]))
}

pub(crate) fn on_mouse_click_ex(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  if refs[5] == "middle" {
    new_mouse_response(req, format!("{}{}middleclick", refs[3], refs[4]))
  } else {
    Ok(new_response_nocontent())
  }
}

pub(crate) fn on_mouse_move(req: &Request) -> Result<Response, ShioriError> {
  let vars = get_global_vars();
  let refs = get_references(req);
  let status = Status::from_request(req);
  if refs[4].is_empty() || status.talking {
    Ok(new_response_nocontent())
  } else {
    let now = SystemTime::now();
    if vars.volatility.last_nade_part() == refs[4] {
      let dur = check_error!(
        now.duration_since(vars.volatility.last_nade_count_unixtime()),
        ShioriError::SystemTimeError
      )
      .as_millis();
      if dur > NADE_LIFETIME {
        vars.volatility.set_nade_counter(1);
        vars.volatility.set_last_nade_count_unixtime(now);
      } else if dur >= NADE_DURATION {
        vars
          .volatility
          .set_nade_counter(vars.volatility.nade_counter() + 1);
        vars.volatility.set_last_nade_count_unixtime(now);
      }
      debug!("{} {} {}", refs[4], dur, vars.volatility.nade_counter());
    } else {
      vars.volatility.set_nade_counter(1);
    }
    vars.volatility.set_last_nade_part(refs[4].to_string());
    if vars.volatility.nade_counter() > NADE_THRESHOLD {
      vars.volatility.set_nade_counter(0);
      new_mouse_response(req, format!("{}{}nade", refs[3], refs[4]))
    } else {
      Ok(new_response_nocontent())
    }
  }
}
