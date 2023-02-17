use lazy_static::lazy_static;
use regex::Regex;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Error};

mod compression;
mod freqs;
mod huffman;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = fs::read_to_string(DATA_PATH)?;
    let lines: Vec<_> = text.split('\n').map(|x| x.to_string()).collect();

    // let compressed = compression::compress_file("data/wikisent2.txt");
    // let mut out_f = File::create(output_path)?;
    // out_f.write(&bytes).map_err(|err| err.into())

    // let file = File::open(path)?;
    // let CompressedData { encoder, data }: CompressedData<T> = rmp_serde::decode::from_read(file)?;

    // let decompressed: Vec<Vec<char>> = compression::decompress_file("data/compressed.huffman")?;
    // dbg!(&decompressed[..2]);
    // dbg!(&decompressed[(decompressed.len() - 2)..]);

    Ok(())
}

const DATA_PATH: &'static str = "data/wikisent2.txt";

fn read_data() -> Result<Vec<String>, Error> {
    let file = File::open(DATA_PATH)?;
    return BufReader::new(file).lines().collect();
}

fn preprocess(line: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[^0-9A-Za-z]+").unwrap();
    }

    let mut s: String = RE.replace_all(line, " ").to_string();
    s.make_ascii_lowercase();
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocess_test() {
        assert_eq!(preprocess("hey何か YO ПРИВЕТ!14wow"), "hey yo 14wow");
    }
}
