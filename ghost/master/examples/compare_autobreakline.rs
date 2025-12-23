//! 自動改行テスト駆動開発用のexample
//!
//! 使用方法:
//! ```
//! cd ghost/master
//! cargo run --example compare_autobreakline
//! ```
//!
//! テストモード（理想値との比較）:
//! ```
//! cargo run --example compare_autobreakline -- --test
//! ```

use haine::events::translate::translate;
use haine::system::autobreakline::{count_chars, process_autobreak};
use once_cell::sync::Lazy;
use regex::Regex;
use vibrato::{Dictionary, Tokenizer};

// さくらスクリプト除去用（表示時のみ使用）
static SAKURA_SCRIPT_RE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r###"\\_{0,2}[a-zA-Z0-9*!&](\d|\[("([^"]|\\")+?"|([^\]]|\\\])+?)+?\])?"###).unwrap()
});

static SAKURA_SCRIPT_WITHOUT_BREAKLINE_RE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r###"\\_{0,2}[abcdefghijklmopqrstuvwxyzA-Z0-9*!&](\d|\[("([^"]|\\")+?"|([^\]]|\\\])+?)+?\])?"###).unwrap()
});
// ============================================================
// テストケース定義
// ============================================================

struct TestCase {
  name: &'static str,
  /// randomtalk.rsからの生テキスト（translate前）
  raw_input: &'static str,
  /// 理想的な自動改行結果（translate後 + 自動改行後）
  expected: &'static str,
}

fn get_test_cases() -> Vec<TestCase> {
  vec![
    // テストケース1: 科学への興味
    // - 中程度の長さ
    // - 複数の表情コード
    // - 句読点での自然な改行が期待される
    TestCase {
      name: "科学への興味",
      raw_input: "\
生きていた頃、科学に興味を持っていたわ。\\n\
物質の構造や、宇宙の謎、生命の起源。\\n\
一見して無秩序で不確かなものたちが、\\n\
じつに単純な秩序によって結びついているの。\\n\
そのさまは、目が覚めるように美しい。\\n\\n[half]\
今日はどんな新しい発見があるのかと、\\n\
いまでも楽しみにしているのよ。",
      expected: "\
\\![quicksection,0]生きていた頃、\\_w[600]科学に興味を持っていたわ \\_w[1200]\\n\
物質の構造や、\\_w[600]宇宙の謎、\\_w[600]生命の起源 \\_w[1200]\\n\
一見して無秩序で不確かなものたちが、\\_w[600]\\n\
じつに単純な秩序によって結びついているの \\_w[1200]\\n\
そのさまは、\\_w[600]目が覚めるように美しい \\_w[1200]\\n\\n[half]\
\\_w[700]今日はどんな新しい発見があるのかと、\\_w[600]\\n\
いまでも楽しみにしているのよ ",
    },
    // テストケース2: 別れの悲しみ
    // - 感情的な内容
    // - 長めの文章が連続
    // - 自然な区切りでの改行が重要
    TestCase {
      name: "別れの悲しみ",
      raw_input: "\
「別れがこんなに悲しいなら、最初から出会わなければよかった」\\n\
……使い古された句だけど、私も、その時が来たらきっとそう感じると思う。\\n\
過程がどうであれ、別れてしまえば残った傷は他の思い出を変質させてしまう。\\n\
元通りの幸せな感情は決して戻らない。そう思うの。",
      expected: "\
\\![quicksection,0]「別れがこんなに悲しいなら、\\n\
\\_w[600]最初から出会わなければよかった」\\_w[600]\\n\
\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]使い古された句だけど、\\_w[600]私も、\\n\
\\_w[600]その時が来たらきっとそう感じると思う \\_w[1200]\\n\
過程がどうであれ、\\_w[600]別れてしまえば\\n\
残った傷は他の思い出を変質させてしまう \\_w[1200]\\n\
元通りの幸せな感情は決して戻らない \\n\
\\_w[1200]そう思うの ",
    },
    // テストケース3: 姿は変えられない
    // - \\n\\n[half] による段落区切り
    // - 長文と短文の混在
    TestCase {
      name: "姿は変えられない",
      raw_input: "\
霊は不定形だけれど、自由に形を変えられるわけではないわ。\\n\
魂の形は一つしかない。変えられるとしたら、自分が誰かもわからなくなってしまった者くらいよ。\\n\\n[half]\
だから、私が昼に出歩くことはないわ。\\n\
10年、20年経とうが姿の変わらない女。\\n\
余計な面倒は避けるに越したことはないもの。",
      expected: "\
\\![quicksection,0]霊は不定形だけれど、\\n\
\\_w[600]自由に形を変えられるわけではないわ \\_w[1200]\\n\
魂の形は一つしかない \\_w[1200]変えられるとしたら、\\n\
\\_w[600]自分が誰かもわからなくなってしまった者\\n\
くらいよ \\_w[1200]\\n\\n[half]\\_w[700]\
だから、\\_w[600]私が昼に出歩くことはないわ \\_w[1200]\\n\
10年、\\_w[600]20年経とうが姿の変わらない女 \\_w[1200]\\n\
余計な面倒は避けるに越したことはないもの ",
    },
    // テストケース4: 霊力の多寡
    // - アンカーリンク \\_a[...]\\_a を含む
    // - 複雑なSakuraScript構造
    TestCase {
      name: "霊力の多寡",
      raw_input: "\
霊力の多寡は年月や才能、特別な契約の有無などで変わるけれど、\\n\
最も大きな要因は環境──つまり、その地との関わりの深さによるの。\\n\
私のように生家に根付いた霊は言わずもがな。\\n\
……まあ、強いからといって良いことばかりでもないわ。\\n\
霊にも社会がある。\\_a[AnchorTalk,NoblesseOblige,義務ってどんなこと？]上位者の義務\\_aというものも。\\n\\n[half]\
……はじめは億劫だと思っていたのだけどね。\\n\
悪くないものよ。感謝され、慕われるというのは。",
      expected: "\
\\![quicksection,0]霊力の多寡は年月や才能、\\n\
\\_w[600]特別な契約の有無などで変わるけれど、\\_w[600]\\n\
最も大きな要因は環境──\\n\
つまり、\\_w[600]その地との関わりの深さによるの \\_w[1200]\\n\
私のように生家に根付いた霊は言わずもがな \\_w[1200]\\n\
\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]まあ、\\n\
\\_w[600]強いからといって良いことばかりでもないわ \\_w[1200]\\n\
霊にも社会がある \\_w[1200]\\_a[AnchorTalk,NoblesseOblige,義務ってどんなこと？]上位者の義務\\_aというものも \\_w[1200]\\n\\n[half]\\_w[700]\
\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]はじめは億劫だと思っていたのだけどね \\_w[1200]\\n\
悪くないものよ \\_w[1200]感謝され、\\_w[600]慕われるというのは ",
    },
    // テストケース5: 生家の広さ
    // - アンカーリンクを含む
    // - 日常的な話題
    // - 適度な長さ
    TestCase {
      name: "生家の広さ",
      raw_input: "\
ここは私の生家なの。実際は別荘なのだけど。\\n\
知っての通り、従者がいなければ掃除が行き届かないほど広いの。\\n\
……まあ、\\_a[AnchorTalk,LiveHome,別荘だけど長く住んでいたの？]勝手知ったる場所\\_aなのは不幸中の幸い、といえなくもないかしらね。\\n\
くつろいで暮らすのにこれ以上の場所はないわ。",
      expected: "\
\\![quicksection,0]ここは私の生家なの \\_w[1200]実際は別荘なのだけど \\_w[1200]\\n\
知っての通り、\\n\
\\_w[600]従者がいなければ掃除が行き届かないほど広いの \\_w[1200]\\n\
\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]まあ、\\_w[600]\\_a[AnchorTalk,LiveHome,別荘だけど長く住んでいたの？]勝手知ったる場所\\_aなのは\\n\
不幸中の幸い、\\_w[600]といえなくもないかしらね \\_w[1200]\\n\
くつろいで暮らすのにこれ以上の場所はないわ ",
    },

    TestCase {
      name: "恋愛観",
      raw_input: "\
幽霊は生前の想い……好みや恨みに執着するの。\\n\
想い人がいればその人に、恨みがあればその相手に。\\n\
逆に、死後新たな執着が生まれることは\\n\
ほとんどないわ。\\n\
だから幽霊同士、ましてや人と幽霊の間に恋愛が生まれることは皆無といっていいでしょう。\\n\\n[half]\
……なに、その顔は。あいにく、\\n\
私は生きていた頃から恋愛とは無縁よ。",
      expected: "\
\\![quicksection,0]幽霊は生前の想い\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]好みや恨みに執着するの \\_w[1200]\\n\
想い人がいれば\\n\
その人に、\\_w[600]恨みがあればその相手に \\_w[1200]\\n\
逆に、\\_w[600]死後新たな執着が生まれることは\\n\
ほとんどないわ \\_w[1200]\\n\
だから幽霊同士、\\_w[600]ましてや人と幽霊の間に恋愛が\\n\
生まれることは皆無といっていいでしょう \\_w[1200]\\n\\n[half]\\_w[700]\
\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]なに、\\_w[600]その顔は \\_w[1200]あいにく、\\_w[600]\\n\
私は生きていた頃から恋愛とは無縁よ ",
    },

    TestCase {
      name: "リップクリーム",
      raw_input: "\
あら、それは何？\\n\
リップクリーム……唇の保湿をするのね。\\n\\n[half]\
それ、借りても良いかしら？初めて見たの。\\n\
\\1スティックタイプのものを渡すと、\\n\
ハイネは見様見真似で自分の唇に塗る。\\n\\n[half]\
塗り終えると、唇を小指で拭った。\
\\0\\n[half]ふむ……。保湿のためとはいえ、べたつくのは少し嫌ね。\\n\\n[half]\
ありがとう、返すわ。",
      expected: "\
\\![quicksection,0]あら、\\_w[600]それは何？\\_w[1200]\\n\
リップクリーム\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]唇の保湿をするのね \\_w[1200]\\n\\n[half]\\_w[700]\
それ、\\_w[600]借りても良いかしら？\\_w[1200]初めて見たの \\_w[1200]\\n\
\\![quicksection,0]\\p[1]スティックタイプのものを渡すと、\\_w[600]\\n\
ハイネは見様見真似で自分の唇に塗る。\\_w[1200]\\n\\n[half]\
\\_w[700]塗り終えると、\\_w[600]唇を小指で拭った。\\_w[1200]\\![quicksection,0]\\p[0]\\n\
[half]ふむ\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0] \\_w[1200]保湿の\\n\
ためとはいえ、\\_w[600]べたつくのは少し嫌ね \\_w[1200]\\n\\n[half]\
\\_w[700]ありがとう、\\_w[600]返すわ ",
    },

    TestCase {
      name: "時間の流れ方",
      raw_input: "\
時間というものは不思議なものね。\\n\
同じ一日でも、充実していれば短く感じるし、\\n\
退屈であれば長く感じる。\\n\\n[half]\
この館での時間は、あなたにとってどのように流れているのかしら？\\n\\n[half]\
私はもう時間の感覚が曖昧になってしまったけれど、\\n\
あなたには退屈な瞬間もあるのではないかしら。\\n\
……いえ、\\_a[AnchorTalk,Menohikari,愚問？]愚問ね。\\_a",
      expected: "\
\\![quicksection,0]時間というものは不思議なものね \\_w[1200]\\n\
同じ一日でも、\\_w[600]充実していれば短く感じるし、\\_w[600]\\n\
退屈であれば長く感じる \\_w[1200]\\n\\n[half]\
\\_w[700]この館での時間は、\\n\
\\_w[600]あなたにとってどのように流れているのかしら？\\_w[1200]\\n\\n[half]\
\\_w[700]私はもう時間の\\n\
感覚が曖昧になってしまったけれど、\\_w[600]\\n\
あなたには退屈な瞬間もあるのではないかしら \\_w[1200]\\n\
\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]いえ、\\_w[600]\\_a[AnchorTalk,Menohikari,愚問？]愚問ね \\_w[1200]\\_a",
    },

    TestCase {
      name: "冥府の渡し賃",
      raw_input: "\
古代ギリシャでは、死者に銅貨を持たせて葬っていたの。\\n\
冥界には川を渡っていかなければならなかったから、\\n\
渡し賃を持たせて快適な旅を願っていたのよ。\\n\\n[half]\
死者が川を越えていくという伝承は世界中で見られるわ。\\n\
彼らにとって、境界線といえばまず川が連想されたのかしら。\\n\
あなたなら、あの世とこの世の間にはなにがあると思う？\
      ",
      expected: "\
\\![quicksection,0]古代ギリシャでは、\\n\
\\_w[600]死者に銅貨を持たせて葬っていたの \\_w[1200]\\n\
冥界には川を\\n\
渡っていかなければならなかったから、\\_w[600]\\n\
渡し賃を持たせて快適な旅を願っていたのよ \\_w[1200]\\n\\n[half]\
\\_w[700]死者が川を越えていくという伝承は\\n\
世界中で見られるわ \\_w[1200]\\n\
彼らにとって、\\n\
\\_w[600]境界線といえばまず川が連想されたのかしら \\_w[1200]\\n\
あなたなら、\\n\
\\_w[600]あの世とこの世の間にはなにがあると思う？",
    },

    TestCase {
      name: "早口",
      raw_input: "\
……\\![set,balloonwait,0.7]それでねφ、\\_w[500]だからこそこの仕組みが成り立つのφ。\\_w[700]\\n\
ここまでは良いかしらφ？\\_w[700]それで、次に重要なのが……\\![set,balloonwait,1]\\n\\n[half]\
\\1矢継ぎ早の説明についていけずφ、\\1\\n曖昧に頷いてしまう。\
\\0……ごめんなさい。少し早口だったわね。\\n\
普通に話すと速いと言われるものだから\\n\
ゆっくり話すよう心がけているのだけれど……\\n\
熱が入ると、だめね。\
      ",
      expected: "\
\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![set,balloonwait,0.7]それでね、\\n\
\\_w[500]だからこそこの仕組みが成り立つの。\\_w[700]\\n\
ここまでは良いかしら？\\n\
\\_w[700]それで、\\_w[600]次に重要なのが\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![set,balloonwait,1]\\n\\n[half]\
\\_w[700]\\![quicksection,0]\\p[1]矢継ぎ早の説明についていけず、\\n\
曖昧に頷いてしまう。\\n\
\\_w[1200]\\![quicksection,0]\\p[0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]ごめんなさい \\_w[1200]少し早口だったわね \\_w[1200]\\n\
普通に話すと速いと言われるものだから\\n\
ゆっくり話すよう心がけているのだけれど\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\![quicksection,1]…\\_w[600]\\![quicksection,0]\\n\
熱が入ると、\\_w[600]だめね ",
    },
  ]
}

// ============================================================
// メイン関数
// ============================================================

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let test_mode = args.iter().any(|a| a == "--test");

  // 辞書を読み込み
  let bytes = include_bytes!("../ipadic-mecab-2_7_0/system.dic.bincode");
  let dict = Dictionary::read(&bytes[..]).unwrap();
  let tokenizer = Tokenizer::new(dict);

  let cols = 22.0;

  if test_mode {
    run_tests(&tokenizer, cols);
  } else {
    run_comparison(&tokenizer, cols);
  }
}

/// テストモード: 理想値との比較
fn run_tests(tokenizer: &Tokenizer, cols: f32) {
  println!("=== 自動改行テスト ===");
  println!("行幅: {}文字\n", cols);

  let test_cases = get_test_cases();
  let mut passed = 0;
  let mut failed = 0;
  let mut skipped = 0;

  for tc in &test_cases {
    println!("--- {} ---", tc.name);

    // translate処理を適用
    let translated = match translate(tc.raw_input.to_string(), false) {
      Ok(t) => t,
      Err(e) => {
        println!("❌ translate失敗: {:?}", e);
        failed += 1;
        continue;
      }
    };

    if tc.expected.is_empty() {
      println!("⏭ スキップ（理想値未設定）");
      println!();

      // 現在の出力を表示（理想値設定の参考用）
      let result = process_autobreak(&translated, tokenizer, cols);

      println!("translate後:");
      println!("{}", translated);
      println!();
      println!("現在の出力:");
      for (i, line) in result.split("\\n").enumerate() {
        let visible = SAKURA_SCRIPT_RE.replace_all(line, "");
        println!("  {}: {} ({}文字)", i + 1, visible, count_chars(line));
      }
      println!();
      println!("生出力（コピー用）:");
      println!("{}", result);
      println!();

      skipped += 1;
      continue;
    }

    let r = process_autobreak(&translated, tokenizer, cols);
    let result = SAKURA_SCRIPT_WITHOUT_BREAKLINE_RE.replace(r.as_str(), "");
    let expected = SAKURA_SCRIPT_WITHOUT_BREAKLINE_RE.replace(tc.expected, "");

    if result == expected {
      println!("✅ PASS");
      passed += 1;
    } else {
      println!("❌ FAIL");
      println!();

      let expected_lines: Vec<&str> = expected.split("\\n").collect();
      let result_lines: Vec<&str> = result.split("\\n").collect();
      let max_lines = expected_lines.len().max(result_lines.len());

      println!("行ごとの比較（≠は不一致）:");
      for i in 0..max_lines {
        let exp = expected_lines.get(i).map(|s| *s);
        let res = result_lines.get(i).map(|s| *s);

        let exp_visible = exp
          .map(|s| SAKURA_SCRIPT_RE.replace_all(s, "").to_string())
          .unwrap_or_else(|| "(なし)".to_string());
        let res_visible = res
          .map(|s| SAKURA_SCRIPT_RE.replace_all(s, "").to_string())
          .unwrap_or_else(|| "(なし)".to_string());

        let marker = if exp == res { "  " } else { "≠ " };
        println!(
          "{}行{:2}: 期待「{}」 vs 実際「{}」",
          marker,
          i + 1,
          exp_visible,
          res_visible
        );
      }
      println!();
      println!(
        "改行数: 期待={}, 実際={}",
        expected_lines.len(),
        result_lines.len()
      );
      failed += 1;
    }
    println!();
  }

  println!("=== 結果 ===");
  println!("PASS: {}, FAIL: {}, SKIP: {}", passed, failed, skipped);

  if failed > 0 {
    std::process::exit(1);
  }
}

/// 比較モード: 新実装の出力表示
fn run_comparison(tokenizer: &Tokenizer, cols: f32) {
  println!("=== 自動改行出力 ===");
  println!("行幅: {}文字", cols);
  println!("※ --test フラグでテストモードを実行\n");

  let test_cases = get_test_cases();

  for tc in &test_cases {
    println!("--- {} ---", tc.name);

    // translate処理を適用
    let translated = match translate(tc.raw_input.to_string(), false) {
      Ok(t) => t,
      Err(e) => {
        println!("❌ translate失敗: {:?}", e);
        continue;
      }
    };

    let result = process_autobreak(&translated, tokenizer, cols);
    let breaks = result.matches("\\n").count();

    println!("【translate後】");
    println!("{}", translated);
    println!();

    println!("【自動改行結果】改行数: {}", breaks);
    for (j, line) in result.split("\\n").enumerate() {
      let visible = SAKURA_SCRIPT_RE.replace_all(line, "");
      println!("  {}: {} ({}文字)", j + 1, visible, count_chars(line));
    }
    println!();

    println!("生出力:");
    println!("{}", result);
    println!("\n");
  }
}
