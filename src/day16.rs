// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use rulinalg::matrix::Matrix;
use rulinalg::vector::Vector;
use std::mem;

#[inline]
fn pattern_at(r: usize, c: usize) -> i32 {
    // pattern value at row r, column c
    [0, 1, 0, -1][((c+1)/(r+1)) % 4] // floored division
}

#[allow(non_snake_case)]
fn part1(line: &String, num_phases: u32) -> u32 {
    let input: Vec<u8> = line.chars().map(|c| c.to_string().parse().unwrap()).collect();
    let N = input.len();

    let mut data: Vec<i32> = Vec::with_capacity(N*N);
    for r in 0..N {
        for c in 0..N {
            data.push(pattern_at(r,c));
        }
    }

    let A = Matrix::new(N, N, data);

    let mut result = Vector::from_fn(N, |idx| input[idx % input.len()] as i32);
    for _ in 0..num_phases {
        result = &A * result;
        result.iter_mut().for_each(|v| *v = v.abs() % 10);
    }

    let data: &Vec<i32> = result.data();
    let result = data[0]*10_000_000
               + data[1]*1_000_000
               + data[2]*100_000
               + data[3]*10_000
               + data[4]*1000
               + data[5]*100
               + data[6]*10
               + data[7];
    result as u32
}

#[allow(non_snake_case)]
fn part2(line: &String, num_phases: u32, scale: u32) -> u32 {
    // scale = amount of times the input is repeated
    let input: Vec<u8> = line.chars().map(|c| c.to_string().parse().unwrap()).collect();
    let N = input.len() * (scale as usize);

    //  It's helpful to consider an FFT phase as a matrix multiplication:
    //     A x input = output
    //  where A is a square NxN matrix containing the FFT pattern:
    //
    //   1  0 -1  0  1  0 -1  0  1  0 -1  0  1  0 -1  0  1  0 -1  0  1  0 -1  0  1  0 -1  0  1  0 -1  0
    //   0  1  1  0  0 -1 -1  0  0  1  1  0  0 -1 -1  0  0  1  1  0  0 -1 -1  0  0  1  1  0  0 -1 -1  0
    //   0  0  1  1  1  0  0  0 -1 -1 -1  0  0  0  1  1  1  0  0  0 -1 -1 -1  0  0  0  1  1  1  0  0  0
    //   0  0  0  1  1  1  1  0  0  0  0 -1 -1 -1 -1  0  0  0  0  1  1  1  1  0  0  0  0 -1 -1 -1 -1  0
    //   0  0  0  0  1  1  1  1  1  0  0  0  0  0 -1 -1 -1 -1 -1  0  0  0  0  0  1  1  1  1  1  0  0  0
    //   0  0  0  0  0  1  1  1  1  1  1  0  0  0  0  0  0 -1 -1 -1 -1 -1 -1  0  0  0  0  0  0  1  1  1
    //   0  0  0  0  0  0  1  1  1  1  1  1  1  0  0  0  0  0  0  0 -1 -1 -1 -1 -1 -1 -1  0  0  0  0  0
    //   0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  0  0  0  0  0  0  0  0 -1 -1 -1 -1 -1 -1 -1 -1  0
    //   0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  0  0  0  0  0  0  0  0  0 -1 -1 -1 -1 -1 -1
    //   0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  0  0  0  0  0  0  0  0  0  0 -1 -1 -1
    //   0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1  0  0  0  0  0  0  0  0  0  0  0
    //   0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1  1  0  0  0  0  0  0  0  0  0
    //   0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1  1  1  0  0  0  0  0  0  0
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1  1  1  1  0  0  0  0  0
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  0  0  0
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  0
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  1
    //   0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1
    //
    //  Note the following properties:
    //    * it is an upper-triangular matrix: all elements below the diagonal are zero.
    //        this means that the output value at position k is computed using only elements at positions
    //        k and onwards from the input.
    //
    //        in particular, this means that if we're only interested in the output values starting at
    //        a specific position (as happens to be the case), then we can drop any input values before
    //        that position from the input and use a corresponding sub-matrix as well.
    //
    //    * on rows where there are no -1 elements, we can omit the "take absolute value and take it mod 10"
    //        step after each phase, and instead defer that step until all phases have been completed.
    //        as long as we don't need the output value of a row containing -1's, the algorithm can
    //        thus be simplified into:
    //
    //            1) compute A^num_phases = output
    //            2) replace each element v in the output vector by (v mod 10)
    //
    //        this is possible because for such rows, the output value is always guaranteed to be positive
    //        (since all inputs are single digits and hence also positive), and will remain positive through
    //        repeated multiplication with matrix A because of the upper-triangular property outlined above.
    //        therefore, on those rows the abs() step after every phase has no effect and can be omitted.
    //        the mod 10 operation after every phase can also be deferred as the last digit is preserved
    //        throughout further multiplications with other positive numbers.
    //           (i.e. ((a*b + c*d + ... + y*z)*(x mod 10)) mod 10 = (a*b + c*d + ... + y*z)*x mod 10 for a,b,...,z >= 0)
    //
    //        from the way the pattern repeats on any given row, we can determine the first row that no
    //        longer contains a -1 in it:
    //
    //              first row without a negative one:  ceil((N+1)/3) - 1    (0-based)
    //
    //        where N is the size of the input (and also both sizes of the matrix A).
    //        in the example above, this value is indeed ceil((32+1)/3) - 1 = 10.
    //
    // the problem statement is only asking for the values of 8 output values at a specific offset.
    // if that offset is at or beyond the first row without -1's, then we can solve the problem
    // by computing A^num_phases first, then multiplying with the (repeated) input as described above,
    // and finally taking the values mod 10.

    let first_line_without_negone: usize = (((N+1) as f64)/3.0f64).ceil() as usize - 1;
    let message_offset: usize = (input[0] as usize)*1_000_000
                              + (input[1] as usize)*100_000
                              + (input[2] as usize)*10_000
                              + (input[3] as usize)*1000
                              + (input[4] as usize)*100
                              + (input[5] as usize)*10
                              + (input[6] as usize);

    if message_offset >= N {
        panic!("invalid message offset {}; exceeds input size {}", message_offset, N);
    }
    if message_offset < first_line_without_negone {
        panic!("message offset is not big enough for efficient calculation");
    }

    let N_reduced = N - message_offset;

    let mut input: Vec<u32> = (message_offset..N).map(|x| input[x % input.len()] as u32).collect();
    let mut output: Vec<u32> = Vec::with_capacity(input.len());
    output.resize(input.len(), 0);

    for _ in 0..num_phases {
        // we know that there are no -1's on these rows, and therefore there can only be a
        // single run of 1's on these rows.

        // also, the lengths of these runs of 1's are sufficiently large at this point that
        // the matrix is essentially a clean square matrix with all 1's in the upper diagonal
        // and all 0's in the lower diagonal, so we can avoid duplicate addition work by adding
        // up values from the input backwards:
        //
        // 1  1  1  1  1  1  1  1
        // 0  1  1  1  1  1  1  1
        // 0  0  1  1  1  1  1  1
        // 0  0  0  1  1  1  1  1
        // 0  0  0  0  1  1  1  1
        // 0  0  0  0  0  1  1  1
        // 0  0  0  0  0  0  1  1
        // 0  0  0  0  0  0  0  1

        let mut incr_sum: u32 = 0;
        for k in 0..N_reduced {
            incr_sum = (incr_sum + input[N_reduced-1-k]) % 10;
            output[N_reduced-1-k] = incr_sum;
        }

        mem::swap(&mut input, &mut output);
    }

    let result = input[0]*10_000_000 // note: 'input' is actually output from the last iteration at this point
               + input[1]*1_000_000
               + input[2]*100_000
               + input[3]*10_000
               + input[4]*1000
               + input[5]*100
               + input[6]*10
               + input[7];
    result
}

pub fn main() {
    let line: String = util::file_read_lines("input/day16.txt").into_iter().next().unwrap();

    let (input, num_phases, scale) = (line, 100, 10_000);
    //let (input, num_phases, scale) = (example_input(5).clone().to_string(), 100, 10_000);
    //let (input, num_phases, scale) = (example_input(1).clone(), 4, 1);

    println!("{}", part1(&input, num_phases));
    println!("{}", part2(&input, num_phases, scale));
}

#[allow(dead_code)]
fn example_input(n: i32) -> String {
    match n {
        1 => "12345678",
        2 => "80871224585914546619083218645595",
        3 => "19617804207202209144916044189917",
        4 => "69317163492948606335995924319873",

        5 => "03036732577212944063491565474664",
        6 => "02935109699940807407585447034323",
        7 => "03081770884921959731165446850517",
        _ => panic!(),
    }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn examples() {
        assert_eq!(part1(&example_input(1), 4), 1029498);
        assert_eq!(part1(&example_input(2), 100), 24176176);
        assert_eq!(part1(&example_input(3), 100), 73745418);
        assert_eq!(part1(&example_input(4), 100), 52432133);

        assert_eq!(part2(&example_input(5), 100, 10_000), 84462026);
        assert_eq!(part2(&example_input(6), 100, 10_000), 78725270);
        assert_eq!(part2(&example_input(7), 100, 10_000), 53553731);
    }
}
