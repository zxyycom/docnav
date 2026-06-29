本 change 目标是统一分页默认配置：入口配置表达是否启用 pagination，以及启用时提供给 adapter 的 numeric `limit`。Adapter `invoke` protocol 继续接收显式 `limit` 与 `page`。

## Why

当前配置默认值只表达 numeric `limit`，不能表达入口侧默认分页状态。默认配置需要收敛为 pagination 语义组；标准参数层只校验和合并正整数预算，`limit` 的单位和切分策略由最终 adapter owner 声明。

## What Changes

- 将分页默认配置迁移到 `defaults.pagination.enabled` 与 `defaults.pagination.limit`；当前 numeric 默认值继续由对应 owner 文档记录，除非实施任务明确修改该 owner。
- 将 CLI 显式分页参数定义为 `--pagination enabled|disabled` 与 `--limit <n>`；`--pagination` 控制入口侧分页状态，`--limit` 提供本次调用的正整数预算。
- Core 和 adapter SDK direct CLI 使用同一 pagination 参数身份、来源优先级、校验和 disabled 归一规则，并在 request construction 或 operation handler 前产出最终 `limit` 与 `page`。
- 当最终 pagination disabled 时，入口把 outgoing `limit` 归一为标准参数/typed validation owner 定义的最大正整数预算，不向 protocol request 增加单独的 `pagination` 字段。
- `limit` 的单位和解释权属于 adapter；本 change 不开放用户配置预算单位，也不把预算单位提升为 core/SDK 配置。
- `page` 仍不是配置默认值，入口省略时从 `1` 开始。
- Adapter `invoke` 的 request `arguments` 作为 direct input 进入同一标准参数流程；注册的配置/defaults 可以补足缺失的分页参数，但不回写原始 protocol request。

## Capabilities

### New Capabilities

- 无新增 capability ID。

### Modified Capabilities

- `core-cli`: core 配置、CLI argv 和 adapter request construction 使用通用 pagination 参数模型。
- `adapter-protocol`: adapter SDK direct CLI 与 `invoke` entry 使用同一 pagination 参数来源模型。
- `markdown-navigation`: Markdown direct CLI 配置示例、schema 和测试跟随 SDK pagination 参数规则。

## Impact

- 影响 core config、document command parser、help/default 文案、invoke request construction 和相关测试。
- 影响 adapter SDK direct CLI config projection、argv parser、参数来源合并、disabled finalization 和测试。
- 影响 Markdown adapter config schema/example、smoke/matrix 测试和文档说明。
- 需要同步 `standard-parameters`、CLI、adapter contract、Markdown adapter、schema/example 和测试说明中对 pagination 默认配置形状、来源优先级和 adapter-owned `limit` 的描述。
- Protocol request 继续使用既有 `arguments.limit` 与 `arguments.page`；protocol schema/example 只在描述或验证材料与该 shape 不一致时更新。
