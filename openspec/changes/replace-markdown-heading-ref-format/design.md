## Context

Markdown v0 ref 的生成和解析归 `docnav-markdown` 所有。`docnav`、`docnav-mcp`、共享协议类型、readable schema 和 MCP tool 只把 ref 当作 opaque string 原样传递。

当前 heading ref 已经包含三类定位信息：源码行号、heading breadcrumb path、重复 path 的 occurrence ordinal。本变更只调整序列化形式：把 ordinal 移到 `L{line}#{ordinal}:{path}` 的行号前缀中，并在 ordinal 为 `1` 时从 canonical 输出中省略。

## Goals / Non-Goals

**Goals:**

- 当 heading path occurrence 为 `1` 时，生成 `L{line}:{path}`。
- 当 heading path occurrence 大于 `1` 时，生成 `L{line}#{ordinal}:{path}`。
- 解析 canonical ref，并接受显式 `#1` 输入。
- 生成器始终省略默认序号，不输出 `#1`。
- 删除旧方括号 ordinal 后缀解析路径，使旧 ref 进入现有稳定 ref 错误。
- 保留 `doc:full` 作为全文 fallback ref。
- 更新暴露 Markdown heading ref 的 specs、docs、tests、fixture 断言和示例。

**Non-Goals:**

- 不修改共享协议 envelope、schema、错误 code 或 readable result shape。
- 不修改 `docnav` 路由、`docnav-mcp`、adapter management 或非 Markdown adapter。
- 不修改 heading 解析、章节边界、分页或 `find` match 归属语义。

## Decisions

1. 将新格式的解析和生成继续放在 `docnav-markdown`。

   理由：Markdown heading ref 是格式专属语义。把逻辑留在 adapter 内，可以保持 `docnav` 和 MCP 层只负责原样传递 ref。

   备选方案：在共享协议或 `docnav` 中识别新语法。该方案会把 Markdown 专属字段引入格式无关层，舍弃。

2. 接受显式 `#1` 输入，但 canonical 输出不生成 `#1`。

   理由：`#1` 可唯一定位首个 occurrence，方便机械构造 ref；省略默认序号可以让最常见的无重复 heading 输出更短。

   备选方案：拒绝 `#1`。该方案只能减少一种等价输入，不能提升唯一性，舍弃。

3. read 解析后仍按 line、path、occurrence ordinal 三项匹配 heading。

   理由：新格式只改变字符串表示，不改变重复 heading path 的定位事实。省略 ordinal 的 ref 表示 occurrence `1`，不能退化为按 line/path 启发式消歧。

   备选方案：无 ordinal 时只按 line 和 path 匹配。该方案会让默认省略形式和重复 path 行为边界变弱，舍弃。

4. 旧方括号 ordinal 后缀通过删除解析分支来拒绝。

   理由：目标明确要求不兼容旧格式。旧 ref、格式错误 ref 和无匹配 ref 应复用现有稳定 ref 错误路径。

   备选方案：过渡期同时接受新旧格式。该方案会保留两套可解析拼写，和验收目标冲突，舍弃。

## Risks / Trade-offs

- [已保存旧 ref 无法读取] -> 这是预期破坏性变更；调用方通过重新执行 `outline` 或 `find` 生成新 ref。
- [文档或 fixture 残留旧示例] -> 实现完成前执行旧 suffix marker 仓库扫描，并清理 tests、fixtures、OpenSpec 和文档示例中的残留。
- [解析器错误接受 malformed ordinal] -> adapter 测试覆盖省略 ordinal、显式 `#1`、重复 `#2`、ordinal 为 0、非数字 ordinal 和旧后缀输入。
- [核心 CLI 或 MCP 引入 Markdown 语法依赖] -> 验证重点放在 ref 原样传递；语法解析只在 Markdown adapter 测试中覆盖。
