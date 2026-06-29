本 design 记录 pagination 默认配置的目标决策。它只约束入口配置、CLI argv、标准参数来源合并和进入 adapter 前的最终分页参数；protocol request/result shape 继续由 protocol owner 承接。

## Context

分页默认值需要表达两个概念：是否启用分页，以及启用时提供给 adapter 的 numeric budget。预算数字必须进入统一 `limit` 参数；core 和 SDK 只负责来源合并、正整数校验和入口侧 disabled 归一，预算单位和切分策略由 adapter 的导航策略解释。

## Goals / Non-Goals

**Goals:**

- 在 core 和 adapter SDK direct CLI 中统一 `defaults.pagination.enabled` 与 `defaults.pagination.limit`。
- 让 `--pagination enabled|disabled` 和 `--limit <n>` 映射到同一 pagination 参数来源模型。
- 写清 `enabled` 与 `limit` 分别来自不同来源时的合并、覆盖和最终归一规则。
- 在进入 adapter 前完成 enabled/limit 的默认值解析和 disabled 归一。
- 保持 protocol request shape 为显式 `limit` 和 `page`，不增加 protocol `pagination` 字段。
- 保持 `page` 不可配置，入口省略时仍固定为 `1`。
- 保持 adapter `invoke` request `arguments` 作为 direct input 进入标准参数流程，并允许注册的配置/defaults 补足缺失分页参数。

**Non-Goals:**

- 不开放用户选择 `limit` 单位或 budget function。
- 不要求 core 或 SDK 理解 adapter 如何解释 `limit`。
- 不把 adapter native options 提升为 core pagination 配置。
- 不重新定义 protocol request envelope、operation 或 result shape。

## Decisions

### Decision 1: `limit` 是 adapter-owned numeric budget

Core 和 SDK 只校验 `limit` 是正整数并按来源优先级解析默认值。Adapter 决定该数字代表字符、token、条目、行数或其它稳定预算函数，并在 adapter owner 文档中声明。

### Decision 2: pagination 默认值由 `defaults.pagination` 拥有

`defaults.pagination.enabled` 表示入口侧默认分页状态，`defaults.pagination.limit` 表示启用时提供给 adapter 的正整数预算。目标配置形状由该 pagination group 承接；schema、example、help 和主规范中的当前 `defaults.limit` 描述需要随本 change 迁移。CLI argv、项目配置、用户配置和内置默认值都投影到同一参数来源模型。

### Decision 3: numeric 默认值保持现有 owner 语义

本 change 改变默认配置形状和来源模型，不改变 numeric budget 的现有默认值。实施时应把现有默认 `limit` 数值迁移到 `defaults.pagination.limit` 的 built-in default；如果需要改变数值，必须作为单独决策同步 owner 主规范、schema/example 和测试。

### Decision 4: disabled 只在入口归一为最大 limit

当最终 `enabled=false` 时，core 或 SDK document entry 在进入 adapter operation handler 前把最终 `limit` 归一为标准参数/typed validation owner 定义的最大正整数预算。Adapter operation 仍只看到最终 numeric budget 和 page；protocol request 不携带 `enabled` 或 `pagination` 字段。

### Decision 5: protocol shape 保持现有 `limit` 与 `page`

本 change 不新增 protocol 字段，也不把入口侧 `enabled` 序列化给 adapter。现有 protocol owner 已定义 `arguments.limit` 与 `arguments.page`；本 change 只负责保证 core、SDK direct CLI 和 SDK `invoke` entry 在进入 adapter operation handler 前产出符合标准参数和协议约束的最终值。

### Decision 6: 配置迁移采用 hard switch

`defaults.pagination.limit` 是迁移后的唯一配置路径。Core、SDK、schema、example、help 和 typed config 只声明 `defaults.pagination` group；`defaults.limit` 不形成 `LIMIT` 参数来源。仍使用旧字段的配置按现有 unknown-field、schema 或 typed validation 路径失败，这是 hard switch 的可观察行为。

### Decision 7: `enabled` 与 `limit` 永远按标准参数身份独立合并

`pagination.enabled` 与 `limit` 分别拥有独立标准参数 identity。两个 identity 都按 direct input、project config、user config、built-in default 的通用优先级合并；`--limit` 只提供本次调用的 numeric budget，`--pagination` 只提供本次调用的分页状态。来源合并完成后，入口根据最终 `enabled` 执行 disabled finalization；当最终 `enabled=false` 时，operation 入参中的 `limit` 被最大 limit 覆盖，即使该 `limit` 来自更高优先级来源。

### Decision 8: disabled finalization 的最大 limit 常量归 page/pagination 运行时所有

`enabled=false` 使用的最大正整数 limit 是进入 adapter 前的运行时 finalization 值，不是字段默认值。该命名常量由 page/pagination 参数解析或 finalization 模块承接，并由 core 和 SDK 复用。字段定义、schema facet 和 typed config validation 只校验用户输入和配置中的 `limit` 是正整数；disabled finalization 负责把已合并的 operation limit 覆盖为该最大正整数。

### Decision 9: 文本预算 helper 返回最小切分边界

`text_budget` helper 只承接字符预算测量和文本切分位置计算，不拥有 page、continuation 或 adapter pagination 语义。Helper 输入是有序字符串数组、字符预算和切分边界；切分边界只支持 `element` 与 `character`。输出是切分位置，表达为命中的元素索引和该元素内的 Unicode 字符偏移。`element` 只允许在输入元素边界切分；`character` 允许在元素内部按 Unicode 字符边界切分。若 `boundary=element` 且第一个元素已经超过字符预算，helper 返回失败。

## Risks / Trade-offs

- 不暴露单位会降低跨 adapter 可比性，但保留 adapter-owned navigation 的弹性。
- hard switch 会影响现有 `defaults.limit` 配置文件，但避免长期维护两个配置入口和迁移歧义。
- `enabled` 和 `limit` 的独立 identity 需要矩阵测试证明标准参数合并顺序和 disabled finalization 时点。
- disabled finalization 依赖一个命名的最大 numeric limit，需要由 page/pagination finalization owner 定义，并用测试证明分页不会循环或停滞。
- `text_budget` 只能提供可复用的字符测量与切分位置；adapter owner 仍负责声明自己的 `limit` 单位和 pagination 策略。
- `--pagination` 是入口侧状态；测试需要证明它不会泄漏为 protocol `pagination` 字段，adapter 只接收最终 `limit` 和 `page`。

## Open Questions

当前无未回答问题。已收敛决策：

- `defaults.pagination.limit` 是唯一配置路径；旧 `defaults.limit` 按 hard switch 的诊断路径失败。
- `enabled` 与 `limit` 始终按标准参数 identity 独立合并，来源合并完成后再执行 disabled finalization。
- disabled finalization 的最大正整数常量归 page/pagination finalization owner，不归字段定义或默认值 owner。
