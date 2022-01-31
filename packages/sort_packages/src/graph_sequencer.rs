use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::RangeBounds;
use std::rc::Rc;

type Graph<'a, T> = HashMap<&'a T, Vec<&'a T>>;
type Groups<'a, T> = Vec<Vec<&'a T>>;

pub struct SequencerOptions<'a, T> {
    pub graph: Graph<'a, T>,
    pub groups: Groups<'a, T>,
}

pub struct SequencerResult<'a, T> {
    pub safe: bool,
    pub chunks: Groups<'a, T>,
    pub cycles: Groups<'a, T>,
}

fn get_cycles<'a, 'r, T>(
    curr_deps_map: &'a mut Graph<'r, T>,
    visited: &'a mut Graph<'r, T>,
) -> Groups<'r, T>
where
    T: Eq + Hash,
{
    let items = curr_deps_map.iter().map(|(&k, _)| k);

    todo!()
}

pub fn graph_sequencer<'a, T: Eq + Hash + Clone + Ord /* + std::fmt::Debug */>(
    opts: SequencerOptions<'a, T>,
) -> SequencerResult<'a, T> {
    let SequencerOptions { graph, groups } = &opts;
    let graph_items = graph.iter().map(|(&k, _)| k).collect::<Vec<_>>();

    // assert_eq!(
    //     {
    //         let mut graph_items = graph_items.clone();
    //         graph_items.sort();
    //         graph_items
    //     },
    //     {
    //         let mut groups = groups.iter().flatten().collect::<Vec<_>>();
    //         groups.sort();
    //         groups
    //     }
    // );

    // We'll push to these with the results.
    let mut chunks: Groups<T> = vec![];
    let mut cycles: Groups<T> = vec![];
    let mut safe = true;

    // We'll keep replacing this queue as we unload items into chunks.
    let mut queue = graph_items;
    let mut chunked = HashSet::<&T>::new();
    let mut visited: Graph<T> = HashMap::new();

    while !queue.is_empty() {
        let mut next_queue = vec![];
        let mut chunk = vec![];
        let mut curr_deps_map: Graph<T> = HashMap::new();

        for &item in queue.iter() {
            let deps = graph.get(item);

            match deps {
                Some(deps) => {
                    // this should return -1 if the item is not found
                    // this is to match js behaviour in the original lib
                    let item_group = groups
                        .iter()
                        .position(|group| group.contains(&item))
                        .map(|i| i as isize)
                        .unwrap_or(-1);
                    let curr_deps = deps
                        .iter()
                        .filter(|dep| {
                            let dep_group = groups
                                .iter()
                                .position(|group| group.contains(dep))
                                .map(|i| i as isize)
                                .unwrap_or(-1);

                            if dep_group > item_group {
                                false
                            } else {
                                !chunked.contains(*dep)
                            }
                        })
                        .map(|dep| *dep)
                        .collect::<Vec<_>>();

                    if curr_deps.is_empty() {
                        chunk.push(item);
                    } else {
                        next_queue.push(item);
                    }

                    curr_deps_map.insert(item, curr_deps);
                }
                None => continue,
            }
        }

        if chunk.is_empty() {
            for cycle in get_cycles(&mut curr_deps_map, &mut visited) {
                cycles.push(cycle);
            }

            queue.sort_by(|&a, &b| {
                let vec_ref = vec![];
                let a_deps = curr_deps_map.get(a).unwrap_or(&vec_ref);
                let b_deps = curr_deps_map.get(b).unwrap_or(&vec_ref);

                a_deps.len().cmp(&b_deps.len())
            });

            chunk.push(queue[0]);
            next_queue = (&queue[1..].iter().map(|&x| x).collect::<Vec<_>>()).clone();
            safe = false;
        }

        for item in chunk.iter() {
            chunked.insert(*item);
        }

        chunk.sort();
        chunks.push(chunk);
        queue = next_queue;
    }

    SequencerResult {
        safe,
        chunks,
        cycles,
    }
}
