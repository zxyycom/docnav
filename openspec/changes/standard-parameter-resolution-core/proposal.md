本 change 定义标准参数来源解析核心的实现边界和验收入口。

## Why

标准参数需要从 direct input、项目配置、用户配置和默认值中形成 typed runtime values。来源构造和来源合并应作为独立实现 slice，消费 typed field metadata，并为后续 core/SDK 迁移提供稳定边界。

基础审计确认：`docs/standard-parameters.md` 拥有长期行为模型，`docnav-typed-fields` 拥有字段 identity、extraction strategy、schema metadata、默认值和 typed value 校验能力。`docnav-standard-parameters` 已实现手工 sources 的 resolver 后半段；本 change 继续补齐 source construction、配置 source 读取和按 registration/extraction strategy 构造 direct/project/user/default sources 的 API。

## What Changes

- `docnav-standard-parameters` 消费 typed field definition，使用字段 identity、extraction strategy、schema metadata、默认值和 typed value 校验能力。
- Source construction 将 direct input、project config、user config 和 default 映射为标准参数 sources，并保留未映射字段 passthrough。
- Resolver 按固定顺序合并 sources：direct input、project config、user config、default；输出 typed values、source info、diagnostics 和 passthrough handoff。
- Config source loading 读取 caller 提供的 project/user config source path，校验 JSON 顶层 object，跳过不可用 source，并返回 structured source-skipped diagnostic data。
- Operation argument binding 只记录标准参数 identity 到 protocol request `arguments` path 的映射；request construction 由后续 owner 处理。

## Capabilities

### New Capabilities

- `standard-parameter-resolution`: 标准参数 source construction、配置 source loading、来源合并、typed runtime values、diagnostics、passthrough 和 operation argument binding。

### Modified Capabilities

本 change 新增 `standard-parameter-resolution` capability delta，不直接修改已归档主 spec requirement。

## Baseline Audit

- `typed-field-definitions` 仍是字段 metadata owner；本 change 以其 value kind、enum、range、requiredness 和 default 规则作为单一事实源。
- Loose CLI argv tokenization、unused flag warning、native option semantic validation、warning formatting、output channel 和 exit behavior 由入口 owner 处理。
- `docnav` 和 `docnav-adapter-sdk` 尚未消费 `docnav-standard-parameters`；consumer migration 不属于本 change 的完成条件。

## Impact

- 后续会影响 core CLI、adapter direct CLI、adapter invoke 和 config handling 的参数解析方式。
- 当前 change 只定义标准参数来源核心，不改变 observable CLI output、protocol-json 或配置文件行为。
- 后续 consumer 迁移必须在独立 change 中更新 docs、examples、tests 和 output expectations。
