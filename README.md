# SpecScript | Executable Architecture Contracts

SpecScript is a declarative DSL and analysis engine for enforcing
software architecture rules directly against your codebase. Inspired
by Reflexion Models (Murphy & Notkin), SpecScript turns architecture
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
system "ProLancer" {
  structure {
    layer Application {
      component TaskService
      component ProjectService
    }
    layer Infrastructure {
      component DBAdapter
    }
  }

  contracts {
    forbid Application -> Infrastructure
  }

  mapping {
    "src/app/**" -> Application
    "src/infra/**" -> Infrastructure
  }
}
```
## End-to-End SpecScript + Engine Pipeline

The full SpecScript + Reflexion Engine pipeline works as follows:

An architect writes a `.specscript` specification describing the intended architecture (layers, services, datastores, allowed and forbidden dependencies, styles, mapping rules, and data-access policies). The SpecScript parser converts this into a `SystemSpec` AST, and the SpecScript compiler turns that AST into engine-ready JSON IR consisting of `ArchitectureModel`, `MappingRules`, and `ContractSet`. 

In parallel, language-specific extractors analyze the actual codebase and produce `ImplementationFacts` (implementation nodes and dependency edges). The normalizer merges these two inputs architecture IR and implementation facts into a `ReflexionGraph`, applying mapping rules to associate code elements with their architectural roles. 

The core Reflexion Engine then initializes edge states, propagates implementation edges into the architectural space, lifts them against declared architecture edges, and classifies every edge as Convergent, Divergent, Allowed, Absent, AllowedAbsent, ImplicitlyAllowed, or Unmapped, taking into account SpecScript contracts such as forbidden, optional, must-exist edges, and style rules. 

Finally, the CLI or API outputs structured conformance reports and diagnostics in JSON or terminal form for CI pipelines, editors, SEE integration, and architecture dashboards, enabling continuous structural validation and incremental drift detection across the entire system.
