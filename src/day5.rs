// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use crate::intcode::CPU;

pub fn main() {
    let line: &String = &util::file_read_lines("input/day5.txt")[0];
    let program: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();
    part1(&program);
    part2(&program);
}

fn part1(program: &Vec<i64>) {
    let mut cpu = CPU::new(program);
    cpu.send_input(1);
    cpu.run();
    println!("{}", cpu.consume_output_last().unwrap());
}
fn part2(program: &Vec<i64>) {
    let mut cpu = CPU::new(program);
    cpu.send_input(5);
    cpu.run();
    println!("{}", cpu.consume_output_last().unwrap());
}

