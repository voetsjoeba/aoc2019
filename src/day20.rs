// vim: set ai et ts=4 sts=4 sw=4:
use std::fmt;
use std::ops::{Add, Index, IndexMut};
use std::collections::{HashMap};
use crate::util;
use crate::path;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
struct Pos {
    pub x: i32,
    pub y: i32,
    pub level: i32,
}
macro_rules! pos {
    [$x:expr, $y:expr, $lvl:expr] => { Pos { x: $x as i32, y: $y as i32, level: $lvl as i32 } };
    [$x:expr, $y:expr]            => { Pos { x: $x as i32, y: $y as i32, level: 0 } };
}
impl Pos {
    pub fn x_one()     -> Self { pos![1, 0] }
    pub fn x_neg_one() -> Self { pos![-1,0] }
    pub fn y_one()     -> Self { pos![0, 1] }
    pub fn y_neg_one() -> Self { pos![0,-1] }

    pub fn at_level(&self, level: i32) -> Pos {
        // returns a new position that's the same as this one but with its level set to the given value
        pos![self.x, self.y, level]
    }
}
impl Add for Pos {
    type Output = Pos;
    fn add(self, other: Self) -> Self::Output {
        Self { x: self.x + other.x, y: self.y + other.y, level: self.level }
    }
}
impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(x={},y={},d={})", self.x, self.y, self.level)
    }
}

#[derive(PartialEq, Eq, Debug, Hash)]
struct PortalInfo {
    label: String,
    attached_passage: Pos,
    on_outer_edge: bool,
}

#[derive(PartialEq, Eq, Debug, Hash)]
enum TileKind {
    Void,
    Passage,
    Wall,
    Portal(PortalInfo),
}

#[derive(PartialEq, Eq, Debug, Hash)]
struct Tile {
    pos: Pos,
    kind: TileKind,
}
impl Tile {
    pub fn is_wall(&self)    -> bool { match self.kind { TileKind::Wall      => true, _ => false } }
    pub fn is_passage(&self) -> bool { match self.kind { TileKind::Passage   => true, _ => false } }
    pub fn is_portal(&self)  -> bool { match self.kind { TileKind::Portal(_) => true, _ => false } }
    pub fn portal_info(&self) -> &PortalInfo {
        match self.kind {
            TileKind::Portal(ref info) => {
                info
            },
            _ => panic!("tile at position {} is not a portal", self.pos),
        }
    }
}

macro_rules! tile_index {
    ($pos:ident, $map_width:expr) => { ($pos.y as usize) * $map_width + $pos.x as usize };
}
struct Map {
    w: usize,
    h: usize,
    tiles: Vec<Tile>,
    starting_pos: Pos,
    target_pos: Pos,
    portal_pairs: HashMap<Pos, Pos>, // for every portal position, records the other end of the portal
    recursive_portals: bool,
}
#[allow(dead_code)]
impl Map {
    pub fn new(lines: &Vec<String>, recursive_portals: bool) -> Self {
        let h = lines.len();
        let w = lines.iter().map(|line| line.len()).max().unwrap();

        // make sure all lines are of the same width, appending whitespace where necessary
        let mut lines = lines.clone();
        for line in &mut lines {
            line.push_str(&" ".repeat(w-line.len()));
        }

        let mut tiles = Vec::<Tile>::with_capacity(w * h);
        let mut starting_pos: Option<Pos> = None;
        let mut target_pos: Option<Pos> = None;
        let mut portal_locations = HashMap::<String, Vec<Pos>>::new(); // label -> pair of locations

        for (y, line) in lines.iter().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let pos = pos![x,y];

                let tile: Tile;
                match c {
                    ' ' => {
                        tile = Tile { pos, kind: TileKind::Void };
                    },
                    '#' => {
                        tile = Tile { pos, kind: TileKind::Wall };
                    },
                    '.' => {
                        tile = Tile { pos, kind: TileKind::Passage };
                    }
                    'A'..='Z' => {
                        macro_rules! record_portal_location {
                            ($pos:ident, $label:ident, $attached_passage_pos:ident) => {{
                                // "AA" is not an actual portal but a marker label for the starting position
                                if $label == "AA" {
                                    starting_pos = Some($attached_passage_pos);
                                    tile = Tile { pos: $pos, kind: TileKind::Void };
                                }
                                // "ZZ" is not an actual portal but a marker label for the target position
                                else if $label == "ZZ" {
                                    target_pos = Some($attached_passage_pos);
                                    tile = Tile { pos: $pos, kind: TileKind::Void };
                                }
                                else {
                                    portal_locations.entry($label.clone()).or_insert(vec![]).push($pos.clone());
                                    tile = Tile {
                                        pos: $pos,
                                        kind: TileKind::Portal(PortalInfo {
                                            label: $label,
                                            attached_passage: $attached_passage_pos,
                                            on_outer_edge: false, // to be revised later
                                        }),
                                    };
                                }
                            }}
                        }
                        // there are four ways in which portals can be specified on the map:
                        //   1) XY. or .XY
                        //   2) X   or .
                        //      Y      X
                        //      .      Y
                        // when we're on the character that's right next to a passage (i.e. '.'),
                        // record the position of this portal; when we're on the other label, record a void tile instead.
                        let mut label = String::new();
                        if y > 0 && lines[y-1].chars().nth(x).unwrap() == '.' {
                            // passage is above us, so we're in this case: .
                            //                                             X <--
                            //                                             Y
                            label.push(c);
                            label.push(lines[y+1].chars().nth(x).unwrap());
                            let passage_pos = pos + Pos::y_neg_one();
                            record_portal_location!(pos, label, passage_pos);
                        }
                        else if y < h-1 && lines[y+1].chars().nth(x).unwrap() == '.' {
                            // passage is below us, so we're in this case: X
                            //                                             Y <--
                            //                                             .
                            label.push(lines[y-1].chars().nth(x).unwrap());
                            label.push(c);
                            let passage_pos = pos + Pos::y_one();
                            record_portal_location!(pos, label, passage_pos);
                        }
                        else if x > 0 && line.chars().nth(x-1).unwrap() == '.' {
                            // passage is to our left, so we're in this case: . X Y
                            //                                                  ^
                            label.push(c);
                            label.push(line.chars().nth(x+1).unwrap());
                            let passage_pos = pos + Pos::x_neg_one();
                            record_portal_location!(pos, label, passage_pos);
                        }
                        else if x < w-1 && line.chars().nth(x+1).unwrap() == '.' {
                            // passage is to our right, so we're in this case: X Y .
                            //                                                   ^
                            label.push(line.chars().nth(x-1).unwrap());
                            label.push(c);
                            let passage_pos = pos + Pos::x_one();
                            record_portal_location!(pos, label, passage_pos);
                        }
                        else {
                            tile = Tile { pos, kind: TileKind::Void };
                        }
                    },
                    _ => panic!("unexpected character on map: {}", c),
                }
                tiles.push(tile);
            }
        }

        // for each portal location, record the location of the other portal, and determine
        // whether this portal is on the outer or on the inner edge of the map.
        let mut portal_pairs = HashMap::<Pos, Pos>::new();

        #[allow(unused_parens)]
        for (_, locations) in portal_locations {
            assert!(locations.len() == 2);
            let (pos1, pos2) = (locations[0], locations[1]);

            portal_pairs.insert(pos1.clone(), pos2.clone());
            portal_pairs.insert(pos2,         pos1);

            // a portal is located on the outer edge iff either of the following is true:
            //   - there are no walls or passages with the same x coordinate (left or right outer edge)
            //   - there are no walls or passages with the same y coordinate (top or bottom outer edge)
            let p1_lonely_x = !tiles.iter().any(|t| (t.is_wall() || t.is_passage()) && t.pos.x == pos1.x);
            let p1_lonely_y = !tiles.iter().any(|t| (t.is_wall() || t.is_passage()) && t.pos.y == pos1.y);
            let p2_lonely_x = !tiles.iter().any(|t| (t.is_wall() || t.is_passage()) && t.pos.x == pos2.x);
            let p2_lonely_y = !tiles.iter().any(|t| (t.is_wall() || t.is_passage()) && t.pos.y == pos2.y);

            let tile1 = &mut tiles[tile_index!(pos1, w)];
            match tile1.kind {
                TileKind::Portal(ref mut info) => { info.on_outer_edge = (p1_lonely_x || p1_lonely_y); },
                _ => panic!(),
            };
            let tile2 = &mut tiles[tile_index!(pos2, w)];
            match tile2.kind {
                TileKind::Portal(ref mut info) => { info.on_outer_edge = (p2_lonely_x || p2_lonely_y); },
                _ => panic!(),
            }
        }

        Self {
            w,
            h,
            tiles,
            starting_pos: starting_pos.unwrap(),
            target_pos: target_pos.unwrap(),
            portal_pairs,
            recursive_portals,
        }
    }
    pub fn iter(&self) -> MapIterator {
        MapIterator { map: &self, counter: 0 }
    }
    pub fn paired_portal_location(&self, portal_pos: &Pos) -> &Pos {
        // given an input position of a portal, returns the location of the other end of the portal
        // TODO: record this info inside PortalInfo instead?
        match self[portal_pos].kind {
            TileKind::Portal(_) => {
                // note: need to perform the lookup into the portal_pairs map using a Pos with level 0,
                // since that's how these are recorded at Map construction time
                let other_end: &Pos = &self.portal_pairs[&portal_pos.at_level(0)];
                assert!(self[other_end].is_portal());
                return other_end;
            },
            _ => panic!("tile at position {} is not a portal", portal_pos),
        }
    }
    pub fn visualize(&self) -> String {
        // run in two passes; in the first, just emit the tiles without any portals;
        // in the second, add in the portal labels.
        let mut result = String::new();
        for y in 0..self.h {
            for x in 0..self.w {
                let pos = pos![x,y];
                if pos == self.starting_pos {
                    result.push_str("@ ");
                }
                else if pos == self.target_pos {
                    result.push_str("$ ");
                }
                else {
                    let tile: &Tile = &self[&pos];
                    match tile.kind {
                        TileKind::Void      => { result.push_str("  "); },
                        TileKind::Passage   => { result.push_str(". "); },
                        TileKind::Wall      => { result.push_str("# "); },
                        TileKind::Portal(_) => { result.push_str("  "); }, // to be overwritten later
                    }
                }
            }
            result.push_str("\n");
        }

        // second phase: add in portal labels
        #[allow(non_snake_case)]
        let mut lines: Vec<String> = result.lines().map(|L| L.to_owned()).collect();
        assert!(lines.len() == self.h);

        for y in 0..self.h {
            for x in 0..self.w {
                let pos = pos![x,y];
                match self[&pos].kind {
                    TileKind::Portal(ref info) => {
                        let char1 = info.label.chars().nth(0).unwrap();
                        let char2 = info.label.chars().nth(1).unwrap();
                        // which side is the corresponding passage tile attached to?
                        // edit the previously computed visualization accordingly to add in the portal's label.
                        // (note: all these *2's in the replace_range calls are because the previous pass emits
                        //  a 2-char string for each tile.)
                        if x > 0 && self[&(pos + Pos::x_neg_one())].is_passage() {
                            // attached to the left
                            lines[y].replace_range(x*2..(x+2)*2, &format!("{} {} ", char1, char2));
                        }
                        else if x < self.w-1 && self[&(pos + Pos::x_one())].is_passage() {
                            // attached to the right
                            lines[y].replace_range((x-1)*2..(x+1)*2, &format!("{} {} ", char1, char2));
                        }
                        else if y > 0 && self[&(pos + Pos::y_neg_one())].is_passage() {
                            // attached to the top
                            assert!(y+1 < self.h);
                            lines[y].replace_range(  x*2..(x+1)*2, &format!("{} ", char1));
                            lines[y+1].replace_range(x*2..(x+1)*2, &format!("{} ", char2));
                        }
                        else if y < self.h-1 && self[&(pos + Pos::y_one())].is_passage() {
                            // attached to the bottom
                            assert!(y > 0);
                            lines[y-1].replace_range(x*2..(x+1)*2, &format!("{} ", char1));
                            lines[y].replace_range(  x*2..(x+1)*2, &format!("{} ", char2));
                        }
                        else {
                            panic!("found portal tile at {} that's not connected to a passage on any side", pos);
                        }
                    },
                    _ => { continue; },
                }
            }
        }
        let result = lines.join("\n");
        return result;
    }
}
impl Index<&Pos> for Map {
    type Output = Tile;
    fn index(&self, pos: &Pos) -> &Self::Output {
        &self.tiles[tile_index!(pos, self.w)]
    }
}
impl IndexMut<&Pos> for Map {
    fn index_mut(&mut self, pos: &Pos) -> &mut Self::Output {
        &mut self.tiles[tile_index!(pos, self.w)]
    }
}


struct MapIterator<'a> {
    map: &'a Map,
    counter: usize,
}
impl<'a> Iterator for MapIterator<'a> {
    type Item = &'a Tile;
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.map.tiles.len() {
            self.counter += 1;
            return Some(&self.map.tiles[self.counter-1]);
        } else {
            return None;
        }
    }
}

impl path::Node for Pos {}
impl path::Map for Map {
    type Node = Pos;
    type Cost = u32;

    fn neighbours(&self, pos: &Pos) -> Vec<(Pos, Self::Cost)>
    {
        macro_rules! add_neighbour {
            ($tile_pos:ident, $neighbours:ident) => {{
                match self[&$tile_pos].kind {
                    TileKind::Passage     => {
                        $neighbours.push(($tile_pos, 1));
                    },
                    TileKind::Portal(ref portal_info) => {
                        let on_outer_edge = portal_info.on_outer_edge;
                        let paired_portal: &Pos = self.paired_portal_location(&$tile_pos);

                        let mut warp_location = self[paired_portal].portal_info().attached_passage.clone(); // note: always at level 0 (since that's how it was originally recorded)!

                        if self.recursive_portals {
                            // in recursive mode, portals on the outer edge are only accessible if we're at
                            // level > 0, and the depth of the warped-to position is either incremented
                            // or decremented depending on whether we're taking an outer or inner portal.
                            let accessible = ($tile_pos.level > 0 || !on_outer_edge);
                            if accessible {
                                warp_location.level = $tile_pos.level + if on_outer_edge { -1 }  else { 1 };
                                assert!(warp_location.level >= 0);
                                $neighbours.push((warp_location, 1));
                            }
                        } else {
                            // in non-recursive mode, portals can always be taken
                            $neighbours.push((warp_location, 1));
                        }
                    },
                    _ => {},
                }
            }}
        }

        let mut result = Vec::new();
        if pos.x > 0 {
            let left = *pos + Pos::x_neg_one();
            add_neighbour!(left, result);
        }
        if pos.y > 0 {
            let up = *pos + Pos::y_neg_one();
            add_neighbour!(up, result);
        }
        if pos.x < (self.w-1) as i32 {
            let right = *pos + Pos::x_one();
            add_neighbour!(right, result);
        }
        if pos.y < (self.h-1) as i32 {
            let down = *pos + Pos::y_one();
            add_neighbour!(down, result);
        }
        result
    }
}

pub fn main() {
    let lines = util::file_read_lines("input/day20.txt");
    println!("{}", part1(&lines));
    println!("{}", part2(&lines));
}

fn part1(lines: &Vec<String>) -> u32 {
    let map = Map::new(&lines, false);

    // we can't use A* because taking a portal would cause the heuristic to change drastically
    // midway during the operation, which is likely to render it inadmissible, so we'll use dijkstra instead.
    // note that the pathfinder should never encounter nodes of type Portal during operation, as the .neighbours()
    // call implementation transparently replaces them with the passageways attached to their other end.
    let path_maybe = path::dijkstra_to_target(&map, &map.starting_pos, &map.target_pos,
        |map,pos| match map[pos].kind {
                      TileKind::Passage => true,
                      TileKind::Portal(_) => panic!("encountered portal node during pathfinding"), // should be transp.
                      _ => false,
                  }
        );

    if let Some(path) = path_maybe {
        assert!(path.nodes.iter().all(|p| p.level == 0)); // in part 1, we should stay entirely within the same level
        return path.cost;
    } else {
        panic!("no path found between {} and {}", map.starting_pos, map.target_pos);
    }
}

fn part2(lines: &Vec<String>) -> u32 {
    let map = Map::new(&lines, true);

    // same thing as before, but now points contain an active third coordinate, i.e. the recursion depth.
    // note: in this variant it's possible to infinitely descend into recursively nested maps, and also
    // for the exit to not be reachable, so there's a real risk of the pathfinding never terminating.

    // indeed, as stated in the problem description, running this on example map 2 will never terminate,
    // so don't do that :o)
    let path_maybe = path::dijkstra_to_target(&map, &map.starting_pos, &map.target_pos,
        |map,pos| match map[pos].kind {
                      TileKind::Passage => true,
                      TileKind::Portal(_) => panic!("encountered portal node during pathfinding"), // should be transp.
                      _ => false,
                  }
        );

    if let Some(path) = path_maybe {
        return path.cost;
    } else {
        panic!("no path found between {} and {}", map.starting_pos, map.target_pos);
    }
}

#[allow(dead_code)]
fn example_map(n: i32) -> Vec<String> {
    match n {
        1 => vec![
            "         A           ",
            "         A           ",
            "  #######.#########  ",
            "  #######.........#  ",
            "  #######.#######.#  ",
            "  #######.#######.#  ",
            "  #######.#######.#  ",
            "  #####  B    ###.#  ",
            "BC...##  C    ###.#  ",
            "  ##.##       ###.#  ",
            "  ##...DE  F  ###.#  ",
            "  #####    G  ###.#  ",
            "  #########.#####.#  ",
            "DE..#######...###.#  ",
            "  #.#########.###.#  ",
            "FG..#########.....#  ",
            "  ###########.#####  ",
            "             Z       ",
            "             Z       ",
        ],
        2 => vec![
            "                   A               ",
            "                   A               ",
            "  #################.#############  ",
            "  #.#...#...................#.#.#  ",
            "  #.#.#.###.###.###.#########.#.#  ",
            "  #.#.#.......#...#.....#.#.#...#  ",
            "  #.#########.###.#####.#.#.###.#  ",
            "  #.............#.#.....#.......#  ",
            "  ###.###########.###.#####.#.#.#  ",
            "  #.....#        A   C    #.#.#.#  ",
            "  #######        S   P    #####.#  ",
            "  #.#...#                 #......VT",
            "  #.#.#.#                 #.#####  ",
            "  #...#.#               YN....#.#  ",
            "  #.###.#                 #####.#  ",
            "DI....#.#                 #.....#  ",
            "  #####.#                 #.###.#  ",
            "ZZ......#               QG....#..AS",
            "  ###.###                 #######  ",
            "JO..#.#.#                 #.....#  ",
            "  #.#.#.#                 ###.#.#  ",
            "  #...#..DI             BU....#..LF",
            "  #####.#                 #.#####  ",
            "YN......#               VT..#....QG",
            "  #.###.#                 #.###.#  ",
            "  #.#...#                 #.....#  ",
            "  ###.###    J L     J    #.#.###  ",
            "  #.....#    O F     P    #.#...#  ",
            "  #.###.#####.#.#####.#####.###.#  ",
            "  #...#.#.#...#.....#.....#.#...#  ",
            "  #.#####.###.###.#.#.#########.#  ",
            "  #...#.#.....#...#.#.#.#.....#.#  ",
            "  #.###.#####.###.###.#.#.#######  ",
            "  #.#.........#...#.............#  ",
            "  #########.###.###.#############  ",
            "           B   J   C               ",
            "           U   P   P               ",
        ],
        3 => vec![
            "             Z L X W       C                 ",
            "             Z P Q B       K                 ",
            "  ###########.#.#.#.#######.###############  ",
            "  #...#.......#.#.......#.#.......#.#.#...#  ",
            "  ###.#.#.#.#.#.#.#.###.#.#.#######.#.#.###  ",
            "  #.#...#.#.#...#.#.#...#...#...#.#.......#  ",
            "  #.###.#######.###.###.#.###.###.#.#######  ",
            "  #...#.......#.#...#...#.............#...#  ",
            "  #.#########.#######.#.#######.#######.###  ",
            "  #...#.#    F       R I       Z    #.#.#.#  ",
            "  #.###.#    D       E C       H    #.#.#.#  ",
            "  #.#...#                           #...#.#  ",
            "  #.###.#                           #.###.#  ",
            "  #.#....OA                       WB..#.#..ZH",
            "  #.###.#                           #.#.#.#  ",
            "CJ......#                           #.....#  ",
            "  #######                           #######  ",
            "  #.#....CK                         #......IC",
            "  #.###.#                           #.###.#  ",
            "  #.....#                           #...#.#  ",
            "  ###.###                           #.#.#.#  ",
            "XF....#.#                         RF..#.#.#  ",
            "  #####.#                           #######  ",
            "  #......CJ                       NM..#...#  ",
            "  ###.#.#                           #.###.#  ",
            "RE....#.#                           #......RF",
            "  ###.###        X   X       L      #.#.#.#  ",
            "  #.....#        F   Q       P      #.#.#.#  ",
            "  ###.###########.###.#######.#########.###  ",
            "  #.....#...#.....#.......#...#.....#.#...#  ",
            "  #####.#.###.#######.#######.###.###.#.#.#  ",
            "  #.......#.......#.#.#.#.#...#...#...#.#.#  ",
            "  #####.###.#####.#.#.#.#.###.###.#.###.###  ",
            "  #.......#.....#.#...#...............#...#  ",
            "  #############.#.#.###.###################  ",
            "               A O F   N                     ",
            "               A A D   M                     ",
        ],
        _ => panic!(),
    }.iter().map(|s| s.to_string()).collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::Map as PFMap; // bring the path::Map trait into scope so we can call .neighbours()

    #[test]
    fn portal_neighbours() {
        // in the first example map, check that the set of neighbours of each of the portal nodes contains
        // the passage attached to the other end of the portal (with step cost 1)
        let map = Map::new(&example_map(1), false);
        // portal "BC"
        assert!(map.neighbours(&pos![9,6]).contains(&(pos![2,8], 1)));
        assert!(map.neighbours(&pos![2,8]).contains(&(pos![9,6], 1)));
        // portal "DE"
        assert!(map.neighbours(&pos![6,10]).contains(&(pos![2,13], 1)));
        assert!(map.neighbours(&pos![2,13]).contains(&(pos![6,10], 1)));
        // portal "FG"
        assert!(map.neighbours(&pos![11,12]).contains(&(pos![2,15], 1)));
        assert!(map.neighbours(&pos![2,15]).contains(&(pos![11,12], 1)));
    }

    #[test]
    fn recursive_portal_neighbours() {
        // same as the portal_neighbours test, but now additionally check for proper incrementing/decrementing
        // of the third level coordinate, and for inaccessible outer edge portals on level 0
        let map = Map::new(&example_map(1), true);
        // at level 0, taking the inner portal "BC" should work and return a position at a deeper level ..
        assert!(map.neighbours(&pos![9,6,0]).contains(&(pos![2,8,1], 1)));
        assert!(map.neighbours(&pos![2,8,1]).contains(&(pos![9,6,0], 1)));
        assert!(map.neighbours(&pos![9,6,1]).contains(&(pos![2,8,2], 1)));
        // .. but taking the outer portal at level 0 shouldn't.
        assert_eq!(map.neighbours(&pos![2,8,0]), vec![(pos![3,8,0], 1)]);
    }

    #[test]
    fn example_solutions() {
        assert_eq!(part1(&example_map(1)), 23);
        assert_eq!(part1(&example_map(2)), 58);
        assert_eq!(part2(&example_map(1)), 26);
        assert_eq!(part2(&example_map(3)), 396);
    }
}
