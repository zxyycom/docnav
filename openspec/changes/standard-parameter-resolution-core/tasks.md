本 tasks 只给出标准参数解析核心的推进顺序；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## 1. 审计门禁

- [x] 1.1 阻塞级审计：确认本 change 只定义标准参数来源解析核心，不迁移 core CLI、adapter SDK、adapter direct CLI 或 `clap`。
- [x] 1.2 阻塞级审计：确认 typed field definition 是字段 metadata owner，standard parameter resolution 只消费字段 identity、schema metadata、默认值和 typed value 校验能力。
- [x] 1.3 阻塞级审计：确认 passthrough 与 owner validation 的边界没有提前限制 adapter native options、unknown argv 或未映射 invoke arguments。
- [x] 1.4 阻塞级审计：确认已废弃的 `unify-standard-parameter-definitions` 只作为历史背景，当前执行入口为本 change 的 2.x/3.x tasks。

## 2. 轮廓实现

- [x] 2.1 决定 resolver core 的 crate/module 放置和最小可见性，先保持窄边界 API。
- [x] 2.2 定义标准参数 registration、source kind、source object、source info、diagnostic 和 merge result 的最小结构。
- [x] 2.3 接入 typed-field metadata，以现有 value kind、enum、range、requiredness 和 default validation 作为字段约束单一事实源。
- [x] 2.4 实现 `direct input > project config > user config > default` 来源合并和 typed runtime value 查询。
- [x] 2.5 实现 required/default 的 runtime 处理，确保 static/dynamic default 结果进入同一 typed-field validation。
- [x] 2.6 实现 passthrough handoff 结构，只对已映射标准参数执行标准参数 validation。
- [x] 2.7 建模 operation argument binding 的 identity-to-arguments-path 关系，并把 protocol request construction 留给后续 owner。

## 2A. typed-field extraction strategy 前置实现

- [x] 2A.1 删除 `FieldDefBuilder::path` 兼容入口，叶子字段只能通过 `FieldDefBuilder::extract(strategy_id, strategy)` 声明抽取来源。
- [x] 2A.2 新增 `FieldDefBuilder::extract(strategy_id, strategy)`，允许同一字段按策略 id 声明不同抽取策略。
- [x] 2A.3 在 `FieldDefSet` build 阶段校验同一 strategy id 在同一 definition set 内只对应一种 input kind。
- [x] 2A.4 新增 JSON path strategy 的 `fields.extract(strategy_id, json)`、`extract_with_static_defaults(strategy_id, json)` 和对应 validate typed object projection。
- [x] 2A.5 建模 Rust field strategy input kind，后续 direct-input derive projection 不在本 slice 展开。

## 3. 验证

- [x] 3.1 添加小范围 fixture，证明 direct/project/user/default 来源优先级和 source info。
- [x] 3.2 添加 fixture，证明 required/default、typed value validation 和 invalid mapped value diagnostic。
- [x] 3.3 添加 fixture，证明标准参数 validation 只覆盖已映射字段，passthrough 保留给 entry owner。
- [x] 3.4 添加 fixture，证明 operation argument binding 保留 direct/config/default 的 resolved source info。
- [x] 3.5 运行 resolver 所在 Rust crate 的 targeted tests。
- [x] 3.6 若实现触及多个 crate 或 observable contract surface，运行 `bun run verify:docnav-workspace`。
- [x] 3A.1 添加 typed-field fixture，证明同一 strategy id 的 JSON path strategy 可抽取 typed object。
- [x] 3A.2 添加 typed-field fixture，证明同一 strategy id 混用 JSON input kind 和 Rust field input kind 时 build 失败。
