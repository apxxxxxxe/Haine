use crate::events::common::*;
use shiorust::message::{Response, *};

pub fn on_web_clap_input(req: &Request) -> Response {
  let refs = get_references(req);
  let m = format!(
    "\\1\\![execute,http-post,http://clap.webclap.com/clap.php?id=apxxxxxxe,--param=message_body=Haine:{},--async=webclap]",
    refs[0]
  );
  new_response_with_value(m, true)
}

pub fn on_execute_http_complete(req: &Request) -> Response {
  let refs = get_references(req);
  if refs[1] == "webclap" {
    new_response_with_value("\\1送信しました。".to_string(), true)
  } else {
    new_response_nocontent()
  }
}

pub fn on_execute_http_failure(_req: &Request) -> Response {
  let refs = get_references(_req);
  if refs[1] == "webclap" {
    new_response_with_value("\\1送信に失敗しました。".to_string(), true)
  } else {
    new_response_nocontent()
  }
}
