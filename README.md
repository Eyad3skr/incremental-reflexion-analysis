# Reflexion Engine (Rust) - Architecture, Phases & Project Plan

### *Core Structural Conformance Engine for SpecScript (Architecture-as-Code)*

This document describes the **Rust Reflexion Engine**, the backend structural conformance checker that powers **SpecScript**, the Architecture-as-Code language.

The engine is inspired by Prof. Rainer Koschke's *Incremental Reflexion Analysis* but modernized and designed for full automation, Contract-based architecture checking, IDE/CI integration, and SpecScript compilation support.

---

## 1. Repository Structure

```
src/
  core/
    graph.rs            # nodes, edges, IR
    classify.rs         # classification logic
    delta.rs            # incremental diffs
    lifting.rs          # lifting/hierarchy logic
    propagate.rs        # propagation logic
    mapping.rs          # maps_to + rule-based mapping
    state.rs            # Convergent, Divergent, etc.
    types.rs            # enums + shared types

  io/
    json_loader.rs
    json_writer.rs
    normalize.rs        # JSON → ReflexionGraph
    specscript_loader.rs  # SpecScript compiler input

  specscript/
    parser.rs           # lexer/parser for the SpecScript DSL
    model.rs            # DSL AST (SystemSpec)
    compiler.rs         # AST → JSON IR (Architecture + MappingRules + Contracts)
    patterns.rs         # style/pattern rules → dependency constraints
    policies.rs         # optional/must_exist/forbidden/severity constraints

  cli/
    main.rs
    commands.rs         # support multiple commands
    args.rs

tests/
  simple_layered.rs
  mismatch_examples.rs
  incremental.rs
```

---

## 2. Phase Breakdown

### Phase 0 - In-Memory Engine (Now)

**Goal:**
- Build the `ReflexionGraph`
- Implement propagation + lifting + classification
- Hardcode a simple architecture + implementation
- Pass a basic test end-to-end

**Deliverables:**
- `graph.rs`, `propagate.rs`, `lifting.rs`, `classify.rs`
- `tests/basic.rs`

---

### Phase 1 - IR JSON Formats

Define JSON for the engine:

#### Architecture Model

```json
{
  "nodes": [{ "id": 1, "name": "UI", "parent": null }],
  "edges": [{ "id": 100, "from": 1, "to": 2, "kind": "Calls" }]
}
```

#### Implementation Facts

```json
{
  "nodes": [{ "id": 10, "name": "LoginPage", "path": "src/ui/login.rs" }],
  "edges": [{ "id": 200, "from": 10, "to": 11, "kind": "Calls" }]
}
```

#### Mapping Table

```json
{
  "maps_to": [
    { "impl": 10, "arch": 1 },
    { "impl": 11, "arch": 2 }
  ]
}
```

Serde structs will be defined in `io/json_loader.rs`.

---

### Phase 2 - Normalizer (JSON → ReflexionGraph)

Implement:

```
ArchitectureModel + ImplementationFacts + MappingRules
        ↓
    ReflexionGraph
```

This prepares the internal graph representation for the core engine.

---

### Phase 3 - Hierarchy-Aware Lifting

- Walk parent/ancestor nodes during lifting
- Add `AllowedAbsent`, `ImplicitlyAllowed`
- Handle optional & must_exist edges
- Apply pattern/policy constraints

---

### Phase 4 - Incremental API

Expose:

```rust
add_impl_edge()
remove_impl_edge()
remap_impl_node()
```

So editors (LSP), CI watchers, and SEE can update results in real time.

---

### Phase 5 - Contract Attributes (SpecScript Integration)

Architecture edges and nodes can declare:
- `optional`
- `must_exist`
- `forbidden`
- `ghost`
- `severity` (warning/error)

The engine classifies violations using these constraints.

---

### Phase 6 - Mapping Rules (from SpecScript)

SpecScript compiler emits:

```
mapping_rules.json
```

Core engine adds:

```rust
apply_mapping_rules(&ImplementationFacts, &MappingRules)
    -> HashMap<impl_node, arch_node>
```

Stops relying on `mapping.json`.

---

### Phase 7 - CLI Tooling

Binary name example:

```bash
specscript-reflect --spec system.specscript --impl impl.json --out result.json
```

CLI performs:
1. Parse `.specscript`
2. Compile to IR
3. Load implementation facts
4. Normalize to ReflexionGraph
5. Run reflexion
6. Print violations or export JSON

---

### Phase 8 - IDE & CI Integration

Expose:
- JSON reports
- Watch mode
- LSP hooks for editors
- GitHub Action integration
- (optional) SEE integration (visual frontend)

This enables continuous conformance checking with zero manual effort.

---

## 3. Phase 0 Code Summary

### The minimal working engine includes:

- Node/Edge definitions
- ReflexionGraph struct
- `run_from_scratch()`
- Propagation: impl edge → propagated arch edge
- Lifting: propagated → specified architecture edge
- Final classification:
  - Convergent
  - Allowed
  - Divergent
  - Absent

This forms the foundation before JSON, CLI, or SpecScript support.

---

## 4. End-to-End SpecScript + Engine Pipeline

The full SpecScript + Reflexion Engine pipeline works as follows:

An architect writes a `.specscript` specification describing the intended architecture (layers, services, datastores, allowed and forbidden dependencies, styles, mapping rules, and data-access policies). The SpecScript parser converts this into a `SystemSpec` AST, and the SpecScript compiler turns that AST into engine-ready JSON IR consisting of `ArchitectureModel`, `MappingRules`, and `ContractSet`. 

In parallel, language-specific extractors analyze the actual codebase and produce `ImplementationFacts` (implementation nodes and dependency edges). The normalizer merges these two inputs—architecture IR and implementation facts into a `ReflexionGraph`, applying mapping rules to associate code elements with their architectural roles. 

The core Reflexion Engine then initializes edge states, propagates implementation edges into the architectural space, lifts them against declared architecture edges, and classifies every edge as Convergent, Divergent, Allowed, Absent, AllowedAbsent, ImplicitlyAllowed, or Unmapped, taking into account SpecScript contracts such as forbidden, optional, must-exist edges, and style rules. 

Finally, the CLI or API outputs structured conformance reports and diagnostics in JSON or terminal form for CI pipelines, editors, SEE integration, and architecture dashboards, enabling continuous structural validation and incremental drift detection across the entire system.
