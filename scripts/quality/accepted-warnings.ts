import type { AcceptedWarningConfig } from "../tools/quality-core/src/model/schema.ts";

export const ACCEPTED_WARNINGS = Object.freeze([
  {
    ruleId: "scc-file-code-lines",
    sourceTool: "scc",
    path: "subrepos/cli-config-resolution/crates/cli-config-resolution-clap/src/lib.rs",
    codeArea: "rust-production",
    metric: "code-lines",
    value: 414,
    reason:
      "This companion file currently co-locates clap command construction, candidate extraction, and conflict checks. The user explicitly deferred that split to a dedicated CLI parsing refactor; splitting it now would create duplicate churn. Remove this acceptance when that refactor starts or this file structure changes."
  },
  {
    ruleId: "lizard-function-code-density",
    sourceTool: "lizard",
    path: "subrepos/cli-config-resolution/crates/cli-config-resolution-clap/src/lib.rs",
    codeArea: "rust-production",
    metric: "function-code-density",
    value: 82,
    messageIncludes: ["Function \"candidate_from_matches\""],
    reason:
      "candidate_from_matches owns canonical value-kind and invalid-input candidate conversion in the clap companion. The user explicitly deferred its split to a dedicated CLI parsing refactor; splitting it now would create duplicate churn. Remove this acceptance when that refactor starts or this function structure changes."
  },
  {
    ruleId: "lizard-cyclomatic-complexity",
    sourceTool: "lizard",
    path: "subrepos/cli-config-resolution/crates/cli-config-resolution-clap/src/lib.rs",
    codeArea: "rust-production",
    metric: "cyclomatic-complexity",
    value: 16,
    messageIncludes: ["Function \"candidate_from_matches\""],
    reason:
      "candidate_from_matches branches across the canonical value kinds and their invalid-input outcomes in the clap companion. The user explicitly deferred this restructuring to a dedicated CLI parsing refactor; changing it now would create duplicate churn. Remove this acceptance when that refactor starts or this function structure changes."
  },
  {
    ruleId: "lizard-cyclomatic-complexity",
    sourceTool: "lizard",
    path: "subrepos/cli-config-resolution/crates/cli-config-resolution-clap/src/lib.rs",
    codeArea: "rust-production",
    metric: "cyclomatic-complexity",
    value: 16,
    messageIncludes: ["Function \"command_conflicts\""],
    reason:
      "command_conflicts checks short and long locator collisions against both generated and existing clap arguments. The user explicitly deferred this restructuring to a dedicated CLI parsing refactor; changing it now would create duplicate churn. Remove this acceptance when that refactor starts or this function structure changes."
  },
  {
    ruleId: "lizard-cyclomatic-complexity",
    sourceTool: "lizard",
    path: "subrepos/cli-config-resolution/crates/cli-config-resolution-clap/src/lib.rs",
    codeArea: "rust-production",
    metric: "cyclomatic-complexity",
    value: 12,
    messageIncludes: ["Function \"parse\""],
    reason:
      "parse coordinates clap command construction, argument parsing, match lookup, and candidate collection. The user explicitly deferred this restructuring to a dedicated CLI parsing refactor; changing it now would create duplicate churn. Remove this acceptance when that refactor starts or this function structure changes."
  },
  {
    ruleId: "scc-file-code-lines",
    sourceTool: "scc",
    path: "subrepos/cli-config-resolution/crates/cli-config-resolution-clap/src/tests.rs",
    codeArea: "rust-tests",
    metric: "code-lines",
    value: 319,
    reason:
      "The clap companion tests keep command registration, candidate conversion, and conflict behavior in one owner test module. The user explicitly deferred reorganizing this suite to a dedicated CLI parsing refactor; splitting it now would create duplicate churn. Remove this acceptance when that refactor starts or this test file structure changes."
  },
  {
    ruleId: "lizard-function-code-density",
    sourceTool: "lizard",
    path: "subrepos/cli-config-resolution/crates/cli-config-resolution-clap/examples/resolution_flow.rs",
    codeArea: "fixtures-examples",
    metric: "function-code-density",
    value: 73,
    messageIncludes: ["Function \"run\""],
    reason:
      "The run example keeps the complete clap-to-resolution flow together so the companion integration remains readable end to end. The user explicitly deferred restructuring it to a dedicated CLI parsing refactor; changing it now would create duplicate churn. Remove this acceptance when that refactor starts or this example function changes."
  },
  {
    ruleId: "lizard-function-code-density",
    sourceTool: "lizard",
    path: "subrepos/cli-config-resolution/crates/cli-config-resolution-clap/examples/resolution_flow.rs",
    codeArea: "fixtures-examples",
    metric: "function-code-density",
    value: 54,
    messageIncludes: ["Function \"resolve_flow\""],
    reason:
      "resolve_flow keeps candidate resolution, diagnostics, and example output projection in one demonstrative flow. The user explicitly deferred restructuring it to a dedicated CLI parsing refactor; changing it now would create duplicate churn. Remove this acceptance when that refactor starts or this example function changes."
  }
] satisfies AcceptedWarningConfig[]);
