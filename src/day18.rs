// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use std::fmt;
use std::iter::{FromIterator, Iterator, IntoIterator, Extend};
use std::ops::{Index, IndexMut, Add, Sub, AddAssign};
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::From;
use crate::path;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
struct Pos {
    pub x: i32,
    pub y: i32,
}
impl Pos {
    pub fn new<T>(x: T, y: T) -> Self
        where i32: From<T>
    {
        Self { x: i32::from(x), y: i32::from(y) }
    }
    pub fn x_one()     -> Self { Pos { x:1,  y:0 } }
    pub fn x_neg_one() -> Self { Pos { x:-1, y:0 } }
    pub fn y_one()     -> Self { Pos { x:0,  y:1 } }
    pub fn y_neg_one() -> Self { Pos { x:0,  y:-1} }
}
impl Add for Pos {
    type Output = Pos;
    fn add(self, other: Self) -> Self::Output {
        Self { x: self.x + other.x, y: self.y + other.y }
    }
}
impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(x={},y={})", self.x, self.y)
    }
}

#[derive(PartialEq, Eq, Debug, Hash)]
enum TileKind {
    Empty,
    Wall,
    Key(char),
    Door(char),
}
impl From<char> for TileKind {
    fn from(c: char) -> Self {
        match c {
            '.' => Self::Empty,
            '#' => Self::Wall,
            c if c.is_lowercase() => Self::Key(c),
            c if c.is_uppercase() => Self::Door(c.to_ascii_lowercase()), // match the same char as the key
            _ => panic!(),
        }
    }
}
impl fmt::Display for TileKind {
    #[allow(non_snake_case)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Empty   => '.',
            Self::Wall    => '#',
            Self::Key(c)  => *c,
            Self::Door(c) => c.to_ascii_uppercase(),
        })
    }
}

#[derive(PartialEq, Eq, Debug, Hash)]
struct Tile {
    pos: Pos,
    kind: TileKind,
}
impl Tile {
    fn new(pos: Pos, c: char) -> Self {
        Self {
            pos,
            kind: TileKind::from(c),
        }
    }
    fn key_char(&self) -> Option<char> {
        match self.kind {
            TileKind::Key(c) => Some(c),
            _                => None,
        }
    }
}
impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} at {}", self.kind, self.pos)
    }
}

struct Map {
    w: usize,
    h: usize,
    tiles: Vec<Vec<Tile>>,
    starting_pos: Pos,
}
impl Map {
    pub fn new(lines: &Vec<String>) -> Self {
        let h = lines.len();
        let w = lines[0].len();

        let mut tiles = Vec::new();
        let mut starting_pos: Option<Pos> = None;

        for (y, line) in lines.iter().enumerate() {
            let mut row_tiles = Vec::new();
            for (x, c) in line.chars().enumerate() {
                let pos = Pos::new(x as i32, y as i32);
                let tile: Tile;
                match c {
                    '@' => {
                        starting_pos = Some(pos.clone());
                        tile = Tile { pos, kind: TileKind::Empty };
                    },
                    _   => {
                        tile = Tile::new(pos, c);
                    },
                };
                row_tiles.push(tile);
            }
            tiles.push(row_tiles);
        }

        Self {
            w,
            h,
            tiles,
            starting_pos: starting_pos.unwrap(),
        }
    }
    pub fn iter(&self) -> MapIterator {
        MapIterator { map: &self, counter: 0 }
    }
    #[allow(dead_code)]
    pub fn label_at(&self, pos: &Pos) -> String {
        if *pos == self.starting_pos {
            "@".to_owned()
        } else {
            self[*pos].kind.to_string()
        }
    }
    #[allow(dead_code)]
    pub fn visualize(&self) -> String {
        self.visualize_at(&self.starting_pos, &vec![])
    }
    #[allow(dead_code)]
    pub fn visualize_at(&self, pos: &Pos, keys_collected: &Vec<char>) -> String {
        let mut result = String::new();
        for y in 0..self.h {
            for x in 0..self.w {
                if *pos == Pos::new(x as i32, y as i32) {
                    result.push_str("@");
                } else {
                    let tile = &self.tiles[y][x];
                    result.push_str(&match tile.kind {
                        TileKind::Key(c) | TileKind::Door(c) if keys_collected.contains(&c) => TileKind::Empty.to_string(),
                        _ => tile.kind.to_string(),
                    });
                }
                result.push_str(" ");
            }
            result.push_str("\n");
        }
        return result;
    }
}
impl Index<Pos> for Map {
    type Output = Tile;
    fn index(&self, pos: Pos) -> &Self::Output {
        &self.tiles[pos.y as usize][pos.x as usize]
    }
}
impl IndexMut<Pos> for Map {
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        &mut self.tiles[pos.y as usize][pos.x as usize]
    }
}


struct MapIterator<'a> {
    map: &'a Map,
    counter: usize,
}
impl<'a> Iterator for MapIterator<'a> {
    type Item = &'a Tile;
    fn next(&mut self) -> Option<Self::Item> {
        let y = self.counter / self.map.w;
        let x = self.counter % self.map.w;
        self.counter += 1;
        if y < self.map.h {
            Some(&self.map.tiles[y][x])
        } else {
            None
        }
    }
}

impl path::Node for Pos {}
impl path::Map for Map {
    type Node = Pos;
    type Cost = u32;

    fn neighbours(&self, pos: &Pos) -> Vec<(Pos, Self::Cost)> {
        let mut result = Vec::new();
        if pos.x > 0 { result.push((*pos + Pos::x_neg_one(), 1)); }
        if pos.y > 0 { result.push((*pos + Pos::y_neg_one(), 1)); }
        if pos.x < (self.w-1) as i32 { result.push((*pos + Pos::x_one(), 1)); }
        if pos.y < (self.h-1) as i32 { result.push((*pos + Pos::y_one(), 1)); }
        result
    }
}

#[derive(Copy,Clone,Debug,Hash,Eq,PartialEq)]
struct KeySet(u32);

#[allow(dead_code)]
impl KeySet {
    fn contains(&self, k: &char) -> bool {
        self.0 & Self::bit(k) != 0
    }
    fn remove(&mut self, k: &char) {
        self.0 = self.0 - (self.0 & Self::bit(k));
    }
    fn is_empty(&self) -> bool {
        self.0 == 0u32
    }
    fn iter(&self) -> KeySetIterator {
        KeySetIterator {
            bit_pattern: self.0,
            counter: 0,
        }
    }
    fn len(&self) -> usize {
        self.iter().count()
    }
    fn bit(k: &char) -> u32 {
        2u32.pow(((*k as u8) - ('a' as u8)) as u32)
    }
}
impl Default for KeySet {
    fn default() -> Self { Self(0) }
}
impl<'a> Extend<&'a char> for KeySet {
    fn extend<T>(&mut self, iter: T)
        where T: IntoIterator<Item=&'a char>
    {
        for elem in iter {
            *self += self.add(*elem);
        }
    }
}
impl From<u32> for KeySet {
    fn from(k: u32) -> Self { Self(k) }
}
impl From<char> for KeySet {
    fn from(c: char) -> Self { Self( KeySet::bit(&c) ) }
}
impl FromIterator<char> for KeySet {
    fn from_iter<I: IntoIterator<Item=char>>(iter: I) -> Self {
        let mut result = Self::default();
        for c in iter { result.add_assign(KeySet::from(c)); }
        result
    }
}
impl From<&Vec<char>> for KeySet {
    fn from(v: &Vec<char>) -> Self {
        let mut result = Self::default();
        for &c in v { result = result + Self::from(c); }
        result
    }
}
impl From<&HashSet<char>> for KeySet {
    fn from(hs: &HashSet<char>) -> Self {
        let mut result = Self::default();
        for &c in hs { result = result + Self::from(c); }
        result
    }
}
impl From<&str> for KeySet {
    fn from(s: &str) -> Self {
        let mut result = Self::default();
        for slice in s.split(",") {
            match slice.chars().nth(0) {
                Some(c) => result = result + Self::from(c),
                None    => continue,
            }
        }
        result
    }
}
impl fmt::Display for KeySet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars: Vec<String> = (0u8..32).filter(|&n| self.0 & ((1 << n) as u32) != 0)
                                          .map(|n| ((('a' as u8) + n) as char).to_string())
                                          .collect();
        write!(f, "{}", chars.join(","))
    }
}
impl Add<KeySet> for KeySet {
    type Output = Self;
    fn add(self, other: KeySet) -> Self::Output { KeySet(self.0 | other.0) }
}
impl AddAssign for KeySet {
    fn add_assign(&mut self, other: Self) {
        self.0 = self.0 | other.0;
    }
}
impl Sub<KeySet> for KeySet {
    type Output = Self;
    fn sub(self, other: KeySet) -> Self::Output { KeySet(self.0 - (self.0 & other.0)) } // keys that are in self but not in other
}
impl Sub<char> for KeySet {
    type Output = Self;
    fn sub(self, other: char) -> Self::Output { self - KeySet::from(other) }
}
impl Add<&Vec<char>> for KeySet {
    type Output = Self;
    fn add(self, other: &Vec<char>) -> Self::Output {
        self + Self::from(other)
    }
}
impl Add<char> for KeySet {
    type Output = Self;
    fn add(self, other: char) -> Self::Output { self + KeySet::from(other) }
}
impl Add<&str> for KeySet {
    type Output = Self;
    fn add(self, s: &str) -> Self::Output {
        self + Self::from(s)
    }
}

struct KeySetIterator {
    bit_pattern: u32,
    counter: usize,
}
impl Iterator for KeySetIterator {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        while self.bit_pattern > 0 {
            let mut result: Option<char> = None;
            if self.bit_pattern & 1 == 1 {
                result = Some((('a' as u8) + (self.counter as u8)) as char);
            }
            self.bit_pattern = self.bit_pattern >> 1;
            self.counter += 1;
            if result.is_some() {
                return result;
            }
        }
        None
    }
}

struct Solver<'a> {
    // contains statically-computed information about a map that we want to be able to query for
    map: &'a Map, // for ease of having the map visualize itself during debugging
    key_locations: HashMap<char, Pos>,
}
impl<'a> Solver<'a> {
    fn new(map: &'a Map) -> Self {
        let mut key_locations = HashMap::<char, Pos>::new();
        let mut door_locations = HashMap::<char, Pos>::new();
        for tile in map.iter() {
            match tile.kind {
                TileKind::Key(c)  => { key_locations.insert(c, tile.pos.clone()); },
                TileKind::Door(c) => { door_locations.insert(c, tile.pos.clone()); },
                _ => {}
            }
        }
        Self {
            map,
            key_locations,
        }
    }
    fn minimal_collection_cost(&self) -> u32 {
        // BFS over (pos, keys_collected) states, each one with an associated cost to reach it.
        // a complete path is found in states where all keys have been collected; the one of those with the
        // smallest cost is the answer. when the same state is encountered with a higher cost than previously seen,
        // we can stop expanding that path.
        let all_keys: HashSet<char> = self.key_locations.keys().copied().collect();

        let mut states_seen = HashMap::<(Pos, KeySet), u32>::new(); // state -> cost map
        let mut queue: VecDeque<(Pos, KeySet, u32)> = VecDeque::new();
        queue.push_back((self.map.starting_pos.clone(), KeySet::default(), 0));

        let mut result: Option<u32> = None;
        while !queue.is_empty() {
            let (current_pos, keys_collected, cost) = queue.pop_front().unwrap();

            // is this a final state, i.e. one in which all keys have been collected? if so, record its cost
            // and make it the new solution if it's better than any seen before.
            let remaining_keys: HashSet<char> = all_keys.difference(&keys_collected.iter().collect()).copied().collect();
            if remaining_keys.is_empty() {
                if result.is_none() || cost < result.unwrap() {
                    result = Some(cost);
                }
                continue;
            }

            // have we seen this state before, and if so, did we arrive in it through a more expensive path?
            // if so, ignore this state and don't expand on it. otherwise, record this state and
            // discover new states, i.e. reachable keys from this position with the current set of keys, and add
            // them to the queue for further exploration.
            if let Some(previously_seen_cost) = states_seen.get(&(current_pos, keys_collected)) {
                if cost > *previously_seen_cost {
                    continue;
                }
            };
            states_seen.insert((current_pos, keys_collected), cost);

            // discover new states reachable from this one, and the cost associated with reaching them
            // find shortest paths from the current position to all other keys in the map,
            assert!(remaining_keys.len() > 0);
            let (dists, came_from) = path::dijkstra(self.map, &current_pos,
                                                    |map, &pos| match map[pos].kind {
                                                        TileKind::Wall => false,
                                                        TileKind::Door(d) => keys_collected.contains(&d),
                                                        _ => true,
                                                    });
            for remaining_key in remaining_keys
            {
                let key_location: &Pos = &self.key_locations[&remaining_key];

                if let Some(path_cost) = dists.get(key_location) {
                    let path_nodes = path::Path::<Pos,Map>::reconstruct_from(key_location, &came_from);
                    assert!(path_nodes[path_nodes.len()-1] == *key_location);
                    assert!(path_nodes[0] == current_pos);

                    // for simplicity, reject paths that pick up other keys along the way to $remaining_key;
                    // i.e. we only want paths that pick up exactly one key (keys that lie behind it will be picked up
                    // in a later iteration when evaluating the states we're adding here)
                    if path_nodes[1..path_nodes.len()-1].iter().any(|p| match self.map[*p].key_char() {
                        Some(k) => !keys_collected.contains(&k),
                        None    => false,
                    }) {
                        continue;
                    }

                    let new_state = (path_nodes[path_nodes.len()-1], keys_collected + remaining_key, cost + path_cost);
                    queue.push_back(new_state);
                } else {
                    continue; // key is not directly reachable from here, try the next one
                }
            }
        }
        result.unwrap()
    }
}

pub fn main() {
    let lines = util::file_read_lines("input/day18.txt");
    let map = Map::new(&lines);
    part1(&map);
}

fn part1(map: &Map) {
    let solver = Solver::new(map);
    println!("{}", solver.minimal_collection_cost());
}

#[allow(dead_code)]
fn example_map(n: i32) -> Vec<String> {
    match n {
        1 => vec!["#########",
                  "#b.A.@.a#",
                  "#########"],

        2 => vec!["########################",
                  "#f.D.E.e.C.b.A.@.a.B.c.#",
                  "######################.#",
                  "#d.....................#",
                  "########################"],

        3 => vec!["########################",
                  "#...............b.C.D.f#",
                  "#.######################",
                  "#.....@.a.B.c.d.A.e.F.g#",
                  "########################"],

        4 => vec!["#################",
                  "#i.G..c...e..H.p#",
                  "########.########",
                  "#j.A..b...f..D.o#",
                  "########@########",
                  "#k.E..a...g..B.n#",
                  "########.########",
                  "#l.F..d...h..C.m#",
                  "#################"],

        5 => vec!["########################",
                  "#@..............ac.GI.b#",
                  "###d#e#f################",
                  "###A#B#C################",
                  "###g#h#i################",
                  "########################"],

        _ => panic!(),
    }.iter().map(|s| s.to_string()).collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples() {
        assert_eq!(Solver::new(&Map::new(&example_map(1))).minimal_collection_cost(), 8);
        assert_eq!(Solver::new(&Map::new(&example_map(2))).minimal_collection_cost(), 86);
        assert_eq!(Solver::new(&Map::new(&example_map(3))).minimal_collection_cost(), 132);
        assert_eq!(Solver::new(&Map::new(&example_map(4))).minimal_collection_cost(), 136);
        assert_eq!(Solver::new(&Map::new(&example_map(5))).minimal_collection_cost(), 81);
    }
}
