use bit_vec::BitVec;
use rayon::prelude::*;
use std::{collections::HashMap, fs, hash::Hash, path::Path};

use crate::{
    freqs,
    huffman::{self, Tree},
};
use Tree::*;

// TODO: use preprocess
pub fn compress_file<P: AsRef<Path>>(path: P) -> Result<BitVec, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let lines: Vec<_> = text.split_inclusive('\n').map(|x| x.to_string()).collect();

    let freqs = freqs::learn_word_frequencies(&lines);
    let tree = huffman::build_huffman_tree(&freqs);
    let encoder = tree.to_encoder();

    let compressed = lines
        .par_iter()
        .flat_map(|line| {
            // TODO: comparison between words & chars compression performance
            // it seems that words compression yields 1847299618 / 8 / 1024 / 1024 ~= 220MB (w/o header)
            // and chars compression yields 4385985563 / 8 / 1024 / 1024 ~= 523MB (w/o header)
            // let chs: Vec<_> = line.chars().collect();
            let chs: Vec<_> = line
                .split_ascii_whitespace()
                .map(|s| s.to_string())
                .collect();
            chs.into_par_iter()
                .map(|ch| encoder.get(&ch).unwrap().clone())
        })
        .reduce(
            || BitVec::new(),
            |mut v1: BitVec, v2: BitVec| {
                v1.extend(v2);
                v1
            },
        );
    Ok(compressed)
}

pub fn decompress_file() {
    // TODO:
    // et codes: Vec<BitVec> = rmp_serde::from_slice(data)?;
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
}
