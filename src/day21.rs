// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use crate::intcode::{CPU, CpuState};

pub fn main() {
    let line: String = util::file_read_lines("input/day21.txt").into_iter().next().unwrap();
    let program: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();

    println!("{}", part1(&program));
    println!("{}", part2(&program));
}

fn part1(program: &Vec<i64>) -> usize {
    let mut cpu = CPU::new(program);
    cpu.run();
    cpu.consume_output_all(); // skip the input prompt
    assert!(cpu.get_state() == CpuState::WaitIO);

    // note: I found this a bit unclear in the problem statement, so to clarify from what I've observed:
    // This program is run by the droid whenever it is located on a floor tile, to decide its next action.
    // If the register J is TRUE at the end of the program, a jump is performed no matter what (even
    // if there are follow-up WALK instructions). Otherwise, despite the problem description saying that
    // "The springdroid will move forward automatically", a WALK instruction is in fact required to make it
    // move forward. If you omit the WALK instruction, the CPU stays waiting for input indefinitely.

    // jump if D is a floor tile and any of A,B or C is a hole:
    //
    //    D ^ (-A v -B v -C)
    // == D ^ -(A ^ B ^ C)
    cpu.send_input_string(concat!(
        "OR A J\n",     // J = A
        "AND B J\n",    // J = A ^ B
        "AND C J\n",    // J = A ^ B ^ C
        "NOT J J\n",    // J = -(A ^ B ^ C)
        "AND D J\n",    // J = -(A ^ B ^ C) ^ D
        "WALK\n"
    ));
    cpu.run();
    assert!(cpu.is_halted());
    return cpu.consume_output_last().unwrap() as usize;
}

fn part2(program: &Vec<i64>) -> usize {
    let mut cpu = CPU::new(program);
    cpu.run();
    cpu.consume_output_all(); // skip the input prompt
    assert!(cpu.get_state() == CpuState::WaitIO);

    // same principle as before, but with some additional constraints to ensure that we can "get away" after
    // having made the jump A -> D:
    //
    // jump if:
    //      at least one of A,B,C is a hole    (otherwise there's no reason to jump)
    // AND: D is a ground tile                 (so we can land safely)
    // AND: we have one of the following escape options from D in case of another jump:
    //      - we can launch from D and land on H
    //          i.e. H is a ground tile (D must also be a ground tile, but this is already satisifed)
    //      - OR: we can launch from E and land on I
    //          i.e. E and I are both ground tiles
    //      - OR: we can launch from F and land on the tile beyond I (which must exist, otherwise there is no solution)
    //          i.e. E and F are both ground tiles (since we must walk over E to launch from F)
    //
    // i.e.: jump if:
    //    D ^ (-A v -B v -C) ^ (H v (E ^ I) v (E ^ F))
    // == D ^ -(A ^ B ^ C) ^ (H v (E ^ (I v F)))

    cpu.send_input_string(concat!(
        "OR A J\n",     // J = A
        "AND B J\n",    // J = A ^ B
        "AND C J\n",    // J = A ^ B ^ C
        "NOT J J\n",    // J = -(A ^ B ^ C)
        "AND D J\n",    // J = -(A ^ B ^ C) ^ D
        "OR I T\n",     // T = I
        "OR F T\n",     // T = I v F
        "AND E T\n",    // T = (I v F) ^ E
        "OR H T\n",     // T = ((I v F) ^ E) v H
        "AND T J\n",    // J = -(A ^ B ^ C) ^ D ^ ((I v F) ^ E) v H
        "RUN\n"
    ));
    cpu.run();
    assert!(cpu.is_halted());
    return cpu.consume_output_last().unwrap() as usize;
}

