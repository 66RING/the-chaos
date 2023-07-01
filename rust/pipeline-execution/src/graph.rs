//! Graph wrapper

use std::sync::Arc;

use petgraph::stable_graph::{DefaultIx, NodeIndex, StableDiGraph};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use crate::processor::Processor;

#[derive(Debug)]
pub struct ExGraph(StableDiGraph<Arc<dyn Processor>, ()>);

pub type Index = NodeIndex<DefaultIx>;

impl ExGraph {
    pub fn new() -> Self {
        ExGraph(StableDiGraph::new())
    }

    pub fn add_node(&mut self, exec: Arc<dyn Processor>) -> Index {
        self.0.add_node(exec)
    }

    pub fn add_edge(&mut self, from: Index, to: Index) {
        self.0.add_edge(from, to, ());
    }

    pub fn get_node_by_index(&self, index: Index) -> Arc<dyn Processor> {
        self.0[index].clone()
    }

    pub fn get_prev_nodes(&self, index: Index) -> Vec<Arc<dyn Processor>> {
        let mut execs = vec![];
        // iter all edge the is the imcoming edge of index
        for e in self.0.edges_directed(index, Direction::Incoming) {
            let node = self.get_node_by_index(e.source());
            execs.push(node);
        }
        execs
    }

    pub fn get_next_nodes(&self, index: Index) -> Vec<Arc<dyn Processor>> {
        let mut execs = vec![];
        // iter all edge the is the outgoing edge of index
        for e in self.0.edges_directed(index, Direction::Outgoing) {
            let node = self.get_node_by_index(e.target());
            execs.push(node);
        }
        execs
    }

    pub fn get_all_nodes(&self) -> Vec<Arc<dyn Processor>> {
        self.0.node_weights().cloned().collect()
    }

    /// get the final executor, which should be call only when merge node was added
    pub fn get_last_node(&self) -> Option<Arc<dyn Processor>> {
        match self.0.node_indices().find(|node| self.0.edges_directed(*node, Direction::Outgoing).count() == 0) {
            Some(idx) => Some(self.get_node_by_index(idx)),
            None => None
        }
    }
}

