//! 自動改行処理モジュール
//!
//! テキストを指定された行幅に収まるよう自動的に改行を挿入する。
//! さくらスクリプトを保護しつつ、句読点やダッシュなどの意味的区切りで
//! 優先的に改行を行う。

use crate::system::error::ShioriError;
use crate::{lazy_fancy_regex, lazy_regex};
use fancy_regex::Regex as FancyRegex;
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use vibrato::{Dictionary, Tokenizer};

// ============================================================
// 正規表現定義
// ============================================================

static SAKURA_SCRIPT_RE: Lazy<Regex> =
  lazy_regex!(r###"\\_{0,2}[a-zA-Z0-9*!&](\d|\[("([^"]|\\")+?"|([^\]]|\\\])+?)+?\])?"###);

pub(crate) static CHANGE_SCOPE_RE: Lazy<FancyRegex> =
  lazy_fancy_regex!(r"(\\[01])(?!w)|(\\p\[\d+\])");

/// 段落区切り（\\n\\n[half]...）
static PARAGRAPH_DELIMITER_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(\\n\\n\[half\](\\_w\[\d+\])?)").unwrap());

// ============================================================
// スコープ処理（既存APIの維持）
// ============================================================

fn find_change_scope(text: &str) -> Option<String> {
  if let Ok(Some(captures)) = CHANGE_SCOPE_RE.captures(text) {
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

// ============================================================
// 改行優先度定義
// ============================================================

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy)]
pub enum BreakPriority {
  None = 0,     // 改行不可
  Low = 1,      // 助詞の後など
  Medium = 2,   // 読点「、」の後
  High = 3,     // 句点「。」「！」「？」の後
  VeryHigh = 4, // 閉じ括弧「」」の後、ダッシュ
}

/// 文字数をカウント（ASCII: 0.5, 非ASCII: 1.0）
pub fn count_chars(text: &str) -> f32 {
  let removed = SAKURA_SCRIPT_RE.replace_all(text, "");
  let ascii = removed.chars().filter(|c| c.is_ascii()).count() as f32 * 0.5;
  let non_ascii = removed.chars().filter(|c| !c.is_ascii()).count() as f32;
  ascii + non_ascii
}

// ============================================================
// トークン処理
// ============================================================

#[derive(Debug, Clone)]
pub struct Token {
  pub text: String,
  pub width: f32,
  pub break_priority: BreakPriority,
  pub is_sakura_script: bool,
}

/// トークンの改行優先度を決定
fn get_break_priority(surface: &str, feature: &str) -> BreakPriority {
  // ダッシュ（意味的区切り）は最優先
  // MeCabがダッシュを分割する場合も考慮して、単体でもVeryHigh
  if surface.contains('─') || surface.contains('―') || surface.contains('—') {
    return BreakPriority::VeryHigh;
  }
  // 閉じ括弧
  if surface.ends_with('」') || surface.ends_with('』') || surface.ends_with('）') {
    return BreakPriority::High;
  }
  // 句点
  if surface.ends_with('。') || surface.ends_with('！') || surface.ends_with('？') {
    return BreakPriority::High;
  }
  // 読点
  if surface.ends_with('、') {
    return BreakPriority::Medium;
  }
  // 助詞類は低優先度
  if feature.contains("助詞,") {
    return BreakPriority::Low;
  }
  BreakPriority::None
}

/// テキストをトークンに分解（さくらスクリプトを保護）
fn tokenize_text(text: &str, tokenizer: &Tokenizer) -> Vec<Token> {
  let mut tokens: Vec<Token> = Vec::new();
  let mut worker = tokenizer.new_worker();

  // さくらスクリプトの位置を取得
  let script_matches: Vec<_> = SAKURA_SCRIPT_RE.find_iter(text).collect();
  let plain_parts: Vec<&str> = SAKURA_SCRIPT_RE.split(text).collect();

  let mut script_iter = script_matches.iter();

  for plain in plain_parts {
    if !plain.is_empty() {
      worker.reset_sentence(plain);
      worker.tokenize();

      for tok in worker.token_iter() {
        let surface = tok.surface().to_string();
        let width = count_chars(&surface);
        let priority = get_break_priority(&surface, tok.feature());
        tokens.push(Token {
          text: surface,
          width,
          break_priority: priority,
          is_sakura_script: false,
        });
      }
    }

    // 対応するさくらスクリプトを追加
    if let Some(script) = script_iter.next() {
      tokens.push(Token {
        text: script.as_str().to_string(),
        width: 0.0, // さくらスクリプトは幅0
        break_priority: BreakPriority::None,
        is_sakura_script: true,
      });
    }
  }

  tokens
}

// ============================================================
// 自動改行ロジック
// ============================================================

/// メインの自動改行処理
/// 1. \\n\\n[half]で段落分割
/// 2. 各段落内の\\nを除去して最適位置に再配置
/// 3. 段落を\\n\\n[half]で再結合
/// 4. ポストプロセス: 空白行の除去、行頭スペースのトリム
pub fn process_autobreak(src: &str, tokenizer: &Tokenizer, cols: f32) -> String {
  // 段落区切りで分割
  let mut paragraphs: Vec<String> = Vec::new();
  let mut delimiters: Vec<String> = Vec::new();

  let parts: Vec<&str> = PARAGRAPH_DELIMITER_RE.split(src).collect();
  let delim_matches: Vec<_> = PARAGRAPH_DELIMITER_RE.find_iter(src).collect();

  for (i, part) in parts.iter().enumerate() {
    if !part.is_empty() {
      let processed = process_paragraph(part, tokenizer, cols);
      paragraphs.push(processed);
    }
    if i < delim_matches.len() {
      delimiters.push(delim_matches[i].as_str().to_string());
    }
  }

  // 段落を再結合
  let mut result = String::new();
  for (i, para) in paragraphs.iter().enumerate() {
    result.push_str(para);
    if i < delimiters.len() {
      result.push_str(&delimiters[i]);
    }
  }

  // ポストプロセス: 空白のみの行を除去、行頭スペースをトリム
  postprocess_lines(&result)
}

/// ポストプロセス: 空白行の除去、sakuraスクリプトのみの行をマージ
fn postprocess_lines(src: &str) -> String {
  let lines: Vec<&str> = src.split("\\n").collect();
  let mut result_lines: Vec<String> = Vec::new();

  for line in lines {
    // 行頭スペースをトリム
    let trimmed = line.trim_start();

    // 可視テキストがあるかチェック（sakuraスクリプトを除去して確認）
    let visible = SAKURA_SCRIPT_RE.replace_all(trimmed, "");
    let has_visible_text = !visible.trim().is_empty();

    if has_visible_text {
      // 可視テキストがある行はそのまま追加
      result_lines.push(trimmed.to_string());
    } else if !trimmed.is_empty() {
      // sakuraスクリプトのみの行は前の行にマージ
      if let Some(last) = result_lines.last_mut() {
        last.push_str(trimmed);
      } else {
        result_lines.push(trimmed.to_string());
      }
    } else if line.is_empty() {
      // 空行は保持（段落区切りの一部）
      result_lines.push(String::new());
    }
    // 空白のみの行はスキップ
  }

  result_lines.join("\\n")
}

/// 段落内の処理
/// 句読点後の\\nは尊重し、長い行のみ分割
fn process_paragraph(para: &str, tokenizer: &Tokenizer, cols: f32) -> String {
  // \\nで分割して各「意味的行」を処理
  let lines: Vec<&str> = para.split("\\n").collect();
  let mut result_lines: Vec<String> = Vec::new();

  for line in lines {
    if line.is_empty() {
      result_lines.push(String::new());
      continue;
    }

    // 行幅を計算
    let width = count_chars(line);

    if width <= cols {
      // 行幅内なら、そのまま保持
      result_lines.push(line.to_string());
    } else {
      // 長い行は分割
      let tokens = tokenize_text(line, tokenizer);
      let processed = render_with_breaks(&tokens, cols);
      // 処理結果を\\nで分割して追加
      for sub_line in processed.split("\\n") {
        result_lines.push(sub_line.to_string());
      }
    }
  }

  result_lines.join("\\n")
}

/// トークン列を行幅制限に従ってレンダリング
fn render_with_breaks(tokens: &[Token], cols: f32) -> String {
  if tokens.is_empty() {
    return String::new();
  }

  // 全トークンの参照に変換
  let all_tokens: Vec<&Token> = tokens.iter().collect();
  let mut result = String::new();
  let mut start_idx = 0;

  // 許容マージン（0.5文字以内のオーバーは許容）
  let margin = 0.5;

  while start_idx < all_tokens.len() {
    // 現在位置から行幅内に収まる最後の位置を探す
    let remaining = &all_tokens[start_idx..];
    let total_width: f32 = remaining.iter().map(|t| t.width).sum();

    if total_width <= cols + margin {
      // 残り全部が収まる場合（マージン込み）
      for t in remaining {
        result.push_str(&t.text);
      }
      break;
    }

    // 行幅を超える場合、最適な改行位置を探す
    let break_result = find_best_break_position(remaining, cols);

    if break_result.position == 0 {
      // 1トークンも収まらない場合は強制的に1つ出力
      result.push_str(&remaining[0].text);
      result.push_str("\\n");
      start_idx += 1;
    } else {
      // 改行位置までを出力
      for t in remaining.iter().take(break_result.position) {
        result.push_str(&t.text);
      }
      result.push_str("\\n");
      start_idx += break_result.position;
    }
  }

  result
}

struct BreakResult {
  position: usize,
}

/// ダッシュ文字を含むかチェック
fn is_dash_token(text: &str) -> bool {
  text.contains('─') || text.contains('―') || text.contains('—')
}

/// 終助詞かどうかをチェック
/// 典型的な終助詞: よ、ね、わ、の、な、か、さ、ぞ、ぜ
fn is_sentence_final_particle(text: &str) -> bool {
  // スペースを除去して判定
  let trimmed = text.trim();
  matches!(
    trimmed,
    "よ" | "ね" | "わ" | "の" | "な" | "か" | "さ" | "ぞ" | "ぜ" | "かしら"
  )
}

/// フレーズ末尾パターンかどうかをチェック
/// 条件: 残りトークンが≤6文字 かつ 終助詞で終わる
/// 戻り値: フレーズ末尾でない場合は0、フレーズ末尾の場合はその幅
const PHRASE_ENDING_THRESHOLD: f32 = 6.0;

fn get_phrase_ending_width(remaining_tokens: &[&Token]) -> f32 {
  if remaining_tokens.is_empty() {
    return 0.0;
  }

  // 残りトークンの合計幅を計算（さくらスクリプトは0幅なので自動的に除外される）
  let total_width: f32 = remaining_tokens.iter().map(|t| t.width).sum();
  if total_width > PHRASE_ENDING_THRESHOLD {
    return 0.0;
  }

  // 最後の可視トークン（幅>0、空白以外）が終助詞かチェック
  for token in remaining_tokens.iter().rev() {
    if token.width > 0.0 {
      // 空白のみのトークンはスキップ
      let trimmed = token.text.trim();
      if trimmed.is_empty() {
        continue;
      }
      if is_sentence_final_particle(trimmed) {
        return total_width; // フレーズ末尾の場合、その幅を返す
      } else {
        return 0.0;
      }
    }
  }
  0.0
}

/// 最適な改行位置を探す
/// 1. 行幅内で最後に収まる位置を記録（フォールバック用）
/// 2. 優先度付き位置の中で、残りが1行で収まる位置を優先
/// 3. 同じ条件なら高い優先度を選ぶ
/// 4. 同じ優先度なら、残りがフレーズ末尾の位置を優先
/// 5. それも同じなら後ろの位置を選ぶ（行を埋める）
fn find_best_break_position(tokens: &[&Token], cols: f32) -> BreakResult {
  let total_width: f32 = tokens.iter().map(|t| t.width).sum();

  // 行幅内で最後に収まる位置（フォールバック用、優先度関係なし）
  let mut last_fitting_pos = 0;

  // 優先度付き位置の最良候補
  let mut best_pos = 0;
  let mut best_priority = BreakPriority::None;
  let mut best_remaining_fits = false;
  let mut best_phrase_ending_width: f32 = 0.0; // フレーズ末尾の場合、その幅（長いほど優先）
  let mut best_width = 0.0;

  let mut accumulated_width = 0.0;
  for (i, token) in tokens.iter().enumerate() {
    accumulated_width += token.width;

    if accumulated_width <= cols {
      // 行幅内で最後に収まる位置を常に更新
      last_fitting_pos = i + 1;

      let priority = token.break_priority;

      // ダッシュ連続の途中では改行しない
      // 次のトークンもダッシュなら、このダッシュ後での改行をスキップ
      if is_dash_token(&token.text) {
        if let Some(next_token) = tokens.get(i + 1) {
          if is_dash_token(&next_token.text) {
            continue; // ダッシュ連続の途中なのでスキップ
          }
        }
      }

      let remaining_width = total_width - accumulated_width;
      let remaining_fits = remaining_width <= cols;

      // フレーズ末尾検出：残りがフレーズ末尾なら、その幅を取得
      let remaining = &tokens[i + 1..];
      let current_phrase_ending_width = get_phrase_ending_width(remaining);

      // None優先度はフレーズ末尾がある場合のみ候補に含める
      if priority == BreakPriority::None && current_phrase_ending_width == 0.0 {
        continue; // フレーズ末尾でもない場合はスキップ
      }

      // 改行位置の選択基準
      // 核心: 意味的区切りで改行し、次行をフルに活用する
      //
      // 「良い改行」の定義:
      // - 残りが収まり、かつ行幅の75%以上を埋める場合
      // - または、残りがフレーズ末尾パターンの場合
      // - または、High優先度（句点・疑問符）で残りが収まる場合
      let remaining_fills_line = remaining_fits && remaining_width >= cols * 0.75;
      let current_is_good_break = remaining_fills_line
        || current_phrase_ending_width > 0.0
        || (remaining_fits && priority >= BreakPriority::High);

      let best_remaining_width = total_width - best_width;
      let best_fills_line = best_remaining_fits && best_remaining_width >= cols * 0.75;
      let best_is_good_break = best_fills_line
        || best_phrase_ending_width > 0.0
        || (best_remaining_fits && best_priority >= BreakPriority::High);

      let should_update = if remaining_fits && !best_remaining_fits {
        // 残りが収まる位置を優先
        // ただし、現在のbestが意味的区切り（Medium以上）で、残りが「ほぼ」収まる場合
        // （1文字以内のオーバー）は意味的区切りを維持する
        let best_almost_fits = best_remaining_width <= cols + 1.0;
        if best_priority >= BreakPriority::Medium && priority < best_priority && best_almost_fits {
          false
        } else {
          true
        }
      } else if remaining_fits == best_remaining_fits {
        // 両方収まる/両方収まらない場合

        // 「良い改行」同士の比較
        if current_is_good_break && !best_is_good_break {
          // 特殊ケース: bestがLow+優先度で残りが十分（行幅30%以上）な場合、
          // currentがphrase_endingのみで良い場合よりbestを優先
          // これにより「は」(Low, remaining 9)が「見」(None+phrase_ending 4)に勝てる
          let best_substantial = best_remaining_fits && best_remaining_width >= cols * 0.3;
          let current_only_phrase = current_phrase_ending_width > 0.0 && !remaining_fills_line;
          // bestの残りがcurrentの残りより大幅に大きい場合、bestを維持
          // (より多くの内容を次行にまとめる)
          let best_much_larger = best_remaining_width > remaining_width + 3.0;
          if current_only_phrase && best_substantial && best_priority >= BreakPriority::Low && best_much_larger {
            false // bestを維持（残りがより大きい）
          } else {
            true
          }
        } else if current_is_good_break && best_is_good_break {
          // 両方とも「良い改行」の場合
          // 残りが長い(fills_line)場合は早い改行を優先（長い内容をまとめる）
          // 残りが短い(phrase)場合は遅い改行を優先（現在行を埋める）
          let current_has_phrase = current_phrase_ending_width > 0.0;
          let best_has_phrase = best_phrase_ending_width > 0.0;

          if remaining_fills_line && best_fills_line {
            // 両方とも行を埋める → 優先度を先に比較
            // 高い優先度（意味的区切り）を優先し、同じ優先度なら後ろを優先
            if priority > best_priority {
              true
            } else if priority < best_priority {
              false
            } else {
              // 同じ優先度なら後ろの位置を優先（現在行を埋める）
              accumulated_width > best_width
            }
          } else if remaining_fills_line && !best_fills_line {
            // currentが行を埋める、bestはフレーズ末尾
            // 意味的区切り（Medium以上）なら行を埋める方を優先
            // そうでなければ、現在行が十分長いか確認
            if priority >= BreakPriority::Medium || accumulated_width >= cols * 0.4 {
              true
            } else {
              // 短い非意味的区切りよりフレーズ末尾を優先
              false
            }
          } else if !remaining_fills_line && best_fills_line {
            // bestが行を埋める、currentはフレーズ末尾
            // bestが意味的区切りか十分長いなら維持
            if best_priority >= BreakPriority::Medium || best_width >= cols * 0.4 {
              false
            } else {
              // bestが短い非意味的区切りならフレーズ末尾を優先
              true
            }
          } else {
            // 両方ともフレーズ末尾のみ
            // フレーズの長さを先に比較（より多くの内容を保護）
            if current_has_phrase && best_has_phrase {
              if current_phrase_ending_width > best_phrase_ending_width + 1.0 {
                // より長いフレーズを保護する位置を優先
                true
              } else if best_phrase_ending_width > current_phrase_ending_width + 1.0 {
                false
              } else {
                // フレーズ長が同程度なら、現在行が長い方を優先
                accumulated_width > best_width
              }
            } else if current_has_phrase {
              // currentがフレーズ末尾、bestはそうでない
              // ただしbestがLow+優先度で残りが大幅に大きければbestを維持
              // (より多くの内容を次行にまとめる)
              let best_substantial = best_remaining_width >= cols * 0.3;
              if best_priority >= BreakPriority::Low
                && best_substantial
                && best_remaining_width > remaining_width + 3.0
              {
                false // bestを維持（残りがより長い）
              } else {
                true
              }
            } else if best_has_phrase {
              // bestがフレーズ末尾、currentはそうでない
              // currentがLow+優先度で残りが大幅に大きければcurrentを選択
              let current_substantial = remaining_width >= cols * 0.3;
              if priority >= BreakPriority::Low
                && current_substantial
                && remaining_width > best_remaining_width + 3.0
              {
                true // currentを選択（残りがより長い）
              } else {
                false
              }
            } else {
              // どちらもフレーズでない（論理的には到達しないはず）
              accumulated_width > best_width
            }
          }
        } else if !current_is_good_break && !best_is_good_break {
          // 両方とも「良い改行」でない場合は優先度と位置で比較
          if priority > best_priority {
            true
          } else if priority == best_priority {
            accumulated_width > best_width
          } else {
            false
          }
        } else {
          // bestが「良い改行」、currentはそうでない
          false
        }
      } else {
        false
      };

      if should_update {
        best_pos = i + 1;
        best_priority = priority;
        best_remaining_fits = remaining_fits;
        best_phrase_ending_width = current_phrase_ending_width;
        best_width = accumulated_width;
      }
    }
  }

  // 優先度付き位置が見つからなければ、最後に収まる位置を使用
  if best_pos == 0 {
    best_pos = last_fitting_pos.max(1);
  }

  BreakResult { position: best_pos }
}

// ============================================================
// Inserter構造体（既存APIの維持）
// ============================================================

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
    let tokenizer_clone = self.tokenizer.clone();
    let tokenizer = tokenizer_clone.lock().unwrap();
    let t = tokenizer.as_ref().unwrap();

    let result = process_autobreak(&src, t, self.cols_num);
    Ok(result)
  }
}
