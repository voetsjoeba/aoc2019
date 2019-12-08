use clap::{App, Arg};

mod util;
mod day1;
mod day2;

fn main() {
    let args = App::new("Advent of Code 2019")
                   .version("1.0")
                   .arg(Arg::with_name("day")
                            .short("d")
                            .long("day")
                            .help("Problem number to solve")
                            .required(true)
                            .takes_value(true))
                    .get_matches();

    let day: i32 = args.value_of("day").unwrap().parse().unwrap();

    // would put this in a macro but concat_ident! is not yet stable :(
    match day {
        1  => day1::main(),
        2  => day2::main(),
        _  => panic!("invalid day number: {}", day),
    };
}
