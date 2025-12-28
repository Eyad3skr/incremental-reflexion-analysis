// classification logic
use std::collections::HashSet;
use std::ops::Sub;
use crate::core::graph::{GraphError, ReflexionGraph, Edge};
use crate::core::state::EdgeState;
use crate::core::types::{EdgeId, NodeId, SubgraphKind, EdgeKind};


impl ReflexionGraph {
    //Run a full reflexion analysis from scratch:
    // - Clears old propagated edges
    // - Resets states/counters
    // - Propagates + lifts every impl edge
    // - Finalizes arch edge states (Absent/Convergent normalization)

    pub fn run_from_scratch(&mut self) -> Result<(), GraphError> {
        //if we ran before, we must drop old propagated edges, otherwise stale edges can survive
        self.clear_propagated_edges();

        //reset counters/states
        self.init_states();

        //collect implementation edges 
        let impl_edge_ids: Vec<EdgeId> = self.edges.iter().filter_map(|(&eid, e)| {
            if e.subgraph == SubgraphKind::Implementation {
                Some(eid)
            } else {
                None
            }
        }).collect();

        for eid in impl_edge_ids {
            self.propagate_and_lift(eid);
        }

        self.finalize_architecture_states();
        Ok(())
    }

    // Normalize final architecture edge states after propagation+lifting:
    // - Specified + counter==0  -> Absent
    // - Specified + counter>0   -> Convergent  (defensive normalization)
    pub fn finalize_architecture_states(&mut self) {
        for e in self.edges.values_mut() {
            if e.subgraph != SubgraphKind::Architecture {
                continue;
            }

            if matches!(e.state, EdgeState::Specified) && e.counter == 0 {
                e.state = EdgeState::Absent;
            } else if matches!(e.state, EdgeState::Specified) && e.counter > 0 {
                // If lifting forgot to flip it, finalize makes it consistent.
                e.state = EdgeState::Convergent;
            }
        }
    }

    pub fn count_violations(&self) -> usize {
        self.edges.values().filter(|e| e.state.is_violation()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::{Edge, Node, ReflexionGraph};
    use crate::core::state::EdgeState;
    use crate::core::types::{EdgeKind, SubgraphKind};

    fn mk_edge(from: u32, to: u32, subgraph: SubgraphKind, kind: EdgeKind) -> Edge {
        Edge {
            id: 0, // overwritten by add_edge
            from,
            to,
            kind,
            subgraph,
            state: EdgeState::Undefined,
            counter: 0,
        }
    }

    #[test]
    fn run_from_scratch_marks_convergent_allowed_and_no_divergence() {
        let mut g = ReflexionGraph::new();

        //architecture nodes
        let ui = g
            .add_node(Node::new("UI", SubgraphKind::Architecture, None))
            .unwrap();
        let service = g
            .add_node(Node::new("Service", SubgraphKind::Architecture, None))
            .unwrap();

        //implementation nodes
        let login_page = g
            .add_node(Node::new("LoginPage", SubgraphKind::Implementation, None))
            .unwrap();
        let user_service = g
            .add_node(Node::new("UserService", SubgraphKind::Implementation, None))
            .unwrap();

        //arch edge UI -> Service (specified)
        let arch_eid = g
            .add_edge(mk_edge(ui, service, SubgraphKind::Architecture, EdgeKind::calls()))
            .unwrap();

        //impl edge LoginPage -> UserService
        let impl_eid = g
            .add_edge(mk_edge(
                login_page,
                user_service,
                SubgraphKind::Implementation,
                EdgeKind::calls(),
            ))
            .unwrap();

        //mapping impl -> arch
        g.set_mapping(login_page, ui).unwrap();
        g.set_mapping(user_service, service).unwrap();

        //act
        g.run_from_scratch().unwrap();

        //assert arch edge convergent + counter incremented
        let arch_edge = g.edges.get(&arch_eid).unwrap();
        assert!(matches!(arch_edge.state, EdgeState::Convergent));
        assert_eq!(arch_edge.counter, 1);

        //assert impl edge allowed (because it lifted onto specified arch edge)
        let impl_edge = g.edges.get(&impl_eid).unwrap();
        assert!(matches!(impl_edge.state, EdgeState::Allowed));

        //assert no divergent edges anywhere
        let has_divergent = g.edges.values().any(|e| matches!(e.state, EdgeState::Divergent));
        assert!(!has_divergent);

        //also: there should be no Absent edges in this tiny setup (the only arch edge is covered)
        let has_absent = g.edges.values().any(|e| matches!(e.state, EdgeState::Absent));
        assert!(!has_absent);

        //optional: violations count == 0
        assert_eq!(g.count_violations(), 0);
    }
}


