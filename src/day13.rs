// vim: set ai et ts=4 sts=4 sw=4:
use std::convert::From;
use std::collections::HashMap;
use crate::util;
use crate::intcode::{CPU};

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
struct Pos {
    pub x: i64,
    pub y: i64,
}
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
enum TileKind {
    Empty,
    Wall,
    Block,
    HorizPaddle,
    Ball,
}
impl From<i64> for TileKind {
    fn from(id: i64) -> Self {
        match id {
            0 => Self::Empty,
            1 => Self::Wall,
            2 => Self::Block,
            3 => Self::HorizPaddle,
            4 => Self::Ball,
            _ => panic!("invalid tile kind id:  {}", id)
        }
    }
}
struct Tile {
    pos: Pos,
    kind: TileKind,
}
struct Arcade {
    cpu: CPU,
    tiles: HashMap<Pos, Tile>,
    score: i64,
}
impl Arcade {
    pub fn new(program: &Vec<i64>) -> Self {
        Self {
            cpu: CPU::new(program),
            tiles: HashMap::new(),
            score: 0,
        }
    }
    pub fn play_for_free(&mut self) {
        self.cpu.write_mem(0, 2);
    }
    pub fn step_game(&mut self, input: i64) {
        // give the game a single joystick input, let it run for a bit,
        // and update the game state according to any output it produces (if any).
        self.cpu.send_input(input);
        self.cpu.run();
        self.check_output();
    }
    pub fn check_output(&mut self) {
        // check for output from the CPU and update the game state accordingly
        // output comes in pairs of threes
        loop {
            if let Some(x) = self.cpu.consume_output() {
                let y = self.cpu.consume_output().unwrap();
                let id = self.cpu.consume_output().unwrap();
                if x == -1 && y == 0 {
                    self.score = id;
                } else {
                    self.tiles.insert(Pos { x, y },
                                      Tile { pos: Pos { x, y },
                                             kind: TileKind::from(id) });
                }
            } else {
                break;
            }
        }
    }
    pub fn get_ball(&self) -> &Tile {
        // finds the ball tile in the current game (assumes one exists)
        self.tiles.values().filter(|t| t.kind == TileKind::Ball).nth(0).unwrap()

    }
    pub fn get_paddle(&self) -> &Tile {
        // finds the paddle tile in the current game (assumes one exists)
        self.tiles.values().filter(|t| t.kind == TileKind::HorizPaddle).nth(0).unwrap()
    }
    #[allow(unused)]
    pub fn visualize(&self) -> String {
        let mut result = String::new();
        if self.tiles.len() == 0 {
            return result;
        }
        result.push_str(&format!("Score: {}\n", self.score));
        let min_x = self.tiles.values().map(|t| t.pos.x).min().unwrap();
        let max_x = self.tiles.values().map(|t| t.pos.x).max().unwrap();
        let min_y = self.tiles.values().map(|t| t.pos.y).min().unwrap();
        let max_y = self.tiles.values().map(|t| t.pos.y).max().unwrap();

        let w = (max_x - min_x) + 1;
        let h = (max_y - min_y) + 1;
        for y in 0..h {
            for x in 0..w {
                let tile = &self.tiles.get(&Pos{ x: min_x + x, y: min_y + y });
                let tile_kind = match tile {
                    Some(t) => t.kind,
                    None    => TileKind::Empty,
                };
                result.push_str(match tile_kind {
                    TileKind::Empty       => " ",
                    TileKind::Wall        => if y == 0 && (x == 0 || x == w-1) { "+" }
                                             else if y == 0 { "-" }
                                             else { "|" },
                    TileKind::Block       => "x",
                    TileKind::HorizPaddle => "_",
                    TileKind::Ball        => "o",
                });
            }
            result.push_str("\n");
        }

        return result;
    }
}

pub fn main() {
    let line: String = util::file_read_lines("input/day13.txt").into_iter().next().unwrap();
    let program: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();
    part1(&program);
    part2(&program);
}

fn part1(program: &Vec<i64>) {
    let mut arcade = Arcade::new(program);
    arcade.cpu.run();
    arcade.check_output();
    println!("{}", arcade.tiles.values().filter(|t| t.kind == TileKind::Block).count());
}

fn part2(program: &Vec<i64>) {
    //for _ in 0..100 { println!(""); } // create some vertical space
    let mut arcade = Arcade::new(program);
    arcade.play_for_free();

    let mut next_input = 0i64;
    loop {
        arcade.step_game(next_input);
        if arcade.cpu.is_halted() {
            break;
        }
        // find the ball and the paddle, and move the paddle according to the ball's horizontal position
        let ball = arcade.get_ball();
        let paddle = arcade.get_paddle();
        next_input = (ball.pos.x - paddle.pos.x).signum();

        //print!("{}[2J", 27 as char); // clear screen
        //println!("{}", arcade.visualize());
        //println!("Press any key to step the game forward.");
        //io::stdin().read_line(&mut String::new());
    }
    println!("{}", arcade.score);
}

