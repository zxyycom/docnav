## 一句话核心

fast outline 由 `docnav` 负责选择“直接读”或“返回 outline”，adapter 只提供自身拥有的 ref 和现有 `outline`/`read` 能力。

## 文档状态

当前文档是未审计的临时设计草案。实现前必须先审计本文的协议影响、输出形状和 MCP 暴露方式；审计未完成前，不要把本文中的字段名、命令名或流程描述视为最终规范。

## Context

`docnav outline` 当前面向大文档导航：它返回紧凑 ref，调用方再通过 `docnav read` 读取具体内容。这个模型适合大文档，但对一次字符预算内可读完的小文档会产生多余步骤。

核心边界是 ref 所有权。ref 由 adapter 生成和解析，`docnav`、MCP 和其它接入层只能原样传递。fast outline 因此需要 adapter 暴露“全文 ref”元数据，同时由 `docnav` 保持 adapter 选择、分页、输出映射和 MCP 入口转换。

## Goals / Non-Goals

**Goals:**
- 新增核心入口 `docnav fast-outline <path>`，小文档返回全文内容，大文档返回普通 outline。
- 小文档策略、adapter 路由、默认参数和错误映射归 `docnav` 所有。
- 文档解析和全文 ref 生成归 adapter 所有。
- 通过专用 MCP tool 暴露等价 fast outline 行为，并映射到 `docnav`。
- 保持现有 `docnav outline` 行为不变。

**Non-Goals:**
- 不新增 adapter `invoke` operation；实现路径使用现有 `outline` 和 `read`。
- 不让 `docnav-mcp` 解析文档内容、判断文档大小或理解 ref 语法。
- 不让 `docnav` 理解 Markdown 私有 ref，例如 `doc:full`。
- 不以文件字节大小作为最终小文档判断； eligibility 以 adapter read 分页结果为准。

## Decisions

1. 新增核心命令，而不是改变 `outline`。

   `docnav fast-outline <path>` 让调用方显式选择“可能返回 read 或 outline”的混合结果，现有 outline 调用保持稳定。

   替代方案：让 `docnav outline` 自动读取小文档。该方案会影响期望只拿 outline entries 的调用方，也会让现有 readable schema 更难解释。

2. 从 adapter 输出获取全文 ref。

   `OutlineResult` 增加可选 `full_ref` 字段。adapter 能读取全文时填入该字段，`docnav` 将该 ref 原样传给 read，不构造、不解析、不规范化 ref。

   替代方案：在 `docnav` 中硬编码 Markdown 全文 ref。该方案违反 ref 边界，并且无法支持后续格式 adapter。

3. 用受限全文读取判断小文档。

   fast outline 先选择 adapter 并调用第一页 outline。若 outline 结果包含 `full_ref`，`docnav` 使用该 ref、第一页和 fast outline 字符预算调用 read。read 返回 `page: null` 时判定为小文档并返回 read-mode；read 返回非空下一页时判定为大文档并回退 outline-mode。

   替代方案：读取文件大小后决定是否 direct read。该方案成本低，但不反映 UTF-8 字符预算、adapter 解析后的内容范围和实际 read 分页。

4. fast outline readable 输出使用显式 mode。

   `readable-json` 和 MCP structuredContent 使用 `mode: "read"` 或 `mode: "outline"` 标识结果类型，并携带对应 payload。默认文本输出复用现有 read 或 outline 文本模板。

   替代方案：直接复用 read/outline schema，不加 mode。该方案要求客户端从字段集合推断结果类型，校验和展示都更脆弱。

## Risks / Trade-offs

- [Risk] 大文档可能触发两次 adapter 调用。-> Mitigation: 使用调用方字符预算做受限 read，read 有下一页时复用已获取的 outline 结果回退。
- [Risk] 未暴露 `full_ref` 的 adapter 无法直接读取小文档。-> Mitigation: 定义稳定回退路径，缺少 `full_ref` 时返回普通 outline。
- [Risk] `OutlineResult.full_ref` 会影响协议 schema 和示例。-> Mitigation: 字段保持可选，更新 schema、示例和语义校验，确保旧 outline 响应仍合法。
- [Risk] 混合结果不适合长期机器解析。-> Mitigation: 保持 `docnav outline` 不变，fast outline 使用专用命令、专用 MCP tool 和显式 readable schema。

## Migration Plan

1. 审计本 change 的命令名、字段名、输出形状、MCP tool 名称和协议边界。
2. 增加 `OutlineResult.full_ref` 的可选协议/readable schema 支持。
3. 更新 Markdown adapter，使 outline 输出包含 adapter 自有全文 ref。
4. 实现 `docnav fast-outline`，复用现有 adapter 选择和 invoke 管线。
5. 增加 `document_fast_outline` MCP tool，映射到 `docnav fast-outline`。
6. 更新文档、示例和测试。

Rollback: 隐藏或移除新的 `fast-outline` 命令和 MCP tool；现有 `outline` 和 `read` 行为保持不变。

## Open Questions

- `docnav fast-outline --output protocol-json` 应返回底层选中的 protocol envelope，还是拒绝该输出模式以避免暗示存在 `fast-outline` 协议 operation？
- fast outline 应复用普通 `limit_chars` 默认值，还是增加独立的 `fast_outline_limit_chars` 配置？
