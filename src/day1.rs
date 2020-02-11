// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;

pub fn main() {
    let input = util::file_read_i64s("input/day1.txt");
    part1(&input);
    part2(&input);
}

fn fuel_needed(mass: i64) -> i64 {
    mass/3-2
}
fn extended_fuel_needed(mass: i64) -> i64 {
    let mut total_fuel = fuel_needed(mass);
    let mut extra_fuel = total_fuel;
    loop {
        let new_fuel = fuel_needed(extra_fuel);
        if new_fuel <= 0 { break; }
        extra_fuel = new_fuel;
        total_fuel += new_fuel;
    }
    total_fuel
}

fn part1(input: &Vec<i64>) {
    println!("{}", input.iter().map(|&m| fuel_needed(m)).sum::<i64>());
}
fn part2(input: &Vec<i64>) {
    println!("{}", input.iter().map(|&m| extended_fuel_needed(m)).sum::<i64>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_1() {
        assert_eq!(fuel_needed(12), 2);
        assert_eq!(fuel_needed(14), 2);
        assert_eq!(fuel_needed(1969), 654);
        assert_eq!(fuel_needed(100756), 33583);
    }

    #[test]
    fn examples_2() {
        assert_eq!(extended_fuel_needed(14), 2);
        assert_eq!(extended_fuel_needed(1969), 966);
        assert_eq!(extended_fuel_needed(100756), 50346);
    }
}
