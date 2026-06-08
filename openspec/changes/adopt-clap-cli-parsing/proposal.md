**核心决策：** 所有 Rust CLI argv 入口以 `clap` 作为解析基础。文档操作入口在各自传输层解析成功后进入 canonical document operation input。CLI argv 对未知或无关 token 保持宽松；adapter `invoke` JSON 和 schema 入口保持严格。阅读层 warning 使用稳定 warning envelope。本 change 只修改 `openspec/changes/adopt-clap-cli-parsing/` 下的临时 OpenSpec artifacts。

## 为什么

Docnav 是 CLI-first 的结构化文档导航系统，使用者包括人类和 AI agent。当前核心 `docnav` 和 adapter direct CLI 的 Rust 参数解析依赖手写 token loop。这些 loop 同时处理命令识别、类型校验、兼容容错、warning 归属和 token 消费细节，导致行为审计成本偏高。

旧 direct CLI 兼容契约过度约束了 warning 细节，例如精确 `ignored_tokens` 分组、细粒度 `kind` 值和 `reason` 文案。这些细节不是 `outline -> ref -> read` 的核心产品契约，却会把测试绑定到特定 parser 实现，增加迁移到 `clap` 的难度。

本 change 保留“AI 友好的一次成功调用”价值，但收窄稳定契约。Rust CLI 入口通过 `clap` 描述固定命令、flag、默认值、枚举和 help。文档操作入口完成传输层解析后，映射为 canonical document operation input，并共享语义归一、容错校验和 operation 执行。CLI argv 可以收集未知或无关输入；adapter `invoke` JSON 必须在进入文档操作管道前拒绝 malformed JSON、未知字段和错误类型。

`docnav-mcp` 保持 Node.js bridge 边界：它把 MCP tool call 映射到核心 `docnav` CLI，并包装 TextContent/structuredContent 输出；它不拥有 adapter SDK 解析、adapter `invoke`、格式解析或 Rust argv 行为。

## 变更内容

- 将核心 `docnav`、adapter direct CLI 和后续 Rust CLI argv 入口的首选解析基础统一为 `clap`。
- 明确入口分层：
  - CLI argv 由 `clap` 加 Docnav 受控宽松收集解析。
  - Adapter `invoke` JSON 由严格 schema、字段和类型校验解析。
  - 成功解析的文档操作输入先归一到 canonical document operation input，再构造 request 并执行。
- 将 CLI 兼容从“精确 token 归属”调整为“语义成功优先”：
  - 当前 operation 的必需语义输入存在且实际使用参数有效时，未知 flag、多余 positional 和当前 operation 不使用的参数不作为成功路径主失败原因。
  - 当前 operation 实际使用的参数仍保持严格。
- 保留必要失败：malformed JSON、schema/type/field 错误、缺少必需 `path`/`ref`/`query`、实际使用参数非法、文档/ref/格式错误和 adapter/protocol 错误都必须清晰失败。
- 统一 readable warning：
  - 每个 `warnings[]` item 使用稳定 envelope，包含 `kind`、非空 `reason`、`ignored_tokens: string[]` 和可选 family-specific 字段。
  - 非 argv warning 的 `ignored_tokens` 为 `[]`。
  - 当前稳定 warning family 至少包括 `cli_argv_ignored` 和 `adapter_candidate_failure`。
  - `adapter_candidate_failure` 保留 `adapter_id`、`stage`、`code` 等 candidate 字段。
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

## 能力变更

### 新增能力

- 无。

### 修改能力

- `v0-contract-documentation`：说明 `clap` 是 Rust CLI 解析基础，canonical document operation input 是传输层之后的共享模型，并记录 strict invoke 边界、MCP ownership 和 stable warning envelope。
- `docnav-core-cli-routing-output-implementation`：将 core CLI 兼容从精确 token 归属调整为语义成功优先，并增加 core CLI help 验收。
- `protocol-and-adapter-sdk-implementation`：将 adapter direct CLI 解析调整为 `clap` 优先，要求 direct CLI 与有效 invoke request 共享文档操作归一和执行路径，并保留 strict invoke 边界。
- `markdown-adapter-v0-implementation`：调整 Markdown CLI smoke 和矩阵验证，覆盖 help、宽松 argv 成功路径、必要失败、schema 边界和 stable warning envelope，不再断言精确 argv token 细节。

## 影响范围

- Rust 依赖：在 workspace 或相关 crate 中添加 `clap`，只启用实现需要的 feature。
- 代码：更新 core CLI parser、adapter SDK direct CLI parser、valid invoke 到文档操作输入的映射、Markdown native option 映射和 help 输出。
- 测试：更新 core CLI smoke、adapter SDK 单元测试和 Markdown smoke/matrix，断言语义成功、必要失败、stable warning envelope 和 stdout 通道边界。
- 主文档：更新 `docs/cli.md`、`docs/adapter-contract.md` 和 `docs/testing.md`，说明宽松 argv 行为、canonical document operation input、invoke strict 边界、warning envelope 和 help 验收。
- Schema 和示例：更新 `docs/schemas/readable-common.schema.json`、MCP outputSchema 示例和 `docs/examples/json/mcp-*-tool.json`，表达 stable warning envelope 和当前 family marker；protocol response、manifest 和 probe schema 不增加 CLI warning 字段。
