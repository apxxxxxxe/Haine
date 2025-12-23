use crate::system::error::ShioriError;
use crate::{lazy_fancy_regex, lazy_regex};
use core::cmp::Ord;
use fancy_regex::Regex as FancyRegex;
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use vibrato::{Dictionary, Tokenizer};

static SAKURA_SCRIPT_RE: Lazy<Regex> =
  lazy_regex!(r###"\\_{0,2}[a-zA-Z0-9*!&](\d|\[("([^"]|\\")+?"|([^\]]|\\\])+?)+?\])?"###);

pub(crate) static CHANGE_SCOPE_RE: Lazy<FancyRegex> =
  lazy_fancy_regex!(r"(\\[01])(?!w)|(\\p\[\d+\])");

fn find_change_scope(text: &str) -> Option<String> {
  if let Ok(Some(captures)) = CHANGE_SCOPE_RE.captures(text) {
    println!("captures: {:?}", captures);
    if let Some(scope) = captures.get(1) {
      return Some(scope.as_str().to_string());
    } else if let Some(scope) = captures.get(2) {
      return Some(scope.as_str().to_string());
    }
  }
  None
}

pub(crate) fn extract_scope(text: &str) -> Option<usize> {
  static RE_NOT_NUMBER: Lazy<Regex> = lazy_regex!(r"[^\d]");

  if let Some(scope_tag) = find_change_scope(text) {
    debug!("scope_tag: {}", scope_tag);
    if let Ok(s) = RE_NOT_NUMBER.replace_all(&scope_tag, "").parse::<usize>() {
      debug!("scope: {}", s);
      return Some(s);
    }
  }
  None
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub(crate) enum Rank {
  Break,
  Append,
  Normal,
}

pub(crate) struct Inserter {
  cols_num: f32,
  tokenizer: Arc<Mutex<Option<Tokenizer>>>,
  join_handle: Option<JoinHandle<()>>,
}

impl Inserter {
  pub fn new(cols_num: f32) -> Self {
    Inserter {
      cols_num,
      tokenizer: Arc::new(Mutex::new(None)),
      join_handle: None,
    }
  }

  pub fn is_ready(&self) -> bool {
    let tokenizer_clone = self.tokenizer.clone();
    let tokenizer = if let Ok(v) = tokenizer_clone.lock() {
      v
    } else {
      return false;
    };
    tokenizer.is_some()
  }

  pub fn start_init(&mut self) {
    self.tokenizer = Arc::new(Mutex::new(None));
    let tokenizer_clone = self.tokenizer.clone();
    self.join_handle = Some(std::thread::spawn(move || {
      // Use pre-compiled bincode dictionary for faster initialization
      let bytes = include_bytes!("../../ipadic-mecab-2_7_0/system.dic.bincode");
      let dict = Dictionary::read(&bytes[..]).unwrap();
      *tokenizer_clone.lock().unwrap() = Some(Tokenizer::new(dict));
    }));
  }

  #[allow(dead_code)]
  pub fn default() -> Self {
    Self::new(24.0) // 24文字で改行: SSPデフォルト+に合わせた値
  }

  pub fn run(&mut self, src: String) -> Result<String, ShioriError> {
    let parts = self.wakachi(src)?;
    let mut lines = self
      .render(parts)
      .split("\\n")
      .map(|s| s.to_string())
      .collect::<Vec<String>>();
    self.remove_last_whitespace(lines.as_mut());
    Ok(lines.join("\\n"))
  }

  #[allow(dead_code)]
  pub fn tokenize(&mut self, src: String) {
    let tokenizer_clone = self.tokenizer.clone();
    let tokenizer = tokenizer_clone.lock().unwrap();
    let t = tokenizer.as_ref().unwrap();
    let mut worker = t.new_worker();
    worker.reset_sentence(&src);
    worker.tokenize();
    for token in worker.token_iter() {
      println!("{}: {}", token.surface(), token.feature());
    }
  }

  fn wakachi(&mut self, src: String) -> Result<Vec<String>, ShioriError> {
    static RE_LINE_SPLITTER: Lazy<Regex> = lazy_regex!(r"(\\n|\\_l\[0|\\x|\\c)");

    let tokenizer_clone = self.tokenizer.clone();
    let tokenizer = tokenizer_clone.lock().unwrap();
    let t = tokenizer.as_ref().unwrap();
    let mut worker = t.new_worker();
    let mut text = src.clone();
    let mut _word_counts = [0, 0];

    // 候補: これが含まれていたら改行
    let pos_to_break = [
      "形容詞,自立",
      "形容詞,接尾",
      "形容詞,非自立",
      "副詞,一般",
      "副詞,助詞類接続",
      "接続詞,",
      "助詞,係助詞",
      "助詞,終助詞",
      "助詞,副詞化",
      "記号,句点",
      "記号,読点",
      "名詞,副詞可能",
      "助詞,格助詞",
      "助詞,接続助詞",
      "助詞,副助詞",
      "助詞,副助詞／並立助詞／終助詞",
      "助詞,並立助詞",
      "助詞,連体化",
      "記号,括弧閉",
      "フィラー",
    ];

    // forbid: ただし、これが含まれていたら改行しない
    let pos_forbidden_to_break = ["記号,一般,", "未然形,"];

    // add_before: これが含まれていたら、前の行に追加
    let pos_to_append = [
      "非自立,",
      "接続助詞,",
      "助詞,",
      "句点,",
      "読点,",
      "特殊・タ",
      "特殊・デス",
      "体言接続,",
      "特殊・ダ,仮定形,",
    ];

    let pos_combinations = [("動詞,自立", "動詞,自立"), ("助詞,格助詞", "助詞,係助詞")];

    let mut results: Vec<String> = Vec::new();
    let mut result = "".to_string();
    let delim_re = "\x1f$1";
    let delim = "\x1f";
    text = RE_LINE_SPLITTER.replace_all(&text, delim_re).to_string();
    let lines = text
      .split(delim)
      .map(|s| s.to_string())
      .collect::<Vec<String>>();

    for line in lines {
      let mut sakura_scripts = SAKURA_SCRIPT_RE.find_iter(&line);
      let ss_splitted = SAKURA_SCRIPT_RE.split(&line).collect::<Vec<&str>>();
      let mut last_token_feature: Option<String> = None;
      for pieces in ss_splitted {
        worker.reset_sentence(pieces);
        worker.tokenize();
        for token in worker.token_iter() {
          let contains_forbidden_pos = pos_forbidden_to_break
            .iter()
            .any(|&p| token.feature().contains(p));
          let contains_pos_to_append = pos_to_append.iter().any(|&p| token.feature().contains(p));
          let contains_pos_combos = last_token_feature.is_some()
            && pos_combinations.iter().any(|&(a, b)| {
              last_token_feature.as_ref().unwrap().contains(a) && token.feature().contains(b)
            });
          let contains_pos_to_break = pos_to_break.iter().any(|&p| token.feature().contains(p));

          let rank = if contains_forbidden_pos {
            Rank::Normal
          } else if contains_pos_to_append || contains_pos_combos {
            if !SAKURA_SCRIPT_RE.replace(&result, "").is_empty() {
              Rank::Break
            } else {
              Rank::Append
            }
          } else if contains_pos_to_break {
            Rank::Break
          } else {
            Rank::Normal
          };

          result += token.surface();
          match rank {
            Rank::Append => {
              if !results.is_empty() {
                let last = results.iter().rposition(|r| !r.is_empty()).unwrap();
                println!("append: {}", &result);
                results[last] += &result;
                result = "".to_string();
              } else {
                panic!("results is empty");
              }
            }
            Rank::Break => {
              println!("push: {}", &result);
              results.push(result);
              result = "".to_string();
            }
            Rank::Normal => {
              println!("normal: ({})", &result);
            }
          }
          last_token_feature = Some(token.feature().to_string());
        }
        if let Some(s) = sakura_scripts.next() {
          result += s.as_str();
        }
      }
      if !result.is_empty() {
        results.push(result);
        result = "".to_string();
      }
    }
    if !result.is_empty() {
      match results.iter().rposition(|r| !r.is_empty()) {
        Some(last) => {
          results[last] += &result;
        }
        None => {
          results.push(result);
        }
      }
    }

    println!("results: {}", results.len());
    for (i, r) in results.iter().enumerate() {
      println!("{}: {}", i, r);
    }

    Ok(results)
  }

  fn remove_last_whitespace(&mut self, parts: &mut [String]) {
    for part in parts.iter_mut() {
      while SAKURA_SCRIPT_RE
        .replace_all(part, "")
        .chars()
        .last()
        .is_some_and(|c| c.is_whitespace())
      {
        let mut is_whitespace_skipped = false;
        let mut chars: Vec<char> = Vec::new();
        for c in part.chars().rev().collect::<String>().chars() {
          if c.is_whitespace() && !is_whitespace_skipped {
            is_whitespace_skipped = true;
            continue;
          }
          chars.push(c);
        }
        *part = chars.iter().rev().collect::<String>();
      }
    }
  }

  fn render(&mut self, parts: Vec<String>) -> String {
    static RE_OPEN_BRACKET: Lazy<Regex> = lazy_regex!(r"[「『（【]");
    static RE_CLOSE_BRACKET: Lazy<Regex> = lazy_regex!(r"[」』）】]");
    static RE_PERIODS: Lazy<Regex> = lazy_regex!(r"[、。！？]");
    static RE_CHANGE_LINE: Lazy<Regex> = lazy_regex!(r"(\\n|\\_l\[0[,0-9em%]+\]|\\c)");
    static RE_NEW_PAGE: Lazy<FancyRegex> = lazy_fancy_regex!(r"\\x(?!\[noclear\])");

    let mut result = String::new();
    let mut counts = [0.0; 10]; // 外部スクリプトを見越して多めに10個用意しておく
    let mut i = 0;
    let mut scope: usize = 0;
    let mut brackets_depth: i32 = 0;
    loop {
      if i >= parts.len() {
        break;
      }
      let part = parts[i].clone();
      debug!("part: {}", part);
      let c = self.count(part.to_string());
      brackets_depth += RE_OPEN_BRACKET.find_iter(&part).count() as i32;
      brackets_depth -= (RE_CLOSE_BRACKET.find_iter(&part).count() as i32).max(0);

      if let Some(s) = extract_scope(&part) {
        scope = s;
      }

      if RE_CHANGE_LINE.is_match(&part) {
        counts[scope] = 0.0;
      }
      if RE_NEW_PAGE.is_match(&part).is_ok_and(|r| r) {
        // \\xが来たら全てリセット
        // TODO: continueせず以下の処理も通すべき？
        result.push_str(&part);
        counts = [0.0; 10];
        scope = 0;
        i += 1;
        continue;
      }

      if c > self.cols_num {
        result.push_str(&part);
        counts[scope] += c % self.cols_num;
        counts[scope] %= self.cols_num;
        i += 1;
        continue;
      }
      let after_counts = counts[scope] + c;
      if after_counts > self.cols_num {
        // result.push_str(&format!("f{}\\n", counts[scope]));
        result.push_str("\\n");
        counts[scope] = 0.0;
        continue;
      }
      counts[scope] += c;
      result.push_str(&part);

      // 句読点後の文章が1行に収まるなら、一気に出力して次へ
      // ただし括弧内では実行しない ぶつ切りの引用文とか台詞はなんか変な感じがするので
      if RE_PERIODS.is_match(&part) && brackets_depth == 0 {
        let mut j = i + 1;
        let mut next_line = String::new();
        while j < parts.len() {
          let next = parts[j].clone();
          if CHANGE_SCOPE_RE.is_match(&next).is_ok_and(|r| r)
            || RE_CHANGE_LINE.is_match(&next)
            || RE_NEW_PAGE.is_match(&next).is_ok_and(|r| r)
          {
            j -= 1;
            break;
          }
          next_line.push_str(&next);
          j += 1;
        }

        let next_word_count = self.count(next_line.clone());
        if next_word_count <= self.cols_num {
          // 句読点の後に改行を入れるのは、句読点の後の文章が1行に収まらない場合のみ
          if counts[scope] + next_word_count > self.cols_num {
            // result.push_str("@\\n");
            result.push_str("\\n");
            counts[scope] = 0.0;
          } else {
            // result.push_str(";");
          }
          result.push_str(&next_line);
          counts[scope] += next_word_count;
          i = j;
        }
      }
      i += 1;
    }
    result
  }

  fn count(&self, text: String) -> f32 {
    let removed = SAKURA_SCRIPT_RE.replace_all(&text, "");
    let mut count = 0.0;
    count += removed.chars().filter(|c| c.is_ascii()).count() as f32 * 0.5;
    count += removed.chars().filter(|c| !c.is_ascii()).count() as f32;
    count
  }
}
