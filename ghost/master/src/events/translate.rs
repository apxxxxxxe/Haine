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

  let translated = translate(text, complete_shadow);

  let vars = get_global_vars();
  if !vars.volatility.inserter_mut().is_ready() {
    error!("on_translate: inserter is not ready");
  }
  vars.volatility.inserter_mut().run(translated)
}

fn translate(text: String, complete_shadow: bool) -> String {
  static RE_SURFACE_SNIPPET: Lazy<Regex> = Lazy::new(|| Regex::new(r"h([0-9]{7})").unwrap());

  let text = RE_SURFACE_SNIPPET
    .replace_all(
      &text,
      format!(
        "\\0\\![embed,OnSmoothBlink,$1,{}]",
        if complete_shadow { 1 } else { 0 }
      )
      .as_str(),
    )
    .to_string();

  let mut dialogs = Dialog::from_text(&text);

  for dialog in dialogs.iter_mut() {
    translate_dialog(dialog);
  }

  translate_whole(
    dialogs
      .iter()
      .enumerate()
      .map(|(i, d)| d.render(i == 0))
      .collect::<Vec<String>>()
      .join(""),
  )
}

// さくらスクリプトで分割されたテキストに対してそれぞれかける置換処理
fn translate_dialog(dialog: &mut Dialog) {
  // 参考：http://emily.shillest.net/ayaya/?cmd=read&page=Tips%2FOnTranslate%E3%81%AE%E4%BD%BF%E3%81%84%E6%96%B9&word=OnTranslate
  static RE_TEXT_ONLY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\\(\\|q\[.*?\]\[.*?\]|[!&8cfijmpqsn]\[.*?\]|[-*+1014567bcehntuvxz]|_[ablmsuvw]\[.*?\]|__(t|[qw]\[.*?\])|_[!?+nqsV]|[sipw][0-9])").unwrap()
  });

  let tags = RE_TEXT_ONLY
    .find_iter(&dialog.text)
    .map(|m| m.as_str())
    .collect::<Vec<&str>>();
  let splitted_texts = RE_TEXT_ONLY.split(&dialog.text).collect::<Vec<&str>>();

  static QUICK_SECTION_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\\!\[quicksection,true|\\!\[quicksection,1)").unwrap());
  static QUICK_SECTION_END: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\\!\[quicksection,false|\\!\[quicksection,0)").unwrap());
  const PHI: &str = "φ";
  let replaces = [
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
  let mut result = String::new();
  let mut in_quicksection = false;
  for (i, splitted) in splitted_texts.iter().enumerate() {
    let tag = tags.get(i).unwrap_or(&"");
    result.push_str(
      &replace_with_check(splitted, dialog.scope, &replaces, in_quicksection).replace(PHI, ""),
    );

    if QUICK_SECTION_START.is_match(tag) {
      println!("in quicksection");
      in_quicksection = true;
    } else if QUICK_SECTION_END.is_match(tag) {
      println!("out quicksection");
      in_quicksection = false;
    } else if *tag == "\\_q" {
      println!("toggle quicksection");
      in_quicksection = !in_quicksection;
    }
    result.push_str(tag);
  }

  dialog.text = result;
}

fn translate_whole(text: String) -> String {
  static RE_LAST_WAIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\_w\[([0-9]+)\]$").unwrap());

  let mut translated = text.clone();

  translated = RE_LAST_WAIT.replace(&translated, "").to_string();

  let vars = get_global_vars();
  translated = translated.replace("{user_name}", &vars.user_name().clone().unwrap());

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
    let mut i = 0;
    while i < texts.len() {
      let mut text = texts[i].to_string();
      let scope = scopes[i];
      while i + 1 < texts.len() && scope == scopes[i + 1] {
        text.push_str(texts[i + 1]);
        i += 1;
      }
      result.push(Dialog::new(text, scope));
      i += 1;
    }
    result
  }

  fn render(&self, is_first: bool) -> String {
    if is_first {
      self.text.clone()
    } else {
      self.to_string()
    }
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

fn replace_with_check(
  text_part: &str,
  scope: usize,
  replaces: &[Replacee],
  in_quicksection: bool,
) -> String {
  static WAIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\\_w\[[0-9]+\]|\\w[1-9])").unwrap());

  let mut translated = String::new();

  let text_chars_vec = text_part.char_indices().collect::<Vec<(usize, char)>>();
  let mut checking_cursor = 0;
  while let Some((j, c)) = text_chars_vec.get(checking_cursor) {
    let text_slice = &text_part[*j..];

    let mut matched_replacee: Option<&Replacee> = None;
    for r in replaces.iter() {
      if text_slice.starts_with(r.old)
        && r.is_in_scope(scope)
        && !r.has_prefix(text_part, checking_cursor)
        && !r.has_suffix(text_part, checking_cursor)
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
  translated
}

#[test]
fn test_translate() {
  let text = "\
    \\_q\\0これは、0のセリフ。\\_qこれはウェイト付きで置換されるべき文章。\\0なお置換されるべき文章。\\n\
    \\1これは、1のセリフ。\\n\
    \\0そして、0のセリフ。\\n\
    "
  .to_string();
  let translated = translate(text, true);
  println!("{}", translated);
}
