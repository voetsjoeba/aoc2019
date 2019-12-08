// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use crate::intcode::CPU;

pub fn main() {
    let line: &String = &util::file_read_lines("input/day5.txt")[0];
    let data: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();
    part1(&data);
    part2(&data);
}

fn part1(data: &Vec<i64>) {
    let mut cpu = CPU::new(
        data,
        || 1,                            // input function
        |x| println!("output: {}", x)    // output function
    );
    cpu.run();
}
fn part2(data: &Vec<i64>) {
    let mut cpu = CPU::new(
        data,
        || 5,                            // input function
        |x| println!("output: {}", x)    // output function
    );
    cpu.run();
}

