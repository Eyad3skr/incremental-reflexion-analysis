// incremental diffs
use std::collections::HashSet;
use std::ops::Sub;
use crate::core::graph::{GraphError, ReflexionGraph, Edge};
use crate::core::state::EdgeState;
use crate::core::types::{EdgeId, NodeId, SubgraphKind, EdgeKind};

impl ReflexionGraph {
    /// Naive incremental: insert an implementation edge then recompute everything.
    ///
    /// Correct but not efficient. This is an API skeleton that will later be replaced
    /// by a real delta-based incremental algorithm.
    pub fn add_impl_edge_and_recompute(&mut self, edge: Edge) -> Result<EdgeId, GraphError> {
        if edge.subgraph != SubgraphKind::Implementation {
            return Err(GraphError::WrongSubgraph {
                node: edge.from, // best available identifier here
                expected: SubgraphKind::Implementation,
                found: edge.subgraph,
            });
        }

        let id = self.add_edge(edge)?;

        // Avoid stale propagated edges / counters accumulating across runs.
        self.clear_propagated_edges();
        self.run_from_scratch();

        Ok(id)
    }

    /// Naive incremental: remove an implementation edge then recompute everything.
    pub fn remove_impl_edge_and_recompute(&mut self, edge_id: EdgeId) -> Result<(), GraphError> {
        let e = self
            .edges
            .get(&edge_id)
            .ok_or(GraphError::EdgeNotFound(edge_id))?;

        if e.subgraph != SubgraphKind::Implementation {
            return Err(GraphError::WrongSubgraph {
                node: e.from,
                expected: SubgraphKind::Implementation,
                found: e.subgraph,
            });
        }

        let from = e.from;

        // Remove the edge object
        self.edges.remove(&edge_id);

        // Remove from adjacency
        if let Some(v) = self.impl_out.get_mut(&from) {
            v.retain(|&x| x != edge_id);
        }

        // Recompute from scratch
        self.clear_propagated_edges();
        self.run_from_scratch();

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::ops::Sub;
    use crate::core::graph::{Node, GraphError, ReflexionGraph, Edge};
    use crate::core::state::EdgeState;
    use crate::core::types::{EdgeId, NodeId, SubgraphKind, EdgeKind};

    fn mk_node(name: &str, subgraph: SubgraphKind) -> Node {
        Node::new(name, subgraph, None)
    }

    fn mk_edge(from: u32, to: u32, subgraph: SubgraphKind, kind: EdgeKind) -> Edge {
        Edge {
            id: 0,
            from,
            to,
            kind,
            subgraph,
            state: EdgeState::Undefined,
            counter: 0,
        }
    }

    #[test]
    fn naive_incremental_add_then_remove_restores_previous_classification() {
        let mut g = ReflexionGraph::new();

        //architecture
        let ui = g.add_node(mk_node("UI", SubgraphKind::Architecture)).unwrap();
        let service = g.add_node(mk_node("Service", SubgraphKind::Architecture)).unwrap();
        let e_arch = g.add_edge(mk_edge(ui, service, SubgraphKind::Architecture, EdgeKind::depends_on())).unwrap();

        //implementation
        let login = g.add_node(mk_node("LoginPage", SubgraphKind::Implementation)).unwrap();
        let usersvc = g.add_node(mk_node("UserService", SubgraphKind::Implementation)).unwrap();
        let e_impl_ok = g.add_edge(mk_edge(login, usersvc, SubgraphKind::Implementation, EdgeKind::depends_on())).unwrap();

        //mapping
        g.set_mapping_overwrite(login, ui).unwrap();
        g.set_mapping_overwrite(usersvc, service).unwrap();

        //baseline
        g.run_from_scratch();
        assert!(matches!(g.edges.get(&e_arch).unwrap().state, EdgeState::Convergent));
        assert!(matches!(g.edges.get(&e_impl_ok).unwrap().state, EdgeState::Allowed));

        //add a divergent impl edge (reverse direction)
        let new_div = mk_edge(usersvc, login, SubgraphKind::Implementation, EdgeKind::depends_on());
        let e_impl_div = g.add_impl_edge_and_recompute(new_div).unwrap();

        assert!(matches!(g.edges.get(&e_impl_div).unwrap().state, EdgeState::Divergent));

        //remove it and ensure we’re back to baseline
        g.remove_impl_edge_and_recompute(e_impl_div).unwrap();

        assert!(matches!(g.edges.get(&e_arch).unwrap().state, EdgeState::Convergent));
        assert!(matches!(g.edges.get(&e_impl_ok).unwrap().state, EdgeState::Allowed));
    }
}
