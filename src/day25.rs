// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use crate::intcode::{CPU, CpuState};
use std::io::{self, BufRead};
use itertools::Itertools;

pub fn main() {
    let line: String = util::file_read_lines("input/day25.txt").into_iter().next().unwrap();
    let program: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();

    println!("{}", part1(&program));
    // no part 2
}

#[allow(dead_code)]
fn run_interactive(cpu: &mut CPU) {
    // TODO: copy/paste from day17
    loop {
        cpu.run();
        let lines: Vec<String> = cpu.consume_output_all().into_iter()
                                    .map(|n| char::from(n as u8)).collect::<String>()
                                    .trim().lines().map(String::from).collect();
        for line in lines {
            println!("{}", line);
        }
        match cpu.get_state() {
            CpuState::Running => panic!(), // can't be running, we just returned from it running
            CpuState::Halted  => { break; },
            CpuState::WaitIO  => {
                // read a single line from stdin and feed it to the cpu
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line).unwrap(); // includes \n at the end
                cpu.send_input_string(&line);
                println!("");
            },
        }
    }
}

fn part1(program: &Vec<i64>) -> i64
{
    let mut cpu = CPU::new(program);
    // from running the program in interactive mode, we know that there a bunch of collectable items to be found
    // throughout the map, and some combination of them will be the correct weight to pass the security checkpoint.
    // find that combination.
    let collect_items = concat!(
        "south\n",              // engineering
        "south\n",              // arcade
        "west\n",               // science lab
        "north\n",              // warp drive maintenance
        "north\n",              // sick bay
        "take tambourine\n",
        "south\n",
        "south\n",
        "east\n",               // arcade
        "south\n",              // crew quarters
        "take fixed point\n",
        "west\n",               // kitchen
        "take asterisk\n",
        "east\n",               // crew quarters
        "south\n",              // passages
        "take festive hat\n",
        "west\n",               // navigation
        "west\n",               // corridor
        "take jam\n",
        "south\n",              // stables
        "take easter egg\n",
        "north\n",
        "east\n",
        "east\n",               // passages
        "north\n",
        "north\n",
        "north\n",              // engineering
        "west\n",               // hallway
        "south\n",              // gift wrap center
        "take antenna\n",
        "north\n",              // hallway
        "west\n",               // observatory
        "west\n",               // storage
        "take space heater\n",
        "west\n",               // security checkpoint
    );
    cpu.send_input_string(&collect_items);
    cpu.run();

    let items = vec!["antenna", "asterisk", "easter egg", "festive hat",
                     "fixed point", "jam", "space heater", "tambourine"];

    // first, drop all the items we've collected in the current location, then try out all different
    // combinations of items (of different lengths as well) to pass through the weight check with.
    for item in &items {
        cpu.send_input_string(&format!("drop {}", item));
    }
    cpu.run().consume_output_all(); // process instructions and clear output buffer

    for n in 1..9 {
        for combination in items.iter().combinations(n) {
            for item in &combination {
                cpu.send_input_string(&format!("take {}\n", item));
            }
            cpu.run().consume_output_all(); // process the take instructions and clear output buffer

            // now try and pass to the west through the weight detector; if we fail, we'll get a
            // recognizable output message and get kicked back to the security checkpoint.
            // in that case, drop the items we were carrying and try again in the next iteration.
            cpu.send_input_string("west\n");
            let response: String = cpu.run().consume_output_all().into_iter()
                                    .map(|n| char::from(n as u8)).collect::<String>();

            if    !response.contains("Alert! Droids on this ship are heavier than the detected value!")
               && !response.contains("Alert! Droids on this ship are lighter than the detected value!")
            {
                // at this point we've found the correct combination; the answer is contained in a
                // substring of the output message of the form:
                //
                // "You should be able to get in by typing XXXXXXXX on the keypad at the main airlock."
                let match_str = "You should be able to get in by typing ";
                let answer_start = response.find(match_str).unwrap() + match_str.len();
                let answer_end   = answer_start + response[answer_start..].find(" ").unwrap(); // first whitespace after answer_start

                return response[answer_start..answer_end].parse().unwrap();
            }

            for item in &combination {
                cpu.send_input_string(&format!("drop {}\n", item));
            }
        }
    }
    panic!("no solution found");
}

