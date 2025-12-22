use crate::system::autobreakline::{extract_scope, CHANGE_SCOPE_RE};
use crate::system::error::ShioriError;
use crate::system::response::*;
use crate::system::variables::*;
use crate::{lazy_fancy_regex, lazy_regex};
use core::fmt::{Display, Formatter};
use fancy_regex::Regex as FancyRegex;
use once_cell::sync::Lazy;
use regex::Regex;
use shiorust::message::{Request, Response};
use std::thread;
use std::time::Duration;

pub(crate) fn on_wait_translater(_req: &Request) -> Result<Response, ShioriError> {
  while !INSERTER.read().unwrap().is_ready() {
    thread::sleep(Duration::from_millis(100));
  }
  let has_waiting_talk = WAITING_TALK.read().unwrap().is_some();
  let m = if has_waiting_talk {
    WAITING_TALK.read().unwrap().clone().unwrap()
  } else {
    return Err(ShioriError::ArrayAccessError);
  };
  new_response_with_value_with_translate(m.0, m.1)
}

pub(crate) fn on_translate(text: String, complete_shadow: bool) -> Result<String, ShioriError> {
  if text.is_empty() {
    return Ok(text);
  }

  let translated = translate(text, complete_shadow)?;

  let balloonnum_reset = format!("{}{}", REMOVE_BALLOON_NUM, translated);

  if !INSERTER.read().unwrap().is_ready() {
    return Err(ShioriError::TranslaterNotReadyError);
  }
  INSERTER.write().unwrap().run(balloonnum_reset)
}

pub fn translate(text: String, complete_shadow: bool) -> Result<String, ShioriError> {
  static IGNORING_TRANSLATE_RANGE: Lazy<Regex> = lazy_regex!(r"@@@@@(.*?)@@@@@");
  static CHANGE_SCOPE_RE_PREFIX: Lazy<FancyRegex> =
    lazy_fancy_regex!(r"^(\\[01])(?!w)|(\\p\[\d+\])");

  let translate_targets = IGNORING_TRANSLATE_RANGE.split(&text).collect::<Vec<&str>>();
  let ignoring_ranges = IGNORING_TRANSLATE_RANGE
    .captures_iter(&text)
    .map(|c| c.get(1).unwrap().as_str())
    .collect::<Vec<&str>>();

  if translate_targets.len() != 1 || !ignoring_ranges.is_empty() {
    // スコープがわかるように、各テキストの先頭にスコープがついているかチェック
    for v in translate_targets.clone() {
      if !v.is_empty() && !CHANGE_SCOPE_RE_PREFIX.is_match(v).is_ok_and(|v| v) {
        return Err(ShioriError::NotSetScopeError(v.to_string()));
      }
    }
    for v in ignoring_ranges.clone() {
      if !v.is_empty() && !CHANGE_SCOPE_RE_PREFIX.is_match(v).is_ok_and(|v| v) {
        return Err(ShioriError::NotSetScopeError(v.to_string()));
      }
    }
  }

  let mut results = String::new();
  for (i, target) in translate_targets.iter().enumerate() {
    results.push_str(&translate_core(target.to_string(), complete_shadow)?);
    if let Some(v) = ignoring_ranges.get(i) {
      results.push_str(&v.replace("@@@@@", ""));
    }
  }
  Ok(results)
}

fn translate_core(text: String, complete_shadow: bool) -> Result<String, ShioriError> {
  static RE_SURFACE_SNIPPET: Lazy<Regex> = lazy_regex!(r"h(r)?([0-9]{7})");

  let text = RE_SURFACE_SNIPPET
    .replace_all(&text, |caps: &regex::Captures| {
      let use_half_blink = caps.get(1).is_some();
      let surface_id = caps.get(2).unwrap().as_str();
      format!(
        "\\0\\![embed,OnSmoothBlink,{},{},{}]",
        surface_id, complete_shadow as i32, use_half_blink as i32,
      )
    })
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

static QUICK_SECTION_START: Lazy<Regex> =
  lazy_regex!(r"^(\\!\[quicksection,true]|\\!\[quicksection,1])");
static QUICK_SECTION_END: Lazy<Regex> =
  lazy_regex!(r"^(\\!\[quicksection,false]|\\!\[quicksection,0])");

// 参考：http://emily.shillest.net/ayaya/?cmd=read&page=Tips%2FOnTranslate%E3%81%AE%E4%BD%BF%E3%81%84%E6%96%B9&word=OnTranslate
static RE_TEXT_ONLY: Lazy<Regex> = lazy_regex!(
  r"\\(\\|q\[.*?\]\[.*?\]|[!&8bcfijmpqsn]\[.*?\]|[-*+1014567bcehntuvxz]|_[ablmsuvw]\[.*?\]|__(t|[qw]\[.*?\])|_[!?+nqsV]|[sipw][0-9])"
);

// さくらスクリプトで分割されたテキストに対してそれぞれかける置換処理
fn translate_dialog(dialog: &mut Dialog) {
  let tags = RE_TEXT_ONLY
    .find_iter(&dialog.text)
    .map(|m| m.as_str())
    .collect::<Vec<&str>>();
  let splitted_texts = RE_TEXT_ONLY.split(&dialog.text).collect::<Vec<&str>>();

  const PHI: &str = "φ";
  let replaces = [
    Replacee::new("、", "、\\_w[600]", PHI, "", None),
    Replacee::new("。", " \\_w[1200]", PHI, ")）」』", Some(vec![0])),
    Replacee::new("。", "。\\_w[1200]", PHI, ")）」』", Some(vec![1])),
    Replacee::new("！", "！\\_w[1200]", PHI, ")）」』", None),
    Replacee::new("？", "？\\_w[1200]", PHI, ")）」』", None),
    Replacee::new(
      "…",
      "\\![quicksection,1]…\\_w[600]\\![quicksection,0]",
      PHI,
      ")）」』",
      None,
    ),
    Replacee::new("」", "」\\_w[600]", PHI, ")）", None),
    Replacee::new("』", "』\\_w[600]", PHI, ")）」", None),
  ];

  // tagsのquicksection状態を事前計算
  let mut in_quicksection_states: Vec<bool> = Vec::new();
  let mut in_qs = false;
  for tag in tags.iter() {
    in_quicksection_states.push(in_qs);
    if QUICK_SECTION_START.is_match(tag) {
      in_qs = true;
    } else if QUICK_SECTION_END.is_match(tag) {
      in_qs = false;
    } else if *tag == "\\_q" {
      in_qs = !in_qs;
    }
  }

  // tagsの置換を適用
  let replaced_tags = replace_tags(&tags, &splitted_texts, &in_quicksection_states);

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
    if let Some(t) = replaced_tags.get(i) {
      result.push_str(t);
    }
  }

  dialog.text = result;
}

fn translate_whole(text: String) -> Result<String, ShioriError> {
  static RE_LAST_WAIT: Lazy<Regex> = lazy_regex!(r"\\_w\[([0-9]+)\]$");

  let mut translated = text.clone();

  translated = RE_LAST_WAIT.replace(&translated, "").to_string();

  let user_name = USER_NAME.read().unwrap().clone();
  translated = translated.replace("{user_name}", &user_name);

  Ok(translated)
}

#[derive(Debug, Clone)]
pub(crate) struct Dialog {
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
      while i + 1 < texts.len() && scope == scopes[i + 1] && !text.contains("\\x") {
        text.push_str(texts[i + 1]);
        i += 1;
      }
      result.push(Dialog::new(text, scope));
      i += 1;
    }
    result
  }

  fn render(&self, is_first: bool) -> String {
    let text = if is_first {
      self.text.clone()
    } else {
      self.to_string()
    };
    format!("\\![quicksection,0]{}", text)
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

struct TagReplacee {
  patterns: &'static [&'static str],
  replacement: &'static str,
}

static WAIT: Lazy<Regex> = lazy_regex!(r"(\\_w\[[0-9]+\]|\\w[1-9])");

fn replace_tags(
  tags: &[&str],
  splitted_texts: &[&str],
  in_quicksection_states: &[bool],
) -> Vec<String> {
  static TAG_REPLACES: &[TagReplacee] = &[TagReplacee {
    patterns: &["\\n", "\\n[half]"],
    replacement: "\\n\\n[half]\\_w[700]",
  }];

  let mut result: Vec<String> = Vec::new();
  let mut i = 0;
  while i < tags.len() {
    let mut matched = false;
    for r in TAG_REPLACES.iter() {
      let pattern_len = r.patterns.len();
      if i + pattern_len <= tags.len() {
        let slice = &tags[i..i + pattern_len];
        // パターンが一致し、かつ間のsplitted_textsが全て空文字（元テキストで連続）
        let is_contiguous = (1..pattern_len)
          .all(|offset| splitted_texts.get(i + offset).is_some_and(|s| s.is_empty()));
        if slice == r.patterns && is_contiguous {
          let in_qs = in_quicksection_states.get(i).copied().unwrap_or(false);
          let replaced = if in_qs {
            WAIT.replace_all(r.replacement, "").to_string()
          } else {
            r.replacement.to_string()
          };
          result.push(replaced);
          for _ in 1..pattern_len {
            result.push(String::new());
          }
          i += pattern_len;
          matched = true;
          break;
        }
      }
    }
    if !matched {
      result.push(tags[i].to_string());
      i += 1;
    }
  }
  result
}

fn replace_with_check(
  text_part: &str,
  scope: usize,
  replaces: &[Replacee],
  in_quicksection: bool,
) -> String {
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
    if let Some(v) = matched_replacee {
      let r = v;
      if in_quicksection {
        println!("removing wait and additional quicksection");
        let mut s = r.new.to_string();
        for reg in [&WAIT, &QUICK_SECTION_START, &QUICK_SECTION_END].iter() {
          s = reg.replace_all(&s, "").to_string();
        }
        translated.push_str(&s);
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

// 1文字ずつ\\_qで囲めば口パクしなくなる
pub(crate) fn replace_dialog_for_nomouthmove(text: String) -> Result<String, ShioriError> {
  let text = translate(text, true)?;
  let split_parts: Vec<&str> = RE_TEXT_ONLY.split(&text).collect();
  let mut matches: Vec<String> = Vec::new();
  for cap in RE_TEXT_ONLY.find_iter(&text) {
    matches.push(cap.as_str().to_string());
  }

  let mut result = String::new();
  for (i, splitted) in split_parts.iter().enumerate() {
    result.push_str(
      splitted
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("\\_w[50]\\![quicksection,0]\\![quicksection,1]")
        .as_str(),
    );
    if let Some(m) = matches.get(i) {
      result.push_str(m);
    }
  }
  Ok(format!(
    "\\![quicksection,1]@@@@@{}\\![quicksection,0]@@@@@",
    result
  ))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_replace_tags_contiguous_pattern() {
    // \\n\\n[half] が連続している場合 → 置換される
    let tags = vec!["\\n", "\\n[half]"];
    let splitted_texts = vec!["あいう", "", "えお"];
    let in_quicksection_states = vec![false, false];

    let result = replace_tags(&tags, &splitted_texts, &in_quicksection_states);

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "\\n\\n[half]\\_w[700]");
    assert_eq!(result[1], ""); // 2番目は空文字
  }

  #[test]
  fn test_replace_tags_non_contiguous_pattern() {
    // \\n と \\n[half] の間にテキストがある場合 → 置換されない
    let tags = vec!["\\n", "\\n[half]"];
    let splitted_texts = vec!["あいう", "かきく", "えお"];
    let in_quicksection_states = vec![false, false];

    let result = replace_tags(&tags, &splitted_texts, &in_quicksection_states);

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "\\n");
    assert_eq!(result[1], "\\n[half]");
  }

  #[test]
  fn test_replace_tags_in_quicksection() {
    // quicksection内の場合 → ウェイトが除去される
    let tags = vec!["\\n", "\\n[half]"];
    let splitted_texts = vec!["あいう", "", "えお"];
    let in_quicksection_states = vec![true, true];

    let result = replace_tags(&tags, &splitted_texts, &in_quicksection_states);

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "\\n\\n[half]"); // ウェイトが除去される
    assert_eq!(result[1], "");
  }

  #[test]
  fn test_replace_tags_no_match() {
    // パターンにマッチしない場合 → そのまま
    let tags = vec!["\\n", "\\s[10]"];
    let splitted_texts = vec!["あいう", "", "えお"];
    let in_quicksection_states = vec![false, false];

    let result = replace_tags(&tags, &splitted_texts, &in_quicksection_states);

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "\\n");
    assert_eq!(result[1], "\\s[10]");
  }

  #[test]
  fn test_replace_tags_single_element() {
    // 単一要素の場合
    let tags = vec!["\\n[half]"];
    let splitted_texts = vec!["あいう", "えお"];
    let in_quicksection_states = vec![false];

    let result = replace_tags(&tags, &splitted_texts, &in_quicksection_states);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "\\n[half]");
  }

  #[test]
  fn test_replace_tags_empty() {
    // 空の場合
    let tags: Vec<&str> = vec![];
    let splitted_texts: Vec<&str> = vec!["テスト"];
    let in_quicksection_states: Vec<bool> = vec![];

    let result = replace_tags(&tags, &splitted_texts, &in_quicksection_states);

    assert_eq!(result.len(), 0);
  }

  #[test]
  fn test_replace_tags_multiple_patterns() {
    // 複数のパターンが連続する場合
    let tags = vec!["\\n", "\\n[half]", "\\n", "\\n[half]"];
    let splitted_texts = vec!["あ", "", "い", "", "う"];
    let in_quicksection_states = vec![false, false, false, false];

    let result = replace_tags(&tags, &splitted_texts, &in_quicksection_states);

    assert_eq!(result.len(), 4);
    assert_eq!(result[0], "\\n\\n[half]\\_w[700]");
    assert_eq!(result[1], "");
    assert_eq!(result[2], "\\n\\n[half]\\_w[700]");
    assert_eq!(result[3], "");
  }
}
