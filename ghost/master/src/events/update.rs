use crate::events::common::*;
use shiorust::message::{Request, Response};

pub(crate) fn on_update_begin(_req: &Request) -> Response {
   new_response_with_value_with_notranslate("\\1\\_q更新を確認中……".to_string(), TranslateOption::none()) 
}

pub(crate) fn on_update_complete(req: &Request) -> Response {
    let refs = get_references(req);
    let m = match refs[0] {
        "none" => "更新なし",
        "changed" => "更新完了",
        _ => unreachable!(),
    };
   new_response_with_value_with_notranslate(format!("\\1\\_q{}\\n", m), TranslateOption::none())
}
