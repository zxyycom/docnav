本 change 目标是统一分页默认配置，并把分页预算默认值收敛为 adapter-owned numeric `limit`；本文档只是 `openspec/changes/configure-pagination-defaults/` 下的未审核临时 proposal，不影响现有其它文档或主规范。

## Why

当前分页默认值与字符预算字段耦合过强，不利于 adapter 自行解释分页预算。需要先把默认配置表达成通用 pagination limit，再由 adapter 决定该数字的实际含义。

## What Changes

- 将分页默认配置收敛到 `defaults.pagination.enabled` 与 `defaults.pagination.limit`。
- 将 CLI 显式预算参数收敛为 `--limit <n>`，只表示本次调用提供一个正整数预算。
- Core 和 adapter SDK direct CLI 使用同一 pagination 参数来源、优先级、校验和 disabled 归一规则。
- `limit` 的单位和解释权属于 adapter；本 change 不开放用户配置预算单位。
- `page` 仍不是配置默认值，入口省略时从 `1` 开始。
- Adapter `invoke` 仍接收显式 protocol request，不读取 direct CLI 配置。

## Capabilities

### New Capabilities

- 无新增 capability ID。

### Modified Capabilities

- `core-cli`: core 配置、CLI argv 和 invoke request construction 使用通用 pagination limit。
- `adapter-protocol`: adapter SDK direct CLI 使用同一 pagination 参数来源模型。
- `markdown-navigation`: Markdown direct CLI 配置示例和测试跟随 SDK pagination limit 规则。

## Impact

- 影响 core config、document command parser、help/default 文案和相关测试。
- 影响 adapter SDK direct CLI config projection、argv parser、参数来源合并和测试。
- 影响 Markdown adapter config schema/example、smoke/matrix 测试和文档说明。
- 依赖 `structure-protocol-fields-and-readable-output` 定义 protocol request 中 `limit` 的最终字段迁移方式。
