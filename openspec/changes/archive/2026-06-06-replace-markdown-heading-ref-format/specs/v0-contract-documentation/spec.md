## ADDED Requirements

### Requirement: Ref 文档必须描述 Markdown heading ref 新格式
Ref 文档和示例 MUST 把 Markdown heading ref 描述为 adapter 拥有的字符串，格式为 `L{line}:{path}` 或 `L{line}#{ordinal}:{path}`，其中 `path` 表示 heading breadcrumb。文档 MUST 说明 Markdown 解析器接受显式 `#1`，但 canonical 生成结果省略 `#1`；文档 MUST 继续把 `doc:full` 描述为全文 fallback ref，并明确它不属于 heading ref 格式。

#### Scenario: Ref 规范展示无重复 heading
- **WHEN** 读者打开 Ref 文档
- **THEN** 首个 occurrence 的 Markdown heading 示例使用 `L1:Guide` 和 `L5:Guide > Install` 形式的 ref
- **THEN** 示例说明 `Guide > Install` 是 heading breadcrumb

#### Scenario: Ref 规范展示重复 heading path
- **WHEN** 读者查看 Ref 文档中的重复 heading path 示例
- **THEN** 首个 occurrence 使用 `L1:Repeat` 和 `L5:Repeat > Child` 形式的 ref
- **THEN** 后续 occurrence 使用 `L9#2:Repeat` 和 `L13#2:Repeat > Child` 形式的 ref

#### Scenario: 文档示例不保留 legacy ordinal suffix
- **WHEN** 实现者更新本 change 涉及的 docs 和 examples
- **THEN** Ref 文档、Markdown 示例和 OpenSpec task/spec 文案只使用 canonical heading ref 示例或 `doc:full`
- **THEN** 文档示例不保留旧方括号 ordinal 后缀 marker
