use crate::events::common::*;
use crate::events::GlobalVariables;
use shiorust::message::{Response, *};

pub fn on_menu_exec(_req: &Request, vars: &mut GlobalVariables) -> Response {
    let m = "\
    \\_l[0,4em]\
    \\![*]\\q[なにか話して,OnAiTalk]\
    \\_l[0,12em]\\q[×,]\
    ";
    new_response_with_value(m.to_string(), vars, true)
}
