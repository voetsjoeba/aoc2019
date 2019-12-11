// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use std::collections::HashMap;
use crate::intcode::{CPU};

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
enum Facing {
    Up,
    Down,
    Left,
    Right,
}
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
struct Pos {
    pub x: i32,
    pub y: i32,
}
impl Pos {
    pub fn up(&self)    -> Self { Self { x: self.x, y: self.y+1 } }
    pub fn down(&self)  -> Self { Self { x: self.x, y: self.y-1 } }
    pub fn left(&self)  -> Self { Self { x: self.x-1, y: self.y } }
    pub fn right(&self) -> Self { Self { x: self.x+1, y: self.y } }
}

struct Robot {
    cpu: CPU,
    pos: Pos,
    facing: Facing,
    paint_map: HashMap<Pos, i64>,
}
impl Robot {
    pub fn new(program: &Vec<i64>) -> Self {
        Self {
            cpu: CPU::new(program),
            pos: Pos { x:0, y:0 },
            facing: Facing::Up,
            paint_map: HashMap::new(),
        }
    }
    pub fn turn(&mut self, dir: u32) { // dir 0 is turn left, 1 is right
        match self.facing {
            Facing::Up    => { self.facing = if dir == 0 { Facing::Left } else { Facing::Right };
                               self.pos    = if dir == 0 { self.pos.left() } else { self.pos.right() };
                             },
            Facing::Down  => { self.facing = if dir == 0 { Facing::Right } else { Facing::Left };
                               self.pos    = if dir == 0 { self.pos.right() } else { self.pos.left() };
                             },
            Facing::Left  => { self.facing = if dir == 0 { Facing::Down } else { Facing::Up };
                               self.pos    = if dir == 0 { self.pos.down() } else { self.pos.up() };
                             },
            Facing::Right => { self.facing = if dir == 0 { Facing::Up } else { Facing::Down };
                               self.pos    = if dir == 0 { self.pos.up() } else { self.pos.down() };
                             }
        }
    }
    pub fn run(&mut self) {
        loop {
            // send current panel color as input
            let current_panel_color = self.paint_map.get(&self.pos).unwrap_or(&0i64); // default to black
            self.cpu.send_input(*current_panel_color);
            self.cpu.run(); // let CPU run for a while until it halts or needs more input
            if self.cpu.is_halted() {
                break;
            }
            // if we didn't halt, the program is supposed to have produced output
            let new_color = self.cpu.consume_output().unwrap();
            let turn_dir = self.cpu.consume_output().unwrap();

            self.paint_map.insert(self.pos.clone(), new_color);
            self.turn(turn_dir as u32);
        }
    }
    pub fn visualize_map(&self) -> String {
        let mut result = String::new();
        // determine the max extents of the painted area
        let min_x = self.paint_map.keys().map(|&pos| pos.x).min().unwrap();
        let max_x = self.paint_map.keys().map(|&pos| pos.x).max().unwrap();
        let min_y = self.paint_map.keys().map(|&pos| pos.y).min().unwrap();
        let max_y = self.paint_map.keys().map(|&pos| pos.y).max().unwrap();

        let w = (max_x - min_x) + 1;
        let h = (max_y - min_y) + 1;
        for y in 0..h {
            for x in 0..w {
                let pos = Pos { x: min_x + x, y: max_y - y }; // max_y - y because the Y axis points up in our coord system
                let color = self.paint_map.get(&pos).unwrap_or(&0); // default to 0
                result.push_str(match color {
                    0 => " ",
                    1 => "#",
                    _ => panic!("invalid color: {}", color),
                });
            }
            result.push_str("\n");
        }
        return result;
    }
}

pub fn main() {
    let line: String = util::file_read_lines("input/day11.txt").into_iter().next().unwrap();
    let program: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();
    part1(&program);
    part2(&program);
}

fn part1(program: &Vec<i64>) {
    let mut robot = Robot::new(program);
    robot.run();
    println!("{}", robot.paint_map.len());
}

fn part2(program: &Vec<i64>) {
    let mut robot = Robot::new(program);
    robot.paint_map.insert(robot.pos.clone(), 1i64); // start on a white panel this time
    robot.run();
    println!("{}", robot.visualize_map());
}
