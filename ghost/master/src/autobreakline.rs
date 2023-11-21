use core::cmp::Ord;
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use vibrato::{Dictionary, Tokenizer};

static SAKURA_SCRIPT_RE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r###"\\_{0,2}[a-zA-Z0-9*!&](\d|\[("([^"]|\\")+?"|([^\]]|\\\])+?)+?\])?"###).unwrap()
});

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Rank {
  Break,
  Append,
  Normal,
}

pub struct Inserter {
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

  pub fn is_ready(&mut self) -> bool {
    let tokenizer_clone = self.tokenizer.clone();
    let tokenizer = tokenizer_clone.lock().unwrap();
    tokenizer.is_some()
  }

  pub fn start_init(&mut self) {
    self.tokenizer = Arc::new(Mutex::new(None));
    let tokenizer_clone = self.tokenizer.clone();
    self.join_handle = Some(std::thread::spawn(move || {
      let bytes = include_bytes!("../ipadic-mecab-2_7_0/system.dic.zst").to_vec();
      let reader = zstd::Decoder::with_buffer(&bytes[..]).unwrap();
      let dict = Dictionary::read(reader).unwrap();
      *tokenizer_clone.lock().unwrap() = Some(Tokenizer::new(dict));
    }));
  }

  pub fn default() -> Self {
    Self::new(24.0)
  }

  pub fn run(&mut self, src: String) -> String {
    let parts = self.wakachi(src);
    self.render(parts)
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

  fn wakachi(&mut self, src: String) -> Vec<String> {
    let tokenizer_clone = self.tokenizer.clone();
    let tokenizer = tokenizer_clone.lock().unwrap();
    let t = tokenizer.as_ref().unwrap();
    let mut worker = t.new_worker();
    let mut text = src.clone();
    let mut _word_counts = vec![0, 0];

    // 候補: これが含まれていたら改行
    let pos_to_break = vec![
      // "動詞,自立",
      // "動詞,接尾",
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
    let pos_to_append = vec![
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

    let pos_combinations = vec![("動詞,自立", "動詞,自立"), ("助詞,格助詞", "助詞,係助詞")];

    let mut results: Vec<String> = Vec::new();
    let mut result = "".to_string();
    let delim_re = "\x1f$1";
    let delim = "\x1f";
    let line_splitter = Regex::new(r"(\\n|\\_l\[0|\\x|\\c|\\[01]|\\p\[\d+\])").unwrap();
    text = line_splitter.replace_all(&text, delim_re).to_string();
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
            .find(|&&p| token.feature().find(p).is_some())
            .is_some();
          let contains_pos_to_append = pos_to_append
            .iter()
            .find(|&&p| token.feature().find(p).is_some())
            .is_some();
          let contains_pos_combos = last_token_feature.is_some()
            && pos_combinations
              .iter()
              .find(|&&(a, b)| {
                last_token_feature.as_ref().unwrap().find(a).is_some()
                  && token.feature().find(b).is_some()
              })
              .is_some();
          let contains_pos_to_break = pos_to_break
            .iter()
            .find(|&&p| token.feature().find(p).is_some())
            .is_some();

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
              if results.len() > 0 {
                let last = results.iter().rposition(|r| !r.is_empty()).unwrap();
                // results[last].text += &format!("#({})", token.surface());
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
    }
    if !result.is_empty() {
      let last = results.iter().rposition(|r| !r.is_empty()).unwrap();
      results[last] += &result;
    }

    println!("results: {}", results.len());
    for (i, r) in results.iter().enumerate() {
      println!("{}: {}", i, r);
    }

    results
  }

  fn render(&mut self, parts: Vec<String>) -> String {
    let re_open_bracket = Regex::new(r"[「『（【]").unwrap();
    let re_close_bracket = Regex::new(r"[」』）】]").unwrap();
    let re_periods = Regex::new(r"[、。！？]").unwrap();
    let re_change_scope = Regex::new(r"(\\[01][^w]?|\\p\[\d+\])").unwrap();
    let re_not_number = Regex::new(r"[^\d]").unwrap();
    let re_change_line = Regex::new(r"(\\n|\\_l\[0[,0-9em%]+\]|\\x|\\c)").unwrap();
    let mut result = String::new();
    let mut counts = vec![0.0, 0.0];
    let mut i = 0;
    let mut scope: usize = 0;
    let mut brackets_depth: i32 = 0;
    loop {
      if i >= parts.len() {
        break;
      }
      let part = parts[i].clone();
      let c = self.count(part.to_string());
      brackets_depth += re_open_bracket.find_iter(&part).count() as i32;
      brackets_depth -= (re_close_bracket.find_iter(&part).count() as i32).max(0);

      if re_change_scope.is_match(&part) {
        let c = re_change_scope.captures(&part).unwrap()[0].to_string();
        scope = re_not_number.replace_all(&c, "").parse::<usize>().unwrap();
      }

      if re_change_line.is_match(&part) {
        counts[scope] = 0.0;
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
      if re_periods.is_match(&part) && brackets_depth == 0 {
        let mut j = i + 1;
        let mut next_line = String::new();
        while j < parts.len() {
          let next = parts[j].clone();
          if re_change_scope.is_match(&next) || re_change_line.is_match(&next) {
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::events::aitalk::TALKS;
  use crate::events::translate::on_translate;
  use rand::seq::SliceRandom;

  #[test]
  fn inserter() {
    let mut talks = TALKS.clone();
    talks.shuffle(&mut rand::thread_rng());
    let mut ins = Inserter::default();
    ins.start_init();
    while !ins.is_ready() {
      std::thread::sleep(std::time::Duration::from_millis(100));
    }
    for (i, t) in talks.iter().enumerate() {
      println!("talk: {}", i);
      let text = on_translate(t.to_string());
      ins.tokenize(text.clone());
      let breaked = ins.run(text).replace("\\n", "\n");
      let result = SAKURA_SCRIPT_RE.replace_all(&breaked, "");
      println!("\n{}\n", result);
    }
  }
}
