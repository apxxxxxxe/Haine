use crate::events::common::*;
use shiorust::message::{Request, Response};

pub fn on_web_clap_open(_req: &Request) -> Response {
  let m = "\
             \\1\\![open,inputbox,OnWebClapInput,0]Web拍手を送ります。\\n\
             感想やバグ報告、要望などをお送り下さい。\
             "
  .to_string();
  new_response_with_value(m, TranslateOption::simple_translate())
}

pub fn on_web_clap_input(req: &Request) -> Response {
  let refs = get_references(req);
  let m = format!(
    "\\1\\![execute,http-post,http://clap.webclap.com/clap.php?id=apxxxxxxe,--param=message_body=Haine:{},--async=webclap]",
    refs[0]
  );
  new_response_with_value(m, TranslateOption::simple_translate())
}

pub fn on_execute_http_complete(req: &Request) -> Response {
  let refs = get_references(req);
  if refs[1] == "webclap" {
    new_response_with_value(
      "\\1送信しました。".to_string(),
      TranslateOption::simple_translate(),
    )
  } else {
    new_response_nocontent()
  }
}

pub fn on_execute_http_failure(_req: &Request) -> Response {
  let refs = get_references(_req);
  if refs[1] == "webclap" {
    new_response_with_value(
      "\\1送信に失敗しました。".to_string(),
      TranslateOption::simple_translate(),
    )
  } else {
    new_response_nocontent()
  }
}
