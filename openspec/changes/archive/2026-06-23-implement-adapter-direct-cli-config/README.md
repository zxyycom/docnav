# implement-adapter-direct-cli-config

为 adapter direct CLI 规定 JSON 配置文件读取、诊断和合并方案：SDK 提供项目级默认配置路径、默认用户配置目录参数、用户级默认配置路径与覆盖参数，并按“显式 argv > 项目级 adapter 配置 > 用户级 adapter 配置 > 内置默认值”合并为标准 direct CLI 参数来源对象。

本 change 定义配置源读取、路径覆盖、配置读取 warning、字段投影、合并优先级，以及 Markdown adapter 配置 schema/example 的参考边界。

## Implementation Status

- Readiness gate 已完成，本 change 可作为正式实现计划执行。
- Capability 复用 `adapter-protocol`、`docnav-contracts` 和 `markdown-navigation`，未新增一次性 capability。
- Core `docnav`、`docnav-mcp` 和 adapter `invoke` 不读取 adapter direct CLI 配置。
- Adapter direct CLI 项目根发现、默认用户配置目录 fallback、配置源跳过 warning shape、配置字段投影语义和 schema/example 参考边界已在 delta specs 中明确。
- 实现阶段按 `tasks.md` 同步 owner 主规范、代码、schema/example、测试和验证材料。
