// 全トークを all_talks.txt に書き出す開発用ツール。
// 実行: cargo run --bin dump_talks （ghost/master で実行するとカレントに出力される）
fn main() {
  let text = haine::render_all_talks();
  std::fs::write("all_talks.txt", text).expect("failed to write all_talks.txt");
  println!("wrote all_talks.txt");
}
