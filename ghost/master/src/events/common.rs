use crate::events::translate::on_translate;
use crate::variables::get_global_vars;

use shiorust::message::{parts::HeaderName, parts::*, traits::*, Request, Response};

pub fn new_response() -> Response {
  let mut headers = Headers::new();
  headers.insert(
    HeaderName::Standard(StandardHeaderName::Charset),
    String::from("UTF-8"),
  );
  Response {
    version: Version::V30,
    status: Status::OK,
    headers,
  }
}

pub fn new_response_nocontent() -> Response {
  let mut r = new_response();
  r.status = Status::NoContent;
  r
}

pub fn new_response_with_value(value: String, use_translate: bool) -> Response {
  let v;
  if use_translate {
    v = on_translate(value);
  } else {
    v = value;
  }
  let mut r = new_response();
  r.headers.insert(HeaderName::from("Value"), v);
  r
}

pub fn choose_one(values: &Vec<String>, update_weight: bool) -> Option<String> {
  if values.len() == 0 {
    return None;
  }
  let vars = get_global_vars();
  let u = vars.volatility.talk_bias.roulette(values, update_weight);
  Some(values.get(u).unwrap().to_owned())
}

// return all combinations of values
// e.g. [a, b], [c, d], [e, f] => "ace", "acf", "ade", "adf", "bce", "bcf", "bde", "bdf"
pub fn all_combo(values: &Vec<Vec<String>>) -> Vec<String> {
  let mut result = Vec::new();
  let mut current = Vec::new();
  all_combo_inner(values, &mut result, &mut current, 0);
  result.iter().map(|v| v.join("")).collect()
}

fn all_combo_inner(
  values: &Vec<Vec<String>>,
  result: &mut Vec<Vec<String>>,
  current: &mut Vec<String>,
  index: usize,
) {
  if index == values.len() {
    result.push(current.clone());
    return;
  }
  for v in values[index].iter() {
    current.push(v.to_string());
    all_combo_inner(values, result, current, index + 1);
    current.pop();
  }
}

pub fn get_references(req: &Request) -> Vec<&str> {
  let mut references: Vec<&str> = Vec::new();
  let mut i = 0;
  loop {
    match req
      .headers
      .get(&HeaderName::from(&format!("Reference{}", i)))
    {
      Some(value) => {
        references.push(value);
        i += 1;
      }
      None => break,
    }
  }
  references
}

pub fn user_talk(dialog: &str, text: &str, text_first: bool) -> String {
  let mut d = String::new();
  if dialog != "" {
    d = format!("『{}』", dialog);
  }
  let mut t = String::new();
  if text != "" {
    t = format!("{}", text);
  }

  let mut v: Vec<String>;
  if text_first {
    v = vec![t, d];
  } else {
    v = vec![d, t];
  }
  v = v
    .iter()
    .filter(|s| s != &&"")
    .map(|s| s.to_string())
    .collect();

  format!("\\1{}\\n", v.join("\\n"))
}

// サーフェス変更の際に目線が動くとき、なめらかに見えるようにまばたきのサーフェスを補完する関数
pub fn on_smooth_blink(req: &Request) -> Response {
  const DELAY: i32 = 100;

  let refs = get_references(req);

  let dest_surface = refs[0].parse::<i32>().unwrap();
  let dest_eyes = dest_surface % 100;
  let dest_remain = dest_surface - dest_eyes;
  let from_surface = get_global_vars().volatility.current_surface;
  let from_eyes = from_surface % 100;

  if from_surface == 0 {
    return new_response_with_value(format!("\\s[{}]", dest_surface), false);
  } else if from_surface == dest_surface {
    return new_response_nocontent();
  }

  let mut cuts = vec![from_surface];
  if (from_eyes == 7 || from_eyes == 9) && (dest_eyes >= 1 && dest_eyes <= 3) {
    //直前が目閉じかつ目標が全目の場合
    cuts.push(dest_surface + 3);
  } else if (dest_eyes == 7 || dest_eyes == 9) && (from_eyes >= 1 && from_eyes <= 3) {
    // 直前が全目かつ目標が目閉じの場合
    cuts.push(dest_remain + from_eyes + 3);
  } else if (dest_eyes >= 1 && dest_eyes <= 3)
    && (from_eyes >= 1 && from_eyes <= 3)
    && (from_eyes != dest_eyes)
  {
    // 直前が全目かつ目標が全目の場合（直前と目標が同じ場合を除く）
    cuts.push(dest_surface + 3);
    cuts.push(dest_remain + 9);
    cuts.push(dest_surface + 3);
  } else if (dest_eyes >= 4 && dest_eyes <= 6)
    && (from_eyes >= 1 && from_eyes <= 3)
    && ((from_eyes + 3) != dest_eyes)
  {
    // 直前が全目かつ目標が半目の場合（直前と目標が同じ場合, 直前と目標の目線方向が同じ場合を除く）
    cuts.push(from_surface + 3);
    cuts.push(dest_remain + 9);
  } else if (dest_eyes >= 1 && dest_eyes <= 3)
    && (from_eyes >= 4 && from_eyes <= 6)
    && ((from_eyes - 3) != dest_eyes)
  {
    // 直前が半目かつ目標が全目の場合（直前と目標が同じ場合, 直前と目標の目線方向が同じ場合を除く）
    cuts.push(dest_remain + 9);
    cuts.push(dest_surface + 3);
  }
  cuts.push(dest_surface);

  let delay = format!("\\_w[{}]", DELAY);
  let animation = cuts
    .iter()
    .map(|s| format!("\\s[{}]", s))
    .collect::<Vec<String>>()
    .join(delay.as_str());

  new_response_with_value(animation, false)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_all_combo() {
    let values = vec![
      vec!["a".to_string(), "b".to_string()],
      vec!["c".to_string(), "d".to_string()],
      vec!["e".to_string(), "f".to_string()],
    ];
    let result = all_combo(&values);
    println!("{:?}", result);
  }
}
