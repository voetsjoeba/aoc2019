// vim: set ai et ts=4 sts=4 sw=4:
#![allow(unused)]
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::vec::Vec;
use std::f64::consts::PI;

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
pub fn manhattan_distance(p1: (i32,i32), p2: (i32,i32)) -> u32 {
    ((p2.0 - p1.0).abs() + (p2.1 - p1.1).abs()) as u32
}
pub fn gcd(a: i32, b: i32) -> i32 {
    match b {
        0 => a,
        _ => gcd(b, a % b),
    }
}
pub fn rad2deg(rad: f64) -> f64 {
    rad*180f64/PI
}
