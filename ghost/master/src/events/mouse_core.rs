use crate::events::common::*;
use crate::events::menu::on_menu_exec;
use crate::events::mouse::*;
use crate::variables::get_global_vars;

use shiorust::message::{Response, *};
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
pub enum Direction {
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

pub fn on_mouse_wheel(req: &Request) -> Response {
  let vars = get_global_vars();
  let refs = get_references(req);
  if refs[4].is_empty() {
    new_response_nocontent()
  } else {
    let now = SystemTime::now();
    let dur = now
      .duration_since(vars.volatility.last_wheel_count_unixtime())
      .unwrap()
      .as_millis();

    let d: Direction = if refs[2].parse::<i32>().unwrap() > 0 {
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
      new_mouse_response(format!("{}{}{}", refs[3], refs[4], d.to_str()))
    } else {
      vars.volatility.set_last_wheel_count_unixtime(now);
      vars.volatility.set_last_wheel_part(refs[4].to_string());
      vars.volatility.set_wheel_direction(d);
      new_response_nocontent()
    }
  }
}

pub fn on_mouse_double_click(req: &Request) -> Response {
  let refs = get_references(req);
  if refs[4].is_empty() {
    on_menu_exec(req)
  } else {
    new_mouse_response(format!("{}{}doubleclick", refs[3], refs[4]))
  }
}

pub fn on_mouse_click_ex(req: &Request) -> Response {
  let refs = get_references(req);
  if refs[5] == "middle" {
    new_mouse_response(format!("{}{}middleclick", refs[3], refs[4]))
  } else {
    new_response_nocontent()
  }
}

pub fn on_mouse_move(req: &Request) -> Response {
  let vars = get_global_vars();
  let refs = get_references(req);
  if refs[4].is_empty() || vars.volatility.status_mut().get("talking").unwrap() {
    new_response_nocontent()
  } else {
    let now = SystemTime::now();
    if vars.volatility.last_nade_part() == refs[4] {
      let dur = now
        .duration_since(vars.volatility.last_nade_count_unixtime())
        .unwrap()
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
      new_mouse_response(format!("{}{}nade", refs[3], refs[4]))
    } else {
      new_response_nocontent()
    }
  }
}

fn new_mouse_response(info: String) -> Response {
  let vars = get_global_vars();
  if info != *vars.volatility.last_touch_info() {
    vars.volatility.set_touch_count(0);
  }
  vars.volatility.set_last_touch_info(info.clone());
  vars
    .volatility
    .set_touch_count(vars.volatility.touch_count() + 1);

  match mouse_dialogs(info, vars) {
    Some(dialogs) => new_response_with_value(choose_one(&dialogs, true).unwrap(), true),
    None => new_response_nocontent(),
  }
}
