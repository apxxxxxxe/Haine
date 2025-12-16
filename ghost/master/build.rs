use std::fs::File;
use std::path::Path;

fn main() {
  let bincode_path = "ipadic-mecab-2_7_0/system.dic.bincode";

  // Check if pre-compiled dictionary already exists
  if Path::new(bincode_path).exists() {
    println!(
      "cargo:warning=Pre-compiled dictionary found: {}",
      bincode_path
    );
    println!("cargo:rerun-if-changed={}", bincode_path);
    return;
  }

  println!("cargo:warning=Pre-compiled dictionary not found. Compiling from zstd...");
  println!("cargo:warning=This is a one-time operation and may take a few seconds.");

  // Read zstd compressed dictionary
  let zstd_path = "ipadic-mecab-2_7_0/system.dic.zst";
  println!("cargo:rerun-if-changed={}", zstd_path);

  let input_file = File::open(zstd_path).expect("Failed to open zstd dictionary");
  let reader = zstd::Decoder::new(input_file).expect("Failed to create zstd decoder");

  // Parse MeCab dictionary
  let dict = vibrato::Dictionary::read(reader).expect("Failed to parse MeCab dictionary");

  // Write pre-compiled dictionary
  let mut output_file = File::create(bincode_path).expect("Failed to create bincode file");
  dict
    .write(&mut output_file)
    .expect("Failed to write bincode dictionary");

  println!(
    "cargo:warning=✓ Dictionary pre-compilation completed: {}",
    bincode_path
  );
}
