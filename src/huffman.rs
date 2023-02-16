use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Error;

pub fn learn_frequencies(lines: &Vec<String>) -> HashMap<char, i32> {
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
}
