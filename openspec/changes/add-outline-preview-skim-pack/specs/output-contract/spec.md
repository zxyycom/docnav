本 spec delta 为 `add-outline-preview-skim-pack` 增加统一输出要求：outline preview 必须在 typed protocol result 中表达结构、预览内容、未预览原因和 continuation，再由两条 output path 投影；当前 change 只在 `openspec/changes/add-outline-preview-skim-pack/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: unified output for outline preview skim pack

Before output orchestration, outline preview composition SHALL be represented as one typed `ProtocolResponse::Success` result that includes outline entries, preview content produced by read, preview status for skipped or failed entries, and continuation guidance. `ProtocolJson` SHALL serialize that result, and the built-in `readable-view` renderer SHALL render the same facts.

#### Scenario: preview succeeds
- **WHEN** outline preview composition reads one or more selected entries successfully
- **THEN** `protocol-json` SHALL expose the outline facts and preview content through the typed composition result
- **AND** `readable-view` SHALL render the same facts through repository renderer configuration without inventing renderer-only fields

#### Scenario: preview is partially skipped
- **WHEN** some outline entries are not previewed because of budget, missing refs, pagination, or preview count limits
- **THEN** the typed composition result SHALL preserve those outline entries
- **AND** both output paths SHALL expose deterministic skipped reasons or continuation guidance where available

#### Scenario: preview read fails
- **WHEN** outline succeeds but an additional preview read returns a diagnostic
- **THEN** the typed composition result SHALL preserve the outline result
- **AND** both output paths SHALL project the read diagnostic as preview status for that entry rather than replacing the outline outcome

#### Scenario: both output paths consume one response
- **WHEN** a caller selects either `protocol-json` or `readable-view`
- **THEN** output orchestration SHALL consume the same immutable `ProtocolResponse`
- **AND** preview business facts SHALL NOT exist only in renderer-private payloads
