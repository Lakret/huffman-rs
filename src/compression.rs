use bit_vec::BitVec;
use rayon::prelude::*;
use rmp_serde;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    hash::Hash,
    io::Write,
    path::Path,
};

use crate::{
    freqs,
    huffman::{self, Tree},
};
use Tree::*;

#[derive(Serialize, Deserialize)]
struct CompressedData<T: Eq + Hash> {
    encoder: HashMap<T, BitVec>,
    data: Vec<BitVec>,
}

// TODO: use preprocess
// TODO: try bincode
pub fn compress(lines: &Vec<String>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let freqs = freqs::learn_char_frequencies(lines);
    let tree = huffman::build_huffman_tree(&freqs);
    let encoder = tree.to_encoder();

    let data: Vec<_> = lines
        .par_iter()
        .map(|line| {
            // TODO: comparison between words & chars compression performance
            // it seems that words compression yields 1847299618 / 8 / 1024 / 1024 ~= 220MB (w/o header)
            // and chars compression yields 4385985563 / 8 / 1024 / 1024 ~= 523MB (w/o header)
            // let chs: Vec<_> = line.chars().collect();
            //
            // .split_ascii_whitespace()
            // .map(|s| encoder.get(s).unwrap().clone())
            line.chars()
                .map(|ch| encoder.get(&ch).unwrap().clone())
                .fold(BitVec::new(), |mut vec1, vec2| {
                    vec1.extend(vec2);
                    vec1
                })
        })
        .collect();

    let compressed_data = CompressedData { encoder, data };
    rmp_serde::encode::to_vec(&compressed_data).map_err(|err| err.into())
}

pub fn decompress(data: Vec<u8>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let CompressedData { encoder, data }: CompressedData<char> =
        rmp_serde::decode::from_slice(&data[..])?;

    // TODO: extract into separate fun
    let mut decoder = HashMap::new();
    for (token, prefix) in encoder.clone() {
        decoder.insert(prefix, token);
    }

    let lines: Vec<_> = data
        .par_iter()
        .map(|line| {
            let mut pos = 0;
            let mut candidate = BitVec::new();
            let mut tokens = vec![];

            while pos < line.len() {
                let bit = line.get(pos).unwrap();
                candidate.push(bit);
                pos += 1;

                match decoder.get(&candidate) {
                    Some(token) => {
                        tokens.push(token.clone());

                        candidate = BitVec::new();
                    }
                    None => (),
                }
            }

            // TODO: word vs char
            tokens.into_iter().collect()
        })
        .collect();

    Ok(lines)
}

impl<T: Eq + Clone + Hash> Tree<T> {
    // TODO: pass pre-computed encoder if possible
    pub fn encode(&self, data: &[T]) -> Vec<BitVec> {
        let encoder = self.to_encoder();

        let mut encoded = vec![];
        for item in data {
            encoded.push(encoder.get(item).unwrap().clone());
        }
        encoded
    }

    // TODO: pass pre-computed decoder if possible
    pub fn decode(&self, data: &Vec<BitVec>) -> Vec<T> {
        let decoder = self.to_decoder(None);

        let mut res = vec![];
        for code in data {
            res.push(decoder.get(&code).unwrap().clone());
        }
        res
    }

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

    pub fn to_decoder(&self, encoder: Option<&HashMap<T, BitVec>>) -> HashMap<BitVec, T> {
        let encoder = encoder
            .map(|m| m.clone())
            .unwrap_or_else(|| self.to_encoder());

        let mut decoder = HashMap::new();
        for (token, prefix) in encoder.clone() {
            decoder.insert(prefix, token);
        }
        decoder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::huffman::*;

    #[test]
    fn to_encoder_and_to_decoder_test() {
        let mut freqs = HashMap::new();
        freqs.insert('a', 40);
        freqs.insert('b', 35);
        freqs.insert('c', 20);
        freqs.insert('d', 5);

        let tree = build_huffman_tree(&freqs);

        let encoder = tree.to_encoder();
        assert!(encoder.get(&'a').unwrap().eq_vec(&[false]));
        assert!(encoder.get(&'b').unwrap().eq_vec(&[true, true]));
        assert!(encoder.get(&'c').unwrap().eq_vec(&[true, false, true]));
        assert!(encoder.get(&'d').unwrap().eq_vec(&[true, false, false]));

        let decoder = tree.to_decoder(Some(&encoder));
        assert_eq!(decoder.len(), 4);

        let mut c_path = BitVec::new();
        c_path.push(true);
        c_path.push(false);
        c_path.push(true);
        assert_eq!(decoder.get(&c_path), Some(&'c'));

        let test_arr = &['a', 'a', 'a', 'b', 'a', 'd', 'c'];
        let encoded = tree.encode(test_arr);
        let decoded = tree.decode(&encoded);
        assert_eq!(decoded, test_arr);
    }

    #[test]
    fn compress_decompress_test() {
        let lines = vec![
            "hey there! nice to meet you.".to_string(),
            "Serde is a framework for serializing and deserializing Rust data structures"
                .to_string(),
        ];

        let data = compress(&lines).unwrap();
        let res_lines = decompress(data).unwrap();
        assert_eq!(&lines, &res_lines);
    }
}
