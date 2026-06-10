本 spec delta 仅为 `shorten-markdown-refs` change 的未审核临时文档，核心目标是把 Markdown adapter 的 heading ref 从长 breadcrumb 格式完全迁移为短标识格式。

本 change 只在 `openspec/changes/shorten-markdown-refs/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## MODIFIED Requirements

### Requirement: Markdown heading ref 必须使用 canonical line-ordinal-path 格式
`docnav-markdown` MUST 为 heading 生成 canonical short ref，格式为 `H{line}:{token}`。`line` MUST 是 heading 的 1-based 起始行号；`token` MUST 是由 canonical heading breadcrumb 和 heading path occurrence ordinal 派生的 ASCII 短标识。`token` MUST 在当前文档内唯一；如果初始短 token 冲突，adapter MUST 扩展 token 长度直到唯一。`docnav-markdown` MUST 生成并接受 `D` 作为全文 fallback ref；`D` 不属于 heading ref 格式。

`docnav-markdown` MUST NOT 生成或接受旧 `L{line}:{path}`、`L{line}#{ordinal}:{path}` 或 `doc:full` ref。调用方持有旧 ref 时 MUST 重新执行 `outline` 或 `find` 获取新 ref。

#### Scenario: 无重复 heading 时输出短 heading ref
- **WHEN** Markdown 文档包含唯一 heading path，例如 `Guide` 和 `Guide > Install`
- **THEN** outline 输出 `H1:<token>` 和 `H5:<token>` 形式的 ref
- **THEN** ref MUST NOT 包含 heading breadcrumb 文本
- **THEN** ref MUST NOT 包含旧格式中的 `#1` default ordinal

#### Scenario: 重复完整 heading path 时输出唯一短 ref
- **WHEN** Markdown 文档包含重复完整 heading path，例如 `Repeat` 和 `Repeat > Child`
- **THEN** outline 为每个重复 occurrence 输出不同的 `H{line}:{token}` ref
- **THEN** 每个 ref 均可由 read 唯一定位对应 Markdown section

#### Scenario: read 接受 canonical short heading ref
- **WHEN** 调用方把 outline 或 find 返回的 canonical short heading ref 传给 read
- **THEN** read 返回唯一匹配的 Markdown section
- **THEN** content_type 为 `text/markdown`

#### Scenario: read 拒绝旧 line-ordinal-path heading ref
- **WHEN** 调用方把旧 heading ref，例如 `L5:Guide > Install`、`L9#2:Repeat` 或 `L1#1:Guide`，传给 read
- **THEN** read 返回现有稳定 ref 错误
- **THEN** read MUST NOT 把该旧 ref 解析到 Markdown section

#### Scenario: outline 为空时使用短全文 ref
- **WHEN** 当前 outline 参数过滤后没有任何 heading entry
- **THEN** outline 返回一个全文 entry
- **THEN** 该 entry 的 ref 为 `D`
- **THEN** read 使用 `D` 返回整篇 Markdown 文档

#### Scenario: read 拒绝旧全文 ref
- **WHEN** 调用方把旧全文 ref `doc:full` 传给 read
- **THEN** read 返回现有稳定 ref 错误
- **THEN** read MUST NOT 把 `doc:full` 解析为全文 Markdown 文档

#### Scenario: find 返回短 ref
- **WHEN** query 命中文档中某个 heading section 内的内容
- **THEN** find match 的 ref 使用 `H{line}:{token}` 格式指向当前导航粒度下的所属 heading
- **THEN** 调用方可将该 ref 原样传给 read
