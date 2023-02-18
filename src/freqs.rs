use rayon::prelude::*;
use std::collections::HashMap;

pub fn char_frequencies(lines: &Vec<String>) -> HashMap<char, i64> {
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

pub fn word_frequencies(lines: &Vec<String>) -> HashMap<String, i64> {
    lines
        .par_iter()
        .fold(
            || HashMap::new(),
            |mut freqs: HashMap<_, _>, line: &String| {
                for word in line.split_ascii_whitespace() {
                    *freqs.entry(word.to_string()).or_insert(0) += 1;
                }
                freqs
            },
        )
        .reduce(
            || HashMap::new(),
            |mut freqs1, freqs2| {
                freqs2
                    .into_iter()
                    .for_each(|(word, n)| *freqs1.entry(word).or_insert(0) += n);
                freqs1
            },
        )
}
