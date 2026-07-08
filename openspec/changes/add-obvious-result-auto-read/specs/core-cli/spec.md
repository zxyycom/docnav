本 spec delta 为 `add-obvious-result-auto-read` 增加 core CLI 行为要求：outline 和 find 的唯一明确结果可以通过显式组合 surface 自动 read；当前 change 只在 `openspec/changes/add-obvious-result-auto-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: obvious result auto-read composition

Core CLI SHALL expose an explicit composition control for outline and find document commands that auto-reads a single obvious result by sequencing the existing read operation after the base operation succeeds.

#### Scenario: outline single entry auto-reads
- **WHEN** a caller invokes the obvious-result auto-read control for `outline`
- **AND** the outline result contains exactly one entry with a non-empty ref
- **AND** the active readable output budget allows the additional read
- **THEN** core SHALL call the existing read pipeline with the same document context and that ref
- **AND** core SHALL preserve adapter ownership of ref parsing and read semantics

#### Scenario: find single match auto-reads
- **WHEN** a caller invokes the obvious-result auto-read control for `find`
- **AND** the find result contains exactly one match with a non-empty ref
- **AND** the active readable output budget allows the additional read
- **THEN** core SHALL call the existing read pipeline with the same document context and that ref
- **AND** core SHALL preserve the original find result facts for output projection

#### Scenario: ambiguous result does not auto-read
- **WHEN** the base outline or find result has zero candidates, multiple candidates, no readable ref, or insufficient budget
- **THEN** core SHALL NOT call read automatically
- **AND** core SHALL return the base operation outcome with a deterministic auto-read skipped status

#### Scenario: protocol output does not mix composition payload
- **WHEN** a caller requests `protocol-json` together with obvious-result auto-read
- **THEN** core SHALL NOT append read content into the base operation protocol result
- **AND** core SHALL either reject the unsupported combination or route to a separately documented protocol contract before implementation is considered complete
