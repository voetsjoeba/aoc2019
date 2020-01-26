// vim: set ai et ts=4 sts=4 sw=4:
#![allow(unused)]
use std::ops::Add;
use std::hash::Hash;
use std::fmt::Debug;
use std::collections::{VecDeque, HashMap};

pub trait Node: Hash + Eq + Clone       // so we can store references to these in a hashmap
{}

pub trait Map
{
    type Node: Node;
    type Cost: Default +                // so we have a way of starting of with a cost of 0 that keeps the typechecker happy
               Ord +                    // so we can take the minimum of a bunch of these
               Copy +                   // so we can copy and store these around easily
               PartialOrd +             // so we can less-than compare these
               Add<Output=Self::Cost>;  // so that adding two of these yields the same thing

    fn neighbours(&self, of: &Self::Node) -> Vec<(Self::Node, Self::Cost)>;
}

pub struct Path<N,M>
    where N: Node,
          M: Map<Node=N>
{
    pub nodes: Vec<N>,
    pub cost: M::Cost,
}
impl<N,M> Path<N,M>
    where N: Node,
          M: Map<Node=N>
{
    pub fn reconstruct_from(node: &N,
                            came_from: &HashMap<N,N>) -> Vec<N>
        where N: Node
    {
        let mut path = vec![node.clone()];
        let mut current: &N = node;
        while let Some(parent) = came_from.get(current) {
            path.push(parent.clone());
            current = parent;
        }
        path.reverse();
        path
    }
}

pub fn astar<N,M,H,W>(map: &M,
                      from: &N,
                      to: &N,
                      distance_heuristic: H,
                      is_walkable: W) -> Option<Path<N,M>>
    where N: Node,
          M: Map<Node=N>,
          H: Fn(&N, &N) -> M::Cost, // cost heuristic for distance between two nodes
          W: Fn(&M, &N) -> bool, // is a given node on the map walkable?
{
    let mut open_list = VecDeque::<N>::new();
    let mut g_scores  = HashMap::<N, M::Cost>::new();
    let mut f_scores  = HashMap::<N, M::Cost>::new();
    let mut came_from = HashMap::<N, N>::new(); // node immediately preceding it on the cheapest known path from start to n

    open_list.push_back(from.clone());
    g_scores.insert(from.clone(), M::Cost::default());
    f_scores.insert(from.clone(), distance_heuristic(from, to));

    while !open_list.is_empty()
    {
        // TODO: should use a priority queue
        let idx = (0..open_list.len()).min_by_key(|&i| f_scores[&open_list[i]]).unwrap();
        let current = open_list.remove(idx).unwrap();

        //let current = open_list.iter().min_by_key(|n| f_scores[n]).unwrap().clone(); // TODO: should use a priority queue
        if &current == to {
            let path = Path::<N,M>::reconstruct_from(&current, &came_from);
            return Some(Path {
                nodes: path,
                cost: g_scores[&current]
            });
        }

        //open_list.retain(|n| n != &current);
        for (nb, step_cost) in map.neighbours(&current) {
            if !is_walkable(map, &nb) {
                continue;
            }
            let new_g_score = g_scores[&current] + step_cost;
            if !g_scores.contains_key(&nb) || new_g_score < g_scores[&nb] {
                // path to neighbour through this node is better than any previous one; record it
                came_from.insert(nb.clone(), current.clone());
                g_scores.insert(nb.clone(), new_g_score);
                f_scores.insert(nb.clone(), new_g_score + distance_heuristic(&nb, to));

                if !open_list.contains(&nb) {
                    open_list.push_back(nb);
                }
            }
        }
    }
    None
}

pub fn dijkstra<M,N,W>(map: &M,
                       source: &N,
                       is_walkable: W) -> (HashMap<N, M::Cost>, HashMap<N,N>)
    where N: Node,
          M: Map<Node=N>,
          W: Fn(&M, &N) -> bool, // is a given node on the map walkable?
{
    dijkstra_impl(map, source, None, is_walkable)
}

pub fn dijkstra_to_target<M,N,W>(map: &M,
                                source: &N,
                                target: &N,
                                is_walkable: W) -> Option<Path<N,M>>
    where N: Node,
          M: Map<Node=N>,
          W: Fn(&M, &N) -> bool, // is a given node on the map walkable?
{
    let (dists, came_from) = dijkstra_impl(map, source, Some(target), is_walkable);
    assert!(dists.contains_key(target));
    Some(Path {
        nodes: Path::<N,M>::reconstruct_from(target, &came_from),
        cost: dists[target],
    })
}
fn dijkstra_impl<M,N,W>(map: &M,
                        source: &N,
                        target: Option<&N>,
                        is_walkable: W) -> (HashMap<N, M::Cost>, HashMap<N,N>)
    where N: Node,
          M: Map<Node=N>,
          W: Fn(&M, &N) -> bool, // is a given node on the map walkable?
{
    let mut dist      = HashMap::<N, M::Cost>::new();
    let mut came_from = HashMap::<N, N>::new();

    let mut queue = VecDeque::<N>::new(); // TODO: should use a priority queue
    dist.insert(source.clone(), M::Cost::default());
    queue.push_back(source.clone());

    while !queue.is_empty() {
        let min_idx = (0..queue.len()).min_by_key(|&idx| dist[&queue[idx]]).unwrap();
        let node = queue.remove(min_idx).unwrap();

        if let Some(t) = target {
            if node == *t {
                return (dist, came_from);
            }
        }

        for (nb, step_cost) in map.neighbours(&node) {
            if !is_walkable(map, &nb) {
                continue;
            }
            let alt = dist[&node] + step_cost;
            if !dist.contains_key(&nb) || alt < dist[&nb] {
                dist.insert(nb.clone(), alt);
                came_from.insert(nb.clone(), node.clone());
                queue.push_back(nb);
            }
        }
    }

    (dist, came_from)
}
