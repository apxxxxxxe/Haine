use crate::events::common::*;
use shiorust::message::{Request, Response};

pub(crate) fn on_update_begin(_req: &Request) -> Response {
  new_response_with_value_with_notranslate(
    "\\1\\_q更新を確認中……".to_string(),
    TranslateOption::none(),
  )
}

pub(crate) fn on_update_complete(req: &Request) -> Response {
  let refs = get_references(req);
  let m = match refs[0] {
    "none" => "更新なし",
    "changed" => "更新完了",
    _ => &format!("不明なステータス({})", refs[0]),
  };
  new_response_with_value_with_notranslate(format!("\\1\\_q{}", m), TranslateOption::none())
}

pub(crate) fn on_update_other_begin(req: &Request) -> Response {
  let refs = get_references(req);
  let item_type = match refs[3] {
    "ghost" => "ゴースト",
    "shell" => "シェル",
    "balloon" => "バルーン",
    "plugin" => "プラグイン",
    "headline" => "ヘッドライン",
    _ => &format!("不明なアイテム({})", render_refs(&refs)),
  };
  new_response_with_value_with_notranslate(
    format!("\\1\\_q{}の更新を確認中……", item_type),
    TranslateOption::none(),
  )
}

fn render_refs(refs: &Vec<&str>) -> String {
  refs.iter().map(|x| format!("{}, ", x)).collect::<String>()
}

pub(crate) fn on_update_other_complete(req: &Request) -> Response {
  let refs = get_references(req);
  let item_type = match refs[3] {
    "ghost" => "ゴースト",
    "shell" => "シェル",
    "balloon" => "バルーン",
    "plugin" => "プラグイン",
    "headline" => "ヘッドライン",
    _ => &format!("不明なアイテム({})", render_refs(&refs)),
  };
  let m = match refs[0] {
    "none" => "更新なし",
    "changed" => "更新完了",
    _ => &format!("不明なステータス({})", refs[0]),
  };
  new_response_with_value_with_notranslate(
    format!("\\1\\_q{}の{}", item_type, m),
    TranslateOption::none(),
  )
}
