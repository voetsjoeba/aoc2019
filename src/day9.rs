// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use crate::intcode::{CPU};

pub fn main() {
    let line: String = util::file_read_lines("input/day9.txt").into_iter().next().unwrap();
    let program: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();
    println!("{}", part(1, &program));
    println!("{}", part(2, &program));
}

fn part(part_nr: u32, program: &Vec<i64>) -> i64 {
    let mut cpu = CPU::new(program);
    cpu.send_input(match part_nr {
        1 => 1,
        2 => 2,
        _ => panic!(),
    });
    cpu.run();
    cpu.consume_output_last().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples() {
        assert_eq!(
            CPU::new(&vec![109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99]).run().consume_output_all(),
            vec![109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99]
        );
        assert_eq!(
            CPU::new(&vec![1102,34915192,34915192,7,4,7,99,0]).run()
                                                              .consume_output_last().unwrap()
                                                              .to_string().len(),
            16
        );
        assert_eq!(
            CPU::new(&vec![104,1125899906842624,99]).run().consume_output_all(),
            vec![1125899906842624]
        );
    }
}
