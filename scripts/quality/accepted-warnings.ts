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
      value: 1201,
      messageIncludes: [
        "File \"crates/adapters/json/src/adapter/tests.rs\"",
        "1201 code lines"
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
      value: 827,
      messageIncludes: [
        "File \"crates/adapters/json/src/document.rs\"",
        "827 code lines"
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
      value: 483,
      messageIncludes: [
        "File \"crates/adapters/json/src/find.rs\"",
        "483 code lines"
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
      value: 78,
      messageIncludes: [
        "Function \"load\"",
        "document.rs:163",
        "78 code lines"
      ],
      reason:
        "Decision 9 keeps load as the single parser/model boundary: it strips one BOM, decodes UTF-8, invokes the offset-preserving scanner and current serde visitor, maps parser-state failure, rejects trailing input, and finalizes source regions and metrics against the same BuildState. Extracting stages would add one-use state plumbing without an independent owner."
    },
    {
      ruleId: "scc-file-code-lines",
      sourceTool: "scc",
      path: "test/smoke/core/cases/real-json.ts",
      codeArea: "fixtures-examples",
      metric: "code-lines",
      value: 605,
      messageIncludes: [
        "File \"test/smoke/core/cases/real-json.ts\"",
        "605 code lines"
      ],
      reason:
        "This is the shared real-CLI evidence boundary for CORE-JSON-NAV-001 and CORE-JSON-FAIL-001: registry, automatic/explicit selection, outline/read/find, readable output, and selected failure assertions share one SmokeProject audit trail. Splitting it would separate a single executable roundtrip without another test owner."
    },
    {
      ruleId: "lizard-function-code-density",
      sourceTool: "lizard",
      path: "test/smoke/core/cases/real-json.ts",
      codeArea: "fixtures-examples",
      metric: "function-code-density",
      value: 63,
      messageIncludes: [
        "Function \"assertAdapterRegistry\"",
        "real-json.ts:99",
        "63 code lines"
      ],
      reason:
        "This one-use CORE-JSON-NAV-001 descriptor contract inspects one adapter-list record in order: required adapter order, each core_static source, then the JSON format ID and exact ordered extensions, filenames, and content types. Extracting these facts would add one-use helpers or an opaque data loop, while another smoke matrix would duplicate the same registry evidence."
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
        "real-json.ts:611",
        "6 parameters"
      ],
      reason:
        "This thin smoke assertion helper receives six distinct call-site facts—label, arguments, project, operation, protocol code, and exit code—validates one command record, and returns it for detail checks; a one-use parameter object would add no domain boundary."
    },
    {
      ruleId: "scc-file-code-lines",
      sourceTool: "scc",
      path: "crates/adapters/json/src/document/tests.rs",
      codeArea: "rust-tests",
      metric: "code-lines",
      value: 471,
      messageIncludes: [
        "File \"crates/adapters/json/src/document/tests.rs\"",
        "471 code lines"
      ],
      reason:
        "This is the document-model evidence boundary named by the JSON adapter Case ledger: loader grammar, source/model preservation, deterministic attribution, bounds, duplicates, and depth assertions share its private helpers and fixtures. Splitting by assertion shape would fragment those one-model invariants without a separate behavior owner."
    },
    {
      ruleId: "lizard-function-code-density",
      sourceTool: "lizard",
      path: "crates/adapters/json/src/document.rs",
      codeArea: "rust-production",
      metric: "function-code-density",
      value: 136,
      messageIncludes: [
        "Function \"attribute_node\"",
        "document.rs:295",
        "136 code lines"
      ],
      reason:
        "Decision 8 assigns every comment exactly once during one deterministic attribution pass. attribute_node must coordinate source-order cursors, commas, direct bundles, tail bundles, and recursive child regions; extracting portions would only thread that mutable attribution state through one-use helpers and weaken the single-pass ownership boundary."
    },
    {
      ruleId: "lizard-cyclomatic-complexity",
      sourceTool: "lizard",
      path: "crates/adapters/json/src/document.rs",
      codeArea: "rust-production",
      metric: "cyclomatic-complexity",
      value: 37,
      messageIncludes: [
        "Function \"attribute_node\"",
        "document.rs:295",
        "cyclomatic complexity 37"
      ],
      reason:
        "Decision 8 assigns every comment exactly once during one deterministic attribution pass. attribute_node's object, array, empty-container, trailing-comma, and root-tail branches encode those distinct placement rules against the same cursors and bundles; splitting them would add one-use coordination without another responsibility owner."
    },
    {
      ruleId: "lizard-function-code-density",
      sourceTool: "lizard",
      path: "crates/adapters/json/src/jsonc.rs",
      codeArea: "rust-production",
      metric: "function-code-density",
      value: 132,
      messageIncludes: [
        "Function \"scan\"",
        "jsonc.rs:28",
        "132 code lines"
      ],
      reason:
        "Decision 9 requires one offset-preserving lexical pass that recognizes strings, comments, line boundaries, commas, and trailing commas while constructing the equal-length parse view. Extracting token cases would only thread scanner state and parse-view mutation through one-use helpers, not create an independent parser owner."
    },
    {
      ruleId: "lizard-cyclomatic-complexity",
      sourceTool: "lizard",
      path: "crates/adapters/json/src/jsonc.rs",
      codeArea: "rust-production",
      metric: "cyclomatic-complexity",
      value: 21,
      messageIncludes: [
        "Function \"scan\"",
        "jsonc.rs:28",
        "cyclomatic complexity 21"
      ],
      reason:
        "Decision 9 requires one offset-preserving lexical pass that distinguishes strings, both comment forms, permitted whitespace, structural commas, and container closings before the strict parser runs. Those grammar branches share cursor, line, container, and parse-view state, so extraction would add coordination rather than a separate responsibility."
    },
    {
      ruleId: "lizard-function-code-density",
      sourceTool: "lizard",
      path: "crates/adapters/json/src/reference.rs",
      codeArea: "rust-production",
      metric: "function-code-density",
      value: 60,
      messageIncludes: [
        "Function \"resolve_selection\"",
        "reference.rs:72",
        "60 code lines"
      ],
      reason:
        "Decision 11 makes resolution one selected-first borrowed frame-chain transaction: validate canonical object/index traversal, retain each parent frame, then validate the requested view on the selected frame. Extracting traversal or view validation would only pass the same node/frame context to one-use helpers, without a separate resolution owner."
    },
    {
      ruleId: "lizard-cyclomatic-complexity",
      sourceTool: "lizard",
      path: "crates/adapters/json/src/reference.rs",
      codeArea: "rust-production",
      metric: "cyclomatic-complexity",
      value: 11,
      messageIncludes: [
        "Function \"resolve_selection\"",
        "reference.rs:72",
        "cyclomatic complexity 11"
      ],
      reason:
        "Decision 11 makes resolution one selected-first borrowed frame-chain transaction. Its object, array-index, scalar, direct-view, and tail-view branches preserve distinct ref semantics against the same selection chain; splitting them would add one-use coordination without another responsibility owner."
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
