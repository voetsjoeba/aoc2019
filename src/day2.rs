// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;

pub fn main() {
    let line: &String = &util::file_read_lines("input/day2.txt")[0];
    let data: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();
    part1(&data);
    part2(&data);
}

fn part1(data: &Vec<i64>) {
    let mut data = data.clone();
    data[1] = 12;
    data[2] = 2;
    run_intcode(&mut data);
    println!("{}", data[0]);
}
fn part2(data: &Vec<i64>) {
    for noun in 0..100 {
        for verb in 0..100 {
            let mut memory = data.clone();
            memory[1] = noun;
            memory[2] = verb;
            run_intcode(&mut memory);
            if memory[0] == 19690720 {
                let answer = 100*noun + verb;
                println!("{}", answer);
                break;
            }
        }
    }
}

fn run_intcode(data: &mut Vec<i64>){
    let mut pc = 0usize;
    loop {
        let opcode = data[pc];
        match opcode {
            1|2 => {
                let arg1 = data[pc+1] as usize;
                let arg2 = data[pc+2] as usize;
                let arg3 = data[pc+3] as usize;
                data[arg3] = match opcode {
                    1 => data[arg1] + data[arg2],
                    2 => data[arg1] * data[arg2],
                    _ => panic!("bad opcode: {}", opcode),
                }
            },
            99 => break,
            _ => { panic!("bad opcode: {}", opcode); }
        }
        pc += 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! final_state_of {
        ($prog:expr) => {{
            let mut data = $prog;
            run_intcode(&mut data);
            data
        }}
    }

    #[test]
    fn examples_1() {
        assert_eq!(final_state_of!(vec![1,0,0,0,99]), vec![2,0,0,0,99]);
        assert_eq!(final_state_of!(vec![2,3,0,3,99]), vec![2,3,0,6,99]);
        assert_eq!(final_state_of!(vec![2,4,4,5,99,0]), vec![2,4,4,5,99,9801]);
        assert_eq!(final_state_of!(vec![1,1,1,4,99,5,6,0,99]), vec![30,1,1,4,2,5,6,0,99]);
    }
}
