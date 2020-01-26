// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use crate::intcode::{CPU};
use std::collections::VecDeque;
use std::ops::Range;

struct IncrementalBeamRange<'a> {
    // returns the range of affected X coordinates over incremental values of Y
    program: &'a Vec<i64>,
    next_y: usize,
    prev_left_x: usize,
    prev_width: usize,
}
impl<'a> IncrementalBeamRange<'a> {
    pub fn new(program: &'a Vec<i64>) -> Self {
        Self {
            program,
            next_y: 0,
            prev_left_x: 0,
            prev_width: 1,
        }
    }
}
impl<'a> Iterator for IncrementalBeamRange<'a> {
    type Item = Option<Range<usize>>;
    fn next(&mut self) -> Option<Self::Item> {
        let result: Option<Range<usize>> = beam_range_incremental(self.next_y, self.prev_left_x, self.prev_width, self.program);
        if let Some(x_range) = &result {
            self.prev_left_x = x_range.start;
            self.prev_width = x_range.len();
        }
        self.next_y += 1;
        Some(result)
    }
}

fn beam_affects(x: usize, y: usize, program: &Vec<i64>) -> bool {
    let mut cpu = CPU::new(&program);
    cpu.send_input(x as i64);
    cpu.send_input(y as i64);
    cpu.run();
    match cpu.consume_output().unwrap() {
        0 => false,
        1 => true,
        _ => panic!(),
    }
}

fn beam_range_incremental(y: usize, prev_left_x: usize, prev_width: usize, program: &Vec<i64>)
    -> Option<Range<usize>>
{
    // note: returns an Option because at very low Y coordinates, the beam may sometimes 'disappear'
    // (presumably due to computational effects in the beam's calculation program)

    // find the left edge of the beam; because each left edge is monotonically increasing, we only
    // need to search starting at the previous Y coordinate's leftmost X coordinate.
    // but careful: very early on there are pathological cases where the beam 'disappears', so
    // we need to make sure to not loop infinitely searching for the first square to be affected
    // by the beam.
    let mut left_x = prev_left_x;
    loop {
        if left_x - prev_left_x > 2*prev_width {
            // we've searched for twice the previous width and still haven't encountered any affected square;
            // assume we're in an early pathological case and exit early.
            return None;
        }
        if beam_affects(left_x, y, program) {
            break;
        }
        left_x += 1;
    }

    // now find the right edge as well; since the width of the beam barely changes with each incremental Y position,
    // jump ahead by the previous width and scan backwards or forwards to find the edge of the beam.
    let mut right_x = left_x + prev_width;
    match beam_affects(right_x, y, program) {
        true => {
            // scan to the right
            right_x += 1;
            while beam_affects(right_x, y, program) {
                right_x += 1;
            }
        },
        false => {
            // scan to the left
            while !beam_affects(right_x-1, y, program) {
                right_x -= 1;
            }
        },
    };
    Some(left_x..right_x)
}

pub fn main() {
    let line: String = util::file_read_lines("input/day19.txt").into_iter().next().unwrap();
    let program: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();

    println!("{}", part1(&program, 50, false));
    println!("{}", part2(&program, 100));
}

#[allow(non_snake_case)]
fn part1(program: &Vec<i64>, N: usize, visualize: bool) -> usize {
    let mut num_affected = 0usize;

    let mut iterator = IncrementalBeamRange::new(program);
    for _ in 0..N {
        match iterator.next().unwrap() {
            None => {
                if visualize { print!("{}\n", ". ".repeat(N)); }
                continue; // no positions affected at this Y coordinate, skip
            },
            Some(x_range) => {
                num_affected += x_range.len();
                if visualize {
                    print!("{}", ". ".repeat(x_range.start));
                    print!("{}", "# ".repeat(x_range.len()));
                    print!("{}", ". ".repeat(N-x_range.end));
                }
            },
        };
        if visualize { print!("\n"); }
    }
    num_affected
}

#[allow(non_snake_case)]
fn part2(program: &Vec<i64>, N: usize) -> usize {
    // note the following properties about the tractor beam:
    //   - the X location of the first drone affected at each Y coordinate monotonically increases
    //        (i.e. first_affected_x_coord(Y) >= first_affected_x_coord(Y-1))
    //   - the width of the beam never changes 'much' from that of the previous Y coordinate's width
    //     (in some early pathological cases it sometimes disappears, but other than that only appears to ever grow
    //      or shrink by tiny fractions of its width, i.e. 1 or 2 positions)

    // for an NxN square to fit inside the beam with the top left coordinate at (X,Y), the following must hold:
    //   - the width of the beam at Y must be >= N (and therefore Y >= N as well, due to the monotically increasing beam width)
    //   - all positions (X, Y+[0..N]) must be affected by the tractor beam as well

    // we can ignore all the Y coordinates where the beam width has not yet reached N for the first time;
    // the beam width fluctuates slightly at each incremental Y position but displays an overall growth, so we can
    // save time by finding the Y coordinate where the beam first reaches a width of at least N.

    // in the general case looking for it incrementally is likely to waste a lot of time since the beam width grows slowly,
    // so e.g. a binary search is likely to save time, but for our particular problem input it turns out to be
    // 'quick enough' to find it incrementally.

    let mut iter = IncrementalBeamRange::new(program);

    // keep a window of the last N ranges seen of width >= N; if at any point they all share the same N consecutive
    // X coordinates then we found a place for the square to fit
    // (or equivalently, since the X coordinates are monotonically increasing: if the oldest range in the window
    // contains both the newest range's leftmost X and leftmost X + N coordinates).
    let mut result: Option<(usize, usize)> = None;
    let mut window = VecDeque::<Range<usize>>::with_capacity(N);
    while let Some(range_maybe) = iter.next() {
        if range_maybe.is_none() || range_maybe.as_ref().unwrap().len() < N {
            // haven't reached the required width yet or found a gap in the beam; reset the window
            window.clear();
            continue;
        }

        let range = range_maybe.unwrap(); // ok, found a range of the required length; add it to the window.
        window.push_back(range.clone());
        if window.len() < N {
            continue; // window needs to fill up for a bit longer
        }

        // as soon as we have N ranges in the window, look for N consecutive X coordinates that are present in all of them.
        // if we didn't find any, drop the oldest range from the window to make room on the next iteration.
        assert!(window.len() == N);
        if window[0].contains(&range.start) && window[0].contains(&(range.start+N-1)) {
            assert!(window.iter().all(|r| r.contains(&range.start) && r.contains(&(range.start+N-1))));
            result = Some((range.start, iter.next_y - N));
            break;
        } else {
            window.pop_front();
        }
    }
    let (x,y) = result.unwrap();
    x*10_000 + y
}

