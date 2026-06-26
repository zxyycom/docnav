本 tasks 给出标准参数来源解析核心的推进顺序。打勾只表示已有可验证实现；未完成项必须有代码、测试和验证命令支撑后才能标记完成。

## 1. 审计门禁

- [x] 1.1 确认本 change 只定义标准参数来源解析核心，不迁移 core CLI、adapter SDK direct CLI、adapter `invoke` 或 CLI frontend。
- [x] 1.2 确认 typed field definition 是字段 metadata owner，standard parameter resolution 只消费字段 identity、extraction strategy、schema metadata、默认值和 typed value 校验能力。
- [x] 1.3 阻塞级审计：确认 passthrough 与 owner validation 的边界没有提前限制 adapter native options、unknown argv 或未映射 invoke arguments。
- [x] 1.4 确认当前执行入口为本 change 的 2.x/3.x tasks；更大的 consumer migration 由后续 change 承接。

## 2. 轮廓实现

- [x] 2.1 决定 resolver core 的 crate/module 放置和最小可见性，先保持窄边界 API。
- [x] 2.2 定义标准参数 registration、source kind、source object、source info 和 merge result 的最小结构。
- [x] 2.3 接入 typed-field metadata，以现有 value kind、enum、range、requiredness 和 default validation 作为字段约束单一事实源。
- [x] 2.4 实现 `direct input > project config > user config > default` 来源合并和 typed runtime value 查询。
- [x] 2.5 实现 required/static default 的 runtime 处理，确保 default 结果进入同一 typed-field validation。
- [x] 2.6 实现 passthrough handoff 结构，只对已映射标准参数执行标准参数 validation。
- [x] 2.7 建模 operation argument binding 的 identity-to-arguments-path 关系，并把 protocol request construction 留给后续 owner。
- [x] 2.8 接入现有 diagnostics handoff：standard validation error、source-skipped warning 和 ignored/passthrough warning handoff 交给 caller-provided diagnostics sink 或返回的等价 event collection。

## 2A. typed-field extraction strategy 前置实现

- [x] 2A.1 删除 `FieldDefBuilder::path` 兼容入口，叶子字段只能通过 `FieldDefBuilder::extract(strategy_id, strategy)` 声明抽取来源。
- [x] 2A.2 新增 `FieldDefBuilder::extract(strategy_id, strategy)`，允许同一字段按策略 id 声明不同抽取策略。
- [x] 2A.3 在 `FieldDefSet` build 阶段校验同一 strategy id 在同一 definition set 内只对应一种 input kind。
- [x] 2A.4 新增 JSON path strategy 的 `fields.extract(strategy_id, json)`、`extract_with_static_defaults(strategy_id, json)` 和对应 validate typed object projection。
- [x] 2A.5 建模 Rust field strategy input kind，后续 direct-input derive projection 不在本 slice 展开。

## 2B. source construction 与配置 source 读取

- [x] 2B.1 定义 strategy-specific metadata projection，包含 standard parameter identity、strategy id、path segments、value kind、constraints 和 default metadata。
- [x] 2B.2 扩展 standard parameter registration，声明 direct input binding、config binding、operation argument binding 和 no-config/no-direct 策略，并在 build 阶段校验 identity/path 冲突。
- [x] 2B.3 实现 direct input source construction：调用方提供已结构化的 CLI/invoke input，标准参数层按 registration 映射 `source=direct`，未映射字段进入 passthrough。
- [x] 2B.4 实现 config source descriptor 和 load result：调用方提供 project/user config path、path origin、source level 和 diagnostics sink，标准参数层读取 JSON、校验顶层 object，并返回 loaded source；显式 source 被跳过时追加 source-skipped recoverable diagnostic event。
- [x] 2B.5 实现 project/user config source construction：按 registered config path 抽取标准参数值，未知配置字段按 entry passthrough policy 处理。
- [x] 2B.6 实现 default source construction：static default 来自 typed-field metadata；dynamic default 由调用方 provider 提供，并进入同一 validation 与 source attribution。
- [x] 2B.7 提供 resolver facade：调用方传入 source construction inputs 后得到 `StandardParameterResolution`；低层 `StandardParameterSources` API 保留给测试和高级调用。
- [x] 2B.8 测试 source-skipped diagnostic handoff：标准参数层交接 source-skipped event，不决定 warning 文案、stderr/stdout 或 exit code。

## 3. 验证

- [x] 3.1 添加小范围 fixture，证明 direct/project/user/default 来源优先级和 source info。
- [x] 3.2 添加 fixture，证明 required/default、typed value validation 和 invalid mapped value 都交给 diagnostics handoff，且 invalid mapped value 不暴露为 safe typed runtime value。
- [x] 3.3 添加 fixture，证明标准参数 validation 只覆盖已映射字段，passthrough 保留给 entry owner。
- [x] 3.4 添加 fixture，证明 operation argument binding 保留 direct/config/default 的 resolved source info。
- [x] 3.5 在统一 diagnostics/error handoff 实现后，运行 resolver 所在 Rust crate 的 targeted tests。
- [x] 3.6 添加 source construction fixture，证明 direct input、project config、user config 和 default 都由 registration/extraction strategy 构造。
- [x] 3.7 添加配置 source 读取 fixture，证明默认缺失 source 不产生 diagnostic event，显式缺失、不可读、invalid JSON、non-object 产生 source-skipped recoverable diagnostic event 且继续合并其它来源。
- [x] 3.8 添加 passthrough 合并 fixture，证明 direct/project/user/default 未映射 path 按优先级覆盖且不参与 typed validation。
- [x] 3.9 若实现触及多个 crate 或 observable contract surface，运行 `bun run verify:docnav-workspace`；否则记录 targeted Rust tests 和跳过 wider verification 的原因。
- [x] 3A.1 添加 typed-field fixture，证明同一 strategy id 的 JSON path strategy 可抽取 typed object。
- [x] 3A.2 添加 typed-field fixture，证明同一 strategy id 混用 JSON input kind 和 Rust field input kind 时 build 失败。
