// lifting/hierarchy logic
//does this propagated relationship correspond to something the architecture explicitly allows?
/*

Specified: architecture edge exists

Convergent: specified ∧ implemented

Divergent: implemented ∧ not specified

Absent: specified ∧ not implemented (handled later)

*/
use std::collections::HashSet;
use std::ops::Sub;
use crate::core::graph::{GraphError, ReflexionGraph, Edge};
use crate::core::state::EdgeState;
use crate::core::types::{EdgeId, NodeId, SubgraphKind, EdgeKind};

impl ReflexionGraph {
    //find an architecture graph that exactly matches (from, to, kind). if found return EdgeId, else none
    
    pub fn lift_exact(&self, from_arch: NodeId, to_arch: NodeId, kind: &EdgeKind) -> Result<Option<EdgeId>, GraphError> {
        //search outgoing edges from this arch node (covers architecture and propagation)
        if let Some(out) = self.arch_out.get(&from_arch) {
            for &eid in out {
                let e = self.edges.get(&eid).ok_or(GraphError::EdgeNotFound(eid))?;
                if e.subgraph == SubgraphKind::Architecture && e.to == to_arch && &e.kind == kind {
                    return Ok(Some(eid));
                }
            }
        }
        Ok(None)
    }

    //propagate one impl edge -> then lift it against specified architecture edges 
    pub fn propagate_and_lift(&mut self, impl_edge_id: EdgeId) -> Result<(), GraphError> {
        //1) propagate (creates/reuses propagated edge + counter++ + propagation_table entry)
        self.propagate_impl_edge(impl_edge_id)?;

        {
            let ie = self.edges.get(&impl_edge_id).ok_or(GraphError::EdgeNotFound(impl_edge_id))?;
            if matches!(ie.state, EdgeState::Unmapped) {
                return Ok(());
            }
        }

        //2) find the propagated edge id corresponding to this impl edge 
        let prop_id = self.propagation_table.iter().find_map( |(&prop_eid, impls)| {
            if impls.contains(&impl_edge_id) {
                Some(prop_eid)
            } else {
                None 
            }
        }).ok_or(GraphError::EdgeNotFound(impl_edge_id))?; //"not found in table" -> treat as edge not found-ish

        //3) read propagated edge endpoints/kind 
        let (from_arch, to_arch, kind) = {
            let pe = self.edges.get(&prop_id).ok_or(GraphError::EdgeNotFound(prop_id))?;
            (pe.from, pe.to, pe.kind.clone())
        };

        //4) lift: match propagated edge to specified architecture edge 
        if let Some(arch_eid) = self.lift_exact(from_arch, to_arch, &kind)? {
            //architecture edge is convergent 
            if let Some(ae) = self.edges.get_mut(&arch_eid) {
                ae.counter += 1;
                ae.state = EdgeState::Convergent;
            }

            //propagated +impl are allowed
            if let Some(pe) = self.edges.get_mut(&prop_id) {
                pe.state = EdgeState::Allowed;
            }
            if let Some(ie) = self.edges.get_mut(&impl_edge_id) {
                ie.state = EdgeState::Allowed;
            }
        } else {
            //divergent
            if let Some(pe) = self.edges.get_mut(&prop_id) {
                pe.state = EdgeState::Divergent;
            }
            if let Some(ie) = self.edges.get_mut(&impl_edge_id) {
                ie.state = EdgeState::Divergent;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::{Edge, Node, ReflexionGraph};
    use crate::core::state::EdgeState;
    use crate::core::types::{EdgeKind, SubgraphKind};

    fn mk_node(name: &str, subgraph: SubgraphKind) -> Node {
        Node::new(name, subgraph, None)
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
    fn propagate_and_lift_marks_convergent_when_arch_edge_exists() {
        let mut g = ReflexionGraph::new();

        // Architecture nodes
        let ui = g.add_node(mk_node("UI", SubgraphKind::Architecture)).unwrap();
        let service = g
            .add_node(mk_node("Service", SubgraphKind::Architecture))
            .unwrap();

        // Implementation nodes
        let login_page = g
            .add_node(mk_node("LoginPage", SubgraphKind::Implementation))
            .unwrap();
        let user_service = g
            .add_node(mk_node("UserService", SubgraphKind::Implementation))
            .unwrap();

        // Specified arch edge UI -> Service
        let arch_edge_id = g
            .add_edge(mk_edge(ui, service, SubgraphKind::Architecture, EdgeKind::calls()))
            .unwrap();

        // Impl edge LoginPage -> UserService
        let impl_edge_id = g
            .add_edge(mk_edge(
                login_page,
                user_service,
                SubgraphKind::Implementation,
                EdgeKind::calls(),
            ))
            .unwrap();

        // Mapping impl -> arch
        g.set_mapping(login_page, ui).unwrap();
        g.set_mapping(user_service, service).unwrap();

        // Init states (arch edges become Specified)
        g.init_states();

        // Act
        g.propagate_and_lift(impl_edge_id).unwrap();

        // Assert arch edge convergent + counter incremented
        let ae = g.edges.get(&arch_edge_id).unwrap();
        assert!(matches!(ae.state, EdgeState::Convergent));
        assert_eq!(ae.counter, 1);

        // Assert impl edge allowed
        let ie = g.edges.get(&impl_edge_id).unwrap();
        assert!(matches!(ie.state, EdgeState::Allowed));

        // Assert propagated edge exists and is allowed
        let prop_id = g
            .propagation_table
            .iter()
            .find_map(|(&prop_eid, impls)| impls.contains(&impl_edge_id).then_some(prop_eid))
            .unwrap();

        let pe = g.edges.get(&prop_id).unwrap();
        assert_eq!(pe.from, ui);
        assert_eq!(pe.to, service);
        assert!(matches!(pe.state, EdgeState::Allowed));
        assert_eq!(pe.counter, 1);
    }
}
