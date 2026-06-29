本 change 目标是统一分页默认配置：入口配置表达是否启用 pagination，以及启用时提供给 adapter 的 numeric `limit`。Adapter `invoke` protocol 继续接收显式 `limit` 与 `page`。

## Why

当前分页默认值只能表达预算数字，不能表达入口侧默认分页是否启用；历史命名也容易把预算理解为字符单位。需要把默认配置收敛为 pagination 语义组，再由 adapter 决定 `limit` 数字的实际含义。

## What Changes

- 将分页默认配置从 flat budget default 迁移到 `defaults.pagination.enabled` 与 `defaults.pagination.limit`。
- 将 CLI 显式分页参数收敛为 `--pagination enabled|disabled` 与 `--limit <n>`；`--limit` 只表示本次调用提供一个正整数预算。
- Core 和 adapter SDK direct CLI 使用同一 pagination 参数来源、优先级、校验和 disabled 归一规则，并在 request construction 或 operation handler 前产出最终 `limit` 与 `page`。
- 当最终 pagination disabled 时，入口把 outgoing `limit` 归一为 protocol 正整数域可表示的最大预算，不向 protocol request 增加单独的 `pagination` 字段。
- `limit` 的单位和解释权属于 adapter；本 change 不开放用户配置预算单位。
- `page` 仍不是配置默认值，入口省略时从 `1` 开始。
- Adapter `invoke` 的 request `arguments` 作为 direct input 进入同一标准参数流程；注册的配置/defaults 可以补足缺失的分页参数，但不回写原始 protocol request。

## Capabilities

### New Capabilities

- 无新增 capability ID。

### Modified Capabilities

- `core-cli`: core 配置、CLI argv 和 invoke request construction 使用通用 pagination limit。
- `adapter-protocol`: adapter SDK direct CLI 使用同一 pagination 参数来源模型。
- `markdown-navigation`: Markdown direct CLI 配置示例和测试跟随 SDK pagination limit 规则。

## Impact

- 影响 core config、document command parser、help/default 文案、invoke request construction 和相关测试。
- 影响 adapter SDK direct CLI config projection、argv parser、参数来源合并、disabled finalization 和测试。
- 影响 Markdown adapter config schema/example、smoke/matrix 测试和文档说明。
- 需要同步 `standard-parameters`、CLI、adapter contract、Markdown adapter、schema/example 和测试说明中对 `defaults.limit` 或旧预算字段的描述。
- Protocol request 继续使用既有 `arguments.limit` 与 `arguments.page`；只在现有 schema/example 仍残留旧预算字段或配置形状时更新验证材料。
