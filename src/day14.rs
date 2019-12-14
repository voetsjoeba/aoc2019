// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use std::collections::{HashMap, VecDeque};
use std::convert::From;
use std::ops::{AddAssign, Mul};
use std::fmt;

#[derive(Debug, Clone)]
struct Resource {
    name: String,
    batch_size: usize,
    batch_inputs: TermSet,
}
impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} => {} {}", self.batch_inputs, self.batch_size, self.name)
    }
}

#[derive(Clone, Debug)]
struct Term {
    resource: String,
    quantity: usize,
}
macro_rules! term {
    ($resource_name:expr, $quant:expr) => { Term { resource: $resource_name.to_string(), quantity: $quant as usize } };
}
impl From<&str> for Term {
    fn from(s: &str) -> Self { // e.g. "5 ORE"
        let parts: Vec<&str> = s.trim().split(" ").collect();
        Self {
            quantity: parts[0].parse().unwrap(),
            resource: parts[1].to_string(),
        }
    }
}
impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.quantity, self.resource)
    }
}

#[derive(Debug,Clone)]
struct TermSet(HashMap<String, usize>); // a set of resources and their quantities, e.g. "5 A, 7 B"
impl TermSet {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn terms(&self) -> Vec<Term> {
        self.0.iter().map(|(name, &quant)| term![name, quant]).collect()
    }
}
impl Mul<usize> for &TermSet {
    type Output = TermSet;
    fn mul(self, rhs: usize) -> TermSet {
        TermSet(self.0.iter()
                      .map(|(name, &quant)| (name.clone(), quant*rhs))
                      .collect())
    }
}
impl From<Vec<Term>> for TermSet {
    fn from(rq_vec: Vec<Term>) -> Self {
        let mut map = HashMap::new();
        for rq in rq_vec {
            map.insert(rq.resource.clone(), rq.quantity);
        }
        Self(map)
    }
}
impl AddAssign<&Term> for TermSet {
    fn add_assign(&mut self, rq: &Term) {
        if let Some(q) = self.0.get_mut(&rq.resource) {
            *q += rq.quantity;
        } else {
            self.0.insert(rq.resource.clone(), rq.quantity);
        }
    }
}
impl fmt::Display for TermSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sorted_names: Vec<&String> = self.0.keys().collect();
        sorted_names.sort_unstable();
        write!(f, "{}", sorted_names.into_iter()
                                    .map(|name| term![name, self.0[name]].to_string())
                                    .collect::<Vec<_>>()
                                    .join(", "))
    }
}

struct Problem {
    resources: HashMap<String, Resource>,
}
impl Problem {
    pub fn new(lines: &Vec<String>) -> Self {
        let mut resources = HashMap::<String, Resource>::new();
        for line in lines {
            // lines are of the form: "5 XYZ, 7 ABC => 3 IJK"
            let parts: Vec<&str> = line.split("=>").collect();
            let lhs = TermSet::from(parts[0].split(",").map(Term::from).collect::<Vec<Term>>());
            let rhs = Term::from(parts[1]);

            let resource = Resource {
                name: rhs.resource,
                batch_size: rhs.quantity,
                batch_inputs: lhs,
            };
            resources.insert(resource.name.clone(), resource);
        }
        // add in a fictitious production rule for ORE with a batch size of 1 and no inputs
        // to simplify the solver logic a bit
        resources.insert("ORE".to_string(), Resource {
            name: "ORE".to_string(),
            batch_size: 1,
            batch_inputs: TermSet::new(),
        });
        Self {
            resources,
        }
    }
    pub fn ore_cost(&self, needed: Term)
        -> (usize, HashMap<String, usize>) // (ore cost, waste products)
    {
        self.ore_cost_with_initial_waste(needed, HashMap::new())
    }

    pub fn ore_cost_with_initial_waste(&self, needed: Term,
                                              initial_waste: HashMap<String, usize>)
        -> (usize, HashMap<String, usize>) // (ore cost, waste products)
    {
        let mut ore_needed = 0usize;
        let mut waste = initial_waste.clone();

        let mut to_expand = VecDeque::<Term>::new();
        to_expand.push_front(needed);

        while let Some(term) = to_expand.pop_front()
        {
            let resource = &self.resources[&term.resource];
            let wasted: &mut usize = waste.entry(resource.name.clone())
                                          .or_insert(0usize);

            // we need $term.quantity of $resource; do we have any of that left over
            // from a previous expansion? if so, reduce it by that amount.
            let mut needed = term.quantity;
            if needed <= *wasted {
                // don't need to do anything, still have this amount "in stock"
                // just need to update the new amount left over
                *wasted -= needed as usize;
                continue;
            }
            needed -= *wasted;
            assert!(needed > 0); // we already checked for needed <= wasted

            if resource.name == "ORE" {
                // can't expand ORE any further (and ORE can never be wasted, nothing produces it)
                ore_needed += needed as usize;
                continue;
            }

            // given the adjusted amount of this resource that we still need to produce,
            // how many batches of its production formula do we need to run?
            // and how many of this resource will we have produced in excess of what's needed?
            // that becomes the new amount wasted of this resource.
            let num_batches = ((needed as f64)/(resource.batch_size as f64)).ceil() as usize;
            *wasted = (num_batches * resource.batch_size) - needed;

            let scaled_inputs = &resource.batch_inputs * num_batches;
            for input_term in scaled_inputs.terms() {
                to_expand.push_front(input_term); // we can do this DFS (push_front) or BFS (push_back), doesn't matter
            }
        }

        (ore_needed, waste)
    }

}
impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sorted_names: Vec<&String> = self.resources.keys().collect();
        // skip the fictitious ORE production rule we added in the constructor of Problem
        sorted_names.remove(sorted_names.iter().position(|&name| name == "ORE").unwrap());

        sorted_names.sort_unstable_by_key(|&name| {
            let input_terms = self.resources[name].batch_inputs.terms();
            // lines of the form "x ORE => y Z" at the top, then the rest sorted by
            // ascending number of inputs, and finally resource name.
            (
                !(input_terms.len() == 1 && input_terms[0].resource == "ORE"),
                input_terms.len(),
                name
            )
        });
        #[allow(unused_must_use)]
        for name in sorted_names {
            write!(f, "{}\n", self.resources[name]);
        }
        Ok(())
    }
}

pub fn main() {
    let lines = util::file_read_lines("input/day14.txt");
    let problem = Problem::new(&lines);
    println!("{}", part1(&problem));
    println!("{}", part2(&problem));
}

fn part1(problem: &Problem) -> usize {
    let (ore_needed, _waste) = problem.ore_cost(term!["FUEL", 1]);
    ore_needed
}

fn part2(problem: &Problem) -> usize
{
    // feed the waste products of the last FUEL production back into the next one,
    // for maximal reuse of wasted resources.
    // TODO: slow, can probably speed this up by guesstimating an amount of FUEL that we can
    // produce and search around that neighbourhood
    let mut fuel_produced = 0usize;
    let mut ore_remaining = 1_000_000_000_000usize;

    let mut waste = HashMap::<String, usize>::new();
    loop {
        let (ore_cost, new_waste) = problem.ore_cost_with_initial_waste(term!["FUEL", 1], waste);
        waste = new_waste;

        if ore_cost > ore_remaining {
            break;
        }

        fuel_produced += 1;
        ore_remaining -= ore_cost;
    }

    fuel_produced
}

#[allow(unused)]
fn example_input(n: i32) -> Vec<String> {
    match n {
        1 => vec![
            "10 ORE => 10 A",
            "1 ORE => 1 B",
            "7 A, 1 B => 1 C",
            "7 A, 1 C => 1 D",
            "7 A, 1 D => 1 E",
            "7 A, 1 E => 1 FUEL",
        ],
        2 => vec![
            "9 ORE => 2 A",
            "8 ORE => 3 B",
            "7 ORE => 5 C",
            "3 A, 4 B => 1 AB",
            "5 B, 7 C => 1 BC",
            "4 C, 1 A => 1 CA",
            "2 AB, 3 BC, 4 CA => 1 FUEL",
        ],
        3 => vec![
            "157 ORE => 5 NZVS",
            "165 ORE => 6 DCFZ",
            "44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL",
            "12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ",
            "179 ORE => 7 PSHF",
            "177 ORE => 5 HKGWZ",
            "7 DCFZ, 7 PSHF => 2 XJWVT",
            "165 ORE => 2 GPVTF",
            "3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT",
        ],
        4 => vec![
            "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG",
            "17 NVRVD, 3 JNWZP => 8 VPVL",
            "53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL",
            "22 VJHF, 37 MNCFX => 5 FWMGM",
            "139 ORE => 4 NVRVD",
            "144 ORE => 7 JNWZP",
            "5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC",
            "5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV",
            "145 ORE => 6 MNCFX",
            "1 NVRVD => 8 CXFTF",
            "1 VJHF, 6 MNCFX => 4 RFSQX",
            "176 ORE => 6 VJHF",
        ],
        5 => vec![
            "171 ORE => 8 CNZTR",
            "7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL",
            "114 ORE => 4 BHXH",
            "14 VRPVC => 6 BMBT",
            "6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL",
            "6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT",
            "15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW",
            "13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW",
            "5 BMBT => 4 WPTQ",
            "189 ORE => 9 KTJDG",
            "1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP",
            "12 VRPVC, 27 CNZTR => 2 XDBXC",
            "15 KTJDG, 12 BHXH => 5 XCVML",
            "3 BHXH, 2 VRPVC => 7 MZWV",
            "121 ORE => 7 VRPVC",
            "7 XCVML => 6 RJRHP",
            "5 BHXH, 4 VRPVC => 5 LTCX",
        ],

        _ => panic!(),
    }.iter().map(|s| s.to_string()).collect::<Vec<_>>()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples() {
        assert_eq!(part1(&Problem::new(&example_input(1))), 31);
        assert_eq!(part1(&Problem::new(&example_input(2))), 165);
        assert_eq!(part1(&Problem::new(&example_input(3))), 13312);
        assert_eq!(part1(&Problem::new(&example_input(4))), 180697);
        assert_eq!(part1(&Problem::new(&example_input(5))), 2210736);
    }
}
