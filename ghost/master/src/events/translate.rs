use crate::autobreakline::{extract_scope, CHANGE_SCOPE_RE};
use crate::variables::get_global_vars;
use core::fmt::{Display, Formatter};
use once_cell::sync::Lazy;
use regex::Regex;

pub fn on_translate(text: String) -> String {
  if text.is_empty() {
    return text;
  }

  let mut translated = text.clone();

  translated = text_only_translater(translated);

  let vars = get_global_vars();
  if vars.volatility.inserter_mut().is_ready() {
    vars.volatility.inserter_mut().run(translated)
  } else {
    translated
  }
}

// 参考：http://emily.shillest.net/ayaya/?cmd=read&page=Tips%2FOnTranslate%E3%81%AE%E4%BD%BF%E3%81%84%E6%96%B9&word=OnTranslate
fn text_only_translater(text: String) -> String {
  let re_tags = Regex::new(r"\\(\\|q\[.*?\]\[.*?\]|[!&8cfijmpqsn]\[.*?\]|[-*+1014567bcehntuvxz]|_[ablmsuvw]\[.*?\]|__(t|[qw]\[.*?\])|_[!?+nqsV]|[sipw][0-9])").unwrap();
  let tags = re_tags.find_iter(&text);
  let splitted = re_tags.split(&text).collect::<Vec<&str>>();
  let mut result = String::new();

  for (i, tag) in tags.enumerate() {
    result.push_str(translate_part(splitted[i].to_string()).as_str());
    result.push_str(tag.as_str());
  }
  result.push_str(translate_part(splitted[splitted.len() - 1].to_string()).as_str());

  translate_whole(result)
}

// さくらスクリプトで分割されたテキストに対してそれぞれかける置換処理
fn translate_part(text: String) -> String {
  let surface_snippet = Regex::new(r"h([0-9]{7})").unwrap();

  let surface_replaced = surface_snippet.replace_all(&text, "\\0\\s[$1]").to_string();

  let vars = get_global_vars();
  surface_replaced.replace("{user_name}", &vars.user_name().clone().unwrap())
}

fn translate_whole(text: String) -> String {
  let last_wait = Regex::new(r"\\_w\[([0-9]+)\]$").unwrap();
  let mut translated = text.clone();

  let phi = "φ";
  let replaces = vec![
    Replacee::new("、", "、\\_w[600]", phi, "", None),
    Replacee::new("。", " \\_w[1200]", phi, ")）」』", Some(vec![0])),
    Replacee::new("。", "。\\_w[1200]", phi, ")）」』", Some(vec![1])),
    Replacee::new("！", "！\\_w[1200]", phi, ")）」』", None),
    Replacee::new("？", "？\\_w[1200]", phi, ")）」』", None),
    Replacee::new("…", "…\\_w[600]", phi, ")）」』", None),
    Replacee::new("」", "」\\_w[600]", phi, ")）", None),
    Replacee::new("』", "』\\_w[600]", phi, ")）」", None),
    Replacee::new("\\n\\n", "\\n\\n\\_w[700]", phi, "", None),
  ];
  translated = replace_with_check(&translated, replaces);
  translated = translated.replace(phi, "");

  translated = last_wait.replace(&translated, "").to_string();

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
    let texts = pre_texts
      .split(delim)
      .filter(|t| !t.is_empty())
      .collect::<Vec<_>>();

    if scopes.len() != texts.len() {
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
  let translated = text_only_translater(text);
  println!("{}", translated);
}
