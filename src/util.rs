use std::fs::File;
use std::io::{BufReader, BufRead};
use std::vec::Vec;

pub fn file_read_lines(filename: &str) -> Vec<String> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    reader.lines().map(|l| l.unwrap()).collect()
}
pub fn file_read_i64s(filename: &str) -> Vec<i64> {
    file_read_lines(filename).iter()
                             .map(|s| s.parse().unwrap())
                             .collect()
}
