本 design 说明如何把 Markdown heading canonical ref 从 `H:L{line}:H{level}:I{index}` 收敛为 `H:L{line}:H{level}`；它只在 `openspec/changes/remove-markdown-heading-ref-index/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Markdown adapter 当前拥有 Markdown heading ref 的生成、解析、匹配和错误分类。共享协议、`docnav` core 和 MCP bridge 只把 ref 视为非空 opaque string，并原样传给 adapter。

现行 Markdown canonical heading ref 包含 line、level 和 index 三个结构字段。index 的主要效果是在文档结构变化后增加旧 ref 失效概率；但 Markdown ref 已被定义为生成时解析结果中的结构坐标，不承诺跨文档变化保持身份或一定失败。保留 index 会增加 ref 长度、示例噪声和测试 fixture 更新成本。

## Goals / Non-Goals

**Goals:**

- 将 Markdown heading canonical ref 改为 `H:L{line}:H{level}`。
- 让 outline 和 find 只生成新 canonical heading ref。
- 让 read 只按当前解析结果中的 line 和 level 精确匹配 heading。
- 同步 Markdown 主规范、OpenSpec requirement、examples、fixtures 和测试断言，保持 public surface 一致。
- 保持 `doc:full`、分页、display 截断、readable/protocol/MCP wrapper 和共享 ref opaque contract 不变。

**Non-Goals:**

- 不兼容旧 `H:L{line}:H{level}:I{index}` 作为 canonical heading ref。
- 不引入 title、breadcrumb、hash、文档版本或持久身份。
- 不改变其它 adapter 的 ref grammar。
- 不让 `docnav` core、MCP bridge、schema 或共享协议解析 Markdown ref。

## Decisions

1. Canonical grammar 使用 `H:L{line}:H{level}`。

   选择 line + level 是因为它保留了人工可读的结构坐标，且同一 Markdown 解析结果中 heading 起始行天然可区分条目。替代方案是保留 index 或改用 title/breadcrumb；保留 index 继续承担不必要的结构变更防护，title/breadcrumb 会引入长度、编码、重命名和重复标题问题。

2. 旧 `:I{index}` 格式按非法 grammar 处理。

   这是有意的 breaking change。兼容旧格式会保留两套 grammar 和测试路径，并削弱本 change “去掉 index”的契约清晰度。调用方的正确迁移方式是重新执行 outline 或 find 获取当前 ref，而不是继续持有旧 ref。

3. `read` 匹配只使用 line 和 level，不补充 title、breadcrumb 或 section 内容校验。

   这保持 ref 是结构坐标而非内容身份。替代方案是增加标题校验来弥补去掉 index 后的旧 ref 误读风险；但这会把 heading title 变成隐式身份字段，与现有结构快照语义冲突。

4. `docnav-mcp` 和共享层不做任何格式专属迁移。

   MCP bridge 继续把 tool 参数映射到 `docnav` CLI 并透传结果；核心 CLI 继续按 path 选择 adapter 并原样传 ref。所有 Markdown grammar 变化留在 Markdown adapter 和对应验证材料中。

## Risks / Trade-offs

- [Risk] 文档编辑后，旧 line + level ref 比旧 line + level + index 更可能匹配到当前结构中的另一个 heading。→ Mitigation: 在规范中继续明确 Markdown ref 是结构快照，不是持久身份；需要当前结构时必须重新执行 outline 或 find。
- [Risk] 现有外部脚本或示例中硬编码旧 `:I` ref 会失败。→ Mitigation: 将本 change 标记为 breaking，并同步 examples、fixtures、error tests 和 release 验证入口。
- [Risk] 测试只更新字符串快照但遗漏错误边界。→ Mitigation: tasks 中要求覆盖合法新 ref 的 read roundtrip、旧 `:I` 格式的 `REF_INVALID`、合法但不匹配新 ref 的 `REF_NOT_FOUND`。

## Migration Plan

1. 更新 `docs/adapters/markdown.md` 和 OpenSpec delta spec，把 canonical grammar、正则、字段表、长度保证、唯一性说明、重复 heading 行为、read 匹配字段和错误边界改为 line + level。
2. 更新 `crates/docnav-markdown` 的 ref 生成、解析和匹配逻辑，删除 canonical heading ref 中的 index 字段。
3. 更新 adapter/unit/CLI smoke 测试和 fixtures：新输出断言使用 `H:L{line}:H{level}`，旧 `H:L{line}:H{level}:I{index}` 进入 `REF_INVALID` 覆盖。
4. 更新 `docs/examples/json/**`、MCP 示例和相关 readable/protocol 示例中展示的 Markdown heading ref。
5. 运行 Markdown adapter 范围测试、schema/example 验证和必要的 workspace 验证；若发现跨层示例或 output schema 断言依赖旧 ref 字符串，同步修正。

Rollback 策略：若实现后发现 line + level 不能满足当前 parser 唯一定位，回滚本 change 的 code/docs/examples/tests 到 `H:L{line}:H{level}:I{index}`，或在进入实现前重新提出包含兼容期的新 change。

## Open Questions

- 无待决问题；本 change 按 breaking canonical grammar 处理，不设计旧 ref 兼容期。
