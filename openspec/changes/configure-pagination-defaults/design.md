本 design 记录通用 pagination limit 默认配置的初始设计方向；当前只在 `openspec/changes/configure-pagination-defaults/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

分页默认值需要表达两个概念：是否启用分页，以及启用时提供给 adapter 的 numeric budget。预算数字的单位不应由 core 或 SDK 解释，而应由 adapter 的导航策略解释。

## Goals / Non-Goals

**Goals:**

- 在 core 和 adapter SDK direct CLI 中统一 `defaults.pagination.enabled` 与 `defaults.pagination.limit`。
- 让 `--pagination enabled|disabled` 和 `--limit <n>` 映射到同一 pagination 参数来源模型。
- 在进入 adapter 前完成 enabled/limit 的默认值解析和 disabled 归一。
- 保持 `page` 不可配置，入口省略时仍固定为 `1`。
- 保持 adapter `invoke` 不读取 direct CLI 配置。

**Non-Goals:**

- 不开放用户选择 limit 单位或 budget function。
- 不要求 core 理解 adapter 如何解释 limit。
- 不把 adapter native options 提升为 core pagination 配置。
- 不在本 change 中决定协议字段迁移的全部兼容策略。

## Decisions

### Decision 1: `limit` 是 adapter-owned numeric budget

Core 和 SDK 只校验 `limit` 是正整数并按来源优先级解析默认值。Adapter 决定该数字代表字符、token、条目、行数或其它稳定预算函数。

### Decision 2: pagination 默认值由 `defaults.pagination` 拥有

`defaults.pagination.enabled` 表示本次是否启用分页默认预算，`defaults.pagination.limit` 表示启用时提供给 adapter 的正整数预算。CLI argv、项目配置、用户配置和内置默认值都投影到同一参数来源模型。

### Decision 3: disabled 只在入口归一为最大 limit

当最终 `enabled=false` 时，core 或 SDK direct CLI 在进入 adapter 前把最终 `limit` 归一为协议正整数域可表示的最大值。Adapter 仍只看到显式 numeric budget 和 page。

### Decision 4: protocol 字段迁移由结构化协议 change 确认

本 change 以 `limit` 为目标字段名，但实际 schema、example 和兼容路径需要等待 `explore-structured-protocol-fields` 确认。

## Risks / Trade-offs

- 不暴露单位会降低跨 adapter 可比性，但保留 adapter-owned navigation 的弹性。
- disabled 归一依赖足够大的 numeric limit，仍需要测试避免分页循环或不可前进。
- 协议字段迁移会影响 examples、schema、core、SDK 和 adapter fixture，需要在协议探索收敛后细化任务。

## Open Questions

- `limit_chars` 到 `limit` 是否需要双字段兼容阶段？
- 默认 `limit` 值是否继续沿用当前数值？
- 哪些 readable 文案需要解释 adapter-owned limit？
