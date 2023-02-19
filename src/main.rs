use clap::{Parser, ValueEnum};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time;

mod compression;
mod freqs;
mod huffman;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(value_enum)]
    action: Action,
    #[arg(value_enum)]
    mode: Mode,
    input: PathBuf,
    output: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Action {
    Compress,
    Extract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Mode {
    Words,
    Chars,
}

// cargo run --release -- compress words data/wikisent2.txt data/words.huffman
// cargo run --release -- extract words data/words.huffman data/extracted.txt
// (to check if the extraction was correct: `diff data/wikisent2.txt data/extracted.txt`)
//
// cargo run --release -- compress chars data/wikisent2.txt data/chars.huffman
// cargo run --release -- extract chars data/chars.huffman data/extracted.txt
//
// to compare with zip:
// time zip data/test.zip data/wikisent2.txt
// time unzip data/test.zip -d data/test_zip
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.action {
        Action::Compress => {
            let timer = time::Instant::now();
            let text = fs::read_to_string(args.input)?;
            let lines: Vec<_> = text.split('\n').map(|x| x.to_string()).collect();
            let time = timer.elapsed();
            let lines_count = lines.len();
            println!("Read the source file with {lines_count} lines in {time:?}");

            let timer = time::Instant::now();
            let compressed = match args.mode {
                Mode::Words => compression::compress(&lines, freqs::word_frequencies, |line| {
                    line.split_ascii_whitespace().map(|token| token.to_string())
                }),
                Mode::Chars => {
                    compression::compress(&lines, freqs::char_frequencies, |line| line.chars())
                }
            }?;
            let time = timer.elapsed();
            println!("Compressed as {mode:?} in {time:?}.", mode = args.mode);

            let timer = time::Instant::now();
            let mut out_f = File::create(&args.output)?;
            out_f.write(&compressed)?;
            let time = timer.elapsed();
            println!(
                "Wrote to {output_path:?} in {time:?}",
                output_path = args.output
            );
        }
        Action::Extract => {
            let timer = time::Instant::now();
            let data = fs::read(&args.input)?;
            let time = timer.elapsed();
            println!("Read the compressed file in {time:?}");

            let timer = time::Instant::now();
            let content = match args.mode {
                Mode::Words => compression::extract(&data, |tokens: Vec<String>| tokens.join(" "))?,
                Mode::Chars => {
                    compression::extract(&data, |tokens: Vec<char>| tokens.into_iter().collect())?
                }
            };
            let time = timer.elapsed();
            let lines_count = content.len();
            println!("Extracted file with {lines_count} lines in {time:?}.");

            let timer = time::Instant::now();
            fs::write(&args.output, content.join("\n"))?;
            let time = timer.elapsed();
            println!(
                "Wrote to {output_path:?} in {time:?}",
                output_path = args.output
            );
        }
    }

    Ok(())
}
