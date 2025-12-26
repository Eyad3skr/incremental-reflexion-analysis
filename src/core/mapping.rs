// maps_to + rule based mapping
use crate::core::types::NodeId;
use crate::core::graph::Node;
use crate::core::graph::ReflexionGraph;
use crate::core::graph::GraphError;
use crate::core::types::SubgraphKind;

impl ReflexionGraph {
    //store/overwrite a mapping from implementation node to architecture node
    //1. impl_node must exist and must be in Implementation subgraph.
    //2. arch_node must exist and must be in Architecture subgraph.
    //3. Each impl node maps to at most one arch node (easy).
    //4. Many impl nodes may map to the same arch node (normal).
    //5. (Optional but recommended) No overwrites unless explicit (prevents silent bugs).
    //6. Provide helpers that are O(1) average, and let higher layers validate “coverage”.


    //validation helpers
    fn expect_impl_node(&self, impl_node: NodeId) -> Result<(), GraphError> {
        let sg = self.node_subgraph(impl_node)?;
        if sg != SubgraphKind::Implementation {
            return Err(GraphError::WrongSubgraph { node: impl_node, expected: SubgraphKind::Implementation, found: sg, });
        }
        Ok(())
    }

    fn expect_arch_node(&self, arch_node: NodeId) -> Result<(), GraphError> {
        let sg = self.node_subgraph(arch_node)?;
        if sg != SubgraphKind::Architecture {
            return Err(GraphError::WrongSubgraph { node: arch_node, expected: SubgraphKind::Architecture, found: sg, });
        }
        Ok(())
    }


    pub fn set_mapping(&mut self, impl_node: NodeId, arch_node: NodeId) -> Result<(), GraphError> {
        self.expect_impl_node(impl_node)?;        
        self.expect_arch_node(arch_node)?;

        match self.maps_to.get(&impl_node).copied() {
            None => {
                self.maps_to.insert(impl_node, arch_node);
                Ok(())            
            }
            Some(old_arch) if old_arch == arch_node => Ok(()), //Idempotent if mapping is identical, no overwrites
            Some(old_arch) => Err(GraphError::MappingAlreadyExists { impl_node, old_arch, new_arch: arch_node, }),
        }
    }

    //returns Ok(Some(arch)) if mapped, Ok(None) if not mapped. errors only if impl_node doesn't
    //exist or wrong subgraph.
    pub fn get_arch_node(&self, impl_node: NodeId) -> Result<Option<NodeId>, GraphError> {
        self.expect_impl_node(impl_node)?;
        Ok(self.maps_to.get(&impl_node).copied())
    }

    pub fn is_mapped(&self, impl_node: NodeId) -> Result<bool, GraphError> {
        self.expect_impl_node(impl_node)?;
        Ok(self.maps_to.contains_key(&impl_node))
    }

    pub fn remove_mapping(&mut self, impl_node: NodeId) -> Result<Option<NodeId>, GraphError> {
        self.expect_impl_node(impl_node)?;
        Ok(self.maps_to.remove(&impl_node))
    }

    pub fn clear_mappings(&mut self) {
        self.maps_to.clear()
    }

    pub fn mapping_len(&self) -> usize {
        self.maps_to.len()
    }

    //for reports
    pub fn iter_mapping(&self) -> impl Iterator<Item=(NodeId, NodeId)> + '_ {
        self.maps_to.iter().map(|(&i, &a)| (i, a))
    }

    fn validate_impl_node(&self, impl_node: NodeId) -> Result<(), GraphError> {
        let found = self.node_subgraph(impl_node)?;

        if found != SubgraphKind::Implementation {
            return Err(GraphError::WrongSubgraph {
                node: impl_node,
                expected: SubgraphKind::Implementation,
                found,
            });
        }

        Ok(())
    }

    fn validate_arch_node(&self, arch_node: NodeId) -> Result<(), GraphError> {
        let found = self.node_subgraph(arch_node)?;

        if found != SubgraphKind::Architecture {
            return Err(GraphError::WrongSubgraph {
                node: arch_node,
                expected: SubgraphKind::Architecture,
                found,
            });
        }

        Ok(())
    }


    pub fn set_mapping_overwrite(
        &mut self,
        impl_node: NodeId,
        arch_node: NodeId,
    ) -> Result<Option<NodeId>, GraphError> {
        self.validate_impl_node(impl_node)?;
        self.validate_arch_node(arch_node)?;

        Ok(self.maps_to.insert(impl_node, arch_node))
    }

    pub fn validate_all_mappings(&self) -> Result<(), GraphError> {
        for (&impl_node, &arch_node) in self.maps_to.iter() {
            self.validate_impl_node(impl_node)?;
            self.validate_arch_node(arch_node)?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::Node;
    use crate::core::types::SubgraphKind;

    #[test]
    fn mapping_insert_and_lookup_and_unmapped_none() {
        let mut g = ReflexionGraph::new();

        // create nodes via add_node(Node)
        let impl1 = g
            .add_node(Node::new("impl1", SubgraphKind::Implementation, None))
            .unwrap();
        let impl2 = g
            .add_node(Node::new("impl2", SubgraphKind::Implementation, None))
            .unwrap();
        let impl3 = g
            .add_node(Node::new("impl3", SubgraphKind::Implementation, None))
            .unwrap();

        let arch1 = g
            .add_node(Node::new("arch1", SubgraphKind::Architecture, None))
            .unwrap();
        let arch2 = g
            .add_node(Node::new("arch2", SubgraphKind::Architecture, None))
            .unwrap();

        // insert mappings
        g.set_mapping(impl1, arch1).expect("impl1 → arch1 should succeed");
        g.set_mapping(impl2, arch2).expect("impl2 → arch2 should succeed");

        // lookups
        assert_eq!(g.get_arch_node(impl1).unwrap(), Some(arch1));
        assert_eq!(g.get_arch_node(impl2).unwrap(), Some(arch2));
        assert_eq!(g.get_arch_node(impl3).unwrap(), None);

        // mapped?
        assert_eq!(g.is_mapped(impl1).unwrap(), true);
        assert_eq!(g.is_mapped(impl3).unwrap(), false);
    }

    #[test]
    fn mapping_rejects_overwrite() {
        let mut g = ReflexionGraph::new();

        let impl1 = g
            .add_node(Node::new("impl1", SubgraphKind::Implementation, None))
            .unwrap();
        let arch1 = g
            .add_node(Node::new("arch1", SubgraphKind::Architecture, None))
            .unwrap();
        let arch2 = g
            .add_node(Node::new("arch2", SubgraphKind::Architecture, None))
            .unwrap();

        g.set_mapping(impl1, arch1).unwrap();

        let err = g.set_mapping(impl1, arch2).unwrap_err();

        match err {
            GraphError::MappingAlreadyExists {
                impl_node,
                old_arch,
                new_arch,
            } => {
                assert_eq!(impl_node, impl1);
                assert_eq!(old_arch, arch1);
                assert_eq!(new_arch, arch2);
            }
            other => panic!("unexpected error: {}", other),
        }
    }
}
