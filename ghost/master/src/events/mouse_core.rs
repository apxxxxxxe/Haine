use crate::check_error;
use crate::events::mouse::*;
use crate::system::error::ShioriError;
use crate::system::response::*;
use crate::system::status::Status;
use crate::system::variables::*;
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
  let refs = get_references(req);
  let now = SystemTime::now();
  let dur = check_error!(
    now.duration_since(*get_read(&LAST_WHEEL_COUNT_UNIXTIME)),
    ShioriError::SystemTimeError
  )
  .as_millis();

  let d: Direction = if check_error!(refs[2].parse::<i32>(), ShioriError::ParseIntError) > 0 {
    Direction::Up
  } else {
    Direction::Down
  };

  if *get_read(&LAST_WHEEL_PART) != refs[4] || dur > WHEEL_LIFETIME {
    *get_write(&WHEEL_COUNTER) = 1;
  } else {
    *get_write(&WHEEL_COUNTER) += 1;
  }

  if *get_read(&WHEEL_COUNTER) >= WHEEL_THRESHOLD {
    *get_write(&WHEEL_COUNTER) = 0;
    new_mouse_response(req, format!("{}{}{}", refs[3], refs[4], d.to_str()))
  } else {
    *get_write(&LAST_WHEEL_COUNT_UNIXTIME) = now;
    *get_write(&LAST_WHEEL_PART) = refs[4].to_string();
    *get_write(&WHEEL_DIRECTION) = d;
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
  let refs = get_references(req);
  let status = Status::from_request(req);
  if refs[4].is_empty() || status.talking {
    Ok(new_response_nocontent())
  } else {
    let now = SystemTime::now();
    if *get_read(&LAST_NADE_PART) == refs[4] {
      let dur = check_error!(
        now.duration_since(*get_read(&LAST_NADE_COUNT_UNIXTIME)),
        ShioriError::SystemTimeError
      )
      .as_millis();
      if dur > NADE_LIFETIME {
        *get_write(&NADE_COUNTER) = 1;
        *get_write(&LAST_NADE_COUNT_UNIXTIME) = now;
      } else if dur >= NADE_DURATION {
        *get_write(&NADE_COUNTER) += 1;
        *get_write(&LAST_NADE_COUNT_UNIXTIME) = now;
      }
      debug!("{} {} {}", refs[4], dur, *get_read(&NADE_COUNTER));
    } else {
      *get_write(&NADE_COUNTER) = 1;
    }
    *get_write(&LAST_NADE_PART) = refs[4].to_string();
    if *get_read(&NADE_COUNTER) > NADE_THRESHOLD {
      *get_write(&NADE_COUNTER) = 0;
      new_mouse_response(req, format!("{}{}nade", refs[3], refs[4]))
    } else {
      Ok(new_response_nocontent())
    }
  }
}
