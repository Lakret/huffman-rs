use rayon::prelude::*;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
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
pub enum Tree {
    Leaf {
        freq: i64,
        ch: char,
    },
    Node {
        freq: i64,
        left: Box<Tree>,
        right: Box<Tree>,
    },
}

use Tree::*;

impl Tree {
    pub fn freq(&self) -> i64 {
        match self {
            Leaf { freq, .. } => *freq,
            Node { freq, .. } => *freq,
        }
    }

    pub fn ch(&self) -> Option<char> {
        match self {
            Leaf { ch, .. } => Some(*ch),
            Node { .. } => None,
        }
    }

    pub fn left(&self) -> Option<&Tree> {
        match self {
            Node { left, .. } => Some(left),
            Leaf { .. } => None,
        }
    }

    pub fn right(&self) -> Option<&Tree> {
        match self {
            Node { right, .. } => Some(right),
            Leaf { .. } => None,
        }
    }
}

impl Ord for Tree {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.freq().cmp(&other.freq())
    }
}

impl PartialOrd for Tree {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn build_huffman_tree(freqs: &HashMap<char, i64>) -> Tree {
    let mut heap = BinaryHeap::new();
    for (ch, freq) in freqs {
        let (freq, ch) = (*freq, *ch);
        heap.push(Reverse(Leaf { freq, ch }))
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
    fn build_huffman_tree_test() {
        let mut freqs = HashMap::new();
        freqs.insert('a', 40);
        freqs.insert('b', 35);
        freqs.insert('c', 20);
        freqs.insert('d', 5);

        let tree = build_huffman_tree(&freqs);
        assert_eq!(tree.freq(), 100);

        // the most frequent character only requires 1 bit
        assert_eq!(tree.left().and_then(|n| n.ch()), Some('a'));
        assert_eq!(tree.left().map(|n| n.freq()), Some(40));

        // the second most frequent character requires 2 bits
        assert_eq!(
            tree.right().and_then(|t| t.right()).and_then(|n| n.ch()),
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
                .and_then(|n| n.ch()),
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
                .and_then(|n| n.ch()),
            Some('c')
        );
        assert_eq!(
            tree.right()
                .and_then(|t| t.left())
                .and_then(|t| t.right())
                .map(|n| n.freq()),
            Some(20)
        );
    }
}
