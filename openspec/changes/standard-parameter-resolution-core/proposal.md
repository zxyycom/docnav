本 change 定义标准参数来源解析核心的实现边界和验收入口；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Why

标准参数需要从 CLI argv、adapter invoke arguments、项目配置、用户配置和默认值中形成 typed runtime values。来源合并应作为独立实现 slice，消费 typed field metadata，并为后续 core/SDK 迁移提供稳定边界。

基础审计确认：`docs/standard-parameters.md` 已经拥有长期行为模型，当前代码中 `docnav-typed-fields` 已提供字段 identity、schema metadata、默认值和 typed value 校验能力；缺口是一个只消费这些 metadata 的来源解析层，而不是重新定义字段模型或替换 CLI frontend。

## What Changes

- 新增标准参数 resolution core 草案，基于 typed field definition 消费字段 identity、schema metadata、默认值和 typed value 校验能力。
- 记录 direct input、project config、user config 和 default 的来源模型、固定合并顺序、source info、diagnostics 和 passthrough policy。
- 定义 resolver 的最小输入/输出边界：registration、source objects、entry passthrough policy、typed values、source attribution、diagnostics 和 passthrough handoff。
- 保留 operation argument binding 的方向：标准参数 identity 映射到 protocol request `arguments` path，request construction 仍由后续 owner 处理。
- 明确本 change 的交付边界：解析核心先落地；core CLI、adapter SDK、adapter direct CLI、CLI frontend、public schema、examples、protocol/readable output 和 current output behavior 留给后续独立 change。

## Capabilities

### New Capabilities

- `standard-parameter-resolution`: 标准参数来源映射、合并、typed runtime value 和 passthrough 的解析边界。

### Modified Capabilities

本 change 新增 `standard-parameter-resolution` capability delta，不直接修改现有已归档主 spec requirement。

## Baseline Audit

- `typed-field-definitions` 仍是字段 metadata owner；本 change 以其 value kind、enum、range、requiredness 和 default 规则作为单一事实源。
- 现有 loose CLI argv 行为仍由当前 CLI/adapter consumer 拥有；本 change 保持 unknown argv、unused flag、native option 和 readable warning 的 owner 边界。
- 已废弃的 `unify-standard-parameter-definitions` 仅作为历史背景；当前执行入口是本 change 的 `tasks.md`。

## Impact

- 后续会影响 core CLI、adapter direct CLI、adapter invoke 和 config handling 的参数解析方式。
- 当前 change 只定义解析核心，不改变任何 observable CLI output、protocol-json 或配置文件行为。
- 后续 consumer 迁移必须在独立 change 中更新 docs、examples、tests 和 output expectations。
