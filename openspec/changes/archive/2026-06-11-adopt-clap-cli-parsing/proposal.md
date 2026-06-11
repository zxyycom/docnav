**核心决策：** 本 change 覆盖的 Rust document CLI、已存在 core non-document 命令、adapter direct CLI 和 help 入口以 `clap` 作为 argv 结构解析基础。文档操作入口在各自传输层解析成功后进入 canonical document operation input 或等价 semantic request。CLI argv 对未知或无关 token 保持宽松；adapter `invoke` JSON 和 schema 入口保持严格。阅读层 warning envelope 迁移是本 change 的一等契约变更。当前提案阶段只修改 `openspec/changes/adopt-clap-cli-parsing/` 下的临时 OpenSpec artifacts；进入实现阶段后将按本 change 范围同步更新代码、主规范、schema、示例和测试。

## Why

Docnav 是 CLI-first 的结构化文档导航系统，使用者包括人类和 AI agent。当前核心 `docnav` 和 adapter direct CLI 的 Rust 参数解析依赖手写 token loop。这些 loop 同时处理命令识别、类型校验、兼容容错、warning 归属和 token 消费细节，导致行为审计成本偏高。

旧 direct CLI 兼容契约过度约束了 warning 细节，例如精确 token 分组、细粒度 warning family 值和 `reason` 文案。这些细节不是 `outline -> ref -> read` 的核心产品契约，却会把测试绑定到特定 parser 实现，增加迁移到 `clap` 的难度。

本 change 保留“AI 友好的一次成功调用”价值，但收窄稳定契约。Rust CLI 入口通过 `clap` 声明固定命令、flag、默认值、枚举和 help，并先确定 command/operation；Docnav 只对当前 operation 实际使用的参数做类型、范围和枚举校验。unknown、extra positional 和 unused known 参数以原始 token 形式受控收集为 warning metadata，不重新实现业务参数解释。文档操作入口完成传输层解析后，映射为 canonical document operation input 或等价 semantic request，并共享语义归一、容错校验和 operation 执行。adapter `invoke` JSON 必须在进入文档操作管道前拒绝 malformed JSON、未知字段和错误类型。

`docnav-mcp` 保持 Node.js bridge 边界：它把 MCP tool call 映射到核心 `docnav` CLI，并包装 TextContent/structuredContent 输出；它不拥有 adapter SDK 解析、adapter `invoke`、格式解析或 Rust argv 行为。

## What Changes

- 将本 change 覆盖的 Rust document CLI、已存在 core non-document 命令、adapter direct CLI 和 help 入口的首选 argv 结构解析基础统一为 `clap`。
- 明确入口分层：
  - CLI argv 中的已知命令、已知参数声明、默认值、枚举和 help 由 `clap` 承载；参数值的类型、范围和枚举校验按当前 operation 实际使用集合执行。
  - Docnav 受控收集 unknown、extra positional 和 unused known 参数的原始 token，只生成 warning metadata，不复制业务参数解释路径。
  - Adapter `invoke` JSON 由严格 schema、字段和类型校验解析。
  - 成功解析的文档操作输入先归一到 canonical document operation input 或等价 semantic request，再构造 request 并执行。
- 将 CLI 兼容从“精确 token 归属”调整为“语义成功优先”：
  - 当前 operation 的必需语义输入存在且实际使用参数有效时，未知 flag、多余 positional 和当前 operation 不使用的参数不作为成功路径主失败原因。
  - 当前 operation 不使用的 known 参数即使值不符合其它 operation 的类型、范围或枚举约束，也不作为成功路径主失败原因。
  - 当前 operation 实际使用的参数仍保持严格。
- 保留必要失败：malformed JSON、schema/type/field 错误、缺少必需 `path`/`ref`/`query`、实际使用参数非法、文档/ref/格式错误和 adapter/protocol 错误都必须清晰失败。
- 将 warning envelope 契约迁移作为一等目标：
  - 每个 `warnings[]` item 使用稳定 envelope，包含 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象。
  - `id` 是稳定 warning family marker；当前至少包括 `cli_argv_ignored` 和 `adapter_candidate_failure`。
  - `effect` 表达稳定影响，例如 `operation_continued`、`candidate_skipped` 或 `diagnostic_only`。
  - `details` 承载 family-specific 字段；CLI argv warning 可以在 `details.tokens` 中列出相关 argv token，adapter candidate warning 保留 `details.adapter_id`、`details.stage`、`details.code` 和可选 `details.preselected`。
  - CLI argv warning 的 token 分组、`reason` 文案和 token 消费顺序不作为稳定契约。
- 保持 protocol-shaped stdout 边界：
  - `protocol-json`、manifest、probe 和 adapter `invoke` stdout 只输出对应 schema payload。
  - CLI warning 或诊断进入 stderr 或阅读层输出通道。
- 保持 adapter `invoke` 的传输层严格校验；schema-valid invoke request 仍必须与 direct CLI 共享文档操作归一和 handler 路径。
- 增加核心 `docnav` 和 `docnav-markdown` 的 help 验收，使 `--help` 和子命令 help 能作为 AI 与人类调用者的纠错入口。

## 非目标

- 不改变 `outline -> ref -> read` 业务语义。
- 不改变 ref 生成、解析或所有权。
- 不把 Markdown 或其它格式解析移入核心 `docnav`。
- 不让 adapter `invoke` JSON 或 MCP tool arguments 变成 argv 式宽松传输协议。
- 不给 protocol response、manifest 或 probe schema 增加 CLI warning 字段。
- 不扩大 `docnav-mcp` 在 MCP-to-core-CLI 映射和 MCP 输出包装之外的 ownership。
- 不改变 readable operation 字段集合；只收紧 readable schema、MCP outputSchema 示例和相关验证材料中的 warning item envelope。
- 不新增、迁移或验收核心 `docnav adapter list/install/update/remove` 管理命令；adapter 管理命令继续由 `implement-docnav-adapter-management` change 拥有，本 change 只在命令族矩阵中记录其 owner、边界和非验收状态。

## 能力变更

### 新增能力

- 无。

### 修改能力

- `v0-contract-documentation`：说明 `clap` 是本 change 覆盖 Rust CLI 入口的 argv 结构解析基础，canonical document operation input 或等价 semantic request 是传输层之后的内部语义模型，并记录 strict invoke 边界、MCP ownership 和 stable warning envelope。
- `docnav-core-cli-routing-output-implementation`：将 core CLI 兼容从精确 token 归属调整为语义成功优先，并增加 core CLI help 验收。
- `protocol-and-adapter-sdk-implementation`：将 adapter direct CLI 解析调整为 `clap` 优先，要求 direct CLI 与有效 invoke request 共享文档操作归一和执行路径，并保留 strict invoke 边界。
- `markdown-adapter-v0-implementation`：调整 Markdown CLI smoke 和矩阵验证，覆盖 help、宽松 argv 成功路径、必要失败、schema 边界和 stable warning envelope，不再断言精确 argv token 细节。

## Impact

- Rust 依赖：在 workspace 或相关 crate 中添加 `clap`，只启用实现需要的 feature。
- 代码：更新 core CLI parser、adapter SDK direct CLI parser、valid invoke 到文档操作输入或等价 semantic request 的映射、Markdown native option 映射和 help 输出。
- 测试：更新 core CLI smoke、adapter SDK 单元测试和 Markdown smoke/matrix，断言 operation-first 参数校验、语义成功、必要失败、stable warning envelope、readable-json warning 必须性、core non-document 命令代表性行为和 stdout 通道边界。
- 主文档：更新 `docs/cli.md`、`docs/adapter-contract.md` 和 `docs/testing.md`，说明命令族矩阵、宽松 argv 行为、canonical document operation input 或等价 semantic request、invoke strict 边界、warning envelope 和 help 验收。
- Schema 和示例：更新 `docs/schemas/readable-common.schema.json`、MCP outputSchema 示例和 `docs/examples/json/mcp-*-tool.json`，表达 stable warning envelope、当前 family marker、稳定 effect 和 family-specific details；protocol response、manifest 和 probe schema 不增加 CLI warning 字段。
