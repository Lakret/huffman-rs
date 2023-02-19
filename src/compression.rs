use bit_vec::BitVec;
use rayon::prelude::*;
use rmp_serde;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

use crate::huffman::{self, Tree};
use Tree::*;

#[derive(Serialize, Deserialize)]
struct CompressedData<T: Eq + Hash> {
    encoder: HashMap<T, BitVec>,
    data: Vec<BitVec>,
}

pub fn compress<'a, T, FreqsF, TokenExtractor, TokensIter>(
    lines: &'a Vec<String>,
    get_freqs: FreqsF,
    line_to_tokens: TokenExtractor,
) -> Result<Vec<u8>, Box<dyn std::error::Error>>
where
    T: Clone + Eq + Hash + Send + Sync + Serialize,
    FreqsF: Fn(&'a Vec<String>) -> HashMap<T, u64>,
    TokenExtractor: Fn(&'a str) -> TokensIter + Send + Sync,
    TokensIter: Iterator<Item = T>,
{
    let freqs = get_freqs(lines);
    let tree = huffman::huffman_tree(&freqs);
    let encoder = tree.to_encoder();

    let data = lines
        .par_iter()
        .map(|line| {
            line_to_tokens(line)
                .map(|token| encoder.get(&token).unwrap().clone())
                .fold(BitVec::new(), |mut vec1, vec2| {
                    vec1.extend(vec2);
                    vec1
                })
        })
        .collect();

    let compressed_data = CompressedData { encoder, data };
    rmp_serde::encode::to_vec(&compressed_data).map_err(|err| err.into())
}

pub fn extract<T, F>(
    data: &Vec<u8>,
    tokens_to_line: F,
) -> Result<Vec<String>, Box<dyn std::error::Error>>
where
    T: Clone + Eq + Hash + Send + Sync + for<'a> Deserialize<'a>,
    F: Fn(Vec<T>) -> String + Send + Sync,
{
    let CompressedData { encoder, data }: CompressedData<T> = rmp_serde::decode::from_slice(data)?;

    let decoder = encoder_to_decoder(&encoder);
    let lines = data
        .par_iter()
        .map(|line| {
            let mut tokens = vec![];
            let mut candidate = BitVec::new();

            for bit in line {
                candidate.push(bit);

                match decoder.get(&candidate) {
                    Some(token) => {
                        tokens.push(token.clone());

                        candidate = BitVec::new();
                    }
                    None => (),
                }
            }
            tokens_to_line(tokens)
        })
        .collect();

    Ok(lines)
}

fn encoder_to_decoder<T: Clone>(encoder: &HashMap<T, BitVec>) -> HashMap<BitVec, T> {
    let mut decoder = HashMap::new();
    for (token, prefix) in encoder.clone() {
        decoder.insert(prefix, token);
    }
    decoder
}

impl<T: Eq + Clone + Hash> Tree<T> {
    pub fn to_encoder(&self) -> HashMap<T, BitVec> {
        let mut encoder = HashMap::new();

        let mut stack = vec![(self, BitVec::new())];
        while !stack.is_empty() {
            let (node, path) = stack.pop().unwrap();
            match node {
                Leaf { token, .. } => {
                    encoder.insert(token.clone(), path.clone());
                }
                Node { left, right, .. } => {
                    let mut left_path = path.clone();
                    left_path.push(false);
                    stack.push((left, left_path));

                    let mut right_path = path.clone();
                    right_path.push(true);
                    stack.push((right, right_path));
                }
            }
        }

        encoder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::freqs;

    #[test]
    fn compress_decompress_test() {
        let lines = vec![
            "hey there! nice to meet you.".to_string(),
            "Serde is a framework for serializing and deserializing Rust data structures"
                .to_string(),
        ];

        let data = compress(&lines, freqs::char_frequencies, |line| line.chars()).unwrap();
        let res_lines = extract(&data, |x: Vec<char>| x.into_iter().collect()).unwrap();
        assert_eq!(&lines, &res_lines);

        let data = compress(&lines, freqs::word_frequencies, |line| {
            line.split_ascii_whitespace().map(|token| token.to_string())
        })
        .unwrap();
        let res_lines = extract(&data, |x: Vec<String>| x.join(" ")).unwrap();
        assert_eq!(&lines, &res_lines);
    }
}
