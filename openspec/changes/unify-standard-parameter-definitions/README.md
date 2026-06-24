# unify-standard-parameter-definitions

统一 core `docnav`、`docnav-adapter-sdk` direct CLI、adapter `invoke` 和 MCP tool mapping 的标准参数共享基础层。

本 change 规划共享 Rust 参数实现，并新增 `docs/standard-parameters.md` 作为标准参数机制 owner。共享机制统一声明参数身份、入口字段映射、配置字段映射、来源标记、合并顺序、透传策略、validation、operation binding 和 MCP metadata。CLI argv、MCP tool input、invoke request arguments、项目配置、用户配置和默认值先归一为标准参数来源，再按固定顺序合并；未映射输入由入口策略决定保留、丢弃或交给 owner 校验。Protocol request/result envelope 不变，但 `arguments` 的标准参数语义从调用方最终参数改为 adapter `invoke` 的显式输入。`docs/architecture.md`、`docs/cli.md`、`docs/adapter-contract.md`、`docs/mcp.md` 和 `docs/protocol.md` 只同步各自消费边界，不重新定义共享标准参数规则。

关键决策记录在 [design.md](design.md#key-decision-log)，并以 D1-D7 编号供 tasks 和 review 引用。后续若改变任一决策，必须同步更新 proposal、spec delta、tasks 和对应验证材料。
