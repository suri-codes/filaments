#![expect(dead_code)]
use std::{cmp::max, collections::HashMap};

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

pub struct Filaments {
    graph: ZkGraph,
    /// simple conversions
    zid_to_gid: HashMap<ZettelId, NodeIndex>,
}

// pub type FilamentsHandle = Arc<RwLock>
//

// impl Filaments {
//     pub fn construct() -> Result<Self> {}
// }

impl From<&Index> for Filaments {
    fn from(value: &Index) -> Self {
        let number_of_zettels = value.zods().len();

        let mut _graph: ZkGraph = ZkGraph::from(&StableGraph::with_capacity(
            max(number_of_zettels * 2, GRAPH_MIN_EDGES),
            max(number_of_zettels * 3, GRAPH_MIN_EDGES),
        ));

        #[expect(clippy::for_kv_map)]
        for (_id, _zod) in value.zods() {}

        todo!()
    }
}
