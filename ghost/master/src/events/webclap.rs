use crate::error::ShioriError;
use crate::events::common::*;
use shiorust::message::{Request, Response};

use super::talk::Talk;

pub(crate) fn derivative_talk_request_open(event_id: &str) -> Result<Response, ShioriError> {
  let last_talk = match Talk::all_talks().unwrap().iter().find(|t| t.id == event_id) {
    Some(t) => t.text.clone(),
    None => "".to_string(),
  };
  new_response_with_value_with_translate(
    format!("\\1\\![open,inputbox,OnDerivativeTalkRequestInput,0,このトークに対するリアクションの要望を送信できます。,--reference={}]{}", event_id,
    last_talk),
    TranslateOption::simple_translate(),
  )
}

pub(crate) fn on_derivative_talk_request_input(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let text = refs[0];
  let event_id = refs[2];
  let m = format!(
    "\\1\\![execute,http-post,https://webclap.apxxxxxxe.dev/clap,--param=Haine:{}:{},--async=webclap]",
    event_id, text
  );
  new_response_with_value_with_translate(m, TranslateOption::simple_translate())
}

pub(crate) fn on_web_clap_open(_req: &Request) -> Result<Response, ShioriError> {
  let m = "\
             \\1\\![open,inputbox,OnWebClapInput,0]Web拍手を送ります。\\n\
             感想やバグ報告、要望などをお送り下さい。\
             "
  .to_string();
  new_response_with_value_with_translate(m, TranslateOption::simple_translate())
}

pub(crate) fn on_web_clap_input(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let m = format!(
    "\\1\\![execute,http-post,https://webclap.apxxxxxxe.dev/clap,--param=Haine:{},--async=webclap]",
    refs[0]
  );
  new_response_with_value_with_translate(m, TranslateOption::simple_translate())
}

pub(crate) fn on_execute_http_complete(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  if refs[1] == "webclap" {
    new_response_with_value_with_translate(
      "\\1送信しました。".to_string(),
      TranslateOption::simple_translate(),
    )
  } else {
    Ok(new_response_nocontent())
  }
}

pub(crate) fn on_execute_http_failure(_req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(_req);
  if refs[1] == "webclap" {
    new_response_with_value_with_translate(
      format!("\\1送信に失敗しました: {}", refs[4]),
      TranslateOption::simple_translate(),
    )
  } else {
    Ok(new_response_nocontent())
  }
}
