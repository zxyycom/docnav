本 design 记录 pagination 默认配置的目标决策。它只约束入口配置、CLI argv、标准参数来源合并和进入 adapter 前的最终分页参数。

## Context

分页默认值需要表达两个概念：是否启用分页，以及启用时提供给 adapter 的 numeric budget。预算数字必须进入统一 `limit` 参数，但它的单位和切分策略不应由 core 或 SDK 解释，而应由 adapter 的导航策略解释。

## Goals / Non-Goals

**Goals:**

- 在 core 和 adapter SDK direct CLI 中统一 `defaults.pagination.enabled` 与 `defaults.pagination.limit`。
- 让 `--pagination enabled|disabled` 和 `--limit <n>` 映射到同一 pagination 参数来源模型。
- 在进入 adapter 前完成 enabled/limit 的默认值解析和 disabled 归一。
- 保持 protocol request shape 为显式 `limit` 和 `page`，不增加 protocol `pagination` 字段。
- 保持 `page` 不可配置，入口省略时仍固定为 `1`。
- 保持 adapter `invoke` request `arguments` 作为 direct input 进入标准参数流程，并允许注册的配置/defaults 补足缺失分页参数。

**Non-Goals:**

- 不开放用户选择 limit 单位或 budget function。
- 不要求 core 理解 adapter 如何解释 limit。
- 不把 adapter native options 提升为 core pagination 配置。
- 不重新定义 protocol request envelope、operation 或 result shape。

## Decisions

### Decision 1: `limit` 是 adapter-owned numeric budget

Core 和 SDK 只校验 `limit` 是正整数并按来源优先级解析默认值。Adapter 决定该数字代表字符、token、条目、行数或其它稳定预算函数。

### Decision 2: pagination 默认值由 `defaults.pagination` 拥有

`defaults.pagination.enabled` 表示本次是否启用分页默认预算，`defaults.pagination.limit` 表示启用时提供给 adapter 的正整数预算。目标配置形状由该 pagination group 承接；schema、example、help 和主规范中的 flat `defaults.limit` 描述需要随本 change 迁移或明确兼容策略。CLI argv、项目配置、用户配置和内置默认值都投影到同一参数来源模型。

### Decision 3: disabled 只在入口归一为最大 limit

当最终 `enabled=false` 时，core 或 SDK document entry 在进入 adapter operation handler 前把最终 `limit` 归一为协议正整数域可表示的最大值。Adapter operation 仍只看到最终 numeric budget 和 page；protocol request 不携带 `enabled` 或 `pagination` 字段。

### Decision 4: protocol shape 保持现有 `limit` 与 `page`

本 change 不新增 protocol 字段，也不把入口侧 `enabled` 序列化给 adapter。现有 protocol owner 已定义 `arguments.limit` 与 `arguments.page`；本 change 只负责保证 core、SDK direct CLI 和 SDK `invoke` entry 在进入 adapter operation handler 前产出符合标准参数和协议约束的最终值。

## Risks / Trade-offs

- 不暴露单位会降低跨 adapter 可比性，但保留 adapter-owned navigation 的弹性。
- 从 flat `defaults.limit` 迁移到 `defaults.pagination.limit` 会影响现有配置文件，需要在实施前确定 hard switch、过渡 alias 或诊断策略。
- disabled 归一依赖足够大的 numeric limit，仍需要测试避免分页循环或不可前进。
- `--pagination` 是入口侧状态；测试需要证明它不会泄漏为 protocol `pagination` 字段，adapter 只接收最终 `limit` 和 `page`。

## Open Questions

- `defaults.limit` 到 `defaults.pagination.limit` 采用 hard switch、过渡 alias 还是诊断提示？
- disabled 归一使用的最大正整数常量由哪个 schema facet 或 typed field definition 承接？
- 默认 `limit` 值是否继续沿用当前数值？
- 哪些 CLI/help、schema/example 和 readable 文案需要解释 adapter-owned limit？
