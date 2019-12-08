// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use std::collections::{HashMap, HashSet};

pub fn main() {
    let lines = util::file_read_lines("input/day6.txt");
    let mut data = Vec::<(String,String)>::new();
    for line in lines {
        let parts = line.split(")").collect::<Vec<_>>();
        data.push((parts[0].to_string(), parts[1].to_string()));
    }
    let mut parents = HashMap::<String, String>::new(); // maps a node name to its parent node name
    for (parent, child) in data {
        parents.insert(child.to_string(), parent.to_string());
    }
    println!("{}", part1(&parents));
    println!("{}", part2(&parents));
}

fn get_path(node: &String, parents: &HashMap<String, String>) -> Vec<String> {
    let mut result = vec![node.to_string()];
    let mut current_node: &String = node;
    while parents.contains_key(current_node) {
        let parent = parents.get(current_node).unwrap();
        result.push(parent.to_string());
        current_node = parent;
    }
    return result;
}

fn part1(parents: &HashMap<String, String>) -> usize {
    let mut result = 0;
    for node in parents.keys() {
        result += get_path(node, &parents).len()-1; // -1 because the path includes the node itself
    }
    result
}

fn part2(parents: &HashMap<String, String>) -> usize {
    let you_parent: &String = parents.get(&"YOU".to_string()).unwrap();
    let san_parent: &String = parents.get(&"SAN".to_string()).unwrap();
    let you_parent_path: HashSet<String> = get_path(you_parent, &parents).into_iter().collect();
    let san_parent_path: HashSet<String> = get_path(san_parent, &parents).into_iter().collect();

    // find common nodes between the two paths, and find the one with the longest distance
    // (i.e. the one that's closest to both YOU and SAN)
    let common = you_parent_path.intersection(&san_parent_path);
    let mut common: Vec<String> = common.into_iter().map(|s| s.to_string()).collect();
    common.sort_by_key(|node| get_path(node, parents).len());

    let result = (you_parent_path.len() - common.len()) +
                 (san_parent_path.len() - common.len());
    result
}
