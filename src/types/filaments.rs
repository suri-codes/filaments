#![expect(dead_code)]
use std::{cmp::max, collections::HashMap, sync::Arc};

use eframe::emath;
use egui_graphs::{
    Graph,
    petgraph::{Directed, graph::NodeIndex, prelude::StableGraph},
};

use crate::types::{Index, Link, ZettelId};

pub type ZkGraph = Graph<ZettelId, Link, Directed>;

/// Minimum number of nodes in our graph.
const GRAPH_MIN_NODES: usize = 128;
/// Arbitrarily chosen minimum number of edges
const GRAPH_MIN_EDGES: usize = GRAPH_MIN_NODES * 3;

#[derive(Debug)]
pub struct Filaments {
    pub graph: ZkGraph,
    /// simple conversions
    zid_to_gid: HashMap<ZettelId, NodeIndex>,
}

pub type FilamentsHandle = Arc<std::sync::Mutex<Filaments>>;

impl From<&Index> for Filaments {
    fn from(value: &Index) -> Self {
        let number_of_zettels = value.zods().len();

        let mut zid_to_gid = HashMap::new();

        let mut graph: ZkGraph = ZkGraph::from(&StableGraph::with_capacity(
            max(number_of_zettels * 2, GRAPH_MIN_EDGES),
            max(number_of_zettels * 3, GRAPH_MIN_EDGES),
        ));

        for (zid, zod) in value.zods() {
            let node_idx = graph.add_node_custom(zid.clone(), |node| {
                node.set_label(zod.fm.title.clone());
                let disp = node.display_mut();
                disp.radius = 50.0;

                // randomize position
                let x = rand::random_range(0.0..=100.0);
                let y = rand::random_range(0.0..=100.0);
                node.set_location(emath::Pos2 { x, y });
                node.set_hovered(true);
            });

            let _ = zid_to_gid.insert(zid.clone(), node_idx);
        }

        for (_, links) in value.outgoing_links.clone() {
            for link in links {
                let start = zid_to_gid
                    .get(&link.source)
                    .expect("Invariant broken, must exist in here if its in the index");
                let end = zid_to_gid
                    .get(&link.dest)
                    .expect("Invariant broken, must exist in here if its in the index");

                let _ = graph.add_edge(*start, *end, link);
            }
        }

        Self { graph, zid_to_gid }
    }
}
