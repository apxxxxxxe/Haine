use std::fs::File;
use std::io::Write;
use vibrato::{Dictionary, Tokenizer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting dictionary pre-compilation...");
    
    // Input: zstd compressed dictionary
    let input_path = "ipadic-mecab-2_7_0/system.dic.zst";
    let output_path = "ipadic-mecab-2_7_0/system.dic.bincode";
    
    println!("Reading zstd compressed dictionary from: {}", input_path);
    let input_file = File::open(input_path)?;
    let reader = zstd::Decoder::new(input_file)?;
    
    println!("Parsing MeCab dictionary...");
    let dict = Dictionary::read(reader)?;
    
    println!("Writing pre-compiled dictionary to: {}", output_path);
    let mut output_file = File::create(output_path)?;
    let bytes_written = dict.write(&mut output_file)?;
    output_file.flush()?;
    
    println!("Successfully pre-compiled dictionary!");
    println!("  Output size: {} bytes ({:.2} MB)", bytes_written, bytes_written as f64 / 1024.0 / 1024.0);
    
    // Verify the compiled dictionary can be loaded
    println!("Verifying the compiled dictionary...");
    let verify_file = File::open(output_path)?;
    let _verify_dict = Dictionary::read(verify_file)?;
    let _tokenizer = Tokenizer::new(_verify_dict);
    
    println!("✓ Verification successful! The dictionary is ready to use.");
    
    Ok(())
}
