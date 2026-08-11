本 delta spec 记录 Markdown adapter 使用 shared text cost calculator helper 的目标。该目标基于 current `cost.measurements[]` shape，不改变 Markdown readable payload shape 或当前 `limit` 预算语义。

## ADDED Requirements

### Requirement: Markdown adapter uses shared text cost calculator for selected output text
`docnav-markdown` MUST use shared text cost helper functions to compute text cost for Markdown text it has already selected for operation output. The adapter MUST choose which helper functions to call and call them directly. The adapter MUST report helper results through the current `cost.measurements[]` protocol shape and MUST NOT change raw protocol cost shape, readable output payload shape, or current `limit` budget semantics as part of helper-based cost reporting.

#### Scenario: Markdown cost comes from selected plain text
- **WHEN** Markdown read or outline output includes text cost
- **THEN** Markdown adapter selects the Markdown text region according to existing operation semantics
- **THEN** Markdown adapter calls `line_cost`, `byte_cost`, and `token_cost` for that operation
- **THEN** Markdown adapter passes that selected plain text as the only required input to each chosen helper function
- **THEN** Markdown adapter embeds the returned measurements in `cost.measurements[]` ordered as `lines`, `bytes`, `tokens`
- **THEN** readable output continues to derive its cost summary from the adapter-provided measurements

#### Scenario: Markdown preserves existing cost scopes
- **WHEN** Markdown adapter reports cost for a read result
- **THEN** the helper measurements are embedded with the existing `selection` scope
- **WHEN** Markdown adapter reports cost for an outline full entry or heading section entry
- **THEN** the helper measurements are embedded with the existing `entry` scope

#### Scenario: Markdown reports token cost without changing limit semantics
- **WHEN** Markdown adapter reports token cost for selected Markdown text
- **THEN** it calls the token cost helper function with that selected text as the only required input
- **THEN** the `tiktoken-rs` `o200k_base` ordinary plain-text token measurement is reported through `cost.measurements[]`
- **THEN** Markdown `limit` continues to use the existing Unicode character budget unless a separate change updates pagination semantics
