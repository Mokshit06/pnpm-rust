use std::collections::HashSet;

use graph_sequencer::{graph_sequencer, Graph, SequencerOptions, SequencerResult};
use indexmap::IndexMap;
use project::ProjectsGraph;
mod graph_sequencer;

pub fn sequence_graph<'a>(pkg_graph: &'a ProjectsGraph<'a>) -> SequencerResult<'a, String> {
    let keys = pkg_graph.iter().map(|(k, _)| k).collect::<Vec<_>>();
    let set_of_keys = HashSet::<_>::from_iter(keys.clone());
    let mut graph: Graph<String> = IndexMap::<&'a String, Vec<&'a String>>::new();

    for &pkg_path in keys.iter() {
        graph.insert(
            pkg_path,
            pkg_graph
                .get(pkg_path)
                .unwrap()
                .dependencies
                .iter()
                .filter(|d| d != &pkg_path && set_of_keys.contains(d))
                .collect(),
        );
    }

    graph_sequencer(SequencerOptions {
        graph,
        groups: vec![keys.clone()],
    })
}
