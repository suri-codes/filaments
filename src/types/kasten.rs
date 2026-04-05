use crate::types::{Link, Zettel, ZettelId};
use color_eyre::eyre::Result;
use dto::{TagEntity, ZettelEntity};
use eframe::emath;
use egui_graphs::{
    Graph, Node,
    petgraph::{Directed, Direction, graph::NodeIndex, prelude::StableGraph, visit::EdgeRef},
};
use rayon::iter::{ParallelBridge as _, ParallelIterator as _};
use std::{cmp::max, collections::HashMap, path::Path, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::types::Workspace;

#[derive(Debug, Clone)]
#[expect(dead_code)]
pub struct Kasten {
    /// Private field so it can only be instantiated from a `Path`
    _private: (),

    /// The workspace this `Kasten` is in
    pub ws: Workspace,

    /// the graph of `Zettel`s and the `Links` between them
    pub graph: ZkGraph,

    /// simple conversions
    zid_to_gid: HashMap<ZettelId, NodeIndex>,

    pub most_recently_edited: Option<NodeIndex>,
}

pub type ZkGraph = Graph<Zettel, Link, Directed>;

/// Minimum number of nodes in our graph.
const GRAPH_MIN_NODES: usize = 128;
/// Arbitrarily chosen minimum number of edges
const GRAPH_MIN_EDGES: usize = GRAPH_MIN_NODES * 3;

pub type KastenHandle = Arc<RwLock<Kasten>>;

impl Kasten {
    /// Indexes the `Workspace` and constructs a `Kasten`
    pub async fn index(ws: Workspace) -> Result<Self> {
        let paths = std::fs::read_dir(&ws.root)?
            .par_bridge()
            .flatten()
            .filter(|entry| {
                entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                    && entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| ext == "md")
            })
            .map(|entry| entry.path())
            .collect::<Vec<_>>();

        info!(
            "indexing the following paths {paths:#?} at root {:#?}",
            ws.root
        );

        let zettel_tasks = paths
            .into_iter()
            .map(|path| {
                let ws = ws.clone();
                tokio::spawn(async move { Zettel::from_path(path, &ws).await })
            })
            .collect::<Vec<_>>();

        // await all of them
        let zettels = futures::future::join_all(zettel_tasks)
            .await
            .into_iter()
            .filter_map(|result| {
                result
                    .inspect_err(|e| error!("Failed to join on zettel task parsing: {e:#?}"))
                    .ok()?
                    .inspect_err(|e| error!("Failed to parse file into zettel: {e:#?}"))
                    .ok()
            })
            .collect::<Vec<Zettel>>();

        info!("zettels: {zettels:#?}");

        // capacity!
        let mut graph: ZkGraph = ZkGraph::from(&StableGraph::with_capacity(
            max(zettels.len() * 2, GRAPH_MIN_EDGES),
            max(zettels.len() * 3, GRAPH_MIN_EDGES),
        ));

        let mut zid_to_gid = HashMap::new();
        for zettel in &zettels {
            let fm = zettel.front_matter(&ws).await?;
            let id = graph.add_node_custom(zettel.clone(), |node| {
                fm.apply_node_transform(node);
                let x = rand::random_range(0.0..=100.0);
                let y = rand::random_range(0.0..=100.0);
                node.set_location(emath::Pos2 { x, y });
            });
            zid_to_gid.insert(zettel.id.clone(), id);
        }

        for zettel in &zettels {
            let src = zid_to_gid.get(&zettel.id).expect("must exist");
            for link in &zettel.links(&ws).await? {
                let dst = zid_to_gid.get(&link.dest).expect("must exist");
                graph.add_edge(*src, *dst, link.clone());
            }
        }

        info!("parsed graph: {graph:#?}");

        Ok(Self {
            _private: (),
            ws,
            graph,
            zid_to_gid,
            most_recently_edited: None,
        })
    }

    /// processes the `Zettel` for the provided `ZettelId`,
    /// meaning it updates the internal state of the `Kasten`
    /// with the changes in `Zettel`.
    pub async fn process_path(&mut self, path: &Path) -> Result<()> {
        //NOTE: need to clone to get around borrowing rules but
        // ideally we dont have to do this, kind of cringe imo.
        let ws = self.ws.clone();

        let zid = ZettelId::try_from(path)?;

        let mut gid = self.zid_to_gid.get(&zid).copied();
        // sometimes this zid is new, so it wont be in the kasten
        let zettel = if let Some(existing) = self.get_node_by_zettel_id_mut(&zid) {
            existing.payload_mut()
        } else {
            // this should aleady be in the database though so lets get it from there first
            let zettel: Zettel = ZettelEntity::load()
                .filter_by_nano_id(zid)
                .with(TagEntity)
                .one(&ws.db)
                .await?
                .expect("This should be in the database already")
                .into();

            let zid = zettel.id.clone();
            let idx = self.graph.add_node(zettel);

            self.zid_to_gid.insert(zid.clone(), idx);

            gid = Some(idx);

            self.get_node_by_zettel_id_mut(&zid)
                .expect("we just inserted it")
                .payload_mut()
        };

        // and then we sync with the file
        zettel.sync_with_file(&ws).await?;

        // to get past borrowchecker rules
        let zettel = zettel.clone();

        // gid must be set
        let gid = gid.unwrap();

        // and now we manage the links going out of the file

        // remove all the old shit
        self.graph
            .edges_directed(gid, Direction::Outgoing)
            .map(|e| e.id())
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|e| {
                let _ = self.graph.remove_edge(e);
            });

        // add the links that actually exist
        zettel.links(&ws).await?.into_iter().for_each(|link| {
            // this is an option because a user c
            let dest = self
                .zid_to_gid
                .get(&link.dest)
                .expect("Links should be valid");

            self.graph.add_edge(gid, *dest, link);
        });

        Ok(())
    }

    pub fn get_node_by_zettel_id(&self, id: &ZettelId) -> Option<&Node<Zettel, Link>> {
        let idx = self.zid_to_gid.get(id)?;

        let node = self.graph.node(*idx).expect(
            "invariant broken if internal hashmap is not uptodate with
            the state of the graph...",
        );
        Some(node)
    }

    pub fn get_node_by_zettel_id_mut(&mut self, id: &ZettelId) -> Option<&mut Node<Zettel, Link>> {
        let idx = self.zid_to_gid.get(id)?;

        let node = self.graph.node_mut(*idx).expect(
            "invariant broken if internal hashmap is not uptodate with the
            state of the graph...",
        );

        Some(node)
    }
}
