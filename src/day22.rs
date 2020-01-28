// vim: set ai et ts=4 sts=4 sw=4:
#![allow(non_snake_case)]
use crate::util;
use num;
use std::fmt::{self, Debug};
use std::convert::TryFrom;

#[derive(Clone,Debug)]
enum Instr {
    DealNewStack,
    DealIncrement(i128),
    Cut(i128),
}
impl From<&str> for Instr {
    fn from(line: &str) -> Self {
        match line {
            L if L.starts_with("deal into new stack") => {
                Self::DealNewStack
            },
            L if L.starts_with("deal with increment ") => {
                let n: i128 = L["deal with increment ".len()..].parse().unwrap();
                Self::DealIncrement(n)
            },
            L if L.starts_with("cut ") => {
                let n: i128 = L["cut ".len()..].parse().unwrap();
                Self::Cut(n)
            },
            _ => panic!("unrecognized instruction: '{}'", line),
        }
    }
}
impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::DealNewStack     => format!("deal into new stack"),
            Self::DealIncrement(n) => format!("deal with increment {}", n),
            Self::Cut(n)           => format!("cut {}", n),
        })
    }
}

struct Deck {
    // Represents a sequence of cards as a pair of (offset, stride) plus a modulus N.
    // In this form, the value of the card at a given position X in the deck is given by:
    //     deck[X] = (offset + X*stride) mod N
    //
    // A factory order deck corresponds to (offset=0, stride=1).
    // Each shuffling operation on the deck can be translated into a modification of the offset
    // and stride values, in such a way that the resulting values match the shuffled order of the deck.
    //
    // Note that this requires calculating the modular multiplicative inverse of the increment, which
    // can be done efficiently using the extended euclidean gcd algorithm:
    //    https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm
    //
    N: i128,
    offset: i128,
    stride: i128,
    stride_inv: i128, // modular multiplicative inverse of stride,
}
impl Deck {
    fn new(N: u64) -> Self {
        Self {
            N: i128::try_from(N).unwrap(),
            offset: 0,
            stride: 1,
            stride_inv: 1, // modular mult. inverse of 1 is always 1
        }
    }
    fn shuffle(&mut self, instrs: &Vec<Instr>) -> &mut Self {
        // shuffles this deck according to the given sequence of operations.
        //
        // deck[X] = (offset + X*stride) mod N
        //
        // DealNewStack:       (offset= offset+((N-1)*stride),  stride= -stride)       (mod N)
        // DealIncrement(n):   (offset= offset,                 stride= stride*(n^-1)) (mod N)
        // Cut(n):             (offset= offset+n*stride,        stride= stride)        (mod N)
        for inx in instrs.iter() {
            match inx {
                Instr::DealNewStack => {
                    self.offset = self.N.checked_sub(1).unwrap()
                                        .checked_mul(self.stride).unwrap()
                                        .checked_add(self.offset).unwrap()
                                        .checked_rem_euclid(self.N).unwrap();
                    self.stride = self.stride.checked_neg().unwrap()
                                             .checked_rem_euclid(self.N).unwrap();
                    self.stride_inv = util::mod_mult_inverse(self.stride, self.N);
                },
                Instr::DealIncrement(ref n) => {
                    let n_inv = util::mod_mult_inverse(*n, self.N);
                    self.stride = self.stride.checked_mul(n_inv).unwrap()
                                             .checked_rem_euclid(self.N).unwrap();
                    self.stride_inv = util::mod_mult_inverse(self.stride, self.N);
                },
                Instr::Cut(ref n) => {
                    self.offset = self.stride.checked_mul(*n).unwrap()
                                             .checked_add(self.offset).unwrap()
                                             .checked_rem_euclid(self.N).unwrap();
                }
            };
        }
        self
    }
    fn shuffle_n(&mut self, instrs: &Vec<Instr>, k: u64) -> &mut Self
    {
        // shuffles this deck according to the given sequence of operations, n times.
        //
        // Repeatedly applying this deck's shuffle sequence generates:
        //
        //     x1: deck[x] = input[o + x*s]                        (mod N)    -> i.e. Deck(o,s)
        //     x2: deck[x] = input[o + (o + x*s)*s]                (mod N)
        //                 = input[(o + s*o) + x*(s*s)]            (mod N)    -> i.e. Deck(o + s*o, s*s)
        //     x3: deck[x] = input[o + (o + (o + x*s)*s)*s]        (mod N)
        //                 = input[(o + s*o + s*s*o) + x*(s*s*s)]  (mod N)    -> i.e. Deck(o + s*o + s*s*o, s*s*s)
        //     ...
        //     k times: -> Deck(o*(1 + s + s^2 + ... + s^(k-1)), s^k)
        //               = Deck(o*(1-s^k)/(1-s), s^k)                  (for s != 1, because Geometric Series)
        //               = Deck(o*(s^k-1)/(s-1), s^k)                  (for s != 1)
        //

        use num::bigint::{BigInt};
        use num::cast::ToPrimitive;

        assert_ne!(k,0);
        macro_rules! big {
            ($num:ident) => { BigInt::from($num) }
        }

        // shuffle the deck once to determine the values of the 'o' and 's' parameters,
        // then scale those up by k as described.
        self.shuffle(instrs);
        let (N,o,s) = (self.N, self.offset, self.stride);

        if s != 1 {
            let s_pow_k: i128 = big![s].modpow(&big![k], &big![N]).to_i128().unwrap();
            let o2: i128 = o.checked_mul(s_pow_k.checked_sub(1).unwrap()).unwrap() // o*(s^k-1) ...
                            .checked_rem_euclid(N).unwrap() // keep the numbers out of overflow range
                            .checked_mul(util::mod_mult_inverse(s.checked_sub(1).unwrap(), N)).unwrap() // .../(s-1)
                            .checked_rem_euclid(N).unwrap();

            self.offset = o2;
            self.stride = s_pow_k;
        } else {
            self.offset = o*(k as i128); // o*(1 + 1 + 1^2 + ... + 1^(k-1)) = o*k
            self.stride = 1;             // 1^k = 1 for any k
        }
        self
    }
    fn index_original_to_shuffled(&self, i: u64) -> u64 {
        // given an index into the factory-ordered deck, returns the corresponding index of that value in this shuffled deck.
        //
        // the contents of this (shuffled) deck are given by:
        //     shuffled[X] = factory_order[offset + X*stride]          (mod N)
        // <=> shuffled[X - offset] = factory_order[X*stride]          (mod N)
        // <=> shuffled[(X - offset)*stride^-1] = factory_order[X] = X (mod N)
        //
        // so the answer is given by:
        //     (X-o) * s^(-1) mod N
        //
        let ii = i128::try_from(i).unwrap();
        u64::try_from(
            ii.checked_sub(self.offset).unwrap()
              .checked_mul(self.stride_inv).unwrap()
              .checked_rem_euclid(self.N).unwrap() // rem_euclid is guaranteed to output a non-negative int
        ).unwrap()
    }
    fn index_shuffled_to_original(&self, i: u64) -> u64 {
        // given an index into the shuffled deck, returns the corresponding index of that value in the factory deck.
        //
        // the contents of this (shuffled) deck are given by:
        //     shuffled[X] = factory_order[offset + X*stride]          (mod N)
        //
        // so the answer is given by:
        //     (o + X*s) mod N
        //
        let ii = i128::try_from(i).unwrap();
        u64::try_from(
            ii.checked_mul(self.stride).unwrap()
              .checked_add(self.offset).unwrap()
              .checked_rem_euclid(self.N).unwrap() // rem_euclid is guaranteed to output a non-negative int
        ).unwrap()
    }
}
impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Deck(N={}, offset={:20} incr={:20}", self.N,
                                                        format!("{},", self.offset),
                                                        format!("{})", self.stride))
    }
}

pub fn main() {
    let lines: Vec<String> = util::file_read_lines("input/day22.txt");
    let instrs: Vec<Instr> = lines.iter().map(|line| Instr::from(&line[..])).collect();
    println!("{}", part1(&instrs));
    println!("{}", part2(&instrs));
}

fn part1(instrs: &Vec<Instr>) -> u64 {
    let mut deck = Deck::new(10_007);
    deck.shuffle(instrs);
    deck.index_original_to_shuffled(2019)
}
fn part2(instrs: &Vec<Instr>) -> u64 {
    let mut deck = Deck::new(119_315_717_514_047);
    deck.shuffle_n(instrs, 101_741_582_076_661);
    deck.index_shuffled_to_original(2020)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remap<T>(input: &Vec<T>, mapped_indices: &Vec<usize>) -> Vec<T>
        where T: Copy + Default + PartialEq + Debug
    {
        // given an input vec of values and a mapping to new indices, returns a new vector
        // with the elements in their new positions.
        //
        // example:
        //          input: 0 1 2 3 4 5 6 7 8 9  (implicit indices)
        //                          |
        //                          v
        // mapped_indices: a b c ...
        //
        // means:
        //    the value that's at input[0] should go into output[a]
        //    the value that's at input[1] should go into output[b]
        //    the value that's at input[2] should go into output[c]
        //    etc.

        let mut result = Vec::<T>::new();
        result.resize(input.len(), T::default());

        for (pos, value) in input.iter().enumerate() {
            let new_pos = mapped_indices[pos];
            result[new_pos] = *value;
        }
        result
    }

    fn expected_result<T>(input: &Vec<T>,
                               instrs: &Vec<Instr>,
                               expected_output: &Vec<T>)
        where T: Copy + Default + PartialEq + Debug
    {
        let N = input.len() as u64;
        let mut deck = Deck::new(N);
        deck.shuffle(instrs);

        // compute the new index of each card after the shuffle, move the values around accordingly,
        // and check that it matches the expected result
        let forwarded_indices: Vec<usize> = (0..N).map(|idx| deck.index_original_to_shuffled(idx) as usize).collect();
        let forwarded_values = remap(input, &forwarded_indices);
        assert_eq!(forwarded_values, *expected_output);

        // now apply the reverse operation and check that it matches the input again
        let reversed_indices: Vec<usize> = (0..N).map(|idx| deck.index_shuffled_to_original(idx) as usize).collect();
        let reversed_values = remap(&forwarded_values, &reversed_indices);
        assert_eq!(reversed_values, *input);
    }

    #[test]
    fn single_operations() {
        // to help avoid confusion between values and indices, let's use chars as card values instead of numbers

        //                           0   1   2   3   4   5   6   7   8   9
        let cards: Vec<char> = vec!['A','B','C','D','E','F','G','H','I','J'];

        expected_result(&cards,  &vec![Instr::DealNewStack],     &vec!['J','I','H','G','F','E','D','C','B','A']);
        expected_result(&cards,  &vec![Instr::Cut(3)],           &vec!['D','E','F','G','H','I','J','A','B','C']);
        expected_result(&cards,  &vec![Instr::Cut(0)],           &cards);
        expected_result(&cards,  &vec![Instr::Cut(-4)],          &vec!['G','H','I','J','A','B','C','D','E','F']);
        expected_result(&cards,  &vec![Instr::DealIncrement(3)], &vec!['A','H','E','B','I','F','C','J','G','D']);
    }

    #[test]
    fn examples() {
        let factory_order = (0..10).collect();
        expected_result(
            &factory_order,
            &vec![
                Instr::DealIncrement(7),
                Instr::DealNewStack,
                Instr::DealNewStack,
            ],
            &vec![0,3,6,9,2,5,8,1,4,7],
        );
        expected_result(
            &factory_order,
            &vec![
                Instr::Cut(6),
                Instr::DealIncrement(7),
                Instr::DealNewStack,
            ],
            &vec![3,0,7,4,1,8,5,2,9,6],
        );
        expected_result(
            &factory_order,
            &vec![
                Instr::DealIncrement(7),
                Instr::DealIncrement(9),
                Instr::Cut(-2),
            ],
            &vec![6,3,0,7,4,1,8,5,2,9],
        );
        expected_result(
            &factory_order,
            &vec![
                Instr::DealNewStack,
                Instr::Cut(-2),
                Instr::DealIncrement(7),
                Instr::Cut(8),
                Instr::Cut(-4),
                Instr::DealIncrement(7),
                Instr::Cut(3),
                Instr::DealIncrement(9),
                Instr::DealIncrement(3),
                Instr::Cut(-1),
            ],
            &vec![9,2,5,8,1,4,7,0,3,6],
        );
    }
}
