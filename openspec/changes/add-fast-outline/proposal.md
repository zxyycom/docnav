## 一句话核心

为 `docnav` 增加 fast outline：小文档直接读取全文，大文档继续返回 outline 导航。

## 文档状态

当前文档是未审计的临时 change 草案，只用于收敛需求和实现方向；实现前必须先执行审计任务，不能把本文中的方案表述直接当作已确认规范或最终技术结论。

## Why

小文档通常可以在一次字符预算内读完，强制用户或 agent 先执行 `outline -> ref -> read` 会增加不必要的往返。fast outline 让小文档直接进入可读内容，同时保留大文档的结构化导航流程。

## What Changes

- 新增 `fast-outline` 能力，用于表达“先按 outline 意图进入文档；如果文档足够小，则直接读取全文”。
- 小文档满足直接读取条件时，输出 read 结果，而不是 outline 列表。
- 不满足直接读取条件时，保持现有 outline 行为，包括 ref、分页、adapter 选择和输出模式。
- 通过 MCP 暴露对应 fast outline 能力，并保持 `docnav-mcp` 只映射到核心 `docnav`，不复制解析逻辑。
- adapter 仍只暴露现有 `outline` 和 `read` operation，不新增 adapter invoke operation。
- outline 结果需要提供可选的全文 ref 元数据，使 `docnav` 能原样调用 read；`docnav` 不拼接、不解析、不猜测 adapter ref。
- 格式 adapter 不承担跨格式路由、小文档策略或 MCP 展示职责。

## Capabilities

### New Capabilities
- `fast-outline`: 定义 CLI/MCP 层的快速入口行为：大文档展示导航，小文档直接读取。

### Modified Capabilities
- 无。

## Impact

- 受影响可执行文件：`docnav` 核心 CLI。
- 受影响桥接层：`docnav-mcp` 需要新增或映射 fast outline 工具。
- 受影响输出层：默认文本、`readable-json`、MCP TextContent 和 structuredContent。
- 受影响文档与校验材料：`docs/cli.md`、必要时的 `docs/architecture.md`、readable schema、示例和测试。
- 协议影响：不新增 adapter `invoke` operation；`outline` 结果增加可选全文 ref 元数据。
- adapter 影响：adapter 可通过返回自身拥有的全文 ref 元数据支持 direct-read fast outline；未返回该字段的 adapter 仍通过普通 outline 回退工作。
