本 change 定义标准参数来源解析核心的实现边界、推荐入口和验收条件。

## Why

标准参数需要从 direct input、project config、user config 和 default 中形成 typed runtime values。字段事实由 `docnav-typed-fields` 表达；来源读取、来源构造、来源优先级、diagnostic handoff 和 passthrough handoff 由 `docnav-standard-parameters` 统一实现。

完成状态以 `FieldDefSet` pipeline 为准：普通 caller 提供 typed-field 定义、direct/config strategy id、直接输入和 config 路径，标准参数层内部形成 catalog/index 并返回 `StandardParameterResolution`。

## What Changes

- `docnav-typed-fields` 继续拥有字段定义：identity、类型、required/default、range、enum、regex，并提供 processing build 让 caller 以处理 id 和处理函数把 raw input 转为处理值。
- `docnav-standard-parameters` 提供普通 pipeline 入口：caller 提供 `FieldDefSet`、direct/config strategy id、direct input、project/user config path 或 descriptor、dynamic defaults 和 passthrough policy。
- Pipeline 内部按固定顺序读取 `schema_metadata()`、`strategy_metadata("direct")` 和 `strategy_metadata("config")`，形成 catalog/index，并校验同一 source role 内的 path conflict。
- 普通 config 入口是 path/descriptor，由标准参数层负责 JSON loading、顶层 object 校验和 skipped-source diagnostic handoff。
- Loaded config 入口只复用同一标准参数 loader 已经加载过的 source，不作为 caller 自行实现 JSON loading 的普通路径。
- Source construction 将 direct input、project config、user config 和 default 映射为标准参数 sources，并把 caller passthrough processing result 原样作为 passthrough handoff 返回；raw-minus-mapped 是 caller 处理函数可以返回的一种结果。
- Resolver 按固定顺序合并 sources：direct input、project config、user config、default；输出 typed values、source info、diagnostic events 和 passthrough handoff。
- Operation argument binding 只记录标准参数 identity 到 protocol request `arguments` path 的映射；request construction 由后续 owner 处理。
- Catalog/index 是 pipeline 内部编译产物，只承接 typed-field metadata 到 source construction 的映射。

## Capabilities

### 新增能力

- `standard-parameter-resolution`: 标准参数 pipeline facade、direct/config strategy binding、catalog/index 派生、source construction、config source loading、来源合并、typed runtime values、diagnostic handoff、passthrough 和 operation argument binding。

### 修改能力

本 change 新增 `standard-parameter-resolution` capability delta，不直接修改已归档主 spec requirement。

## Baseline Audit

- `typed-field-definitions` 是字段事实 owner；标准参数层以其 metadata 和 validation 作为单一事实源。
- Catalog/index 是标准参数层从 `FieldDefSet`、`schema_metadata()`、direct strategy metadata 和 config strategy metadata 派生的内部产物，只服务 pipeline source construction。
- Loose CLI argv tokenization、unused flag warning、native option semantic validation、diagnostic formatting、output channel 和 exit behavior 由入口 owner 处理；标准参数层只提交 diagnostic events。
- `docnav` 和 `docnav-adapter-sdk` 尚未消费 `docnav-standard-parameters`；consumer migration 不属于本 change 的完成条件。

## Impact

- 后续会影响 core CLI、adapter direct CLI、adapter invoke 和 config handling 的参数解析接入方式。
- 当前 change 只定义标准参数来源核心，不改变 observable CLI output、protocol-json 或配置文件行为。
- 后续 consumer migration 必须在独立 change 中更新 docs、examples、tests 和 output expectations。
