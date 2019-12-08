// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use std::collections::HashMap;
use std::convert::From;
use std::fmt;

#[derive(Debug)]
enum Step {
    U(u32),
    D(u32),
    R(u32),
    L(u32),
}
impl Step {
    pub fn num(&self) -> u32 {
        match self {
            Step::U(x) => *x,
            Step::D(x) => *x,
            Step::R(x) => *x,
            Step::L(x) => *x,
        }
    }
    pub fn char(&self) -> &'static str {
        match self {
            Step::U(_) => "U",
            Step::D(_) => "D",
            Step::R(_) => "R",
            Step::L(_) => "L",
        }
    }
}
impl From<&str> for Step {
    fn from(input: &str) -> Self {
        let dir_char = input.chars().nth(0).unwrap();
        let num: u32 = input[1..].parse().unwrap();
        match dir_char {
            'U' => Step::U(num),
            'D' => Step::D(num),
            'R' => Step::R(num),
            'L' => Step::L(num),
            _ => panic!("bad input: {}", input),
        }
    }
}
impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.char(), self.num())
    }
}

type Pos = (i32,i32);
type PathId = u32;
type PathDist = u32;
type PathMap = HashMap<Pos, PathMapVal>; // (x,y) => map of line_id to distance traveled to get here
type PathMapVal = HashMap<PathId, PathDist>;

#[derive(Debug)]
struct Path {
    id: u32,
    steps: Vec<Step>,
}
impl Path {
    fn parse(line: &str, id: u32) -> Self {
        Path {
            id: id,
            steps: line.split(",").map(Step::from).collect(),
        }
    }
}

fn trace_path(path: &Path, map: &mut PathMap) {
    let mut pos = (0,0); // x,y
    let mut dist = 0;
    for step in &path.steps {
        for _ in 0..step.num() {
            if !map.contains_key(&pos) {
                map.insert(pos, PathMapVal::new());
            }
            let val = map.get_mut(&pos).unwrap();
            if !val.contains_key(&path.id) { // don't overwrite an earlier distance seen for this position
                val.insert(path.id, dist);
            }
            pos = match step {
                Step::U(_) => (pos.0, pos.1 + 1),
                Step::D(_) => (pos.0, pos.1 - 1),
                Step::R(_) => (pos.0 + 1, pos.1),
                Step::L(_) => (pos.0 - 1, pos.1),
            };
            dist += 1;
        }
    }
}

fn closest_intersection_to(point: &Pos,
                           map: &PathMap)
    -> Option<(Pos, u32)>
{
    map.iter().filter(|(pos,val)| *pos != point && val.len() >= 2)
              .map(|(&pos,_)| (pos, util::manhattan_distance(*point, pos)))
              .min_by_key(|&t| t.1)
}

fn lowest_step_count_from(point: &Pos,
                          map:   &PathMap,
                          path1: &Path,
                          path2: &Path)
    -> Option<u32>
{
    map.iter().filter(|(pos,val)| *pos != point && val.len() >= 2)
              .map(|(_,val)| val[&path1.id] + val[&path2.id])
              .min()
}

pub fn main() {
    let lines = util::file_read_lines("input/day3.txt");
    let path1 = Path::parse(&*lines[0], 1);
    let path2 = Path::parse(&*lines[1], 2);

    let mut map = PathMap::new();
    trace_path(&path1, &mut map);
    trace_path(&path2, &mut map);

    part1(&map);
    part2(&map, &path1, &path2);
}

fn part1(map: &PathMap) {
    println!("{}", closest_intersection_to(&(0,0), map).unwrap().1);
}
fn part2(map: &PathMap, path1: &Path, path2: &Path) {
    println!("{}", lowest_step_count_from(&(0,0), map, path1, path2).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples() {
        let p1 = Path::parse("R75,D30,R83,U83,L12,D49,R71,U7,L72",          1);
        let p2 = Path::parse("U62,R66,U55,R34,D71,R55,D58,R83",             2);
        let p3 = Path::parse("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51", 3);
        let p4 = Path::parse("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7",        4);

        let mut map = PathMap::new();
        trace_path(&p1, &mut map);
        trace_path(&p2, &mut map);
        assert_eq!(closest_intersection_to(&(0,0), &map).unwrap().1,        159);
        assert_eq!(lowest_step_count_from(&(0,0), &map, &p1, &p2).unwrap(), 610);

        let mut map = PathMap::new();
        trace_path(&p3, &mut map);
        trace_path(&p4, &mut map);
        assert_eq!(closest_intersection_to(&(0,0), &map).unwrap().1,        135);
        assert_eq!(lowest_step_count_from(&(0,0), &map, &p3, &p4).unwrap(), 410);
    }

}
