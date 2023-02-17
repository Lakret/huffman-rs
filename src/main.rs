use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};

mod compression;
mod freqs;
mod huffman;

fn main() -> Result<(), Error> {
    let compressed = compression::compress_file("data/wikisent2.txt");
    dbg!(compressed.unwrap().len());

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
