# SpecScript | Executable Architecture Contracts

SpecScript is a declarative DSL and analysis engine for enforcing
software architecture rules directly against your codebase. Inspired
by Incremental Reflexion Analysis (Prof. Rainer Koschke), SpecScript turns architecture
into an executable artifact with deterministic, explainable conformance
results.

## Why SpecScript?

• Keep architecture and code in sync  
• Detect forbidden dependencies  
• Define components, layers, and contracts in a clear DSL  
• Prevent drift in microservice and monorepo environments  
• Map source code to architecture explicitly or using profiles  
• Integrates with CI/CD to enforce rules continuously  

## Example

```specscript
system "ExampleSystem" {
      repo_root "./"
      languages ["typescript", "java"]


    // =======================
    // 1) Architecture model
    // =======================
    architecture {

      // High-level components (logical architecture)
      component UI
      component Application
      component Domain
      component Infrastructure

      // Optional hierarchy
      component Application {
        component UseCases
        component Services
      }

      component Infrastructure {
        component Persistence
        component Messaging
      }

      // Dependency kinds the engine understands
      allowed_kinds [depends_on, calls]
    }


    // ====================================
    // 2) Architectural rules (the contract)
    // ====================================
    rules {

      // Layering rules
      allow UI -> Application
      allow Application -> Domain
      allow Application -> Infrastructure

      // Forbidden dependencies
      forbid UI -> Domain
      forbid UI -> Infrastructure
      forbid Domain -> Infrastructure

      // No upward dependencies
      forbid Domain -> Application
      forbid Domain -> UI
    }


    // ====================================
    // 3) Mapping rules (impl → arch)
    // ====================================
    mapping {

      // ---------- Manual pins (highest priority)
      manual {
        "src/main/ui/LoginPage.tsx"      -> UI
        "src/main/app/AuthService.ts"    -> Application.Services
      }

      // ---------- Path-based rules
      rule path_prefix {
        match "src/main/ui/"
        maps_to UI
        priority 100
      }

      rule path_prefix {
        match "src/main/app/"
        maps_to Application
        priority 90
      }

      rule path_prefix {
        match "src/main/domain/"
        maps_to Domain
        priority 80
      }

      rule path_prefix {
        match "src/main/infra/"
        maps_to Infrastructure
        priority 70
      }

      // ---------- Regex-based rules
      rule regex {
        match ".*Controller.*"
        maps_to UI
        priority 60
      }

      rule regex {
        match ".*Service.*"
        maps_to Application.Services
        priority 50
      }

      rule regex {
        match ".*Repository.*"
        maps_to Infrastructure.Persistence
        priority 40
      }

      // ---------- Fallback
      unmatched {
        mark unmapped
        report true
      }
    }


    // ====================================
    // 4) Enforcement configuration
    // ====================================
    enforcement {

      // How strict the check is
      on_unmapped_impl_nodes warn
      on_forbidden_dependency error

      // Fail CI if violations exist
      fail_on_error true
    }

}
```
## Full Flow 
```
dsl/
   └── **Input:** user writes architecture spec (components, layers, allowed deps) and mapping rules
       **Output:** compiled typed artifacts (spec.toon, mapping_rules.toon)

extractors/ (TS/Rust/Java)
   └── produce facts.toon  (impl nodes + impl edges + metadata)

formats/ + io/
   └── load facts.toon, spec.toon, mapping_rules.toon, manual_mappings.toon

mapping/
   └── generate maps_to + unmapped + mapping_trace

core/
   └── run reflexion:
        init_states()
        clear_propagated_edges()
        propagate()
        lift()
        classify()
        incremental recompute skeleton()

report/
   └── pretty report + JSON + DOT graph

cli/
   └── specscript check --spec spec.toon --facts facts.toon
       (and optional --mapping-rules / --manual-mapping / --dot / --json)

```
## End-to-End SpecScript + Engine Pipeline

The full SpecScript + Reflexion Engine pipeline works as follows:

An architect writes a `.specscript` specification describing the intended architecture (layers, services, datastores, allowed and forbidden dependencies, styles, mapping rules, and data-access policies). The SpecScript parser converts this into a `SystemSpec` AST, and the SpecScript compiler turns that AST into engine-ready JSON IR consisting of `ArchitectureModel`, `MappingRules`, and `ContractSet`. 

In parallel, language-specific extractors analyze the actual codebase and produce `ImplementationFacts` (implementation nodes and dependency edges). The normalizer merges these two inputs architecture IR and implementation facts into a `ReflexionGraph`, applying mapping rules to associate code elements with their architectural roles. 

The core Reflexion Engine then initializes edge states, propagates implementation edges into the architectural space, lifts them against declared architecture edges, and classifies every edge as Convergent, Divergent, Allowed, Absent, AllowedAbsent, ImplicitlyAllowed, or Unmapped, taking into account SpecScript contracts such as forbidden, optional, must-exist edges, and style rules. 

Finally, the CLI or API outputs structured conformance reports and diagnostics in JSON or terminal form for CI pipelines, editors, SEE integration, and architecture dashboards, enabling continuous structural validation and incremental drift detection across the entire system.
