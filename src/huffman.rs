use bit_vec::BitVec;
use rayon::prelude::*;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

pub fn learn_frequencies(lines: &Vec<String>) -> HashMap<char, i64> {
    lines
        .par_iter()
        .fold(
            || HashMap::new(),
            |mut freqs: HashMap<_, _>, line: &String| {
                for ch in line.chars() {
                    *freqs.entry(ch).or_insert(0) += 1;
                }
                freqs
            },
        )
        .reduce(
            || HashMap::new(),
            |mut freqs1, freqs2| {
                freqs2
                    .into_iter()
                    .for_each(|(ch, n)| *freqs1.entry(ch).or_insert(0) += n);
                freqs1
            },
        )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tree<T> {
    Leaf {
        freq: i64,
        token: T,
    },
    Node {
        freq: i64,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}

use Tree::*;

impl<T: Clone> Tree<T> {
    pub fn freq(&self) -> i64 {
        match self {
            Leaf { freq, .. } => *freq,
            Node { freq, .. } => *freq,
        }
    }

    pub fn token(&self) -> Option<T> {
        match self {
            Leaf { token, .. } => Some(token.clone()),
            Node { .. } => None,
        }
    }

    pub fn left(&self) -> Option<&Tree<T>> {
        match self {
            Node { left, .. } => Some(left),
            Leaf { .. } => None,
        }
    }

    pub fn right(&self) -> Option<&Tree<T>> {
        match self {
            Node { right, .. } => Some(right),
            Leaf { .. } => None,
        }
    }
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

    pub fn to_decoder(&self, encoder: Option<HashMap<T, BitVec>>) -> HashMap<BitVec, T> {
        let encoder = encoder.unwrap_or(self.to_encoder());

        let mut decoder = HashMap::new();
        for (token, prefix) in encoder {
            decoder.insert(prefix, token);
        }
        decoder
    }
}

impl<T: Clone + Eq> Ord for Tree<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.freq().cmp(&other.freq())
    }
}

impl<T: Clone + Eq> PartialOrd for Tree<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn build_huffman_tree<T: Eq + Clone>(freqs: &HashMap<T, i64>) -> Tree<T> {
    let mut heap = BinaryHeap::new();
    for (token, freq) in freqs {
        let (freq, token) = (*freq, token.clone());
        heap.push(Reverse(Leaf { freq, token }))
    }

    while heap.len() > 1 {
        let node1 = heap.pop().unwrap().0;
        let node2 = heap.pop().unwrap().0;

        let merged_node = Node {
            freq: node1.freq() + node2.freq(),
            left: Box::new(node1),
            right: Box::new(node2),
        };
        heap.push(Reverse(merged_node));
    }

    heap.pop().unwrap().0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learn_frequencies_test() {
        let input = vec!["this is an epic sentence".to_string(), "xyz ".to_string()];
        let freqs = learn_frequencies(&input);
        assert_eq!(freqs[&' '], 5);
        assert_eq!(freqs[&'t'], 2);
        assert_eq!(freqs[&'i'], 3);
        assert_eq!(freqs[&'p'], 1);
        assert_eq!(freqs[&'z'], 1);
        assert_eq!(freqs.keys().len(), 13);
    }

    #[test]
    fn huffman_tree_test() {
        let mut freqs = HashMap::new();
        freqs.insert('a', 40);
        freqs.insert('b', 35);
        freqs.insert('c', 20);
        freqs.insert('d', 5);

        let tree = build_huffman_tree(&freqs);
        assert_eq!(tree.freq(), 100);

        // the most frequent character only requires 1 bit
        assert_eq!(tree.left().and_then(|n| n.token()), Some('a'));
        assert_eq!(tree.left().map(|n| n.freq()), Some(40));

        // the second most frequent character requires 2 bits
        assert_eq!(
            tree.right().and_then(|t| t.right()).and_then(|n| n.token()),
            Some('b')
        );
        assert_eq!(
            tree.right().and_then(|t| t.right()).map(|n| n.freq()),
            Some(35)
        );

        // the least frequent characters require 3 bits
        assert_eq!(
            tree.right()
                .and_then(|t| t.left())
                .and_then(|t| t.left())
                .and_then(|n| n.token()),
            Some('d')
        );
        assert_eq!(
            tree.right()
                .and_then(|t| t.left())
                .and_then(|t| t.left())
                .map(|n| n.freq()),
            Some(5)
        );

        assert_eq!(
            tree.right()
                .and_then(|t| t.left())
                .and_then(|t| t.right())
                .and_then(|n| n.token()),
            Some('c')
        );
        assert_eq!(
            tree.right()
                .and_then(|t| t.left())
                .and_then(|t| t.right())
                .map(|n| n.freq()),
            Some(20)
        );

        let encoder = tree.to_encoder();
        assert!(encoder.get(&'a').unwrap().eq_vec(&[false]));
        assert!(encoder.get(&'b').unwrap().eq_vec(&[true, true]));
        assert!(encoder.get(&'c').unwrap().eq_vec(&[true, false, true]));
        assert!(encoder.get(&'d').unwrap().eq_vec(&[true, false, false]));

        let decoder = tree.to_decoder(Some(encoder.clone()));
        assert_eq!(decoder.len(), 4);

        let mut c_path = BitVec::new();
        c_path.push(true);
        c_path.push(false);
        c_path.push(true);
        assert_eq!(decoder.get(&c_path), Some(&'c'));
    }
}
