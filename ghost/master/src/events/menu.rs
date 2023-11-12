use crate::events::common::*;
use crate::variables::get_global_vars;
use shiorust::message::{Response, *};

pub fn on_menu_exec(_req: &Request) -> Response {
  let current_talk_interval = get_global_vars().random_talk_interval.unwrap_or(180);
  let mut selections = Vec::new();
  for i in [1, 3, 5, 7, 10].iter() {
    if current_talk_interval == *i * 60 {
      selections.push(format!("\\f[underline,1]{}分\\f[underline,0]", i,));
    } else {
      selections.push(format!("\\q[{}分,OnTalkIntervalChanged,{}]", i, i * 60,));
    };
  }
  let talk_interval_selector = format!(
    "\
    ◆トーク頻度  【現在 {}分】\\n\
    {}\
  ",
    current_talk_interval / 60,
    selections.join("  ")
  );

  let m = format!(
    "\\_q\
    \\_l[0,4em]\
    \\![*]\\q[なにか話して,OnAiTalk]\\n\\n\
    {}
    \\_l[0,12em]\\q[×,]\
    ",
    talk_interval_selector
  );

  new_response_with_value(m, true)
}

pub fn on_talk_interval_changed(req: &Request) -> Response {
  let refs = get_references(req);
  let v = refs[0].parse::<u64>().unwrap();
  get_global_vars().random_talk_interval = Some(v);

  on_menu_exec(req)
}
