本 spec delta 为 `add-obvious-result-auto-read` 增加统一输出要求：typed protocol result 必须同时表达 base result、auto-read 内容和未展开状态，再由两条 output path 投影；当前 change 只在 `openspec/changes/add-obvious-result-auto-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: unified output for obvious auto-read

Before output orchestration, obvious-result auto-read SHALL be represented as one typed `ProtocolResponse::Success` result that includes the base outline/find result, the auto-read expansion status, and any read content produced by the existing read pipeline. `ProtocolJson` SHALL serialize that result, and the built-in `readable-view` renderer SHALL render the same facts.

#### Scenario: auto-read succeeds
- **WHEN** obvious-result auto-read reads the single candidate successfully
- **THEN** `protocol-json` SHALL expose both the base operation facts and the read content through the typed composition result
- **AND** `readable-view` SHALL render the same facts through repository renderer configuration without inventing renderer-only fields

#### Scenario: auto-read is skipped
- **WHEN** core decides not to auto-read because the result is ambiguous, missing a ref, paginated, or over budget
- **THEN** the typed composition result SHALL preserve the base operation result
- **AND** both output paths SHALL include a deterministic skipped reason and continuation guidance where available

#### Scenario: auto-read expansion fails
- **WHEN** the base outline/find operation succeeds but the additional read returns a diagnostic
- **THEN** the typed composition result SHALL preserve the base operation result
- **AND** both output paths SHALL project the read diagnostic as auto-read expansion status rather than replacing the base operation outcome

#### Scenario: both output paths consume one response
- **WHEN** a caller selects either `protocol-json` or `readable-view`
- **THEN** output orchestration SHALL consume the same immutable `ProtocolResponse`
- **AND** auto-read business facts SHALL NOT exist only in renderer-private payloads
