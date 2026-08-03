本 delta spec 定义 `replace-probe-traversal-with-inferred-routing` 对 `core-cli` 尚未应用的 Target：automatic pathname routing 发生在目标文档 filesystem I/O 之前，route 命中后的 normalized document path 继续交给既有下游契约；它不表示 Current 主规范或实现已经迁移。

## MODIFIED Requirements

### Requirement: Core CLI normalizes document and project paths

Core CLI MUST derive an invocation-private routing pathname lexically from the caller document path and command cwd before handing automatic-routing facts to navigation. This derivation MUST NOT inspect target-document metadata, open or read the target, or canonicalize it through the filesystem. If automatic routing selects a registered adapter, or caller intent explicitly selects one, core and navigation MUST then perform the filesystem-backed document path/access normalization required by the operation before constructing closed standard input or dispatching the selected strategy. Downstream operation owners MUST receive the stable normalized document path rather than raw argv text. If automatic routing finds no pathname hint, the invocation MUST fail with the routing pathname and MUST NOT perform target-document filesystem I/O merely to normalize or validate a document that will not be dispatched.

#### Scenario: File outside project root

- **WHEN** a caller references a routable document outside the project root
- **THEN** core first derives its routing pathname without target-document filesystem I/O
- **THEN** navigation selects the adapter from the lexical basename
- **THEN** core normalizes the document path after selection
- **THEN** downstream operation owners receive a stable path fact rather than raw argv text

#### Scenario: Unknown basename stops before document I/O

- **WHEN** automatic routing matches neither an exact filename nor a manifest suffix for the lexical basename
- **THEN** the invocation returns the pathname no-match diagnostic using the routing pathname
- **THEN** core does not inspect target metadata, open or read the target, or canonicalize it through the filesystem

#### Scenario: Routable pathname has a filesystem failure

- **WHEN** automatic or explicit selection identifies one registered adapter
- **AND** post-selection document path/access normalization fails
- **THEN** core returns the existing path or access diagnostic for that filesystem-backed failure
- **THEN** no adapter operation is dispatched
- **THEN** routing does not try another adapter
