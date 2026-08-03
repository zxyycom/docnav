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
    },
    {
      ruleId: "scc-file-code-lines",
      sourceTool: "scc",
      path: "crates/adapters/json/src/adapter/tests.rs",
      codeArea: "rust-tests",
      metric: "code-lines",
      value: 771,
      messageIncludes: [
        "File \"crates/adapters/json/src/adapter/tests.rs\"",
        "771 code lines"
      ],
      reason:
        "This is the JSON adapter contract suite: manifest hints, selected-operation parsing, every operation, diagnostic projection, and full-read hooks share the same TempDocument and operation-result helpers; splitting it would duplicate test-only fixtures without creating a separate behavior owner."
    },
    {
      ruleId: "scc-file-code-lines",
      sourceTool: "scc",
      path: "crates/adapters/json/src/document.rs",
      codeArea: "rust-production",
      metric: "code-lines",
      value: 483,
      messageIncludes: [
        "File \"crates/adapters/json/src/document.rs\"",
        "483 code lines"
      ],
      reason:
        "This is the adapter-private JSON document boundary: the source cursor, serde visitor, duplicate-key and depth enforcement, raw-number capture, and source regions evolve against one tree representation; splitting it would expose coordination rather than separate responsibilities."
    },
    {
      ruleId: "scc-file-code-lines",
      sourceTool: "scc",
      path: "crates/adapters/json/src/find.rs",
      codeArea: "rust-production",
      metric: "code-lines",
      value: 341,
      messageIncludes: [
        "File \"crates/adapters/json/src/find.rs\"",
        "341 code lines"
      ],
      reason:
        "This file owns one adapter-private find pipeline: source occurrence iteration, deepest-region ref attribution, line location, and bounded Unicode/whitespace excerpt construction. Its small private state types enforce the label memory bound; splitting them would spread the same source-region and occurrence invariants across modules without another consumer."
    },
    {
      ruleId: "scc-file-code-lines",
      sourceTool: "scc",
      path: "test/tools/smoke-harness.ts",
      codeArea: "typescript-validation-smoke",
      metric: "code-lines",
      value: 310,
      messageIncludes: [
        "File \"test/tools/smoke-harness.ts\"",
        "310 code lines"
      ],
      reason:
        "This file is the shared smoke execution and audit boundary; the helper path reuses command preparation, process execution, assertion recording, and executable logging, so splitting it would fragment one stateful harness responsibility."
    },
    {
      ruleId: "lizard-function-code-density",
      sourceTool: "lizard",
      path: "crates/adapters/json/src/document.rs",
      codeArea: "rust-production",
      metric: "function-code-density",
      value: 54,
      messageIncludes: [
        "Function \"load\"",
        "document.rs:96",
        "54 code lines"
      ],
      reason:
        "load is one ordered loader transaction: it strips the BOM, validates UTF-8, drives the custom visitor, classifies parser-state failure, rejects trailing input, and finalizes root regions and metrics against the same source and BuildState."
    },
    {
      ruleId: "lizard-parameter-count",
      sourceTool: "lizard",
      path: "crates/adapters/json/src/paging.rs",
      codeArea: "rust-production",
      metric: "parameter-count",
      value: 7,
      messageIncludes: [
        "Function \"paginate_entry_slice\"",
        "paging.rs:119",
        "7 parameters"
      ],
      reason:
        "The function has five top-level parameters; Lizard counts two nested commas in its function-pointer types as extra parameters. Those callbacks are the minimal adaptation that lets outline and find entries share the same private pagination mechanics."
    },
    {
      ruleId: "lizard-parameter-count",
      sourceTool: "lizard",
      path: "crates/adapters/json/src/paging.rs",
      codeArea: "rust-production",
      metric: "parameter-count",
      value: 7,
      messageIncludes: [
        "Function \"entries_page\"",
        "paging.rs:147",
        "7 parameters"
      ],
      reason:
        "The function has five top-level parameters; Lizard counts two nested commas in its function-pointer types as extra parameters. The callbacks keep the page-building loop shared without introducing a public entry abstraction."
    },
    {
      ruleId: "lizard-parameter-count",
      sourceTool: "lizard",
      path: "crates/adapters/json/src/paging.rs",
      codeArea: "rust-production",
      metric: "parameter-count",
      value: 6,
      messageIncludes: [
        "Function \"fit_entry\"",
        "paging.rs:178",
        "6 parameters"
      ],
      reason:
        "The function has four top-level parameters; Lizard counts two nested commas in its function-pointer types as extra parameters. A one-use parameter object would only hide the outline/find field and label callbacks used by this private fit operation."
    },
    {
      ruleId: "scc-file-code-lines",
      sourceTool: "scc",
      path: "test/smoke/core/cases/real-json.ts",
      codeArea: "fixtures-examples",
      metric: "code-lines",
      value: 425,
      messageIncludes: [
        "File \"test/smoke/core/cases/real-json.ts\"",
        "425 code lines"
      ],
      reason:
        "This file is one real-CLI JSON scenario, preserving registry, automatic/explicit selection, outline/read, find, readable-view, and selected failure evidence around a shared SmokeProject; splitting it would scatter one executable roundtrip's audit trail."
    },
    {
      ruleId: "lizard-parameter-count",
      sourceTool: "lizard",
      path: "test/smoke/core/cases/real-json.ts",
      codeArea: "fixtures-examples",
      metric: "parameter-count",
      value: 6,
      messageIncludes: [
        "Function \"runProtocolFailure\"",
        "real-json.ts:425",
        "6 parameters"
      ],
      reason:
        "This thin smoke assertion helper receives six distinct call-site facts—label, arguments, project, operation, protocol code, and exit code—validates one command record, and returns it for detail checks; a one-use parameter object would add no domain boundary."
    },
    {
      ruleId: "lizard-function-code-density",
      sourceTool: "lizard",
      path: "crates/docnav/src/runtime.rs",
      codeArea: "rust-production",
      metric: "function-code-density",
      value: 57,
      messageIncludes: [
        "Function \"execute_document\"",
        "runtime.rs:35",
        "57 code lines at cyclomatic complexity 5"
      ],
      reason:
        "This is one ordered runtime transaction: prepare routing, normalize the path, build the parameter catalog, execute navigation, and record each stage's failure against shared logging and timing state. Splitting it only to cross the observation threshold would add one-use state plumbing without a separate behavior owner; remove this acceptance if a stage gains an independent owner or the warning disappears."
    },
    {
      ruleId: "lizard-cyclomatic-complexity",
      sourceTool: "lizard",
      path: "crates/shared/navigation/src/routing.rs",
      codeArea: "rust-production",
      metric: "cyclomatic-complexity",
      value: 11,
      messageIncludes: [
        "Function \"select_adapter\"",
        "routing.rs:124",
        "cyclomatic complexity 11"
      ],
      reason:
        "This is one adapter-selection decision: validate registry routing once, honor an explicit selection, otherwise infer the adapter from the pathname, then resolve its definition. Extracting either short branch would add a one-use wrapper without another responsibility owner; remove this acceptance if selection gains a separate responsibility or the warning disappears."
    }
  ] satisfies AcceptedWarningConfig[]
);
