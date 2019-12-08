// vim: set ai et ts=4 sts=4 sw=4:
pub fn main() {
    part(1);
    part(2);
}

fn part(num: i32) {
    let mut result = 0;
    for i in 231832..767346 {
        if meets_conditions(i, num == 2) {
            result += 1;
        }
    }
    println!("{}", result);
}

#[allow(unused_parens)]
fn meets_conditions(num: i32, part2: bool) -> bool {
    let digits = num.to_string().chars()
                                .map(|x| x.to_string().parse().unwrap())
                                .collect::<Vec<u32>>();

    let mut has_group = false;
    let mut has_exact_pair = false;

    let mut i = 0;
    while i < 6 {
        let digit = digits[i];
        // scan forward to find any groupings and to make sure we're going on ascending-or-equal order
        let mut group_length = 1;
        let mut k = i+1;
        while k < 6 {
            if digits[k] == digit { group_length += 1; }
            else if digits[k] < digit { return false; }
            else { break; }
            k += 1;
        }
        has_group      = (has_group      || group_length >= 2);
        has_exact_pair = (has_exact_pair || group_length == 2);

        i = k;
    }

    if part2 {
        // if there is an exact pair in the number, then the rule of "no triples or more" goes away
        return has_exact_pair;
    } else {
        return has_group;
    }
}
