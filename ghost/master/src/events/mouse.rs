use crate::events::common::*;
use crate::events::menu::on_menu_exec;
use crate::events::GlobalVariables;

use shiorust::message::{Response, *};
use std::time::SystemTime;

const NADE_THRESHOLD: i32 = 12;
const NADE_DURATION: u128 = 100;
const NADE_LIFETIME: u128 = 3000;

pub fn on_mouse_double_click(req: &Request, vars: &mut GlobalVariables) -> Response {
    let refs = get_references(req);
    if refs[4] == "" {
        on_menu_exec(req, vars)
    } else {
        new_response_with_value(refs[4].to_string(), true)
    }
}

pub fn on_mouse_move(req: &Request, vars: &mut GlobalVariables) -> Response {
    let refs = get_references(req);
    if refs[4] == "" {
        new_response_nocontent()
    } else {
        let now = SystemTime::now();
        if vars.volatility.last_nade_part == refs[4] {
            let dur = now
                .duration_since(vars.volatility.last_nade_count_unixtime)
                .unwrap()
                .as_millis();
            if dur > NADE_LIFETIME {
                vars.volatility.nade_counter = 0;
                vars.volatility.last_nade_count_unixtime = now;
            } else if dur >= NADE_DURATION {
                vars.volatility.nade_counter += 1;
                vars.volatility.last_nade_count_unixtime = now;
            }
            debug!("{} {} {}", refs[4], dur, vars.volatility.nade_counter);
        } else {
            vars.volatility.nade_counter = 0;
        }
        vars.volatility.last_nade_part = refs[4].to_string();
        if vars.volatility.nade_counter > NADE_THRESHOLD {
            vars.volatility.nade_counter = 0;
            new_response_with_value(refs[4].to_string() + "なで", true)
        } else {
            new_response_nocontent()
        }
    }
}
