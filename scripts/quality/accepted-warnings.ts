import type { AcceptedWarningConfig } from "../tools/quality-core/src/model/schema.ts";

export const ACCEPTED_WARNINGS = Object.freeze(
  [
    {
      ruleId: "scc-file-code-lines",
      sourceTool: "scc",
      path: "crates/shared/navigation/src/parameters/config.rs",
      codeArea: "rust-production",
      metric: "code-lines",
      value: 301,
      reason:
        "This file is the cohesive owner for ordered navigation config validation; splitting at the current one-line threshold excess would obscure error precedence."
    },
    {
      ruleId: "scc-file-code-lines",
      sourceTool: "scc",
      path: "crates/shared/protocol/src/contract_validation/response/results.rs",
      codeArea: "rust-production",
      metric: "code-lines",
      value: 301,
      reason:
        "This file is the cohesive owner for protocol success-result shape validation; revisit the split when an independent validation boundary emerges."
    },
    {
      ruleId: "lizard-cyclomatic-complexity",
      sourceTool: "lizard",
      path: "crates/shared/typed-fields/src/field.rs",
      codeArea: "rust-production",
      metric: "cyclomatic-complexity",
      value: 12,
      reason:
        "The function is an ordered linear validation pipeline; Lizard counts Rust error-propagation operators and the process-validation loop as branches."
    }
  ] satisfies AcceptedWarningConfig[]
);
