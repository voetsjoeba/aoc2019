// vim: set ai et ts=4 sts=4 sw=4:
use std::collections::{HashSet};
use std::iter::FromIterator;
use std::io::{self, BufRead};
use std::fmt;
use std::cmp::{min, max};
use crate::util;
use crate::intcode::{CPU, CpuState};

type NodeId = usize;
type Edge = (NodeId,NodeId);
type Walk = Vec<NodeId>;

enum CallbackResult {
    Continue,
    Stop,
}

#[derive(Debug)]
struct Node {
    x: i32, // actually unsigned, but let's use signed to avoid underflows in computations
    y: i32,
    id: NodeId,
}
impl Node {
    pub fn new(x: i32, y: i32, id: NodeId) -> Self {
        Self { x, y, id }
    }
}
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(x={},y={})", self.x, self.y)
    }
}

macro_rules! undirected_edge {
    ($n1:expr, $n2:expr) => {
        (min($n1, $n2), max($n1, $n2))
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
enum Instr {
    TurnLeft,
    TurnRight,
    Forward(usize),
    SubProgram(usize),
}
impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Instr::TurnLeft      => "L".to_string(),
            Instr::TurnRight     => "R".to_string(),
            Instr::Forward(n)    => n.to_string(),
            Instr::SubProgram(n) => match n {
                0 => "A".to_string(),
                1 => "B".to_string(),
                2 => "C".to_string(),
                _ => panic!(),
            }
        })
    }
}
impl Instr {
    #[allow(dead_code)]
    fn decode(s: &str) -> Option<Self> {
        if let Ok(x) = s.parse::<usize>() {
            return Some(Self::Forward(x));
        }
        match s.chars().next() {
            Some('L') => Some(Self::TurnLeft),
            Some('R') => Some(Self::TurnRight),
            Some('A') => Some(Self::SubProgram(0)),
            Some('B') => Some(Self::SubProgram(1)),
            Some('C') => Some(Self::SubProgram(2)),
            _         => None,
        }
    }
}

const PROGRAM_MAX_LEN: usize = 20; // when expressed in string form! (no relation to amount of instructions in a program)

#[allow(unused_macros)]
macro_rules! format_program {
    ($prog:expr) => {
        $prog.iter().map(|instr| instr.to_string()).collect::<Vec<_>>().join(",")
    }
}
#[allow(unused_macros)]
macro_rules! instrs {
    ($str:expr) => { // comma separated string
        $str.split(",").map(|s| Instr::decode(s).unwrap()).collect::<Vec<_>>()
    }
}

#[derive(Debug)]
struct Program {
    main_program: Vec<Instr>,     // only allowed to contain Instr::SubProgram values
    subprograms: Vec<Vec<Instr>>, // not allowed to contain Instr::SubProgram calls
}
impl fmt::Display for Program {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fmt_main = format_program!(self.main_program);
        write!(f, "main program: {:-20} (len {})\n", fmt_main, fmt_main.len());
        for i in 0..3 {
            let fmt_sub = format_program!(self.subprograms.get(i).unwrap_or(&vec![]));
            write!(f, "  subprogram {}: {:-20} (len {})\n",
                ('A' as u8 + i as u8) as char,
                fmt_sub, fmt_sub.len()
            );
        }
        Ok(())
    }
}
impl Program {
    fn from_instructions(instrs: &Vec<Instr>) -> Option<Self> {
        // the main program can only contain subprogram calls, and subprograms cannot call other subprograms
        // either, so the problem here is to find a way to fully segment the program into (up to) 3 segments
        // such that each segment is <= 20 chars in string form.
        if let Some(segmentation) = Segmentation::find_segmentation(&instrs) {
            let (segments, arrangement) = segmentation;
            return Some(Self {
                main_program: arrangement.iter().map(|&idx| Instr::SubProgram(idx)).collect(),
                subprograms:  segments.iter().map(|slice| slice.to_vec()).collect(),
            });
        }
        None
    }
}
struct Segmentation {

}
impl Segmentation {
    fn find_segmentation<'a>(input: &'a Vec<Instr>)
        -> Option<(Vec<&'a [Instr]>, Vec<usize>)>
    {
        let mut arrangement = Vec::new(); // order of segments (e.g. 0,1,0,2)
        let mut segments    = Vec::new(); // segment definitions (each segment is a slice of instrs of the input)
        if Self::find_segmentation_r(input, &mut segments, &mut arrangement) {
            return Some((segments, arrangement));
        }
        None
    }
    #[allow(non_snake_case)]
    fn find_segmentation_r<'a>(input: &'a Vec<Instr>,
                               segments: &mut Vec<&'a [Instr]>,
                               arrangement: &mut Vec<usize>) -> bool
    {
        // how far along in the input are we? (i.e. how much have the segments consumed yet?)
        let L = input.len();
        let offset = arrangement.iter().fold(0, |acc, &idx| acc + segments[idx].len());

        // if we've consumed all instructions, and the total string length of the main
        // program fits in the allowed space, then we've found a solution
        if offset == L && 2*arrangement.len() - 1 <= PROGRAM_MAX_LEN {
            // TODO: hardcodes knowledge that each subprogram instruction takes up 1 char in size
            return true;
        }

        // for a segment of N instructions to fit a maximum size of M chars in string form
        // (where the smallest instruction is 1 char and they are ','-separated):
        //   N + (N-1) <= M
        //   N <= (M+1)/2
        let max_instrs_per_segment = (PROGRAM_MAX_LEN + 1)/2;

        for len in (1..max_instrs_per_segment+1).rev() {
            if offset + len > L { continue; } // can't go past the end of the input
            let new_segment = &input[offset..offset+len];

            // only a valid segment if its total length in string form is <= PROGRAM_MAX_LEN
            let string_form = format_program!(new_segment);
            if string_form.len() > PROGRAM_MAX_LEN {
                continue;
            }

            // maybe this one exactly matches another we've seen before? if so, we can just add it to
            // the arrangement and skip ahead.
            if let Some(idx) = (0..segments.len()).filter(|&i| segments[i] == new_segment).next() {
                arrangement.push(idx);
                match Self::find_segmentation_r(input, segments, arrangement) {
                    false => {
                        arrangement.pop(); // didn't work out, undo our change and continue searching
                    }
                    true => { return true; }
                }
            }
            else {
                // try and allocate a new segment at the position where we left off,
                // and see if it leads to a solution down the line.
                // if so, that's our result, otherwise continue searching.
                if segments.len() < 3 {
                    arrangement.push(segments.len());
                    segments.push(new_segment);
                    match Self::find_segmentation_r(input, segments, arrangement) {
                        false => {
                            arrangement.pop();
                            segments.pop();
                        }
                        true => { return true; }
                    }
                }
            }
        }

        // no segmentation found :(
        false
    }

}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
enum Orientation {
    North,
    South,
    East,
    West,
}
impl Orientation {
    pub fn rotation_move_to(&self, other: &Orientation) -> Option<Instr> {
        match self {
            Orientation::North => match other {
                                      Orientation::North => None,
                                      Orientation::South => panic!(),
                                      Orientation::East  => Some(Instr::TurnRight),
                                      Orientation::West  => Some(Instr::TurnLeft),
                                  },
            Orientation::South => match other {
                                      Orientation::North => panic!(),
                                      Orientation::South => None,
                                      Orientation::East  => Some(Instr::TurnLeft),
                                      Orientation::West  => Some(Instr::TurnRight),
                                  },
            Orientation::East  => match other {
                                      Orientation::North => Some(Instr::TurnLeft),
                                      Orientation::South => Some(Instr::TurnRight),
                                      Orientation::East  => None,
                                      Orientation::West  => panic!(),
                                  },
            Orientation::West  => match other {
                                      Orientation::North => Some(Instr::TurnRight),
                                      Orientation::South => Some(Instr::TurnLeft),
                                      Orientation::East  => panic!(),
                                      Orientation::West  => None,
                                  },
        }
    }
}

#[derive(Debug)]
struct Graph {
    nodes: Vec<Node>,
    adjacency: Vec<HashSet<NodeId>>, // nodeID => set of adjacent nodeIDs
    start_node_id: NodeId,
    start_orientation: Orientation,
}
impl Graph {
    pub fn from_lines(lines: &Vec<String>) -> Self {
        // parse a description of the playing field as an incoming set of strings
        // and construct the corresponding graph for it
        let h = lines.len();
        let w = lines[0].len();

        let mut nodes = Vec::<Node>::new();
        let mut start_node_id: Option<NodeId> = None;
        let mut start_orientation: Option<Orientation> = None;

        macro_rules! tile_char_at {
            ($x:expr, $y:expr) => {
                lines[$y as usize].chars().nth($x as usize).unwrap()
            }
        }
        macro_rules! is_scaffold_char {
            ($c:expr) => {
                match $c {
                    '#'|'^'|'<'|'>'|'v' => true, // the robot position markers also count as scaffolding tiles
                    _                   => false,
                }
            }
        }
        macro_rules! is_scaffold_tile_at {
            ($x:expr, $y:expr) => {
                is_scaffold_char!(tile_char_at!($x, $y))
            }
        }

        for y in 0..h {
            for x in 0..w {
                // a spot on the map is a node if it doesn't have exactly two # neighbours along the same axis
                // (i.e. either the start of a segment, a bend in a segment, or an intersection).
                // the initial position of the robot is on a node and marked as the starting node.

                let this_char = tile_char_at!(x, y);
                if !is_scaffold_char!(this_char) {
                    continue;
                }
                let has_two_neighbours_hor = (x > 0 && x < w-1) && is_scaffold_tile_at!(x-1, y)
                                                                && is_scaffold_tile_at!(x+1, y);
                let has_two_neighbours_ver = (y > 0 && y < h-1) && is_scaffold_tile_at!(x, y-1)
                                                                && is_scaffold_tile_at!(x, y+1);

                // we're on a node if either has_two_n_hor and has_two_n_v are both false (a bend or a segment start)
                // or if both are true (an intersection)
                if (!has_two_neighbours_hor && !has_two_neighbours_ver) ||
                    (has_two_neighbours_hor && has_two_neighbours_ver)
                {
                    let node = Node::new(x as i32, y as i32, nodes.len());
                    if ['^', '<', '>', 'v'].contains(&this_char) {
                        start_node_id = Some(node.id);
                        start_orientation = Some(match this_char {
                            '^' => Orientation::North,
                            'v' => Orientation::South,
                            '<' => Orientation::West,
                            '>' => Orientation::East,
                            _   => panic!()
                        });
                    }
                    nodes.push(node);
                }
            }
        }

        // build adjacency lists (list of node_ids directly adjacent to each node)
        let mut adjacency = Vec::new();
        for node in &nodes {
            let mut connected_neighbours = HashSet::new();

            // left neighbour
            if let Some(nb) = nodes.iter().filter(|o| o.y == node.y && o.x < node.x)
                                          .min_by_key(|o| (o.x - node.x).abs())
            {
                if (nb.x+1 .. node.x).all(|i| is_scaffold_tile_at!(i, node.y)) {
                    connected_neighbours.insert(nb.id);
                }
            }
            // right neighbour
            if let Some(nb) = nodes.iter().filter(|o| o.y == node.y && o.x > node.x)
                                          .min_by_key(|o| (o.x - node.x).abs())
            {
                if (node.x+1 .. nb.x).all(|i| is_scaffold_tile_at!(i, node.y)) {
                    connected_neighbours.insert(nb.id);
                }
            }
            // up neighbour
            if let Some(nb) = nodes.iter().filter(|o| o.x == node.x && o.y < node.y) // up
                                          .min_by_key(|o| (o.y - node.y).abs())
            {
                if (nb.y+1 .. node.y).all(|i| is_scaffold_tile_at!(node.x, i)) {
                    connected_neighbours.insert(nb.id);
                }
            }
            // down neighbour
            if let Some(nb) = nodes.iter().filter(|o| o.x == node.x && o.y > node.y) // down
                                          .min_by_key(|o| (o.y - node.y).abs())
            {
                if (node.y+1 .. nb.y).all(|i| is_scaffold_tile_at!(node.x, i)) {
                    connected_neighbours.insert(nb.id);
                }
            }

            adjacency.push(connected_neighbours);
        }

        Self {
            nodes,
            adjacency,
            start_node_id: start_node_id.unwrap(),
            start_orientation: start_orientation.unwrap(),
        }
    }
    #[allow(dead_code)]
    pub fn node_at(&self, x: i32, y: i32) -> Option<&Node> {
        self.nodes.iter().filter(|n| n.x == x && n.y == y).next()
    }
    pub fn edges(&self) -> HashSet<Edge> {
        // returns a set of all edges in the graph
        let mut result = HashSet::new();
        for node in &self.nodes {
            result.extend(self.adjacency[node.id].iter().map(|&nb_id| undirected_edge!(node.id, nb_id)));
        }
        return result;
    }
}

fn generate_walks<F>(g: &Graph, mut callback: F)
    where F: FnMut(&Walk) -> CallbackResult
{
    let mut walk = vec![g.start_node_id];
    let mut remaining_edges = HashSet::from_iter(g.edges());

    generate_walks_r(g, &mut callback, &mut walk, &mut remaining_edges);
}
fn generate_walks_r<F>(g: &Graph,
                       f: &mut F,
                       walk: &mut Walk,
                       remaining_edges: &mut HashSet<Edge>) -> CallbackResult
    where F: FnMut(&Walk) -> CallbackResult
{
    let current_node = walk[walk.len()-1];

    // recursively visit each of the current node's neighbours, assuming that edge has not yet been visited
    for &nb_id in &g.adjacency[current_node] {
        let edge = undirected_edge!(current_node, nb_id);
        if remaining_edges.contains(&edge) {
            remaining_edges.remove(&edge);
            walk.push(nb_id);

            let cb_result = generate_walks_r(g, f, walk, remaining_edges);
            if let CallbackResult::Stop = cb_result {
                return cb_result;
            }

            walk.pop();
            remaining_edges.insert(edge);
        }
    }

    if remaining_edges.len() == 0 {
        return f(&walk); // found a walk, call the callback function
    }
    CallbackResult::Continue
}


fn edge_orientation(from: &Node, to: &Node) -> Orientation {
    // when moving between the given nodes (from and to), what direction would we be walking in?
    if to.y == from.y && to.x < from.x {
        Orientation::West
    } else if to.y == from.y && to.x > from.x {
        Orientation::East
    } else if to.x == from.x && to.y < from.y {
        Orientation::North
    } else if to.x == from.x && to.y > from.y {
        Orientation::South
    } else {
        panic!()
    }
}
fn make_instructions(g: &Graph, walk: &Walk) -> Vec<Instr> {
    let mut result = Vec::new();
    let mut current_orientation = g.start_orientation;
    for i in 0..walk.len()-1 {
        let current_node = &g.nodes[walk[i]];
        let next_node    = &g.nodes[walk[i+1]];

        // do we need to turn before we can take this edge in this direction?
        let edge_orientation = edge_orientation(&current_node, &next_node);
        if edge_orientation != current_orientation {
            if let Some(move_instr) = current_orientation.rotation_move_to(&edge_orientation) {
                result.push(move_instr);
            }
        }
        current_orientation = edge_orientation;

        // we've rotated if needed; now move forward along the edge
        result.push(Instr::Forward(match edge_orientation {
            Orientation::North => (current_node.y - next_node.y) as usize,
            Orientation::South => (next_node.y - current_node.y) as usize,
            Orientation::East  => (next_node.x - current_node.x) as usize,
            Orientation::West  => (current_node.x - next_node.x) as usize,
        }));
    }
    return result;
}

pub fn main() {
    let line: String = util::file_read_lines("input/day17.txt").into_iter().next().unwrap();
    let program: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();

    let mut cpu = CPU::new(&program);
    cpu.run();
    let lines: Vec<String> = cpu.consume_output_all().into_iter()
                                .map(|n| char::from(n as u8)).collect::<String>()
                                .trim().lines().map(String::from).collect();

    let g = Graph::from_lines(&lines);
    part1(&g);
    part2(&g, &program);
}

fn part1(g: &Graph) {
    println!("{}", g.nodes.iter().filter(|n| g.adjacency[n.id].len() > 2)
                                 .map(|n| n.x*n.y)
                                 .sum::<i32>());
}

fn part2(g: &Graph, original_program: &Vec<i64>) {
    let mut cpu = CPU::new(&original_program);
    cpu.write_mem(0, 2);

    let interactive = false;
    if !interactive {
        // strap in, this is gonna take a while
        let p = match find_program(g) {
            Some(p) => p,
            None    => { println!("no solution found :("); return; }
        };

        // note: no subprogram can be empty, will be rejected

        // send main program
        cpu.send_input_string(&format_program!(p.main_program));
        cpu.send_input_string("\n");

        // send subprograms
        for i in 0..3 {
            cpu.send_input_string(&format_program!(p.subprograms.get(i).unwrap_or(&vec![])));
            cpu.send_input_string("\n");
        }

        // video feed prompt?
        cpu.send_input_string("n\n");
        cpu.run();

        println!("{}", cpu.consume_output_last().unwrap());
    }
    else {
        // for interactive mode:
        loop {
            cpu.run();
            let lines: Vec<String> = cpu.consume_output_all().into_iter()
                                        .map(|n| char::from(n as u8)).collect::<String>()
                                        .trim().lines().map(String::from).collect();
            for line in lines {
                println!("{}", line);
            }
            match cpu.get_state() {
                CpuState::Running => panic!(), // can't be running, we just returned from it running
                CpuState::Halted  => { break; },
                CpuState::WaitIO  => {
                    // read a single line from stdin and feed it to the cpu
                    let mut line = String::new();
                    io::stdin().lock().read_line(&mut line).unwrap(); // includes \n at the end
                    cpu.send_input_string(&line);
                },
            }
        }
    }
}


fn find_program(g: &Graph) -> Option<Program> {
    // given a graph representing the puzzle input, find a 'program' for the robot to travel each
    // consisting of a main and 3 subroutines, all of which of string length <= 20,
    // such that the robot travels every path segment at least once.

    let mut program: Option<Program> = None;

    // generate walks through the graph that visit each edge (not node!) exactly once; whenever one is found,
    // run a callback function that produces the set of instructions for that walk, and then tries
    // to see if it will fit in the size limitations of the program to be generated.

    generate_walks(g, |walk| {
        let instrs = make_instructions(g, walk);
        // the list of walk instructions might contain consecutive steps forward that can be merged
        // into a single bigger forward move. And wouldn't you know it, a given puzzle might not have any paths
        // that can be decomposed into A/B/C subprograms unless one or more or any of those mergers are performed
        // (the example given in part 2 appears to be such a puzzle).
        //
        // The amount of variants a single program can have grows very quickly though (order of n factorial?),
        // so checking all of them is infeasible. Instead, we'll only take the maximally-reduced version of
        // each program and check that one.

        let merged_variant = maximally_merge_instructions(&instrs);
        program = Program::from_instructions(&merged_variant);
        match program {
            Some(_) => CallbackResult::Stop,
            None    => CallbackResult::Continue,
        }
    });

    program
}

fn maximally_merge_instructions(instrs: &Vec<Instr>) -> Vec<Instr> {
    let mut working_copy = instrs.clone();
    let mut i = 0usize;
    while i < working_copy.len()-1 {
        match (&working_copy[i], &working_copy[i+1]) {
            (&Instr::Forward(a), &Instr::Forward(b)) => { working_copy.splice(i..i+2, vec![Instr::Forward(a+b)]); }
            _                                        => { i += 1; }
        }
    }
    working_copy
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_example_1() -> Vec<String> {
        vec![
           "..#..........",
           "..#..........",
           "#######...###",
           "#.#...#...#.#",
           "#############",
           "..#...#...#..",
           "..#####...^..",
        ].into_iter().map(|s| s.to_string()).collect()
    }
    #[allow(dead_code)]
    fn get_example_2() -> Vec<String> {
        vec![
            "#######...#####",
            "#.....#...#...#",
            "#.....#...#...#",
            "......#...#...#",
            "......#...###.#",
            "......#.....#.#",
            "^########...#.#",
            "......#.#...#.#",
            "......#########",
            "........#...#..",
            "....#########..",
            "....#...#......",
            "....#...#......",
            "....#...#......",
            "....#####......",
        ].into_iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn example_nodes() {
        let g = Graph::from_lines(&get_example_1());
        assert_eq!(
            HashSet::from_iter(g.nodes.iter().map(|n| (n.x,n.y))),
            vec![(2,0),
                 (0,2), (2,2), (6,2), (10,2), (12,2),
                 (0,4), (2,4), (6,4), (10,4), (12,4),
                 (2,6), (6,6), (10,6)
                ].into_iter().collect::<HashSet<_>>()
        );
        assert_eq!(
            g.edges(),
            vec![
                undirected_edge!(g.node_at(2,0).unwrap().id,  g.node_at(2,2).unwrap().id),
                undirected_edge!(g.node_at(0,2).unwrap().id,  g.node_at(2,2).unwrap().id),
                undirected_edge!(g.node_at(2,2).unwrap().id,  g.node_at(6,2).unwrap().id),
                undirected_edge!(g.node_at(2,2).unwrap().id,  g.node_at(6,2).unwrap().id),
                undirected_edge!(g.node_at(10,2).unwrap().id, g.node_at(12,2).unwrap().id),
                undirected_edge!(g.node_at(0,2).unwrap().id,  g.node_at(0,4).unwrap().id),
                undirected_edge!(g.node_at(2,2).unwrap().id,  g.node_at(2,4).unwrap().id),
                undirected_edge!(g.node_at(6,2).unwrap().id,  g.node_at(6,4).unwrap().id),
                undirected_edge!(g.node_at(10,2).unwrap().id, g.node_at(10,4).unwrap().id),
                undirected_edge!(g.node_at(12,2).unwrap().id, g.node_at(12,4).unwrap().id),
                undirected_edge!(g.node_at(0,4).unwrap().id,  g.node_at(2,4).unwrap().id),
                undirected_edge!(g.node_at(2,4).unwrap().id,  g.node_at(6,4).unwrap().id),
                undirected_edge!(g.node_at(6,4).unwrap().id,  g.node_at(10,4).unwrap().id),
                undirected_edge!(g.node_at(10,4).unwrap().id, g.node_at(12,4).unwrap().id),
                undirected_edge!(g.node_at(2,4).unwrap().id,  g.node_at(2,6).unwrap().id),
                undirected_edge!(g.node_at(6,4).unwrap().id,  g.node_at(6,6).unwrap().id),
                undirected_edge!(g.node_at(10,4).unwrap().id, g.node_at(10,6).unwrap().id),
                undirected_edge!(g.node_at(2,6).unwrap().id,  g.node_at(6,6).unwrap().id),
            ].into_iter().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn example_path_moves() {
        let g = Graph::from_lines(&get_example_1());
        let path = vec![
            g.node_at(10,6).unwrap().id,
            g.node_at(10,4).unwrap().id,
            g.node_at(12,4).unwrap().id,
            g.node_at(12,2).unwrap().id,
            g.node_at(10,2).unwrap().id,
            g.node_at(10,4).unwrap().id,
            g.node_at(6,4).unwrap().id,
            g.node_at(6,2).unwrap().id,
            g.node_at(2,2).unwrap().id,
            g.node_at(2,4).unwrap().id,
            g.node_at(6,4).unwrap().id,
            g.node_at(6,6).unwrap().id,
            g.node_at(2,6).unwrap().id,
            g.node_at(2,4).unwrap().id,
            g.node_at(0,4).unwrap().id,
            g.node_at(0,2).unwrap().id,
            g.node_at(2,2).unwrap().id,
            g.node_at(2,0).unwrap().id,
        ];
        assert_eq!(
            make_instructions(&g, &path),
            vec![
                Instr::Forward(2),                     // to (10,4)
                Instr::TurnRight, Instr::Forward(2),   // to (12,4)
                Instr::TurnLeft,  Instr::Forward(2),   // to (12,2)
                Instr::TurnLeft,  Instr::Forward(2),   // to (10,2)
                Instr::TurnLeft,  Instr::Forward(2),   // to (10,4)
                Instr::TurnRight, Instr::Forward(4),   // to (6,4)
                Instr::TurnRight, Instr::Forward(2),   // to (6,2)
                Instr::TurnLeft,  Instr::Forward(4),   // to (2,2)
                Instr::TurnLeft,  Instr::Forward(2),   // to (2,4)
                Instr::TurnLeft,  Instr::Forward(4),   // to (6,4)
                Instr::TurnRight, Instr::Forward(2),   // to (6,6)
                Instr::TurnRight, Instr::Forward(4),   // to (2,6)
                Instr::TurnRight, Instr::Forward(2),   // to (2,4)
                Instr::TurnLeft,  Instr::Forward(2),   // to (0,4)
                Instr::TurnRight, Instr::Forward(2),   // to (0,2)
                Instr::TurnRight, Instr::Forward(2),   // to (2,2)
                Instr::TurnLeft,  Instr::Forward(2),   // to (2,0)
            ]
        );
    }

    #[test]
    fn segment_program() {
        // we know this program can be segmented; make sure the code agrees
        let program: Vec<Instr> = instrs!("R,12,R,8,R,4,R,4,R,8,L,6,L,2,R,4,R,4,R,8,R,8,R,8,L,6,L,2");
        let reduced = Program::from_instructions(&program);
        assert!(reduced.is_some());
        let reduced = reduced.unwrap();
        assert!(format_program!(&reduced.main_program).len() <= PROGRAM_MAX_LEN);
        assert!(reduced.subprograms.iter().all(|sp| format_program!(sp).len() <= PROGRAM_MAX_LEN));
    }

    #[test]
    fn merged_program_max() {
        assert_eq!(
            maximally_merge_instructions(&instrs!("2,2,R,2,R,2,R,2,4,4,L,2,L,4,L,2,2,L,4,2,L,2,L,2,L,2,2")),
                                          instrs!("4,R,2,R,2,R,10,L,2,L,4,L,4,L,6,L,2,L,2,L,4")
        );
    }

    #[test]
    fn experimentation() {
        let prog = &instrs!("R,8,R,8,R,4,R,4,R,8,L,6,L,2,R,4,R,4,R,8,R,8,R,8,L,6,L,2");
        let s = Segmentation::find_segmentation(&prog).unwrap();
        println!("{:?}", s.0.iter().map(|seg| format_program!(seg)).collect::<Vec<_>>());
    }
}
