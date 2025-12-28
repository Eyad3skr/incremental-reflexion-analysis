/* 
Inputs:

    facts.toon (impl nodes + metadata)

    spec.toon (arch nodes)

    mapping_rules.toon

    optional manual_overrides.toon

Outputs:

    maps_to: HashMap<ImplNodeId, ArchNodeId>

    unmapped_impl_nodes

    mapping_report (traceable explanation)

Responsibilities:

    Normalize identities (paths, modules, fqns)

    Apply rules deterministically

    Resolve conflicts

    Produce stable, reproducible results 
*/
