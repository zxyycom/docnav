本 change 只起草 core 与 adapter SDK 迁移到标准参数解析器的想法和审计入口；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Why

标准参数解析核心完成后，core CLI、adapter direct CLI 和 adapter invoke 需要逐步消费同一套 registration 与 typed runtime values。迁移不应继续塞进底层 definition change，否则审计和回滚边界会变得不清晰。

## What Changes

- 起草 core CLI 和 adapter SDK 消费 standard parameter resolver 的迁移计划。
- 保留现有 observable behavior：unknown argv、extra positional、unused operation flag 继续 warning，当前 operation 实际消费字段严格校验。
- 记录 help/default 文案应来自标准参数 metadata 的方向。
- 保留 adapter invoke 是独立入口的边界。
- 不替换 CLI frontend；`lexopt` 在独立 change 处理。

## Capabilities

### New Capabilities

- `standard-parameter-adoption`: core 和 adapter SDK 消费标准参数解析器的迁移边界。

### Modified Capabilities

当前草案不直接修改已归档主 spec；审计门禁会确认是否需要拆成 `core-cli` 和 `adapter-protocol` delta。

## Impact

- 未来会影响 `docnav` document operations、adapter direct CLI、adapter invoke request handling、help/default 文案和 tests。
- 当前 change 不改变现有 CLI parsing crate、协议 schema、输出模式或 adapter handler。
- 后续实现必须用局部 smoke/integration tests 证明行为保持或有意变化。
