# Reflexion Engine (Rust) — Architecture, Phases & Project Plan

This document describes the **Rust Reflexion Engine** that will become the core structural conformance checker under **SpecScript** (Architecture-as-Code).

The engine is heavily inspired by Prof. Koschke's original Reflexion Model but modernized, made incremental, and designed for full integration with SpecScript.

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
    ardsl_loader.rs     # .ardsl compiler input

  ardsl/
    parser.rs           # lexer/parser for DSL
    model.rs            # DSL AST
    compiler.rs         # AST → JSON IR
    patterns.rs         # style/pattern rules
    policies.rs         # optional/forbidden/etc.

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

### Phase 0 — In-Memory Engine (Now)

**Goal:**
- Build the **ReflexionGraph** data model
- Implement **run_from_scratch**, propagation, and lifting
- Hardcode a simple architecture + implementation + mapping
- Pass a small end-to-end test

**Deliverables:**
- `model.rs`
- `engine.rs`
- `tests/basic.rs`

---

### Phase 1 — IR JSON Formats

Define simple JSON formats for:

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

Matching serde structs will be added in `io.rs`.

---

### Phase 2 — Normalizer (JSON → Graph)

Add a module:
```
src/normalizer.rs
```

That performs:
```
ArchitectureModel + ImplementationFacts + MappingTable
        ↓
   ReflexionGraph (ready to run)
```

This is core for CLI and ArDSL integration.

---

### Phase 3 — Hierarchy-Aware Lifting

Upgrade lifting:
- Walk parent components
- Support `AllowedAbsent`, `ImplicitlyAllowed`
- Handle optional edges vs mandatory edges

This completes a real reflexion engine.

---

### Phase 4 — Incremental API

Add:
```
add_impl_edge
remove_impl_edge
remap_impl_node
```

So editors, file watchers, or CI can use the engine dynamically.

---

### Phase 5 — Contract Attributes (ArDSL Integration)

Architecture edges/nodes gain attributes:
- `optional`
- `must_exist`
- `forbidden`
- `ghost`
- `severity` (warning/error)

Engine classifies violations accordingly.

---

### Phase 6 — Mapping Rules

Stop using `mapping.json`.  
ArDSL compiler emits `mapping_rules.json`.

You add:
```rust
apply_mapping_rules(&ImplementationFacts, &MappingRules) -> HashMap<impl, arch>
```

---

### Phase 7 — CLI & Tooling

Add new crate `reflexion-cli`:
```
ardsl-reflect --arch arch.json --impl impl.json --rules mapping_rules.json --out result.json
```

---

### Phase 8 — IDE & CI Integration

Expose the engine as:
- JSON report API
- Watch mode
- Language Server hooks
- GitHub Action for PR checks

---

## 3. Phase 0 Code Summary (Core Engine)

**Files:**
- `model.rs`
- `engine.rs`
- `tests/basic.rs`

**Contains:**
- `Node`, `Edge`, `ReflexionGraph`
- `run_from_scratch`
- `propagate_and_lift`
- `lift_exact`
- Propagation table
- Initial classification: `Convergent`, `Allowed`, `Divergent`, `Absent`

This is the minimum skeleton needed before adding JSON, CLI, or mapping rules.

---

## 4. Future Fit With ArDSL

```
ArDSL → .ardsl
    ↓
ArDSL Compiler → arch.json + mapping_rules.json
    ↓
Normalizer → ReflexionGraph
    ↓
-> Reflexion Engine → classification result
    ↓
ArDSL Contract System → warnings/errors
    ↓
ArDSL Generator → code scaffold / regeneration
    ↓
IDE/CI integration
```

---

## 5. Summary

This engine becomes:
- The backbone of ArDSL's structural conformance
- Fully incremental
- Fully contract-aware
- Fast (Rust)
- Capable of replacing old mapping (`.gxl`, manual models) with modern mapping rules and architecture-as-code workflows
