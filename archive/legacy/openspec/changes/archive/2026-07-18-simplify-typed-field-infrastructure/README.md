# simplify-typed-field-infrastructure

本 change 的主目标是重做参数执行边界，而不是替换 typed-field：core 定义本 release 接受的全部文档操作参数，并调用 typed-field 完成输入提取、来源合并、默认值、类型 materialization 和 core 选择执行的校验；selected adapter 只接收 core 定义的 standard operation input，并实现格式策略。

Adapter 不再声明、注册或发现自己需要哪些调用方参数。它仍可在策略函数中校验或重复校验已经标准化的值；校验发生在 core、adapter 或两处，都不改变参数定义权只属于 core。

> 状态：方向、契约和执行检查点已收敛，可进入实现；尚未修改生产代码。

## Decision

- Core-owned closed catalog 是调用方可配置参数的唯一接受面；通用参数不带 adapter 标记，format-specific 参数带一个精确的 static adapter-id 标记。
- Config validation 与 selected-operation resolution 从同一 catalog 派生：前者使用完整 catalog projection，后者再按 selected adapter id 和 operation binding 过滤。
- Standard operation input 是 operation-specific closed typed contract；navigation 从 resolution result 构造它，adapter strategy 只消费该输入。
- Core 必须完成标准类型 materialization，可按字段执行无、极简或完整的 adapter 语义校验；adapter 策略可执行或重复执行其算法所需校验。
- Adapter definition 只提供 routing、capability 与 strategy facts；调用方参数事实只来自 core catalog。
- `FieldDef`、四种 `MergeStrategy`、ordered multi-source resolution、env extraction、validation、defaults、typed materialization 与 provenance 保留。
- Runtime 最终只保留一条 catalog → typed-field → standard input → strategy 路径；额外静态清理在主路径等价后独立验收，不能改变该契约。
- Env locator 表示该字段已启用 env source；未启用字段保持 `explicit > project > user > built_in`，后续启用字段使用 `explicit > env > project > user > built_in`。

## Reading Order

1. [`proposal.md`](proposal.md)：判断为什么做、改什么和承担什么代价。
2. [`design.md`](design.md)：实现 core/adapter 边界、两个 checkpoint 与风险控制。
3. [`specs/`](specs/)：核对归档后生效的可验证契约。
4. [`tasks.md`](tasks.md)：按状态转换实施和验收。
5. [`type-field-maintenance-report.md`](type-field-maintenance-report.md)：需要复核取舍依据时查看调用链与消费者证据。
