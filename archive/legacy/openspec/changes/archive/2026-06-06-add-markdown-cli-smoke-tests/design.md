## Context

`docnav-markdown` 目前已有 parser 单元测试、adapter 级测试、`invoke_once` 测试和少量直接 CLI 测试。缺口在于：还没有一个明确的黑盒 CLI smoke 契约，把构建后的 `docnav-markdown` 可执行文件本身作为被测对象，系统验证进程边界、三种输出模式、stdout/stderr 分离、退出码和 schema 形状。

这些测试必须停留在 Markdown adapter 的所有权边界内。核心 `docnav` 的 adapter 路由、配置解析、adapter install 管理和 MCP bridge smoke 属于其它 active change。

## Goals / Non-Goals

**Goals:**

- 增加启动真实 `docnav-markdown` 二进制的黑盒测试。
- 覆盖 normal 和 Markdown 边界 fixture corpus。
- 覆盖 `outline -> ref -> read`、`find`、`info`、`probe`、`manifest` 和有效 `invoke`。
- 覆盖 `text`、`readable-json`、`protocol-json` 三种输出模式。
- 对 JSON 输出做 schema 或等价结构断言，确认 readable 层不泄漏 protocol envelope。
- 增加负向 CLI 矩阵，校验 stdout、stderr 和 exit code。
- 为每条命令写入审计日志，记录命令、cwd、exit code、stdout、stderr 和断言摘要。

**Non-Goals:**

- 不实现或测试核心 `docnav` adapter 选择、配置合并或路由。
- 不增加 MCP bridge smoke 测试。
- 不改变 protocol schema、manifest shape、probe shape 或 Markdown adapter runtime 行为，除非新增测试暴露了与既有主规范冲突的缺陷。

## Decisions

1. 使用 Node.js 黑盒脚本批量启动 adapter binary。
   - 理由：黑盒 smoke 的核心是批量执行进程、传 stdin、解析 stdout/stderr、校验 JSON/schema 和 exit code。Node.js 的 `child_process` 更直接，避免为了拿构建产物路径而把大量黑盒流程塞进 Rust integration tests。
   - 备选方案：Rust integration tests。拒绝，因为这会把进程批量编排、fixture 管理和 JSON/schema 校验写得更重，复杂度不匹配。

2. 先显式构建，再把 binary 路径交给 Node.js。
   - 理由：测试脚本不应该猜测 `target/` 目录布局。推荐通过 package script 或 wrapper 命令先执行 `cargo build -p docnav-markdown`，再用确定性路径或环境变量传入 adapter binary。
   - 备选方案：Node.js 脚本内部自动调用 cargo 并猜路径。暂不采用，因为构建步骤作为外层命令更清晰，也便于 CI 单独复用。

3. fixture 必须作为项目文件固定放置。
   - 理由：黑盒测试用例本身是审计材料，应该能被直接查看、review 和复现。临时生成 fixture 会隐藏案例内容，降低测试矩阵可读性。
   - 位置建议：`crates/docnav-markdown/tests/fixtures/cli-smoke/`，非 UTF-8 fixture 可使用二进制文件。
   - 备选方案：测试运行时临时创建 fixture。拒绝，因为用户明确要求测试用例直接写成项目文件。

4. fixture 保持在 Markdown adapter 测试范围内。
   - 理由：这些 fixture 是 adapter 黑盒边界验证材料，不应进入 `docs/examples` 成为第二套规范来源。
   - 备选方案：放到 `docs/examples`。暂不采用，因为 examples 主要服务 schema 和语义映射校验，不适合承载穷举边界 corpus。

5. 输出层按模式分别断言。
   - 理由：`protocol-json` 必须保留完整 envelope；`text` 和 `readable-json` 属于阅读输出层，不能包含 `protocol_version`、`request_id`、`operation`、`ok`。测试要能抓住 envelope 泄漏、stderr/stdout 混用和错误包装漂移。
   - 备选方案：只断言命令成功。拒绝，因为这无法覆盖本 change 的核心风险。

6. CLI 参数校验错误与 operation 稳定错误分开测试。
   - 理由：缺参数、未知 flag、非法数字属于 CLI 诊断；missing file、invalid ref、non-UTF-8 属于 operation 层稳定错误。两类错误的 stdout/stderr 期望不同。

7. 增加少量格式兼容 fixture。
   - 理由：UTF-8 BOM、CRLF、`.MD` 和 `.markdown` 都是容易被路径/编码/行号处理影响的黑盒边界，成本低但回归价值高。

8. 审计日志使用固定 `.log` 目录。
   - 做法：Node.js runner 将完整原始输出写入 `.log/docnav-markdown-cli-smoke/latest.log` 和时间戳日志；终端只输出通过/失败摘要和日志路径。
   - 理由：黑盒 smoke 一次会执行大量命令，完整输出全部打到终端会降低可读性；写入日志更利于审计和失败复查。
   - 边界：日志只能记录测试命令、fixture 路径、stdout/stderr、exit code 和断言结果，不写入与测试无关的环境变量或机器私密信息。

## Risks / Trade-offs

- [文本输出测试过脆] → 只断言稳定关键片段，例如 ref、page、content_type、error code 和关键 details，不做完整快照。
- [large pagination fixture 变慢] → 使用固定但适度大小的项目 fixture，刚好跨页即可，不使用超大文件。
- [与现有单元测试重复] → 黑盒测试只覆盖进程、输出层和 smoke flow；parser 细节仍由单元/adapter 测试承担。
- [schema 校验引入测试复杂度] → Node.js 脚本优先复用现有 schema 文件和 AJV 依赖；若局部不适合 schema 编译，则做等价字段集合断言，避免引入新的规范来源。
- [审计日志过大] → 记录完整命令结果，但 fixture 保持适度大小；终端只打印摘要和日志路径。

## Migration Plan

1. 增加固定 fixture corpus 目录和测试文件。
2. 增加 Node.js 黑盒 smoke runner 和可复用断言工具。
3. 为 runner 增加 `.log/docnav-markdown-cli-smoke/` 审计日志输出。
4. 增加 package script 或统一验证集成：先构建 `docnav-markdown`，再运行 Node.js smoke runner。
5. 补齐正向 smoke，包括 `outline -> ref -> read`、`find`、`info`、`probe`、`manifest` 和有效 `invoke`。
6. 补齐负向矩阵，包括 CLI 参数错误、operation 错误和 malformed invoke JSON。
7. 先运行 Markdown CLI smoke script，再运行 `pnpm run verify:docnav-workspace`。

## Open Questions

- 无。当前范围可以完全限制在 Markdown adapter 测试内。
