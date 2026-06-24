本 change 只起草标准参数解析核心的想法和审计入口；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Why

标准参数需要从 CLI argv、adapter invoke arguments、项目配置、用户配置和默认值中形成 typed runtime values。这个来源合并问题应与 typed field 的字段 metadata 分离，也不应和 core/SDK 迁移绑在同一批实现里。

## What Changes

- 新增标准参数 resolution core 草案，基于 typed field definition 消费字段 metadata。
- 记录 direct/project config/user config/default 的来源模型、合并顺序、source info 和 passthrough policy。
- 保留 operation argument binding 的方向，但不迁移任何现有 core CLI 或 adapter SDK 行为。
- 不决定具体 CLI frontend，不引入 `lexopt`。
- 不修改 `unify-standard-parameter-definitions`。

## Capabilities

### New Capabilities

- `standard-parameter-resolution`: 标准参数来源映射、合并、typed runtime value 和 passthrough 的解析边界。

### Modified Capabilities

当前草案不直接修改已归档主 spec；审计门禁会确认是否需要改为 existing capability delta。

## Impact

- 后续会影响 core CLI、adapter direct CLI、adapter invoke 和 config handling 的参数解析方式。
- 当前 change 只定义解析核心，不改变任何 observable CLI output、protocol-json 或配置文件行为。
- 后续 consumer 迁移必须在独立 change 中更新 docs、examples、tests 和 output expectations。
