## Why

Markdown heading ref 当前把重复序号放在 path 后面的适配器私有后缀里，可读性和 `path` 表示 heading breadcrumb 的文档表述不一致。此变更在新增更多 adapter 或 MCP 消费方依赖旧写法前，统一 Markdown heading ref 的 canonical 形式，同时保持 `outline -> ref -> read` 的传递模型不变。

## What Changes

- **BREAKING**：Markdown heading ref 的 canonical 输出改为首个出现项 `L{line}:{path}`，重复 heading path 的第 2 次及以后改为 `L{line}#{ordinal}:{path}`。
- **BREAKING**：删除旧方括号 ordinal 后缀的解析路径；旧 ref 进入现有稳定 ref 错误路径，不再解析到章节。
- 解析器接受显式默认序号输入，例如 `L1#1:Guide`；生成器仍省略默认序号，不输出 `#1`。
- `doc:full` 继续作为全文 fallback ref，不纳入 heading ref 格式替换范围。
- 同步更新 Ref 规范、Markdown adapter 测试、CLI smoke fixture 断言、OpenSpec spec/task 文案和文档示例。
- 边界：不修改共享协议 schema、`docnav-mcp`、非 Markdown adapter、分页语义或 `outline -> ref -> read` 的原样传递契约。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `markdown-adapter-v0-implementation`：更新 Markdown heading ref 的生成、解析、重复 path 消歧和旧格式拒绝规则。
- `v0-contract-documentation`：更新 Ref 规范和示例，使用新的 Markdown heading ref 写法描述 canonical 输出和解析边界。

## Impact

- 受影响可执行文件和 adapter：`docnav-markdown`。
- 受影响实现：`crates/docnav-markdown` 中的 heading ref 生成器和解析器。
- 受影响测试和 fixture：Markdown adapter 单元/集成测试、CLI 测试、`scripts/docnav-markdown-cli-smoke` 断言。
- 受影响文档和校验材料：`docs/refs.md`、Markdown 相关示例/fixture、本 change 的 spec 与 tasks。
- 不改变共享协议和进程边界；`docnav`、`docnav-mcp` 和调用方继续把 ref 当作 opaque string 原样传递。
