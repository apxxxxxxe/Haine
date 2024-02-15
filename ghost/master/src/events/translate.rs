use crate::autobreakline::{extract_scope, CHANGE_SCOPE_RE};
use crate::events::common::*;
use crate::variables::get_global_vars;
use core::fmt::{Display, Formatter};
use once_cell::sync::Lazy;
use regex::Regex;
use shiorust::message::{Request, Response};
use std::thread;
use std::time::Duration;

pub fn on_wait_translater(_req: &Request) -> Response {
  while !get_global_vars().volatility.inserter_mut().is_ready() {
    thread::sleep(Duration::from_millis(100));
  }
  let m = get_global_vars().volatility.waiting_talk().unwrap();
  new_response_with_value(m.0, m.1)
}

pub fn on_translate(text: String, complete_shadow: bool) -> String {
  if text.is_empty() {
    return text;
  }

  let mut translated = text.clone();

  translated = text_only_translater(translated, complete_shadow);

  let vars = get_global_vars();
  if !vars.volatility.inserter_mut().is_ready() {
    error!("on_translate: inserter is not ready");
  }
  vars.volatility.inserter_mut().run(translated)
}

// 参考：http://emily.shillest.net/ayaya/?cmd=read&page=Tips%2FOnTranslate%E3%81%AE%E4%BD%BF%E3%81%84%E6%96%B9&word=OnTranslate
fn text_only_translater(text: String, complete_shadow: bool) -> String {
  static RE_TEXT_ONLY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\\(\\|q\[.*?\]\[.*?\]|[!&8cfijmpqsn]\[.*?\]|[-*+1014567bcehntuvxz]|_[ablmsuvw]\[.*?\]|__(t|[qw]\[.*?\])|_[!?+nqsV]|[sipw][0-9])").unwrap()
  });

  let tags = RE_TEXT_ONLY.find_iter(&text);
  let splitted = RE_TEXT_ONLY.split(&text).collect::<Vec<&str>>();
  let mut result = String::new();

  for (i, tag) in tags.enumerate() {
    result.push_str(translate_part(splitted[i].to_string(), complete_shadow).as_str());
    result.push_str(tag.as_str());
  }
  result
    .push_str(translate_part(splitted[splitted.len() - 1].to_string(), complete_shadow).as_str());

  translate_whole(result)
}

// さくらスクリプトで分割されたテキストに対してそれぞれかける置換処理
fn translate_part(text: String, complete_shadow: bool) -> String {
  static RE_SURFACE_SNIPPET: Lazy<Regex> = Lazy::new(|| Regex::new(r"h([0-9]{7})").unwrap());

  const DEFAULT_Y: i32 = -700;
  const MAX_Y: i32 = -350;
  let bind = if complete_shadow {
    format!(
      "\\0\\![bind,シルエット,黒塗り2,1]\\![anim,offset,800002,0,{}]",
      ((MAX_Y - DEFAULT_Y) as f32
        * (get_global_vars().volatility.immersive_degrees() as f32 / 100.0)) as i32
        + DEFAULT_Y
    )
  } else {
    "\\0\\![bind,シルエット,黒塗り2,0]".to_string()
  };

  let surface_replaced = RE_SURFACE_SNIPPET
    .replace_all(&text, format!("\\0\\s[$1]{}", bind).as_str())
    .to_string();

  let vars = get_global_vars();
  surface_replaced.replace("{user_name}", &vars.user_name().clone().unwrap())
}

fn translate_whole(text: String) -> String {
  static RE_LAST_WAIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\_w\[([0-9]+)\]$").unwrap());

  let mut translated = text.clone();

  const PHI: &str = "φ";
  let replaces = vec![
    Replacee::new("、", "、\\_w[600]", PHI, "", None),
    Replacee::new("。", " \\_w[1200]", PHI, ")）」』", Some(vec![0])),
    Replacee::new("。", "。\\_w[1200]", PHI, ")）」』", Some(vec![1])),
    Replacee::new("！", "！\\_w[1200]", PHI, ")）」』", None),
    Replacee::new("？", "？\\_w[1200]", PHI, ")）」』", None),
    Replacee::new("…", "…\\_w[600]", PHI, ")）」』", None),
    Replacee::new("」", "」\\_w[600]", PHI, ")）", None),
    Replacee::new("』", "』\\_w[600]", PHI, ")）」", None),
    Replacee::new("\\n\\n", "\\n\\n\\_w[700]", PHI, "", None),
  ];
  translated = replace_with_check(&translated, replaces);
  translated = translated.replace(PHI, "");

  translated = RE_LAST_WAIT.replace(&translated, "").to_string();

  // \\Cが含まれているなら文頭に\\Cを補完
  if translated.contains("\\C") {
    translated = format!("\\C{}", translated.replace("\\C", ""));
  }

  translated
}

#[derive(Debug, Clone)]
pub struct Dialog {
  text: String,
  pub scope: usize,
}

impl Display for Dialog {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "\\p[{}]{}", self.scope, self.text)
  }
}

impl Dialog {
  pub fn new(text: String, scope: usize) -> Self {
    Dialog { text, scope }
  }

  pub fn from_text(text: &str) -> Vec<Self> {
    let mut scopes = CHANGE_SCOPE_RE
      .captures_iter(text)
      .map(|c| extract_scope(&c.unwrap()[0]).unwrap())
      .collect::<Vec<_>>();

    let delim = "\x01";
    let pre_texts = CHANGE_SCOPE_RE.replace_all(text, delim);
    let texts = pre_texts.split(delim).collect::<Vec<_>>();

    // 文頭の暗黙的な\\0スコープを補完
    if scopes.len() == texts.len() - 1 {
      scopes.insert(0, 0);
    }

    let mut result = Vec::new();
    for i in 0..texts.len() {
      result.push(Dialog::new(texts[i].to_string(), scopes[i]));
    }
    result
  }
}

struct Replacee {
  old: &'static str,
  new: &'static str,
  exclude_prefix: &'static str,
  exclude_suffix: &'static str,
  scope: Option<Vec<usize>>,
}

impl Replacee {
  fn new(
    old: &'static str,
    new: &'static str,
    exclude_prefix: &'static str,
    exclude_suffix: &'static str,
    scope: Option<Vec<usize>>,
  ) -> Replacee {
    Replacee {
      old,
      new,
      exclude_prefix,
      exclude_suffix,
      scope,
    }
  }

  fn is_in_scope(&self, i: usize) -> bool {
    if let Some(scope) = &self.scope {
      scope.contains(&i)
    } else {
      true
    }
  }

  fn has_prefix(&self, text: &str, cursor: usize) -> bool {
    if cursor > 0 {
      if let Some(prev) = text.chars().nth(cursor - 1) {
        self.exclude_prefix.contains(prev)
      } else {
        false
      }
    } else {
      false
    }
  }

  fn has_suffix(&self, text: &str, cursor: usize) -> bool {
    if let Some(next) = text.chars().nth(cursor + self.old.chars().count()) {
      self.exclude_suffix.contains(next)
    } else {
      false
    }
  }
}

fn replace_with_check(src: &str, replaces: Vec<Replacee>) -> String {
  static QUICK_SECTION_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\\!\[quicksection,true|\\!\[quicksection,1)").unwrap());
  static QUICK_SECTION_END: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\\!\[quicksection,false|\\!\[quicksection,0)").unwrap());
  static WAIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\\_w\[[0-9]+\]|\\w[1-9])").unwrap());

  let mut translated = String::new();

  let lines = Dialog::from_text(src);

  for line in lines {
    let mut in_quicksection = false;
    let text_chars_vec = line
      .to_string()
      .char_indices()
      .collect::<Vec<(usize, char)>>();
    let mut checking_cursor = 0;
    while let Some((j, c)) = text_chars_vec.get(checking_cursor) {
      let text_slice = &line.to_string()[*j..];
      if QUICK_SECTION_START.is_match(text_slice) {
        println!("in quicksection");
        in_quicksection = true;
      } else if QUICK_SECTION_END.is_match(text_slice) {
        println!("out quicksection");
        in_quicksection = false;
      } else if text_slice.starts_with("\\_q") {
        println!("toggle quicksection");
        in_quicksection = !in_quicksection;
      }

      let mut matched_replacee: Option<&Replacee> = None;
      for r in replaces.iter() {
        if line.to_string()[*j..].starts_with(r.old)
          && r.is_in_scope(line.scope)
          && !r.has_prefix(&line.to_string(), checking_cursor)
          && !r.has_suffix(&line.to_string(), checking_cursor)
        {
          matched_replacee = Some(r);
          break;
        }
      }
      if matched_replacee.is_some() {
        let r = matched_replacee.unwrap();
        if in_quicksection {
          println!("removing wait");
          translated.push_str(&WAIT.replace(r.new, ""));
        } else {
          translated.push_str(r.new);
        }
        checking_cursor += r.old.chars().count();
      } else {
        translated.push(*c);
        checking_cursor += 1;
      }
    }
  }
  translated
}

#[test]
fn test_translate() {
  let text = "こんにちは、\\n{user_name}さん。\\nお元気ですか。\\1ええ、私は元気です。\\nあなたはどうですか、ゴースト。\\0\\_q私はほげ。\\n\\nふがふが。".to_string();
  let translated = text_only_translater(text, true);
  println!("{}", translated);
}
