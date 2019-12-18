// vim: set ai et ts=4 sts=4 sw=4:
#![allow(unused)]
use std::fs::File;
use std::cmp::{min,max};
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

#[allow(non_snake_case)]
pub fn longest_repeated_substring_no_overlap<'a>(s: &'a str) -> &'a str {
    // the longest repeated substring is a known problem that can be solved efficiently using
    // a suffix tree, but this algorithm allows for overlaps. for our purposes, we need the
    // longest non-overlapping repeated substring, which is trickier.
    //
    // I found an implementation here: from http://rubyquiz.com/quiz153.html

    let L = s.len();
    let mut suffixes: Vec<&str> = (0..L).map(|i| &s[i..]).collect();
    suffixes.sort();

    let mut best: Option<&str> = None;
    let mut at_least_size = 1usize;
    let mut distance = 0usize;
    let mut neighbours_to_check = 1; // look up to N positions behind our own in the sorted list of suffixes

    for i in 1..L {
        let s1 = &suffixes[i];
        for neighbour in (1..neighbours_to_check+1).rev() {
            let s2 = &suffixes[i-neighbour];

            distance = ((s1.len() as i64) - (s2.len() as i64)).abs() as usize;
            if distance < at_least_size {
                if s1.len() >= at_least_size &&
                   s2.len() >= at_least_size &&
                   s1[..at_least_size] == s2[..at_least_size]
                {
                    neighbours_to_check = max(neighbours_to_check, neighbour + 1);
                } else {
                    neighbours_to_check = neighbour;
                }
            }

            if s1[..min(at_least_size,s1.len())] != s2[..min(at_least_size,s2.len())] {
                neighbours_to_check = neighbour;
                continue;
            }

            best = Some(lcp_max(s1, s2, distance));
            at_least_size = best.unwrap().len() + 1;
            if best.unwrap().len() == distance {
                neighbours_to_check = max(neighbours_to_check, neighbour + 1);
            } else {
                neighbours_to_check = neighbour;
            }
        }
    }

    best.unwrap_or("")
}

pub fn lcp<'a, 'b>(s1: &'a str, s2: &'b str) -> &'a str { // longest common prefix
    let common_len: usize = min(s1.len(), s2.len());
    for i in 0..common_len {
        if s1[i..i+1] != s2[i..i+1] {
            return &s1[0..i];
        }
    }
    return &s1[0..common_len];
}
pub fn lcp_max<'a, 'b>(s1: &'a str, s2: &'b str, max: usize) -> &'a str { // longest common prefix, capped to a maximum length
    let lcp = lcp(s1, s2);
    &lcp[..min(lcp.len(), max)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_repeated_substr() {
        assert_eq!(lcp("ABC", "ABCD"), "ABC");
        assert_eq!(lcp("ABCD", "ABC"), "ABC");
        assert_eq!(lcp("ABC", "XYZ"), "");
        assert_eq!(lcp("", ""), "");
        assert_eq!(lcp_max("ABCD", "ABC", 3), "ABC");
        assert_eq!(lcp_max("ABCD", "ABC", 2), "AB");
        assert_eq!(lcp_max("ABCD", "ABC", 0), "");
        assert_eq!(longest_repeated_substring_no_overlap("ABCDxxxABCD"), "ABCD");
        assert_eq!(longest_repeated_substring_no_overlap("ABCDxxxABC"), "ABC");
        assert_eq!(longest_repeated_substring_no_overlap("ABCDEFG"), "");
        assert_eq!(longest_repeated_substring_no_overlap("L,R,U,D,8,L,2,L,R,D,U"), "L,R,");
    }

}
