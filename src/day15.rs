// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use std::convert::From;
use std::collections::{HashMap};
use crate::intcode::{CPU};

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
struct Pos {
    pub x: i32,
    pub y: i32,
}
impl Pos {
    pub fn up(&self)    -> Self { Self { x: self.x, y: self.y+1 } } // positive Y axis points up
    pub fn down(&self)  -> Self { Self { x: self.x, y: self.y-1 } }
    pub fn left(&self)  -> Self { Self { x: self.x-1, y: self.y } }
    pub fn right(&self) -> Self { Self { x: self.x+1, y: self.y } }
}
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
enum TileKind {
    Empty,
    Wall,
    Target,
}
impl From<i64> for TileKind {
    fn from(val: i64) -> Self {
        match val {
            0 => Self::Wall,
            1 => Self::Empty,
            2 => Self::Target,
            _ => panic!(),
        }
    }
}

fn discover_map(program: &Vec<i64>)
    -> (HashMap<Pos, TileKind>,   // map pos -> tile_kind
        HashMap<Pos, Vec<i64>>) // map pos -> shortest inputs to reach it
{
    // walk the terrain and explore the full extent of the map
    let starting_pos = Pos{x:0, y:0};
    let mut cpu = CPU::new(program);
    let mut visited = HashMap::<Pos, TileKind>::new();
    let mut shortest_paths = HashMap::<Pos, Vec<i64>>::new(); // shortest list of inputs to reach the position
    let mut current_path: Vec<i64> = vec![];

    visited.insert(starting_pos.clone(), TileKind::Empty);
    shortest_paths.insert(starting_pos.clone(), vec![]);
    discover_map_r(&starting_pos, &mut cpu, &mut visited, &mut shortest_paths, &mut current_path);
    return (visited, shortest_paths);
}
fn discover_map_r(pos: &Pos,
                  cpu: &mut CPU,
                  visited: &mut HashMap<Pos, TileKind>,
                  shortest_paths: &mut HashMap<Pos, Vec<i64>>,
                  current_path: &mut Vec<i64>)
{
    // from the current position, try each direction in sequence
    // (except squares we've already visited)
    for (move_input, new_pos, return_input) in [(1, pos.up(), 2),
                                                (2, pos.down(), 1),
                                                (3, pos.left(), 4),
                                                (4, pos.right(), 3)].iter()
    {
        current_path.push(*move_input);

        // update the shortest path seen to travel to this position
        let existing_shortest_path = shortest_paths.get(new_pos);
        match existing_shortest_path {
            None    => { shortest_paths.insert(new_pos.clone(), current_path.clone()); },
            Some(p) => {
                if current_path.len() < p.len() {
                    shortest_paths.insert(new_pos.clone(), current_path.clone());
                }
            },
        }
        //shortest_paths.insert(new_pos.clone(),
        //                 min(*shortest_paths.get(new_pos).unwrap_or(&(steps_taken+1)), steps_taken+1));

        if let None = visited.get(&new_pos) {
            cpu.send_input(*move_input);
            let tile_kind = TileKind::from(cpu.run().consume_output().unwrap());
            visited.insert(*new_pos, tile_kind);

            // if we hit a wall, our position hasn't changed so we can just try the next direction;
            // otherwise, continue discovering recursively from the new position
            if tile_kind != TileKind::Wall {
                // recursively discover further locations
                discover_map_r(new_pos, cpu, visited, shortest_paths, current_path);

                // we need to step back to where we were before trying the next direction.
                cpu.send_input(*return_input);
                assert!(TileKind::from(cpu.run().consume_output().unwrap()) != TileKind::Wall);
            }
        }

        current_path.pop();
    }
}
#[allow(unused)]
fn visualize_map(map: &HashMap<Pos, TileKind>) -> String {
    let mut result = String::new();
    if map.len() == 0 {
        return result;
    }
    let min_x = map.keys().map(|p| p.x).min().unwrap();
    let max_x = map.keys().map(|p| p.x).max().unwrap();
    let min_y = map.keys().map(|p| p.y).min().unwrap();
    let max_y = map.keys().map(|p| p.y).max().unwrap();

    let w = (max_x - min_x) + 1;
    let h = (max_y - min_y) + 1;
    for y in 0..h {
        for x in 0..w {
            let pos = Pos{ x: min_x + x, y: min_y + y };
            let tile_kind = map.get(&pos).unwrap_or(&TileKind::Wall);
            result.push_str(if pos.x == 0 && pos.y == 0 {
                                "S "
                            } else { match tile_kind {
                                TileKind::Empty       => "  ",
                                TileKind::Wall        => "# ",
                                TileKind::Target      => "T ",
                            }});
        }
        result.push_str("\n");
    }

    return result;
}

pub fn main() {
    let line: String = util::file_read_lines("input/day15.txt").into_iter().next().unwrap();
    let program: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();
    solve(&program);
}

fn solve(program: &Vec<i64>) {
    let (map, paths) = discover_map(program);
    let target_pos = map.iter().filter(|(_, &tile_kind)| tile_kind == TileKind::Target)
                               .map(|(p, _)| p)
                               .nth(0).unwrap();
    let target_path = paths.get(target_pos).unwrap();
    //println!("{}", visualize_map(&map));
    println!("{}", target_path.len());

    // amount of time to fill the whole map with oxygen = largest shortest distance from the target to
    // any other tile on the map.

    // make a new cpu, move it to the target location, then run another scan from there.
    let mut cpu = CPU::new(program);

    for input in target_path {
        cpu.send_input(*input);
        cpu.run();
        assert!(cpu.consume_output().unwrap() != 0); // we shouldn't be hitting a wall at any point here
    }

    let mut visited = HashMap::<Pos, TileKind>::new(); // unused
    let mut shortest_paths = HashMap::<Pos, Vec<i64>>::new();
    discover_map_r(target_pos, &mut cpu, &mut visited, &mut shortest_paths, &mut vec![]);
    println!("{}", shortest_paths.values().map(|p| p.len()).max().unwrap());

}

