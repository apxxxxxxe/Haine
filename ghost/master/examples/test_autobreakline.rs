//! 自動改行ロジックのテスト用example
//!
//! 使用方法:
//! ```
//! cd ghost/master
//! cargo run --example test_autobreakline
//! ```

use std::sync::RwLock;
use std::sync::LazyLock;

// Inserterの構造を再現（lib.rsからエクスポートできないため）
mod autobreakline_test {
  use regex::Regex;
  use once_cell::sync::Lazy;
  use std::sync::{Arc, Mutex};
  use std::thread::JoinHandle;
  use vibrato::{Dictionary, Tokenizer};

  static SAKURA_SCRIPT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r###"\\_{0,2}[a-zA-Z0-9*!&](\d|\[("([^"]|\\")+?"|([^\]]|\\\])+?)+?\])?"###).unwrap());

  pub struct Inserter {
    cols_num: f32,
    tokenizer: Arc<Mutex<Option<Tokenizer>>>,
    _join_handle: Option<JoinHandle<()>>,
  }

  impl Inserter {
    pub fn new(cols_num: f32) -> Self {
      // 同期的に辞書を読み込む
      let bytes = include_bytes!("../ipadic-mecab-2_7_0/system.dic.bincode");
      let dict = Dictionary::read(&bytes[..]).unwrap();
      let tokenizer = Tokenizer::new(dict);

      Inserter {
        cols_num,
        tokenizer: Arc::new(Mutex::new(Some(tokenizer))),
        _join_handle: None,
      }
    }

    pub fn run(&mut self, src: String) -> String {
      let tokenizer_clone = self.tokenizer.clone();
      let tokenizer = tokenizer_clone.lock().unwrap();
      let t = tokenizer.as_ref().unwrap();
      let mut worker = t.new_worker();

      // 簡易版の処理: トークン化して表示
      worker.reset_sentence(&src);
      worker.tokenize();

      println!("=== 入力テキスト ===");
      println!("{}", src);
      println!("\n=== 形態素解析結果 ===");
      for token in worker.token_iter() {
        println!("{}: {}", token.surface(), token.feature());
      }

      // 簡易的な改行処理（本番のロジックの動作確認用）
      src
    }

    fn count(&self, text: &str) -> f32 {
      let removed = SAKURA_SCRIPT_RE.replace_all(text, "");
      let mut count = 0.0;
      count += removed.chars().filter(|c| c.is_ascii()).count() as f32 * 0.5;
      count += removed.chars().filter(|c| !c.is_ascii()).count() as f32;
      count
    }
  }
}

fn main() {
  println!("=== 自動改行ロジック テスト ===\n");

  let mut inserter = autobreakline_test::Inserter::new(22.0);

  // テストケース
  let test_cases = vec![
    // 通常の文
    "今日は天気が良いですね。",
    // 長い修飾語句
    "美しい花が咲いている庭を眺めていた。",
    // 接続詞を含む複文
    "雨が降っていたが、しかし散歩に出かけた。",
    // 括弧を含む文
    "「こんにちは」と彼女は言った。",
    // さくらスクリプトを含む文
    "\\0今日は\\w[500]天気が良いですね。\\n明日も晴れるでしょう。",
    // 長い文
    "私は毎日朝早く起きて、ゆっくりと朝食を食べてから、会社に向かう電車に乗ります。",
  ];

  for (i, test) in test_cases.iter().enumerate() {
    println!("\n--- テストケース {} ---", i + 1);
    inserter.run(test.to_string());
    println!();
  }
}
