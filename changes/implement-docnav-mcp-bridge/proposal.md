# Proposal

本计划在核心契约稳定且产品方向恢复该工作后，交付一个格式无关的 `docnav-mcp` stdio bridge；当前按[核心契约稳定后再扩张接入与交互面](../../docs/decisions/product-direction/stabilize-core-before-entrypoint-expansion.md)暂停，而不是退回只有方向的 Draft。

## Why

MCP client 需要稳定的 document tools，但格式识别、adapter selection、配置、错误映射和 document operations 已由核心 `docnav` CLI 拥有。Bridge 若复制这些语义，会形成第二套 contract；若在 find、protocol 和 output 仍变化时实施，又会扩大返工面。

## Outcome

产品恢复后，用户可安装并启动 `docnav-mcp`，通过 `document_outline`、`document_read`、`document_find` 和 `document_info` 调用核心 CLI；每个 tool 都从 `docnav --output protocol-json` 的同一结果事实生成受 schema 约束的 `structuredContent` 和精简 `TextContent`。

## Scope

- 纳入：Node.js package 与 bin、stdio MCP transport、四个 document tools、参数到 CLI argv 的映射、tool-owned output schemas、protocol success/failure 到 MCP content 的转换、package 和端到端验证。
- 不纳入：格式解析、adapter 路由或管理、配置解析、ref 解释、重复 readable-view framing、直接调用 adapter、完整 protocol envelope 暴露或常驻 core service。
- 当前暂停只阻止实施；恢复时先确认基础契约稳定并从届时 Current owner 重新基线。

## Success Criteria

- MCP client 能列出并调用四个 tools，输入字段只映射到对应 `docnav` operation 和 `--output protocol-json`。
- `structuredContent` 通过随包 tool schema 校验，只包含 tool-owned result/error facts；`TextContent` 使用同一事实且不发起第二次 CLI 调用。
- Explicit adapter id、automatic routing、ref、diagnostic 和成功 stderr 状态仍由核心 CLI 语义决定，bridge 不复制解释逻辑。
- 离线 package、参数映射、schema、failure 和真实 CLI 端到端验证通过。

## Affected Owners

- [CLI](../../docs/cli.md)、[Navigation Input Resolution](../../docs/navigation-input-resolution.md)、[原始协议](../../docs/protocol.md)和[输出模式](../../docs/output.md)：实施期间作为 bridge 输入与映射边界的 Current 基线；行为证据成立后再同步实际新增的 Current surface。
- [Schema](../../docs/schemas/json-schema.md)与[契约示例](../../docs/examples/contract-examples.md)：实施期间用于确认 protocol result、failure 与 tool-owned output schema 的可追溯映射，不把 legacy shape 或 Target 当成 Current。
- 本 design 登记新增 MCP bridge/package owner 的预期 delta，包括 tool schema、MCP TextContent、structuredContent、transport 与分发；只有实现与行为证据通过后，才把相应 delta 写成 Current。
- [测试策略](../../docs/testing.md)和[发布包验证](../../docs/testing/release.md)：映射、真实 CLI handoff 和离线 package 证据。
