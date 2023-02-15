use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    let sentences = read_data()?.collect::<Result<Vec<_>, _>>()?;
    dbg!(&sentences[..10]);
    dbg!(&sentences[(sentences.len() - 10)..]);
    Ok(())
}

const DATA_PATH: &'static str = "data/wikisent2.txt";

// you can also write `Result<io::Lines<BufReader<File>>, ...>` as the return type, but it's less flexible
fn read_data() -> Result<impl Iterator<Item = Result<String, io::Error>>, Box<dyn Error>> {
    let file = File::open(DATA_PATH)?;
    return Ok(BufReader::new(file).lines());
}
