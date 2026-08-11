## ADDED Requirements

### Requirement: Markdown heading ref 必须使用 canonical line-ordinal-path 格式
`docnav-markdown` MUST 在 heading path occurrence ordinal 为 `1` 时生成 `L{line}:{path}`，并在 occurrence ordinal 大于 `1` 时生成 `L{line}#{ordinal}:{path}`。`path` MUST 表示 heading breadcrumb，不是文件路径。`docnav-markdown` MUST 继续生成并接受 `doc:full` 作为全文 fallback ref；`doc:full` 不属于 heading ref 格式。

#### Scenario: 无重复 heading 时省略默认 ordinal
- **WHEN** Markdown 文档包含唯一 heading path，例如 `Guide` 和 `Guide > Install`
- **THEN** outline 输出 `L1:Guide` 和 `L5:Guide > Install` 形式的 ref
- **THEN** outline MUST NOT 为这些首个 occurrence 输出 `#1`

#### Scenario: 重复完整 heading path 时输出重复 ordinal
- **WHEN** Markdown 文档包含重复完整 heading path，例如 `Repeat` 和 `Repeat > Child`
- **THEN** outline 为首个 occurrence 输出 `L1:Repeat` 和 `L5:Repeat > Child` 形式的 ref
- **THEN** outline 为后续 occurrence 输出 `L9#2:Repeat` 和 `L13#2:Repeat > Child` 形式的 ref

#### Scenario: read 接受 canonical heading ref
- **WHEN** 调用方把 canonical heading ref，例如 `L5:Guide > Install` 或 `L9#2:Repeat`，传给 read
- **THEN** read 返回唯一匹配的 Markdown section
- **THEN** content_type 为 `text/markdown`

#### Scenario: read 接受显式 default ordinal
- **WHEN** 调用方把显式 default ordinal ref，例如 `L1#1:Guide`，传给 read
- **THEN** read 定位该 heading path 的首个 occurrence
- **THEN** 生成器仍省略 `#1`

#### Scenario: read 拒绝 legacy bracketed ordinal suffix
- **WHEN** 调用方把使用旧方括号 ordinal 后缀的 heading ref 传给 read
- **THEN** read 返回现有稳定 ref 错误
- **THEN** read MUST NOT 把该旧 ref 解析到 Markdown section
