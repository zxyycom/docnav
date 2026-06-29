本 delta spec 记录 Markdown adapter 使用 SDK helper 表达 token-informed cost 的目标；当前只在 `openspec/changes/use-token-based-document-cost/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Markdown adapter reports token-informed document cost through adapter-owned policy
`docnav-markdown` MUST use adapter-owned policy to report token-informed document cost, using SDK helper primitives where appropriate. The final protocol field shape MUST follow the separate structured protocol fields change.

#### Scenario: Markdown cost uses adapter-owned measurement choices
- **WHEN** Markdown read or outline output includes document cost
- **THEN** the cost is derived from Markdown content selected by the adapter
- **THEN** Markdown adapter owns which measurements are reported and how readable output aggregates them
