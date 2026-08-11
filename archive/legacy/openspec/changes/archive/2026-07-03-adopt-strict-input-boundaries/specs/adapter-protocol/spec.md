本 spec delta 定义 `adopt-strict-input-boundaries` 对 `adapter-protocol` 的目标变更：在当前 core-linked adapter 架构下，公共输入由 core CLI、protocol request、config source 和 navigation input resolution 严格处理；linked adapter handler 接收已准备的 typed operation input。

## MODIFIED Requirements

### Requirement: Linked adapter handler 接收已准备的 operation input
Docnav core 和 navigation layer MUST 在 dispatch linked adapter handler 前完成 public input boundary 处理。Core MUST classify commands and pass config source descriptors/paths; `docnav-navigation` MUST load raw config sources and construct typed operation input from raw command, protocol request arguments, project/user config and built-in defaults, preserving declared adapter-owned native option source metadata. Linked adapter handlers MUST NOT read CLI argv、stdin、stdout、stderr、process cwd or process exit code to obtain operation input.

Invalid public input MUST fail before linked adapter business execution when it belongs to core CLI parsing、protocol envelope/request shape、config source loading、navigation input resolution mapping or operation applicability。Declared adapter-owned native options MAY be handed to the selected adapter through source-level static native option registry metadata；unsupported option、type mismatch or range invalid MUST be reported by selected adapter typed-field validation before format business handling continues.

#### Scenario: core CLI unknown argv 被拒绝在 adapter dispatch 前
- **WHEN** caller executes `docnav outline docs/guide.md --unknown --output readable-json`
- **THEN** core CLI returns an input diagnostic
- **THEN** navigation does not dispatch the linked adapter handler
- **THEN** failure output projects one primary `DiagnosticRecord`

#### Scenario: protocol request shape failure 停在 protocol owner
- **WHEN** a protocol request JSON value contains unknown envelope fields、missing required fields or malformed request shape
- **THEN** protocol input validation rejects the request at the protocol boundary
- **THEN** navigation input resolution does not receive the invalid envelope
- **THEN** failure output uses the protocol failure projection for the primary `DiagnosticRecord`

#### Scenario: known operation arguments 进入 navigation input resolution
- **WHEN** a protocol request envelope is valid but operation arguments contain wrong type、unmapped arguments or invalid values
- **THEN** navigation input resolution and typed-field processing produce validation diagnostics
- **THEN** linked adapter business handling does not execute
- **THEN** the owning surface projects the diagnostics as a failed document request

#### Scenario: declared native option handoff 保留 owner metadata
- **WHEN** CLI、config or protocol arguments provide `options.max_heading_level: 2`
- **AND** the source-level static native option registry declares the Markdown option source
- **THEN** navigation input resolution preserves source kind、owner、namespace、key and type variant metadata
- **THEN** the linked Markdown handler receives the merged native option value in prepared operation input

#### Scenario: selected adapter typed-field native option validation 返回结构化诊断
- **WHEN** adapter selection succeeds and prepared input contains an unsupported option、type mismatch or range invalid value for the selected adapter
- **THEN** selected adapter typed-field validation returns a structured diagnostic before handler execution
- **THEN** core/output projects that diagnostic through the selected raw or readable failure surface
