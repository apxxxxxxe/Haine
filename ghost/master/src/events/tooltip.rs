use crate::system::response::*;
use shiorust::message::{Request, Response};

pub(crate) fn on_balloon_tooltip(_req: &Request) -> Response {
  new_response_with_value_with_notranslate("\\C\\_l[0,0] ".to_string(), TranslateOption::none())
}

pub(crate) fn balloon_tooltip(req: &Request) -> Response {
  let refs = get_references(req);
  if refs[1] != "OnBalloonTooltip" {
    return new_response_nocontent();
  }
  new_response_nocontent()
}
