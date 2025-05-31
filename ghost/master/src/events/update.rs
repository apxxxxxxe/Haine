use crate::events::common::*;
use shiorust::message::{Request, Response};

pub(crate) fn on_update_begin(req: &Request) -> Response {
  let refs = get_references(req);
  if refs.len() < 5 {
    return new_response_nocontent();
  }
  if refs[0] != "Crave The Grave" || refs[4] != "manual" {
    new_response_nocontent()
  } else {
    new_response_with_value_with_notranslate(
      "\\1\\_qゴーストの更新を確認中……".to_string(),
      TranslateOption::none(),
    )
  }
}

pub(crate) fn on_update_result_ex(req: &Request) -> Response {
  let refs = get_references(req);
  let mut m = String::new();
  if refs.is_empty() {
    return new_response_nocontent();
  }
  for r in refs.iter() {
    let results = r.split(1 as char).collect::<Vec<&str>>();
    if results.len() < 4 {
      continue;
    }
    let item_name = results[0];
    let item_type = match results[1] {
      "ghost" => {
        if results[0] == "Crave The Grave" {
          "ゴースト"
        } else {
          continue;
        }
      }
      "shell" => "シェル",
      "balloon" => {
        if results[0] == "霧の郊外にて" {
          "バルーン"
        } else {
          continue;
        }
      }
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
