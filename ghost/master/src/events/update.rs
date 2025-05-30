use crate::events::common::*;
use shiorust::message::{Request, Response};

pub(crate) fn on_update_begin(_req: &Request) -> Response {
  new_response_with_value_with_notranslate(
    "\\1\\_qゴーストの更新を確認中……\\n".to_string(),
    TranslateOption::none(),
  )
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
    format!("\\C\\1\\_q{}の更新を確認中……\\n", item_type),
    TranslateOption::none(),
  )
}

fn render_refs(refs: &Vec<&str>) -> String {
  refs.iter().map(|x| format!("{}, ", x)).collect::<String>()
}

pub(crate) fn on_update_result_ex(req: &Request) -> Response {
  let refs = get_references(req);
  let mut m = String::new();
  for r in refs.iter() {
    let results = r.split(1 as char).collect::<Vec<&str>>();
    if results.len() < 4 {
      continue;
    }
    let item_name = results[0];
    let item_type = match results[1] {
      "ghost" => "ゴースト",
      "shell" => "シェル",
      "balloon" => "バルーン",
      _ => continue, // ハイネに関係ないアイテムは無視
    };
    let status = match results[2] {
      "OK" => match results[3] {
        "0" => "更新なし",
        _ => "更新成功",
      },
      "NG" => &format!("更新失敗({})", results[3]),
      _ => &format!("不明なステータス({})", refs[0]),
    };
    m.push_str(&format!("{}({}): {}\\n", item_name, item_type, status));
  }
  new_response_with_value_with_notranslate(
    format!(
      "\\1\\_q{}{}",
      m, "\\n\\![*]\\q[更新履歴,https://apxxxxxxe.dev/works]\\x"
    ),
    TranslateOption::none(),
  )
}
