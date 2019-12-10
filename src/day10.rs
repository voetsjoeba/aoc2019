// vim: set ai et ts=4 sts=4 sw=4:
use crate::util::{gcd, file_read_lines, manhattan_distance};
use std::convert::From;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Status {
    Asteroid,
    Empty,
}
impl From<char> for Status {
    fn from(c: char) -> Self {
        match c {
            '#' => Status::Asteroid,
            '.' => Status::Empty,
             _  => Status::Empty,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
struct Dir {
    pub dx: i32,
    pub dy: i32,
}
impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(dx={},dy={})", self.dx, self.dy)
    }
}
impl Dir {
    fn normalized(&self) -> Self {
        if self.dx == 0 && self.dy == 0 {
            panic!("can't normalize the (0,0) direction");
        }
        else if self.dx == 0 {
            Dir { dx: 0, dy: self.dy.signum() } // vertical direction, up or down
        }
        else if self.dy == 0 {
            Dir { dx: self.dx.signum(), dy: 0 }
        }
        else {
            // reduce the direction by their gcd
            let gcd = gcd(self.dx, self.dy).abs();
            Dir { dx: self.dx/gcd, dy: self.dy/gcd }
        }
    }
    fn angle(&self) -> f64 {
        // clockwise angle relative to the 'up' direction
        let rad = (-self.dy as f64).atan2(self.dx as f64); // -y because in our coord system the Y axis points down but atan2 expects it to point up
        let rad = PI/2f64 - rad; // shift to an offset from the vertical axis
        let rad = if rad < 0f64 { 2f64*PI + rad } else { rad }; // correct the negative part to a positive one (so that it 'goes around')
        return rad;
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
struct Pos {
    pub x: i32,
    pub y: i32,
}
impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(x={},y={})", self.x, self.y)
    }
}
struct Asteroid {
    pub pos: Pos,
    pub direction_map: HashMap<Dir, Vec<Pos>>, // maps direction to list of other asteroids along that direction
}
impl Asteroid {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            pos: Pos::new(x,y),
            direction_map: HashMap::new(),
        }
    }
    pub fn record_asteroid_in_direction(&mut self, dir: &Dir, other_pos: Pos) {
        let ndir = dir.normalized();
        if !self.direction_map.contains_key(&ndir) {
            self.direction_map.insert(ndir.clone(), Vec::new());
        }
        self.direction_map.get_mut(&ndir).unwrap()
                          .push(other_pos);
    }
    pub fn sort_other_asteroids(&mut self) {
        let pos_tuple = (self.pos.x, self.pos.y);
        for (_, positions) in self.direction_map.iter_mut() {
            positions.sort_by_key(|other| manhattan_distance(pos_tuple, (other.x, other.y)));
        }
    }
}
impl fmt::Display for Asteroid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A{}", self.pos)
    }
}

struct Map {
    pub w: usize,
    pub h: usize,
    pub asteroids: HashMap<Pos, Asteroid>,
}
impl Map {
    pub fn new(lines: &Vec<String>) -> Self {
        let mut asteroids = HashMap::new();
        for (y, line) in lines.iter().enumerate() {
            for (x, c) in line.chars().enumerate() {
                if c == '#' {
                    asteroids.insert(
                        Pos::new(x as i32, y as i32),
                        Asteroid::new(x as i32, y as i32)
                    );
                }
            }
        }
        Self {
            w: lines[0].len(),
            h: lines.len(),
            asteroids,
        }
    }
    pub fn asteroid_at_mut(&mut self, pos: &Pos) -> &mut Asteroid {
        self.asteroids.get_mut(pos).unwrap()
    }
    pub fn compute_directions(&mut self) {
        // for each asteroid (identified by (x,y) coords), find the directions to all other asteroids
        // on the map. store each other asteroid in a list keyed by the normalized direction along which it lies.
        let mut tuples: Vec<(Pos, Pos, Dir)> = Vec::new();
        for a in self.asteroids.values() {
            for o in self.asteroids.values().filter(|x| x.pos != a.pos) {
                let dir = Dir { dx: o.pos.x - a.pos.x,
                                dy: o.pos.y - a.pos.y };
                //a.record_asteroid_in_direction(&dir, o);
                tuples.push((a.pos.clone(), o.pos.clone(), dir));
            }
        }

        for (a_pos, o_pos, dir) in tuples {
            self.asteroid_at_mut(&a_pos).record_asteroid_in_direction(&dir, o_pos);
        }

        // sort each asteroid's list of other asteroids in a direction by its distance away from it
        for a in self.asteroids.values_mut() {
            a.sort_other_asteroids();
        }
    }
    #[allow(unused)]
    pub fn display(&self) -> String {
        let mut result = String::new();
        for y in 0..self.h {
            for x in 0..self.w {
                let pos = Pos::new(x as i32, y as i32);
                result.push_str(match self.asteroids.get(&pos) {
                    None    => ". ",
                    Some(_) => "# ",
                });
            }
            result.push_str("\n");
        }
        return result;
    }
}
pub fn main() {
    let lines = file_read_lines("input/day10.txt");
    let mut map = Map::new(&lines);
    solve(&mut map);
}

fn solve(map: &mut Map) {
    map.compute_directions();

    // find which asteroids has the most unique (normalized) directions to other asteroids
    #[allow(unused_assignments)]
    let mut station_pos = Pos{x:-1,y:-1};
    {
        let station = map.asteroids.values()
                                   .max_by_key(|a| a.direction_map.len())
                                   .unwrap();
        station_pos = station.pos.clone();
        println!("{}", station.direction_map.len());
    }

    // from that location, determine the order of its unique directions in clockwise order
    // starting from the up direction. at each direction in turn, eliminate the closest asteroid
    // along that direction. repeat until the list of asteroids in all directions is empty.
    let station: &mut Asteroid = map.asteroids.get_mut(&station_pos).unwrap();
    let mut dir_order: Vec<Dir> = station.direction_map.keys().map(|&k| k).clone().collect();
    dir_order.sort_by(|a,b| a.angle().partial_cmp(&b.angle()).unwrap());

    let mut popped: Vec<Pos> = Vec::new(); // positions of asteroids destroyed so far
    loop {
        // visit each direction in order, and pop the first asteroid along that direction
        // (already sorted by distance)
        for dir in &dir_order {
            let others_in_dir = station.direction_map.get_mut(dir).unwrap();
            if others_in_dir.len() == 0 {
                continue; // no more asteroids in this direction, move on to next one
            }
            popped.push(others_in_dir.remove(0));
        }
        if dir_order.iter().all(|&dir| station.direction_map.get(&dir).unwrap().len() == 0) {
            break; // no more asteroids to destroy
        }
    }

    if popped.len() < 200 {
        println!("no solution, fewer than 200 asteroids destroyed from position {}", station);
    } else {
        println!("{}", popped[199].x*100 + popped[199].y);
    }
}

