本 design 说明如何让 Markdown heading canonical ref 使用 `H:L{line}:H{level}`。

## Context

Markdown adapter 当前拥有 Markdown heading ref 的生成、解析、匹配和错误分类。共享协议、`docnav` core 和 MCP bridge 把 ref 视为非空 opaque string，并原样传给 adapter。

Markdown ref 已被定义为生成时解析结果中的结构坐标。line + level 足以表达当前解析结果内的 heading 定位输入，并减少 ref 长度、示例噪声和测试 fixture 更新成本。

## Goals / Scope Boundaries

**Goals:**

- 将 Markdown heading canonical ref 定义为 `H:L{line}:H{level}`。
- 让 outline 和 find 生成当前 canonical heading ref。
- 让 read 按当前解析结果中的 line 和 level 精确匹配 heading。
- 同步 Markdown 主规范、OpenSpec requirement、examples、fixtures 和测试断言，保持 public surface 一致。
- 保持 `doc:full`、分页、display 截断、readable/protocol/MCP wrapper 和共享 ref opaque contract 稳定。

**Scope Boundaries:**

- 调用方通过当前 outline 或 find 获取当前 ref。
- title、breadcrumb、hash、文档版本和持久身份保持在 display、content 或外部状态中。
- 其它 adapter 的 ref grammar 由各 adapter 拥有。
- `docnav` core、MCP bridge、schema 和共享协议继续执行 opaque pass-through。

## Decisions

1. Canonical grammar 使用 `H:L{line}:H{level}`。

   选择 line + level 是因为它保留了人工可读的结构坐标，且同一 Markdown 解析结果中 heading 起始行天然可区分条目。title/breadcrumb 承载 display 阅读语义，ref 身份输入保持结构坐标。

2. `read` 按当前 canonical grammar 解析 heading ref。

   这是有意的 breaking change。调用方的正确迁移方式是重新执行 outline 或 find 获取当前 ref。

3. `read` 匹配使用 line 和 level。

   这保持 ref 是结构坐标。heading title 和 section 内容继续由 display/content 承载。

4. `docnav-mcp` 和共享层保持 opaque pass-through。

   MCP bridge 继续把 tool 参数映射到 `docnav` CLI 并透传结果；核心 CLI 继续按 path 选择 adapter 并原样传 ref。所有 Markdown grammar 变化留在 Markdown adapter 和对应验证材料中。

## Risks / Trade-offs

- [Risk] 文档编辑后，line + level ref 可能匹配当前结构中的另一个 heading。→ Mitigation: 规范明确 Markdown ref 是结构快照；需要当前结构时重新执行 outline 或 find。
- [Risk] 外部脚本或示例硬编码 canonical ref。→ Mitigation: 将本 change 标记为 breaking，并同步 examples、fixtures 和 release 验证入口。
- [Risk] 测试覆盖可能停留在字符串快照。→ Mitigation: tasks 中要求覆盖 canonical ref 的 read roundtrip、grammar 外输入的 `REF_INVALID` 和合法但不匹配 canonical ref 的 `REF_NOT_FOUND`。

## Migration Plan

1. 更新 `docs/adapters/markdown.md` 和 OpenSpec delta spec，把 canonical grammar、正则、字段表、长度保证、唯一性说明、重复 heading 行为、read 匹配字段和错误边界改为 line + level。
2. 更新 `crates/docnav-markdown` 的 ref 生成、解析和匹配逻辑，使 canonical heading ref 使用 line 和 level。
3. 更新 adapter/unit/CLI smoke 测试和 fixtures：输出断言使用 `H:L{line}:H{level}`，错误边界覆盖 canonical ref 未匹配的 `REF_NOT_FOUND` 和 grammar 外输入的 `REF_INVALID`。
4. 更新 `docs/examples/json/**`、MCP 示例和相关 readable/protocol 示例中展示的 Markdown heading ref。
5. 运行 Markdown adapter 范围测试、schema/example 验证和必要的 workspace 验证；跨层示例和 output schema 断言同步使用当前 canonical ref 字符串。

Reversal 策略：若需要替代唯一定位策略，提交新的 OpenSpec change 定义替代 grammar 和迁移计划。

## Open Questions

- 无待决问题；本 change 按 breaking canonical grammar 处理。
