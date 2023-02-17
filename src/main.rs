use std::fs::{self, File};
use std::io::Write;
use std::time;

mod compression;
mod freqs;
mod huffman;

const DATA_PATH: &'static str = "data/wikisent2.txt";
const WORDS_OUT_PATH: &'static str = "data/words.huffman";
const CHARS_OUT_PATH: &'static str = "data/chars.huffman";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let timer = time::Instant::now();
    let text = fs::read_to_string(DATA_PATH)?;
    let lines: Vec<_> = text.split('\n').map(|x| x.to_string()).collect();
    let time = timer.elapsed();
    let lines_count = lines.len();
    println!("Read the source file with {lines_count} lines in {time:?}");

    // words compress & decompress

    let timer = time::Instant::now();
    let compressed = compression::compress(&lines, freqs::learn_word_frequencies, |line| {
        line.split_ascii_whitespace().map(|token| token.to_string())
    })?;
    let time = timer.elapsed();
    println!("Compressed as words in {time:?}");

    let timer = time::Instant::now();
    let mut out_f = File::create(WORDS_OUT_PATH)?;
    out_f.write(&compressed)?;
    let time = timer.elapsed();
    println!("Wrote to {WORDS_OUT_PATH} in {time:?}");

    let timer = time::Instant::now();
    let data = fs::read(WORDS_OUT_PATH)?;
    let time = timer.elapsed();
    println!("Read the compressed file in {time:?}");

    let timer = time::Instant::now();
    let res_lines = compression::decompress(data, |tokens: Vec<String>| tokens.join(" "))?;
    let time = timer.elapsed();
    let res_lines_count = res_lines.len();
    println!("Decompressed file with {res_lines_count} lines in {time:?}");

    dbg!(&res_lines[..4]);
    dbg!(&res_lines[(res_lines.len() - 4)..]);

    // Chars compress & decompress
    let timer = time::Instant::now();
    let compressed =
        compression::compress(&lines, freqs::learn_char_frequencies, |line| line.chars())?;
    let time = timer.elapsed();
    println!("Compressed as chars in {time:?}");

    let timer = time::Instant::now();
    let mut out_f = File::create(CHARS_OUT_PATH)?;
    out_f.write(&compressed)?;
    let time = timer.elapsed();
    println!("Wrote to {CHARS_OUT_PATH} in {time:?}");

    let timer = time::Instant::now();
    let data = fs::read(CHARS_OUT_PATH)?;
    let time = timer.elapsed();
    println!("Read the compressed file in {time:?}");

    let timer = time::Instant::now();
    let res_lines =
        compression::decompress(data, |tokens: Vec<char>| tokens.into_iter().collect())?;
    let time = timer.elapsed();
    let res_lines_count = res_lines.len();
    println!("Decompressed file with {res_lines_count} lines in {time:?}");

    dbg!(&res_lines[..4]);
    dbg!(&res_lines[(res_lines.len() - 4)..]);

    Ok(())
}
