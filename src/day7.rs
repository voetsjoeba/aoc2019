// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use crate::intcode::{CPU};
use std::cmp::max;
use permutohedron;

pub fn main() {
    let line: &String = &util::file_read_lines("input/day7.txt")[0];
    let program: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();
    println!("{}", part(1, &program));
    println!("{}", part(2, &program));
}

fn part(part_nr: u32, program: &Vec<i64>) -> i64 {
    let mut phases = if part_nr == 2 { vec![5,6,7,8,9] } else { vec![0,1,2,3,4] };
    let mut max_output: Option<i64> = None;
    permutohedron::heap_recursive(
        &mut phases,
        |perm| { let output = run_amplifier_chain(program, &perm.to_vec(), part_nr == 2);
                 max_output = match max_output {
                     None    => Some(output),
                     Some(x) => Some(max(x, output)),
                 };
               }
    );
    max_output.unwrap()
}

fn run_amplifier_chain(program: &Vec<i64>, phase_settings: &Vec<u32>, _part2: bool) -> i64 {
    let mut amp0 = CPU::new(program);
    let mut amp1 = CPU::new(program);
    let mut amp2 = CPU::new(program);
    let mut amp3 = CPU::new(program);
    let mut amp4 = CPU::new(program);
    amp0.send_input(phase_settings[0] as i64);
    amp1.send_input(phase_settings[1] as i64);
    amp2.send_input(phase_settings[2] as i64);
    amp3.send_input(phase_settings[3] as i64);
    amp4.send_input(phase_settings[4] as i64);

    amp0.send_input(0);

    // works for both part1 and part2; in part1, the CPUs all exit after the first loop, in part2 they continue
    let mut last_output: Option<i64> = None;
    loop {
        amp0.run();
        amp1.run();
        amp2.run();
        amp3.run();
        amp4.run();
        if let Some(x) = amp0.consume_output() { amp1.send_input(x); }
        if let Some(x) = amp1.consume_output() { amp2.send_input(x); }
        if let Some(x) = amp2.consume_output() { amp3.send_input(x); }
        if let Some(x) = amp3.consume_output() { amp4.send_input(x); }
        if let Some(x) = amp4.consume_output() { amp0.send_input(x); last_output = Some(x); }

        if amp0.is_halted() && amp1.is_halted() && amp2.is_halted() && amp3.is_halted() && amp4.is_halted() {
            break;
        }
    }
    last_output.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples() {
        assert_eq!(part(1, &vec![3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0]),   43210);
        assert_eq!(part(1, &vec![3,23,3,24,1002,24,10,24,1002,23,-1,23,
                                 101,5,23,23,1,24,23,23,4,23,99,0,0]),               54321);
        assert_eq!(part(1, &vec![3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,
                                 1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0]), 65210);

        assert_eq!(part(2, &vec![3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,
                                 27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5]),         139629729);
        assert_eq!(part(2, &vec![3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,
                                 -5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,
                                 53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10]),         18216);
    }
}
