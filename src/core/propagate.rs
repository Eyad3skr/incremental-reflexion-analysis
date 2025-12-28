use std::collections::HashSet;

use crate::core::graph::{GraphError, ReflexionGraph, Edge};
use crate::core::state::EdgeState;
use crate::core::types::{EdgeId, NodeId, SubgraphKind, EdgeKind};

impl ReflexionGraph {
    /// Reuse an existing propagated edge if present, otherwise create it.
    ///
    /// Propagated edges live in architecture-space adjacency (`arch_out`) but are marked
    /// with `subgraph = Propagated`.
    pub fn get_or_create_propagated_edge(
        &mut self,
        from_arch: NodeId,
        to_arch: NodeId,
        kind: EdgeKind,
    ) -> Result<EdgeId, GraphError> {
        // Reuse if exists: scan outgoing edges in arch_out[from_arch]
        if let Some(out) = self.arch_out.get(&from_arch) {
            for &eid in out {
                let e = self
                    .edges
                    .get(&eid)
                    .ok_or(GraphError::EdgeNotFound(eid))?;

                if e.subgraph == SubgraphKind::Propagated && e.to == to_arch && e.kind == kind {
                    return Ok(eid);
                }
            }
        }

        // Else create new propagated edge
        let new_edge = Edge {
            id: 0, // overwritten by add_edge
            from: from_arch,
            to: to_arch,
            kind,
            subgraph: SubgraphKind::Propagated,
            state: EdgeState::Undefined,
            counter: 0,
        };

        self.add_edge(new_edge)
    }

    /// Propagate a single implementation edge into architecture space.
    ///
    /// Steps:
    /// 1) Read (from_impl, to_impl, kind)
    /// 2) Map endpoints using `maps_to`
    ///    - if either endpoint unmapped => mark impl edge Unmapped and return
    /// 3) Create/reuse propagated edge (from_arch -> to_arch, same kind)
    /// 4) Increment propagated edge counter
    /// 5) Record: propagation_table[prop_edge] includes impl_edge
    pub fn propagate_impl_edge(&mut self, impl_edge_id: EdgeId) -> Result<(), GraphError> {
        // Read impl edge info (copy what we need to avoid borrow conflicts)
        let (from_impl, to_impl, kind) = {
            let e = self
                .edges
                .get(&impl_edge_id)
                .ok_or(GraphError::EdgeNotFound(impl_edge_id))?;

            if e.subgraph != SubgraphKind::Implementation {
                return Err(GraphError::WrongSubgraph {
                    node: e.from, // you don’t have EdgeId in the error, so this is the closest anchor
                    expected: SubgraphKind::Implementation,
                    found: e.subgraph,
                });
            }

            (e.from, e.to, e.kind.clone())
        };

        // Map endpoints: impl -> arch
        let from_arch = match self.maps_to.get(&from_impl).copied() {
            Some(x) => x,
            None => {
                if let Some(e) = self.edges.get_mut(&impl_edge_id) {
                    e.state = EdgeState::Unmapped;
                }
                return Ok(());
            }
        };

        let to_arch = match self.maps_to.get(&to_impl).copied() {
            Some(x) => x,
            None => {
                if let Some(e) = self.edges.get_mut(&impl_edge_id) {
                    e.state = EdgeState::Unmapped;
                }
                return Ok(());
            }
        };

        // Create/reuse propagated edge in architecture space
        let prop_id = self.get_or_create_propagated_edge(from_arch, to_arch, kind)?;

        // Increment propagated edge counter
        if let Some(pe) = self.edges.get_mut(&prop_id) {
            pe.counter += 1;
        }

        // Record the relationship: propagated edge <- impl edge(s)
        self.propagation_table
            .entry(prop_id)
            .or_insert_with(HashSet::new)
            .insert(impl_edge_id);

        Ok(())
    }
} 



/*
What does this unit test do:
    
    1. Creates two architecture nodes: UI, Service

    2. Creates two implementation nodes: LoginPage, UserService

    3. Inserts maps_to mappings:
        LoginPage -> UI
        UserService -> Service

    4. Adds an implementation edge: LoginPage calls UserService

    5. Calls propagate_impl_edge(impl_edge_id)

    6. Expects:
        A propagated edge exists in architecture space: UI calls Service
        Its counter == 1 (meaning 1 impl edge contributed to it)
        propagation_table[prop_edge_id] contains the original impl_edge_id

by passing these unit tests the code fully assures a working version of “propagation” contract: impl edge gets projected into arch space via maps_to, and we keep provenance.
*/

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
    fn propagate_creates_propagated_edge_and_registers_table_and_counter() {
        let mut g = ReflexionGraph::new();

        // Architecture nodes
        let ui = g.add_node(Node::new("UI", SubgraphKind::Architecture, None)).unwrap();
        let service = g
            .add_node(Node::new("Service", SubgraphKind::Architecture, None))
            .unwrap();

        // Implementation nodes
        let login_page = g
            .add_node(Node::new("LoginPage", SubgraphKind::Implementation, None))
            .unwrap();
        let user_service = g
            .add_node(Node::new("UserService", SubgraphKind::Implementation, None))
            .unwrap();

        // Mapping: impl -> arch
        g.set_mapping(login_page, ui).unwrap();
        g.set_mapping(user_service, service).unwrap();

        // Implementation edge: LoginPage -> UserService
        let impl_edge_id = g
            .add_edge(mk_edge(
                login_page,
                user_service,
                SubgraphKind::Implementation,
                EdgeKind::calls(),
            ))
            .unwrap();

        // Act
        g.propagate_impl_edge(impl_edge_id).unwrap();

        // Assert: there exists a propagated edge UI -> Service (kind=calls) with counter=1
        let mut found_prop: Option<u32> = None;
        for (eid, e) in g.edges.iter() {
            if e.subgraph == SubgraphKind::Propagated
            && e.from == ui
            && e.to == service
            && e.kind == EdgeKind::calls()
            {
                found_prop = Some(*eid);
                assert_eq!(e.counter, 1, "propagated edge counter must be incremented");
            }
        }
        let prop_id = found_prop.expect("expected propagated edge UI -> Service to exist");

        // propagation_table links the impl edge correctly
        let set = g
            .propagation_table
            .get(&prop_id)
            .expect("propagation table must have entry for propagated edge");
        assert!(
            set.contains(&impl_edge_id),
            "propagation table must contain the implementation edge id"
        );

        // Adjacency also updated (propagated edges live in arch_out)
        let out = g.arch_out.get(&ui).expect("arch_out[UI] must exist");
        assert!(
            out.contains(&prop_id),
            "arch_out[UI] must contain propagated edge id"
        );
    }

    #[test]
    fn propagate_reuses_same_propagated_edge_increments_counter_and_unions_sources() {
        let mut g = ReflexionGraph::new();

        // Architecture nodes
        let ui = g.add_node(Node::new("UI", SubgraphKind::Architecture, None)).unwrap();
        let service = g
            .add_node(Node::new("Service", SubgraphKind::Architecture, None))
            .unwrap();

        // Implementation nodes
        let a = g.add_node(Node::new("A", SubgraphKind::Implementation, None)).unwrap();
        let b = g.add_node(Node::new("B", SubgraphKind::Implementation, None)).unwrap();
        let c = g.add_node(Node::new("C", SubgraphKind::Implementation, None)).unwrap();

        // Map A,B,C -> UI/Service such that both impl edges map to UI->Service
        g.set_mapping(a, ui).unwrap();
        g.set_mapping(b, service).unwrap();
        g.set_mapping(c, service).unwrap();

        let e1 = g
            .add_edge(mk_edge(a, b, SubgraphKind::Implementation, EdgeKind::calls()))
            .unwrap();
        let e2 = g
            .add_edge(mk_edge(a, c, SubgraphKind::Implementation, EdgeKind::calls()))
            .unwrap();

        g.propagate_impl_edge(e1).unwrap();
        g.propagate_impl_edge(e2).unwrap();

        // Exactly ONE propagated edge for UI->Service (calls)
        let props: Vec<(u32, i32)> = g
            .edges
            .iter()
            .filter_map(|(eid, e)| {
                if e.subgraph == SubgraphKind::Propagated
                && e.from == ui
                && e.to == service
                && e.kind == EdgeKind::calls()
                {
                    Some((*eid, e.counter))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(props.len(), 1, "must reuse the same propagated edge");
        let (prop_id, counter) = props[0];
        assert_eq!(counter, 2, "counter must equal number of propagated impl edges");

        // propagation_table should contain both impl edges
        let set = g.propagation_table.get(&prop_id).unwrap();
        assert!(set.contains(&e1));
        assert!(set.contains(&e2));
        assert_eq!(set.len(), 2, "should be a set of unique impl edge ids");
    }

    #[test]
    fn propagate_marks_impl_edge_unmapped_and_creates_nothing_if_mapping_missing() {
        let mut g = ReflexionGraph::new();

        // Architecture nodes
        let ui = g.add_node(Node::new("UI", SubgraphKind::Architecture, None)).unwrap();
        let service = g
            .add_node(Node::new("Service", SubgraphKind::Architecture, None))
            .unwrap();

        // Implementation nodes
        let login_page = g
            .add_node(Node::new("LoginPage", SubgraphKind::Implementation, None))
            .unwrap();
        let user_service = g
            .add_node(Node::new("UserService", SubgraphKind::Implementation, None))
            .unwrap();

        // Only map ONE endpoint
        g.set_mapping(login_page, ui).unwrap();
        // user_service NOT mapped

        let impl_edge_id = g
            .add_edge(mk_edge(
                login_page,
                user_service,
                SubgraphKind::Implementation,
                EdgeKind::calls(),
            ))
            .unwrap();

        g.propagate_impl_edge(impl_edge_id).unwrap();

        // impl edge must be marked Unmapped
        let e = g.edges.get(&impl_edge_id).unwrap();
        assert_eq!(e.state, EdgeState::Unmapped);

        // No propagated edge should exist
        let any_prop = g.edges.values().any(|edge| edge.subgraph == SubgraphKind::Propagated);
        assert!(!any_prop, "should not create propagated edges when mapping is missing");

        // propagation table should remain empty
        assert!(g.propagation_table.is_empty());
    }

    #[test]
    fn propagate_errors_if_edge_is_not_implementation() {
        let mut g = ReflexionGraph::new();

        // Architecture nodes for an ARCH edge
        let a1 = g.add_node(Node::new("A1", SubgraphKind::Architecture, None)).unwrap();
        let a2 = g.add_node(Node::new("A2", SubgraphKind::Architecture, None)).unwrap();

        let arch_edge_id = g
            .add_edge(mk_edge(a1, a2, SubgraphKind::Architecture, EdgeKind::depends_on()))
            .unwrap();

        let err = g.propagate_impl_edge(arch_edge_id).unwrap_err();

        match err {
            GraphError::WrongSubgraph { expected, found, .. } => {
                assert_eq!(expected, SubgraphKind::Implementation);
                assert_eq!(found, SubgraphKind::Architecture);
            }
            _ => panic!("expected WrongSubgraph, got {:?}", err),
        }
    }

    #[test]
    fn get_or_create_propagated_edge_does_not_reuse_architecture_edge() {
        let mut g = ReflexionGraph::new();

        // Two arch nodes
        let ui = g.add_node(Node::new("UI", SubgraphKind::Architecture, None)).unwrap();
        let service = g
            .add_node(Node::new("Service", SubgraphKind::Architecture, None))
            .unwrap();

        // Add an ARCHITECTURE edge with same endpoints+kind
        let arch_eid = g
            .add_edge(mk_edge(
                ui,
                service,
                SubgraphKind::Architecture,
                EdgeKind::calls(),
            ))
            .unwrap();

        // Ask for propagated edge of same endpoints+kind
        let prop_eid = g
            .get_or_create_propagated_edge(ui, service, EdgeKind::calls())
            .unwrap();

        assert_ne!(
            arch_eid, prop_eid,
            "must not reuse architecture edge as propagated edge"
        );

        let prop = g.edges.get(&prop_eid).unwrap();
        assert_eq!(prop.subgraph, SubgraphKind::Propagated);
        assert_eq!(prop.state, EdgeState::Undefined);
        assert_eq!(prop.counter, 0);
    }
}

