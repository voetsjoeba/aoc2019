// vim: set ai et ts=4 sts=4 sw=4:
#![allow(unused)]
use crate::dprint::*;
use num;
use std::str;
use std::fs::File;
use std::cmp::{min,max,PartialEq,Ordering};
use std::ops::Rem;
use std::io::{BufReader, BufRead};
use std::vec::Vec;
use std::fmt::{Debug, Display};
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
pub fn mod_mult_inverse<T>(a: T, n: T) -> T
    where T: num::Integer + Clone + Debug
{
    // computes a^(-1) mod n, i.e. x such that a*x = 1 (mod n)
    let gcd_ex = a.extended_gcd(&n);
    assert_eq!(gcd_ex.gcd, T::one()); // otherwise the modular multiplicative inverse does not exist
    gcd_ex.x
}
pub fn gcd<T>(a: T, b: T) -> T
    where T: num::Integer
{
    a.gcd(&b)
}
pub fn rad2deg(rad: f64) -> f64 {
    rad*180f64/PI
}
pub fn factorial(n: u64) -> u64 {
    match n {
        0 => 1,
        _ => n * factorial(n-1),
    }
}
pub fn index_of<T>(vec: &Vec<T>, needle: &T) -> usize
    where T: PartialEq
{
    vec.iter().position(|item| item == needle).unwrap()
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
        let s1: &str = &suffixes[i];
        for neighbour in (1..neighbours_to_check+1).rev() {
            let s2: &str = &suffixes[i-neighbour];

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

            best = Some(str::from_utf8(lcp_max(s1.as_bytes(), &s2.as_bytes(), distance)).unwrap());
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

pub fn lcp<'a, 'b, T>(s1: &'a [T],
                      s2: &'b [T]) -> &'a [T] // longest common prefix
    where T: PartialEq
{
    let common_len: usize = min(s1.len(), s2.len());
    for i in 0..common_len {
        if s1[i..i+1] != s2[i..i+1] {
            return &s1[0..i];
        }
    }
    return &s1[0..common_len];
}
pub fn lcp_str<'a,'b>(s1: &'a str, s2: &'b str) -> &'a str {
    str::from_utf8(lcp(s1.as_bytes(), s2.as_bytes())).unwrap()
}
pub fn lcp_max<'a, 'b, T>(s1: &'a [T],
                          s2: &'b [T],
                          max: usize) -> &'a [T] // longest common prefix, capped to a maximum length
    where T: PartialEq
{
    let lcp = lcp(s1, s2);
    &lcp[..min(lcp.len(), max)]
}
pub fn lcp_str_max<'a,'b>(s1: &'a str, s2: &'b str, max: usize) -> &'a str {
    str::from_utf8(lcp_max(s1.as_bytes(), s2.as_bytes(), max)).unwrap()
}

pub fn ordered_permutations<T,O,C>(of: &Vec<T>,
                                   mut order_by: O,
                                   mut callback: C)
    where O: FnMut(&T,&T) -> Ordering,
          C: FnMut(&Vec<T>) -> (),
          T: Copy + Display + PartialEq + Debug,
{
    let mut working_copy: Vec<T> = Vec::new();
    let mut remaining: Vec<T> = of.clone();
    ordered_permutations_r(&mut working_copy, &remaining, &mut order_by, &mut callback);
}
pub fn ordered_permutations_r<T,O,C>(working_copy: &mut Vec<T>,
                                     remaining: &Vec<T>,
                                     order_by: &mut O,
                                     callback: &mut C)
    where O: FnMut(&T,&T) -> Ordering,
          C: FnMut(&Vec<T>) -> (),
          T: Copy + Display + PartialEq + Debug,
{
    if remaining.len() == 0 {
        callback(&working_copy);
        return;
    }

    // the possible options for the next position are those who are <= all others in the remaining list,
    // according to the given order
    let options: Vec<&T> = remaining.iter()
                                    .filter(|item| remaining.iter().all(|other| match order_by(item, other) {
                                        Ordering::Less | Ordering::Equal => true,
                                        _ => false,
                                    }))
                                    .collect();

    for option in options {
        let mut new_options = remaining.clone();
        new_options.retain(|item| item != option);

        working_copy.push(*option);
        ordered_permutations_r(working_copy, &new_options, order_by, callback);
        working_copy.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_repeated_substr() {
        assert_eq!(lcp_str("ABC", "ABCD"), "ABC");
        assert_eq!(lcp_str("ABCD", "ABC"), "ABC");
        assert_eq!(lcp_str("ABC", "XYZ"), "");
        assert_eq!(lcp_str("", ""), "");
        assert_eq!(lcp_str_max("ABCD", "ABC", 3), "ABC");
        assert_eq!(lcp_str_max("ABCD", "ABC", 2), "AB");
        assert_eq!(lcp_str_max("ABCD", "ABC", 0), "");
        assert_eq!(longest_repeated_substring_no_overlap("ABCDxxxABCD"), "ABCD");
        assert_eq!(longest_repeated_substring_no_overlap("ABCDxxxABC"), "ABC");
        assert_eq!(longest_repeated_substring_no_overlap("ABCDEFG"), "");
        assert_eq!(longest_repeated_substring_no_overlap("L,R,U,D,8,L,2,L,R,D,U"), "L,R,");
    }

}
