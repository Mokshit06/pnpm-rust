use indexmap::IndexMap as HashMap;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::RangeBounds;
use std::rc::Rc;

type Graph<'a, T> = HashMap<&'a T, Vec<&'a T>>;
type Groups<'a, T> = Vec<Vec<&'a T>>;

pub struct SequencerOptions<'a, T: PartialEq> {
    pub graph: Graph<'a, T>,
    pub groups: Groups<'a, T>,
}

#[derive(Debug, PartialEq)]
pub struct SequencerResult<'a, T: PartialEq> {
    pub safe: bool,
    pub chunks: Groups<'a, T>,
    pub cycles: Groups<'a, T>,
}

fn visit<'a, T: Clone + Eq + Hash + Debug>(
    item: &'a T,
    cycle: &[&'a T],
    visited: &mut Graph<'a, T>,
    curr_deps_map: &mut Graph<'a, T>,
    cycles: &mut Groups<'a, T>,
) {
    if let Some(deps) = curr_deps_map.get(item).map(|deps| deps.clone()) {
        println!("deps = {:?}", deps);
        for dep in deps {
            println!("[{:?}, {:?}]", cycle, dep);
            if cycle[0] == dep {
                cycles.push(cycle.to_vec());
            }

            let mut r = vec![];
            let mut visited_deps_option = visited.get_mut(item);
            let is_none = visited_deps_option.is_none();
            let mut visited_deps = if let Some(v) = visited_deps_option {
                v
            } else {
                &mut r
                // visited.insert(item, r);
            };

            if visited_deps.contains(&dep) || (dep == item && visited_deps.contains(&item)) {
                visited_deps.push(dep);
                visited.insert(item, r);
                visit(dep, cycle, visited, curr_deps_map, cycles);
            } else {
                visited.insert(item, r);
            }
        }
    }
}

fn get_cycles<'a, T: Clone + Eq + Hash + Debug>(
    curr_deps_map: &mut Graph<'a, T>,
    visited: &mut Graph<'a, T>,
) -> Groups<'a, T>
where {
    let items = curr_deps_map
        .iter()
        .map(|(&k, _)| k)
        .clone()
        .collect::<Vec<_>>();
    let mut cycles: Groups<T> = vec![];

    for item in items {
        visit(item, &[item], visited, curr_deps_map, &mut cycles);
    }

    cycles
}

pub fn graph_sequencer<'a, T: Eq + Hash + Clone + Ord + Debug>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn graph_with_no_deps() {
        let groups = vec![vec![&"a", &"b", &"c", &"d"]];

        assert_eq!(
            graph_sequencer(SequencerOptions {
                graph: HashMap::from_iter([
                    (&"a", vec![]),
                    (&"b", vec![]),
                    (&"c", vec![]),
                    (&"d", vec![]),
                ]),
                groups: groups.clone(),
            }),
            SequencerResult {
                chunks: groups.clone(),
                cycles: vec![],
                safe: true
            }
        );
    }

    #[test]
    fn graph_with_multiple_dependencies_on_one_time() {
        assert_eq!(
            graph_sequencer(SequencerOptions {
                graph: HashMap::from_iter([
                    (&"a", vec![&"d"]),
                    (&"b", vec![&"d"]),
                    (&"c", vec![]),
                    (&"d", vec![])
                ]),
                groups: vec![vec![&"a", &"b", &"c", &"d"]]
            }),
            SequencerResult {
                safe: true,
                chunks: vec![vec![&"c", &"d"], vec![&"a", &"b"]],
                cycles: vec![]
            }
        )
    }

    #[test]
    fn graph_with_resolved_cycle() {
        assert_eq!(
            graph_sequencer(SequencerOptions {
                graph: HashMap::from_iter([
                    (&"a", vec![&"b"]),
                    (&"b", vec![&"c"]),
                    (&"c", vec![&"d"]),
                    (&"d", vec![&"a"]),
                ]),
                groups: vec![vec![&"a"], vec![&"b", &"c", &"d"]],
            }),
            SequencerResult {
                safe: true,
                chunks: vec![vec![&"a"], vec![&"d"], vec![&"c"], vec![&"b"]],
                cycles: vec![],
            },
        );
    }

    #[test]
    fn graph_with_resolved_cycle_with_multiple_unblocked_deps() {
        assert_eq!(
            graph_sequencer(SequencerOptions {
                graph: HashMap::from_iter([
                    (&"a", vec![&"d"]),
                    (&"b", vec![&"d"]),
                    (&"c", vec![&"d"]),
                    (&"d", vec![&"a"]),
                ]),
                groups: vec![vec![&"d"], vec![&"a", &"b", &"c"]]
            }),
            SequencerResult {
                safe: true,
                chunks: vec![vec![&"d"], vec![&"a", &"b", &"c"]],
                cycles: vec![]
            }
        )
    }

    #[test]
    fn graph_with_unresolved_cycle() {
        assert_eq!(
            graph_sequencer(SequencerOptions {
                graph: HashMap::from_iter([
                    (&"a", vec![&"b"]),
                    (&"b", vec![&"c"]),
                    (&"c", vec![&"d"]),
                    (&"d", vec![&"a"]),
                ]),
                groups: vec![vec![&"a", &"b", &"c", &"d"]]
            }),
            SequencerResult {
                safe: false,
                chunks: vec![vec![&"a"], vec![&"d"], vec![&"c"], vec![&"b"]],
                cycles: vec![]
            }
        )
    }

    #[test]
    fn graph_with_multiple_resolves_cycles() {
        assert_eq!(
            graph_sequencer(SequencerOptions {
                graph: HashMap::from_iter([
                    (&"a", vec![&"b"]),
                    (&"b", vec![&"a"]),
                    (&"c", vec![&"d"]),
                    (&"d", vec![&"c"]),
                ]),
                groups: vec![vec![&"b", &"c"], vec![&"a", &"d"]],
            }),
            SequencerResult {
                safe: true,
                chunks: vec![vec![&"b", &"c"], vec![&"a", &"d"]],
                cycles: vec![],
            }
        )
    }
}
