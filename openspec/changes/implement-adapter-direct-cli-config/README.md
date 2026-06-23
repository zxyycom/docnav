# implement-adapter-direct-cli-config

为 adapter direct CLI 规定 JSON 配置文件读取、诊断和合并方案：SDK 提供项目级/用户级默认配置路径与覆盖参数，并按“显式 argv > 项目级 adapter 配置 > 用户级 adapter 配置 > 内置默认值”合并为标准 direct CLI 参数来源对象。

本 change 定义配置源读取、路径覆盖、配置读取 warning、字段映射、合并优先级，以及 Markdown adapter 配置 schema/example 的参考边界。

## Readiness Gate

- Capability 复用 `adapter-protocol`、`docnav-contracts` 和 `markdown-navigation`，未新增一次性 capability。
- Core `docnav`、`docnav-mcp` 和 adapter `invoke` 不读取 adapter direct CLI 配置。
- Adapter direct CLI 项目根发现、配置源跳过 warning shape、配置字段映射语义和 schema/example 参考边界已在 delta specs 中明确。
- 本准备阶段除 OpenSpec artifacts 外，只新增用户确认的 config schema/example 参考材料及 docs validator 绑定，不改主 specs 或实现代码。
