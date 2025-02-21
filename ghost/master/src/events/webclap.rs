use crate::error::ShioriError;
use crate::events::common::*;
use shiorust::message::{Request, Response};

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
