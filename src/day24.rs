// vim: set ai et ts=4 sts=4 sw=4:
#![allow(unused)]
use std::convert::From;
use std::collections::{HashSet, HashMap};
use std::fmt;
use crate::util;
use crate::dprint::*;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
struct Biome(u32); // biome is 5x5, so can be encoded in bits
impl Biome {
    fn bit(n: usize) -> u32 {
        1 << n
    }
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
    pub fn biodiversity_rating(&self) -> u32 {
        self.0
    }
    pub fn num_bugs(&self) -> u32 {
        self.0.count_ones() as u32
    }
    pub fn has_bug_at(&self, pos: usize) -> bool {
        let bit = Self::bit(pos);
        self.0 & bit == bit
    }
    pub fn advance_by(&self, n: usize) -> Biome {
        let mut current = self.clone();
        for _ in 0..n {
            current = current.advance();
        }
        current
    }
    pub fn advance(&self) -> Biome {
        let mut new_encoded = 0u32;
        for n in 0usize..25 {
            let num_neighbouring_bugs =   (n >= 5   && self.has_bug_at(n-5)) as usize  // upper edge
                                        + (n%5 != 0 && self.has_bug_at(n-1)) as usize  // left edge
                                        + (n%5 != 4 && self.has_bug_at(n+1)) as usize  // right edge
                                        + (n < 20   && self.has_bug_at(n+5)) as usize; // bottom edge

            if self.has_bug_at(n) {
                if num_neighbouring_bugs == 1 {
                    new_encoded |= Self::bit(n);
                }
            } else {
                if num_neighbouring_bugs == 1 || num_neighbouring_bugs == 2 {
                    new_encoded |= Self::bit(n);
                }
            }
        }
        Biome(new_encoded)
    }
    pub fn visualize(&self) -> String {
        let mut result = String::new();
        for n in 0..25 {
            let mask = Self::bit(n);
            if self.0 & mask == mask {
                result.push_str("# ");
            } else {
                result.push_str(". ");
            }
            if (n+1) % 5 == 0 {
                result.push('\n');
            }
        }
        result.truncate(result.trim_end().len()); // right trim in place
        result
    }
}
impl Default for Biome {
    fn default() -> Biome {
        Biome(0)
    }
}
impl From<&Vec<&str>> for Biome {
    fn from(lines: &Vec<&str>) -> Self {
        let mut encoded = 0u32;
        for n in 0..25 {
            if lines[n/5].chars().nth(n%5).unwrap() == '#' {
                encoded |= Self::bit(n);
            }
        }
        Self(encoded)
    }
}
impl fmt::Display for Biome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.visualize())
    }
}

#[derive(Clone)]
struct RecursiveBiome {
    levels: HashMap<i32, Biome>,
}
struct RecLocation { // identifies a position in the recursive biome
    level: i32,
    index: usize
}
macro_rules! recpos {
    ($level:expr, $index:expr) => {RecLocation { level: $level, index: $index }};
}

impl RecursiveBiome {
    pub fn new(initial_biome: &Biome) -> Self {
        let mut levels = HashMap::<i32, Biome>::new();
        levels.insert(0, initial_biome.clone());
        Self { levels }
    }
    pub fn neighbours_of(pos: &RecLocation) -> Vec<RecLocation> {
        // given a position within the recursive biome, determines its neighbour positions.
        // note: "outer" levels are considered level -1, "inner" levels are considered +1.
        //
        //          |     |         |     |
        //       0  |  1  |    2    |  3  |  4
        //          |     |         |     |
        //     -----+-----+---------+-----+-----
        //          |     |         |     |
        //       5  |  6  |    7    |  8  |  9
        //          |     |         |     |
        //     -----+-----+---------+-----+-----
        //          |     |A|B|C|D|E|     |
        //          |     |-+-+-+-+-|     |
        //          |     |F|G|H|I|J|     |
        //          |     |-+-+-+-+-|     |
        //      10  | 11  |K|L|?|N|O|  13 |  14
        //          |     |-+-+-+-+-|     |
        //          |     |P|Q|R|S|T|     |
        //          |     |-+-+-+-+-|     |
        //          |     |U|V|W|X|Y|     |
        //     -----+-----+---------+-----+-----
        //          |     |         |     |
        //      15  | 16  |    17   |  18 |  19
        //          |     |         |     |
        //     -----+-----+---------+-----+-----
        //          |     |         |     |
        //      20  | 21  |    22   |  23 |  24
        //          |     |         |     |
        //
        let mut result = Vec::<RecLocation>::new();
        
        // determine this position's upper neighbour
        if pos.index < 5 {
            result.push(recpos![pos.level-1, 7]);                                // has 1 upper neighbour in outer level
        } else if pos.index == 17 {
            result.extend((20..25).map(|i| recpos![pos.level+1, i]));           // has 5 upper neighbours in inner level
        } else {
            result.push(recpos![pos.level, pos.index-5]);                        // has 1 upper neighbour in current level
        }

        // determine this position's left neighbour
        if pos.index % 5 == 0 {
            result.push(recpos![pos.level-1, 11]);                               // has 1 left neighbour in outer level
        } else if pos.index == 13 {
            result.extend([4,9,14,19,24].iter().map(|&i| recpos![pos.level+1, i])); // has 5 left neighbours in inner level
        } else {
            result.push(recpos![pos.level, pos.index-1]);                        // has 1 left neighbour in current level
        }

        // determine this position's right neighbour
        if pos.index % 5 == 4 {
            result.push(recpos![pos.level-1, 13]);                               // has 1 right neighbour in outer level
        } else if pos.index == 11 {
            result.extend([0,5,10,15,20].iter().map(|&i| recpos![pos.level+1, i])); // has 5 right neighbours in inner level
        } else {
            result.push(recpos![pos.level, pos.index+1]);                        // has 1 right neighbour in current level
        }

        // determine this position's bottom neighbour
        if pos.index >= 20 {
            result.push(recpos![pos.level-1, 17]);                               // has 1 bottom neighbour in outer level
        } else if pos.index == 7 {
            result.extend((0..5).map(|i| recpos![pos.level+1, i]));             // has 5 bottom neighbours in inner level
        } else {
            result.push(recpos![pos.level, pos.index+5]);                        // has 1 bottom neighbour in current level
        }

        result
    }
    pub fn has_bug_at(&self, pos: &RecLocation) -> bool {
        // look up the requested level in the stack; if that level doesn't exist in the stack,
        // then that means it's empty and the result is therefore necessarily false
        if let Some(biome) = self.levels.get(&pos.level) {
            biome.has_bug_at(pos.index)
        } else {
            false
        }
    }
    pub fn num_bugs(&self) -> u32 {
        self.levels.values().map(|biome| biome.num_bugs()).sum()
    }
    pub fn advance_by(&self, n: usize) -> RecursiveBiome {
        let mut current = self.clone();
        for _ in 0..n {
            current = current.advance();
        }
        current
    }
    pub fn advance(&self) -> RecursiveBiome {
        let mut result = self.clone();

        // record the new state of all the bugs at the currently-recorded biome levels,
        // (but leave out their center position at each biome level since those contain deeper recursion
        //  levels and shouldn't be regarded as containing bugs)
        for (&level, biome) in &self.levels {
            let mut new_encoded = 0u32;
            for n in 0..25 {
                if n == 12 { continue; } // skip center position
                let pos = recpos![level, n];
                let num_neighbouring_bugs = Self::neighbours_of(&pos)
                                                .iter()
                                                .filter(|p| self.has_bug_at(p))
                                                .count();

                // TODO: copy/paste from Biome::advance
                if self.has_bug_at(&pos) {
                    if num_neighbouring_bugs == 1 {
                        new_encoded |= Biome::bit(n);
                    }
                } else {
                    if num_neighbouring_bugs == 1 || num_neighbouring_bugs == 2 {
                        new_encoded |= Biome::bit(n);
                    }
                }
            }
            result.levels.insert(level, Biome(new_encoded));
        }

        let max_level: i32 = *self.levels.keys().max().unwrap();
        let min_level: i32 = *self.levels.keys().min().unwrap();

        // additionally, spawn a new empty outermost and innermost biome, and see if any of the bugs
        // along their rim to the previous level have been affected, and record those as well.
        // if they are non-empty, add those new biomes to the result; otherwise omit them to save some memory.
        let mut new_outermost = Biome::default();
        for &n in [7,11,13,17].iter() {
            let pos = recpos![min_level-1, n];
            let num_neighbouring_bugs = Self::neighbours_of(&pos)
                                            .iter()
                                            .filter(|p| self.has_bug_at(p)) // note: has_bug_at() wil transparently deal with this new level number and return false for unknown levels like this one
                                            .count();

            // we only need to consider whether to change an empty spot into a bug,
            // since these levels start off empty
            if num_neighbouring_bugs == 1 || num_neighbouring_bugs == 2 {
                new_outermost.0 |= Biome::bit(n);
            }
        }

        let mut new_innermost = Biome::default();
        for &n in [ 0,  1,  2,  3,  4,
                    5,              9,
                   10,             14,
                   15,             19,
                   20, 21, 22, 23, 24 ].iter()
        {
            let pos = recpos![max_level+1, n];
            let num_neighbouring_bugs = Self::neighbours_of(&pos)
                                            .iter()
                                            .filter(|p| self.has_bug_at(p)) // note: has_bug_at() wil transparently deal with this new level number and return false for unknown levels like this one
                                            .count();

            // we only need to consider whether to change an empty spot into a bug,
            // since these levels start off empty
            if num_neighbouring_bugs == 1 || num_neighbouring_bugs == 2 {
                new_innermost.0 |= Biome::bit(n);
            }
        }

        if !new_outermost.is_empty() {
            result.levels.insert(min_level-1, new_outermost);
        }
        if !new_innermost.is_empty() {
            result.levels.insert(max_level+1, new_innermost);
        }
        result
    }
    #[allow(non_snake_case)]
    pub fn visualize(&self) -> String {
        let mut result = String::new();

        let mut levels: Vec<i32> = self.levels.keys().copied().collect();
        levels.sort();
        for L in levels {
            let biome = &self.levels[&L];
            result.push_str(&format!("Level {}:\n", L));
            result.push_str(&biome.visualize());
            result.push_str("\n\n");
        }
        result.truncate(result.trim_end().len()); // right trim in place
        result
    }
}

impl fmt::Display for RecursiveBiome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.visualize())
    }
}


pub fn main() {
    let lines: Vec<String> = util::file_read_lines("input/day24.txt");
    let biome = Biome::from(&lines.iter().map(|line| &line[..]).collect());
    println!("{}", part1(&biome));
    println!("{}", part2(&biome));
}

fn part1(biome: &Biome) -> u32 {
    let mut seen = HashSet::<Biome>::new();
    let mut current_state = biome.clone();
    loop {
        if seen.contains(&current_state) {
            return current_state.biodiversity_rating();
        }
        seen.insert(current_state.clone());
        current_state = current_state.advance();
    }
}

fn part2(biome: &Biome) -> u32 {
    let mut biome = RecursiveBiome::new(biome);
    biome.advance_by(200).num_bugs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples() {
        let stages = vec![
            Biome::from(&vec![
                "....#",
                "#..#.",
                "#..##",
                "..#..",
                "#....",
            ]),
            Biome::from(&vec![
                "#..#.",
                "####.",
                "###.#",
                "##.##",
                ".##..",
            ]),
            Biome::from(&vec![
                "#####",
                "....#",
                "....#",
                "...#.",
                "#.###",
            ]),
            Biome::from(&vec![
                "#....",
                "####.",
                "...##",
                "#.##.",
                ".##.#",
            ]),
            Biome::from(&vec![
                "####.",
                "....#",
                "##..#",
                ".....",
                "##...",
            ]),
        ];
        assert_eq!(stages[0].advance(), stages[1]);
        assert_eq!(stages[1].advance(), stages[2]);
        assert_eq!(stages[2].advance(), stages[3]);
        assert_eq!(stages[3].advance(), stages[4]);

        assert_eq!(Biome::from(&vec![
            ".....",
            ".....",
            ".....",
            "#....",
            ".#...",
        ]).biodiversity_rating(), 2129920);
    }

    #[test]
    fn recursive_example() {
        let mut rec_biome = RecursiveBiome::new(
            &Biome::from(&vec![
                "....#",
                "#..#.",
                "#..##",
                "..#..",
                "#....",
            ])
        );
        assert_eq!(rec_biome.advance_by(10).num_bugs(), 99);
    }

}
