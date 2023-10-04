use crate::events::aitalk::on_ai_talk;
use crate::events::common::*;
use crate::variables::get_global_vars;
use shiorust::message::{Request, Response};

pub fn on_second_change(req: &Request) -> Response {
    let vars = get_global_vars();
    let total_time = vars.total_time.unwrap();
    vars.total_time = Some(total_time + 1);
    vars.volatility.ghost_up_time += 1;

    let refs = get_references(req);
    let idle_secs = refs[4].parse::<i32>().unwrap();
    vars.volatility.idle_seconds = idle_secs;

    if vars.volatility.ghost_up_time % vars.random_talk_interval.unwrap() == 0 {
        on_ai_talk(req)
    } else {
        new_response_nocontent()
    }
}
