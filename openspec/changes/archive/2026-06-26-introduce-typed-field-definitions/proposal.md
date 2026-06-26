## Why

标准参数、配置、manifest、probe 和协议 JSON 都需要描述“某个字段在 JSON 中的位置、值类型、基础约束、默认值 metadata 和错误归属”。这些能力目前容易被塞进标准参数或 JSON Schema 相关 change，导致一个 change 同时承担字段模型、来源合并、CLI argv、schema 文件生成和 consumer 迁移。

本 change 建立 typed field definition core，作为字段级事实源。上层 consumer 可以复用同一套 field/path/value metadata、decode/validation 和错误归属，但标准参数、JSON contract 和 schema tooling 继续拥有各自的入口语义和输出契约。

## What Changes

- 新增 `typed-field-definitions` capability，定义 typed field identity、structured JSON path、field validation、default metadata、schema metadata view、validation attribution 和 duplicate identity guard。
- 建立字段级执行流程：consumer registration -> definition set build -> duplicate identity check -> field decode -> field validation -> attributed result。
- 使用 `FieldValidation<T>` 聚合 runtime value kind、Rust value type 和字段级 constraints；`int()`、`string()`、`num()` 等构造函数同时声明 value kind 和 Rust value type，暂不支持任意闭包式兜底校验。
- String enum 只通过真实 Rust enum 的 `FieldStringEnum` metadata 声明，并投影为 JSON enum values；空 enum allowed set 在 build 阶段失败。
- Numeric range 使用显式 open/closed bound，允许单边 `min`/`max`；`int()` range 使用 integer bound，`num()` range 使用 finite floating bound。String 支持 regex 和字符长度约束，array 支持元素数量约束。
- 使用 definition set 汇总多个字段；`#[derive(FieldDefs)]` 的 Rust struct 声明调用侧 typed values object shape，`#[field(group)]` 表达嵌套参数对象，叶子字段通过 `T`/`Option<T>` 显式声明 presence policy 和 leaf Rust value type。
- 叶子使用 `FieldDef::builder(...).path(...)` 一次性定义 identity、输入 JSON path、typed validation 和 static default metadata；编译器校验 `#[field(...)]` 表达式是匹配的 `FieldDefBuilder<T>`。
- Build 后的定义对象只输出 typed extract/validate functions、value kind view、typed default values object、schema metadata view 和 `to_builder()` typed builder 复用入口；动态 identity-string field lookup 不属于 definition set API。
- 运行时默认值来源不属于本层；同一 definition set 内 field identity 必须唯一，不提供动态 identity-string field lookup 或 duplicate identity merge。
- 将跨字段语义、来源合并、operation binding、CLI argv、manifest/probe policy、protocol envelope 和完整 schema document writer 保留给对应 owner。
- 保持现有 CLI、config、manifest、probe、protocol、schema 文件、examples 和 runtime validation 行为不变。

## Scope Boundaries

- Owned here: field identity、structured JSON path、FieldValidation、required/static default metadata、enum、numeric range、string regex、string/array length 等字段级约束、definition set build、duplicate identity check、decode/validation result、error path attribution、schema metadata view 和 value kind/default projections。
- Owned by consumers: standard parameter source precedence、config discovery、CLI flag parsing、adapter native option policy、manifest/probe business rules、protocol request/response envelope、readable output、complete JSON Schema document generation 和 public schema/example updates。
- Related active change handling: `unify-standard-parameter-definitions` 保持不修改；后续是否替代、迁移或归档旧 change，由独立审计决定。

## Capabilities

### New Capabilities

- `typed-field-definitions`: typed field/path/value definition core，拥有字段级 metadata、decode/validation、error attribution 和 duplicate identity check 边界。

### Modified Capabilities

当前 change 不直接修改已归档主 spec。归档时如果该能力实现落地，应新增或指定一个长期 owner 文档承接 typed field definition core；标准参数、adapter/protocol 和 JSON Schema 文档只保留消费边界摘要。

## Impact

- 新增 `docnav-typed-fields` 共享 crate，承接 typed field definition model 的首版实现。
- 首版 API 包含 `FieldValidation<T>`、`FieldDef`、`FieldDefSet` 和 `FieldDefs` derive macro；字段集合 projection 只暴露 typed field facts，不执行标准参数来源合并。
- 常见字段可通过 Rust struct 定义 values object shape，字段类型用 `T`/`Option<T>` 写出 presence policy 和 leaf Rust value type，由叶子 `.path(...)` 声明输入 JSON 取值位置，由 `FieldValidation<T>` 约束叶子 builder 的 Rust 返回类型。
- Definition set build 统一执行字段 build validation、declaration path attribution、declaration presence metadata projection、static default validation、typed open/closed range validation、regex metadata validation 与 duplicate identity check。
- Generated typed builder 与 struct/group shape 一致，built definition set 可通过 `to_builder()` 静态复用并覆盖字段 builder 后重新 build。
- 为 `standard-parameter-resolution-core` 和 `typed-json-contract-validation` 提供依赖边界：它们消费 typed field metadata，但分别拥有来源合并和 JSON contract runtime policy。
- 后续任何 observable behavior 改动必须在对应 owner change 中同步 docs、schema/example 或 tests。
