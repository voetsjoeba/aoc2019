// vim: set ai et ts=4 sts=4 sw=4:
use crate::util::*;
use std::convert::From;
use std::fmt;
use num::bigint::BigInt;
use num::integer::Integer;
use num::traits::identities::One;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Vec3 {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}
impl Vec3 {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }
    pub fn add(&mut self, other: &Vec3) -> &Self {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
        self
    }
}
impl From<&String> for Vec3 {
    fn from(s: &String) -> Self {
        let s = &s[1..s.len()-1]; // drop leading/trailing "<" and ">"
        let mut result = Self::new(0,0,0);
        for coord_s in s.split(",").map(|s| s.trim()).collect::<Vec<_>>() {
            let parts = coord_s.split("=").collect::<Vec<_>>();
            let value: i64 = parts[1].parse().unwrap();
            match parts[0] {
                "x" => { result.x = value; }
                "y" => { result.y = value; }
                "z" => { result.z = value; }
                _   => { panic!(); }
            }
        }
        result
    }
}
impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<x={:3}, y={:3}, z={:3}>", self.x, self.y, self.z)
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Body {
    pub pos: Vec3,
    pub vel: Vec3,
}
impl Body {
    pub fn new(pos: Vec3) -> Self {
        Self { pos: pos, vel: Vec3::new(0,0,0) }
    }
    pub fn update_position(&mut self, vel: Vec3) {
        self.pos.x += vel.x;
        self.pos.y += vel.y;
        self.pos.z += vel.z;
    }
    pub fn pot_energy(&self) -> i64 {
        self.pos.x.abs() + self.pos.y.abs() + self.pos.z.abs()
    }
    pub fn kin_energy(&self) -> i64 {
        self.vel.x.abs() + self.vel.y.abs() + self.vel.z.abs()
    }
    pub fn total_energy(&self) -> i64 {
        self.pot_energy() * self.kin_energy()
    }
    pub fn gravitational_velocity_change(&self, other: &Body) -> Vec3 {
        Vec3::new(
            (other.pos.x - self.pos.x).signum(), // -1 if we are bigger, 1 if we are smaller, 0 otherwise
            (other.pos.y - self.pos.y).signum(),
            (other.pos.z - self.pos.z).signum(),
        )
    }
}
impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pos={}, vel={}", self.pos, self.vel)
    }
}

struct System {
    tick: usize,
    bodies: Vec<Body>,
}
impl From<&Vec<String>> for System {
    fn from(lines: &Vec<String>) -> Self {
        Self {
            tick: 0,
            bodies: lines.iter().map(|line| Body::new(Vec3::from(line))).collect(),
        }
    }
}
impl System {
    pub fn step(&mut self) {
        // for each pair of bodies, adjust their velocity for gravity
        let mut vel_changes = Vec::<(usize, Vec3)>::new(); // idx, {dvx, dvy, dvz}
        let n = self.bodies.len();
        for i in 0..n {
            for j in (i+1)..n {
                let b1 = &self.bodies[i];
                let b2 = &self.bodies[j];
                vel_changes.push((i, b1.gravitational_velocity_change(b2)));
                vel_changes.push((j, b2.gravitational_velocity_change(b1)));
            }
        }
        for (idx, dv) in vel_changes {
            self.bodies[idx].vel.add(&dv);
        }

        // all velocities are updated; now adjust positions
        for b in self.bodies.iter_mut() {
            b.update_position(b.vel);
        }
        self.tick += 1;
    }
    pub fn step_n(&mut self, num_steps: usize) {
        for _ in 0..num_steps {
            self.step();
        }
    }
    pub fn total_energy(&self) -> i64 {
        self.bodies.iter().map(|b| b.total_energy()).sum()
    }
}
impl fmt::Display for System {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for body in &self.bodies {
            write!(f, "{}\n", body);
        }
        Ok(())
    }
}

pub fn main() {
    let lines = file_read_lines("input/day12.txt");
    part1(&lines);
    part2(&lines);
}

fn part1(lines: &Vec<String>) {
    let mut system = System::from(lines);
    system.step_n(1000);
    println!("{}", system.total_energy());
}

fn part2(lines: &Vec<String>) {
    let mut system = System::from(lines);
    // if there's going to be a cycle in the system in which both positions and velocities return to a previous
    // state, then for each individual velocity component there has to be a sequence that both repeats and sums to 0:
    //  - it must repeat in order for that velocity component to cycle
    //  - it must sum to 0 in order for the underlying positions to cycle around as well when applied to them
    // e.g. a velocities cycle of [0,-1,0,1] would work, but [1,2,3,4] would just cause the planets to drift
    // further and further apart without ever repeating.

    // if the system as a whole is to have a cycle, then the length of that cycle cannot be shorter than
    // the least common multiple of the cycle lengths of each of the individual velocity components.

    // iterate the system and keep a history of each velocity component and planet separately (i.e. num_bodies * 3 values).
    // at each iteration, see if there's a window size of values in which they start repeating
    // (i.e. where the first recorded N values in the history equal the last recorded N values, for some value of 1 <= N <= history_size/2)

    let mut vel_histories = Vec::<Vec<i64>>::new(); // hist[body_idx*3 + (0 for x, 1 for y, 2 for z)] -> values
    let mut cycles = Vec::<Option<usize>>::new();   // cycles[<same idx>] = length of cycle if found
    for _ in 0..system.bodies.len()*3 {
        vel_histories.push(Vec::<i64>::new());
        cycles.push(None);
    }

    loop {
        system.step();
        for (body_idx, body) in system.bodies.iter().enumerate() {
            let x_idx = body_idx*3 + 0;
            let y_idx = body_idx*3 + 1;
            let z_idx = body_idx*3 + 2;

            if let None = cycles[x_idx] {
                vel_histories[x_idx].push(body.vel.x);
                if let Some(c) = find_sum0_cycle(&vel_histories[x_idx]) {
                    cycles[x_idx] = Some(c);
                }
            }
            if let None = cycles[y_idx] {
                vel_histories[y_idx].push(body.vel.y);
                if let Some(c) = find_sum0_cycle(&vel_histories[y_idx]) {
                    cycles[y_idx] = Some(c);
                }
            }
            if let None = cycles[z_idx] {
                vel_histories[z_idx].push(body.vel.z);
                if let Some(c) = find_sum0_cycle(&vel_histories[z_idx]) {
                    cycles[z_idx] = Some(c);
                }
            }
        }

        if cycles.iter().all(|v| v.is_some()) {
            break;
        }
    }

    let cycles = cycles.into_iter().map(|v| v.unwrap()).collect::<Vec<_>>();
    println!("{}", cycles.iter().fold(One::one(), |acc, x| BigInt::from(acc).lcm(&BigInt::from(*x))));
}

#[allow(non_snake_case)]
fn find_sum0_cycle(history: &Vec<i64>) -> Option<usize> {
    // starting from the end of the history and going backwards, see if we can find a window size
    // of values that repeats twice and sums to 0
    let L = history.len();

    // something like this should work, unsure why it doesn't ...
    /*
    for N in (2..L/2+1).rev() {
        if    history[(L-2*N)..(L-N)] == history[(L-N)..] // repeats at the end
           && history[(L-N)..].iter().sum::<i64>() == 0   // sums to 0
        {
            //println!("{}", history[(L-N)..].iter().map(|d| d.to_string()).collect::<Vec<_>>().join(","));
            return Some(N);
        }
    }
    None
    */

    let window_size = 40; // trial and error
    if L > window_size && history[..window_size] == history[(L-window_size)..] {
        return Some(history.len()-window_size);
    }
    return None;
}

