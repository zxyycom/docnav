import type { AcceptedWarningConfig } from "../tools/quality-core/src/model/schema.ts";

export const ACCEPTED_WARNINGS = Object.freeze(
  [
    {
      ruleId: "lizard-cyclomatic-complexity",
      sourceTool: "lizard",
      path: "crates/shared/typed-fields/src/field.rs",
      codeArea: "rust-production",
      metric: "cyclomatic-complexity",
      value: 12,
      messageIncludes: ["Function \"into_definition\""],
      reason:
        "The function is an ordered linear validation pipeline; Lizard counts Rust error-propagation operators and the process-validation loop as branches."
    },
    {
      ruleId: "lizard-function-code-density",
      sourceTool: "lizard",
      path: "scripts/test-evidence/ast-scan.ts",
      codeArea: "typescript-production-scripts",
      metric: "function-code-density",
      value: 63,
      messageIncludes: ["Function \"scanAstRule\""],
      reason:
        "This is one ast-grep diagnostic boundary: process execution and match parsing are already separate, while invocation, exit-status, and malformed-output failures remain ordered together."
    },
    {
      ruleId: "lizard-function-code-density",
      sourceTool: "lizard",
      path: "scripts/test-evidence/catalog/markdown.ts",
      codeArea: "typescript-production-scripts",
      metric: "function-code-density",
      value: 74,
      messageIncludes: ["Function \"parseCaseBlock\""],
      reason:
        "This is one ordered Case-block grammar: Owner, Entities, and Proves share a recovery cursor and partial-result diagnostic context, so extraction would add parser-state plumbing."
    },
    {
      ruleId: "lizard-cyclomatic-complexity",
      sourceTool: "lizard",
      path: "scripts/test-evidence/catalog/owner-ref.ts",
      codeArea: "typescript-production-scripts",
      metric: "cyclomatic-complexity",
      value: 14,
      messageIncludes: ["Function \"markdownHeadingAnchors\""],
      reason:
        "This is a single-pass heading lexer with local fence and duplicate-anchor state; stateless frontmatter and fence primitives are already separate."
    },
    {
      ruleId: "lizard-function-code-density",
      sourceTool: "lizard",
      path: "scripts/tools/release-package/workflow.test.ts",
      codeArea: "typescript-validation-smoke",
      metric: "function-code-density",
      value: 152,
      messageIncludes: [
        "Function \"(anonymous)\"",
        "workflow.test.ts:10"
      ],
      reason:
        "Lizard attributes four separately named node:test callbacks to one anonymous function at line 10; the source is already split by observable workflow behavior."
    }
  ] satisfies AcceptedWarningConfig[]
);
