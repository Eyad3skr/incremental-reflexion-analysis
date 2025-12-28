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
    use crate::core::types::{EdgeKind, NodeId, SubgraphKind};

    fn mk_node(name: &str, subgraph: SubgraphKind, parent: Option<NodeId>) -> Node {
        Node::new(name.to_string(), subgraph, parent)
    }

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

    ///scenario 1: Missing implementation edge
    ///architecture has UI -> Service specified, but impl has no corresponding edge.
    ///expected: arch edge becomes Absent after run_from_scratch().
    #[test]
    fn mismatch_missing_impl_edge_marks_arch_absent() {
        let mut g = ReflexionGraph::new();

        let ui = g.add_node(mk_node("UI", SubgraphKind::Architecture, None)).unwrap();
        let service = g
            .add_node(mk_node("Service", SubgraphKind::Architecture, None))
            .unwrap();

        let e_arch = g
            .add_edge(mk_edge(ui, service, SubgraphKind::Architecture, EdgeKind::depends_on()))
            .unwrap();

        //no impl edges at all
        g.run_from_scratch();

        let arch_e = g.edges.get(&e_arch).unwrap();
        assert!(matches!(arch_e.state, EdgeState::Absent));
        assert_eq!(arch_e.counter, 0);
    }

    ///scenario 2: Divergent dependency
    ///architecture specifies only UI -> Service.
    ///implementation produces UI -> DB (mapped), which is not specified.
    ///expected: impl edge becomes Divergent after propagate_and_lift during run_from_scratch().
    #[test]
    fn mismatch_divergent_impl_edge_is_marked_divergent() {
        let mut g = ReflexionGraph::new();

        //arch nodes
        let ui = g.add_node(mk_node("UI", SubgraphKind::Architecture, None)).unwrap();
        let service = g
            .add_node(mk_node("Service", SubgraphKind::Architecture, None))
            .unwrap();
        let db = g.add_node(mk_node("DB", SubgraphKind::Architecture, None)).unwrap();

        //only specified edge: UI -> Service
        let _e_arch = g
            .add_edge(mk_edge(ui, service, SubgraphKind::Architecture, EdgeKind::depends_on()))
            .unwrap();

        //impl nodes
        let login = g
            .add_node(mk_node("LoginPage", SubgraphKind::Implementation, None))
            .unwrap();
        let db_impl = g
            .add_node(mk_node("DBClient", SubgraphKind::Implementation, None))
            .unwrap();

        //mapping: LoginPage -> UI, DBClient -> DB
        g.set_mapping_overwrite(login, ui);
        g.set_mapping_overwrite(db_impl, db);

        //impl edge: LoginPage -> DBClient (mapped to UI -> DB), which is NOT specified
        let e_impl = g
            .add_edge(mk_edge(
                login,
                db_impl,
                SubgraphKind::Implementation,
                EdgeKind::depends_on(),
            ))
            .unwrap();

        g.run_from_scratch();

        let impl_e = g.edges.get(&e_impl).unwrap();
        assert!(matches!(impl_e.state, EdgeState::Divergent));
    }

    ///scenario 3: Unmapped implementation edge
    ///impl edge endpoints have no maps_to entries.
    //expected: impl edge becomes Unmapped, and propagation does nothing.
    #[test]
    fn mismatch_unmapped_impl_edge_marks_unmapped() {
        let mut g = ReflexionGraph::new();

        // Impl nodes (no mappings)
        let a = g
            .add_node(mk_node("A_Impl", SubgraphKind::Implementation, None))
            .unwrap();
        let b = g
            .add_node(mk_node("B_Impl", SubgraphKind::Implementation, None))
            .unwrap();

        let e_impl = g
            .add_edge(mk_edge(
                a,
                b,
                SubgraphKind::Implementation,
                EdgeKind::depends_on(),
            ))
            .unwrap();

        g.run_from_scratch();

        let impl_e = g.edges.get(&e_impl).unwrap();
        assert!(matches!(impl_e.state, EdgeState::Unmapped));
    }
}


