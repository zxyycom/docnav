本 proposal 是 `create-universal-cli-config-crate` 的 change-local 说明：把 Docnav 现有 `docnav-typed-fields` 的 `FieldDef` / `FieldDefSet` 作为 canonical 标准参数模型，由 `FieldDef` 直接拥有 merge strategy，补齐 CLI、env、config 的显式抽取策略，并由通用 resolution crate 统一执行来源排序、校验、合并和 provenance。当前文档只存在于 `openspec/changes/create-universal-cli-config-crate/`，不直接修改主规范。

## Why

稍复杂的 Rust CLI 通常同时接收 CLI flag、环境变量、配置文件和默认值。Docnav 的 `typed-fields` 已经拥有完整的字段 identity、类型、约束、默认值、校验和 typed materialization；真正缺少的是把不同输入来源稳定映射到同一组字段，并按确定优先级与字段级策略合并的通用机制。

第一轮实现另建了 `FieldContract` / `FieldSet` 及平行的值、约束和校验模型，导致消费者和 Docnav 需要在两套字段模型之间转换。这个 change 重新打开后不再扩展第二套字段契约，而是整理现有 API，让 `FieldDef` / `FieldDefSet` 直接成为通用 resolution 的标准参数对象。

## What Changes

- 复用 `docnav-typed-fields` 的 `FieldDef` / `FieldDefSet` 及其类型、约束、默认值、`MergeStrategy`、校验和 typed materialization；每个 field 直接声明 merge strategy，默认 `Replace`。允许为通用命名提供 re-export 或薄别名，但不复制字段模型或另建按 identity 关联的 merge declaration。
- 微调现有 processing API，显式声明 CLI flag、env var 和 config path 抽取策略；每种 extractor 只读取 `FieldDefSet` 已声明的 locator，并把结果映射到统一 source candidate。
- `cli-config-resolution` 负责 ordered sources、deterministic priority、执行 field 声明的 merge strategy、最终校验协调、diagnostics facts 和 provenance trace。Priority 数值越大优先级越高；同 priority 后注册 source 获胜。`Append` / `MapMerge` 按低 priority 到高 priority 应用，同级按注册顺序应用，`MapMerge` 的后值覆盖同名 key。
- `cli-config-resolution-clap` 负责从声明生成或读取已注册的 clap arguments；未知 flag 继续由 clap 原生拒绝。
- env 与 config 抽取只查询声明过的 locator，未声明输入默认静默忽略；本 change 不增加全量扫描、unused-key diagnostics 或通用 `UnknownPolicy`。
- Docnav hard cutover 直接消费 canonical `FieldDefSet`，移除 `generic_field_set` 一类平行字段转换。
- 独立子仓库以 Cargo workspace 为单位，可以包含 typed-fields、typed-fields macros、resolution core、clap companion 和 serde/config companion；`cli-config-resolution` 是主要消费者入口并 re-export canonical 参数类型。Docnav 通过固定 revision 的 Git submodule 消费该 workspace。

非目标：本 change 不改变 Docnav 的 `outline -> ref -> read` 协议、adapter contract、operation 语义、protocol envelope、diagnostic code 或 output behavior；不为未知 env/config 输入建立复杂兜底策略；不要求在本 change 中发布 crates.io artifact。

## Capabilities

### New Capabilities

- `cli-config-resolution`: 基于 canonical `FieldDef` / `FieldDefSet` 的通用 Rust CLI/config 参数解析能力，拥有显式来源抽取、优先级合并、来源追踪、诊断事实和最终 typed materialization 协调的长期规范。

### Modified Capabilities

- 无。`typed-fields` 与 `navigation-input-resolution` 是本 change 的实现输入和集成影响范围；若实现需要改变其可观察 requirement，应先补充对应 delta spec。

## Impact

- Canonical 参数模型：`crates/shared/typed-fields`、`crates/shared/typed-fields-macros`。
- Resolution 与 extractor：`crates/shared/cli-config-resolution`、`crates/shared/cli-config-resolution-clap`、`crates/shared/cli-config-resolution-serde`。
- Docnav 集成：`crates/shared/navigation/src/parameters/**` 及必要的 CLI/native-option 映射层。
- API 调整：删除或内部化重复的 `FieldContract` / `FieldSet`、重复 value/constraint/default/validation 类型和独立 merge declaration；把 `MergeStrategy` 直接纳入 canonical `FieldDef` metadata，公开 resolution 所需的 canonical metadata view，并整理 source/extractor/resolver 的主要使用路径。
- Repository 调整：建立可独立 checkout、build 和 test 的 Cargo workspace 子仓库边界，并以 `.gitmodules` 与 gitlink 固定 Docnav 的消费 revision；该 workspace 不依赖 Docnav protocol、adapter contracts、navigation、output 或 Markdown adapter crates。
- 验证：需要 canonical model reuse、CLI/env/config extraction、明确的 priority/tie/merge ordering、selected/contributing 与 overridden-invalid validation timing、materialization、provenance、Docnav hard cutover 和独立 workspace 的测试与示例。
- 历史状态：此前 46/46 任务和验证证据继续保留为第一轮实现记录，但不再代表本次收敛后的最终验收。
