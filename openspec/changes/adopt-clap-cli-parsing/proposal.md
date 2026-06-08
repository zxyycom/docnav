**一句话核心：本 change 以 `clap` 作为 Rust CLI 参数解析基础，并把直接 CLI 的容错目标收敛为“优先解析已知有效参数、未知输入不阻断成功路径”。当前 change 只在 `openspec/changes/adopt-clap-cli-parsing/` 下形成未审核临时文档，不影响现有其它文档或主规范。**

## Why

Docnav 的 Rust CLI 主要由 AI 维护，人类更多承担方向审核和决策角色；因此 CLI 参数解析应优先选择主流、声明式、可搜索、易审阅的写法，降低不熟 Rust 的维护者阅读 token loop 和特殊分支的成本。

现有直接 CLI 兼容规则过于精确地约束 ignored token、warning kind 和消费细节，增加实现和审计复杂度。核心目标应调整为：在必需参数和已知有效参数足够执行时，未知或无关输入不增加 AI 反复读取与重试次数。

## What Changes

- 将 Rust CLI 参数解析的首选基础统一为 `clap`，用于核心 `docnav` CLI、adapter 直接 CLI 和后续 Rust CLI 扩展。
- 调整直接 CLI 容错契约：未知 flag、多余 positional 和无关参数不作为成功路径的主失败原因；CLI 应优先保留并使用已知有效参数。
- 保留必要失败条件：缺少真正必需的 path/ref/query、已知使用参数值非法、文档或 adapter 业务错误仍返回明确错误。
- 将 warning 语义从精确 token 消费契约降级为用户可见诊断：warning 可以提示未知或忽略输入，但不要求长期稳定的 ignored token 分组、kind 枚举和消费顺序。
- 保持 adapter `invoke` JSON 严格校验；容错策略只适用于人类和 AI 直接调用的 CLI argv。
- 在文档和测试中明确 AI 维护目标：自动 help、结构化子命令、字段级默认值和类型解析应帮助 agent 更快修正调用。
- 增加 `clap` 依赖和必要 feature；实现时优先使用 derive 或清晰的 builder API，而不是继续扩展手写 argv parser。
- 非目标：本 change 不修改 `outline -> ref -> read` 业务语义，不改变 protocol envelope、readable schema、manifest/probe schema，不实现新的 adapter 管理能力，也不把格式解析移出 adapter。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `v0-contract-documentation`: 调整 CLI 容错和输出契约，使主规范从精确 direct CLI warning 规则转向 AI 友好的宽松解析目标。
- `protocol-and-adapter-sdk-implementation`: 调整 adapter SDK 直接 CLI 参数解析要求，允许使用 `clap` 并弱化 ignored token 精确契约。
- `markdown-adapter-v0-implementation`: 调整 `docnav-markdown` CLI smoke 和矩阵验证，验证宽松解析、自动 help 和成功路径，而不是旧的精确 warning token 行为。

## Impact

- 影响 Rust 依赖：workspace 或相关 crate 引入 `clap`。
- 影响代码：`crates/docnav-adapter-sdk/src/direct/args.rs`、`crates/docnav-adapter-sdk/src/direct/cli.rs`、`crates/docnav-markdown/src/cli.rs`，以及未来核心 `docnav` CLI 参数入口。
- 影响测试：需要更新 direct CLI argument matrix、Markdown CLI smoke cases，以及 warning 断言。
- 影响文档：需要同步 `docs/cli.md`、`docs/adapter-contract.md`、`docs/testing.md` 和相关 OpenSpec spec delta。
- 不影响协议层：`adapter invoke` stdin JSON、protocol schema、readable schema、manifest/probe schema 和 ref 语义保持稳定。
