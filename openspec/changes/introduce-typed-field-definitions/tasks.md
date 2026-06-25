## 1. 审计门禁

- [x] 1.1 确认 proposal、design、specs 和 tasks 只围绕 typed field/path/value metadata，不包含标准参数来源合并、CLI argv、manifest/probe 迁移或 schema 文件生成职责。
- [x] 1.2 确认本 change 不修改 `unify-standard-parameter-definitions`，也不要求实现前同步旧 change。
- [x] 1.3 确认 capability ID 保留为 `typed-field-definitions`；归档时再新增或指定长期 owner 文档承接该底层能力。
- [x] 1.4 确认相邻 change 边界：`standard-parameter-resolution-core` 只消费 typed field metadata，`typed-json-contract-validation` 只消费 typed field metadata 并保留 JSON contract owner 语义。

## 2. Core Model

- [x] 2.1 定义最小 typed field definition model：field identity、structured JSON path、value kind、required/static default metadata、field constraints、schema metadata view、validation attribution 和 duplicate identity guard。
- [x] 2.2 实现 duplicate identity check：相同 canonical identity 的重复注册直接失败，并返回带 field identity/path 的 build failure。
- [x] 2.3 实现字段级 decode/validation：支持基础 type、required、typed enum、typed open/closed numeric range、string regex、string/array length 和 default 校验，并把失败归属到 field identity 与 path。
- [x] 2.4 输出 schema metadata view，但不生成完整 JSON Schema 文件、不决定 schema document layout。
- [x] 2.5 接入一个小范围 inert consumer fixture，证明 metadata、decode/validation、error path 和 duplicate identity check 可以被复用，且不改变现有 CLI/config/manifest/probe/protocol 行为。

## 3. 验证

- [x] 3.1 添加正例：typed field metadata 能生成 schema metadata view，并能成功 decode/validate 合法字段值。
- [x] 3.2 添加反例：缺失 required、类型错误、enum/range/regex/length/default 违规、integer range 大整数精度边界和 duplicate identity 都能产生带 field identity/path 的 failure。
- [x] 3.3 复查实现 diff，确认没有修改 public schema、examples、CLI output、adapter behavior 或 protocol envelope。
- [x] 3.4 当前 OpenSpec artifact 已通过 `openspec validate introduce-typed-field-definitions --strict`。
- [x] 3.5 后续实现或 artifact 再修改后，重新运行 OpenSpec validation 和 touched-surface Rust/fixture 验证；若实现跨协议、schema、examples 或多个包边界，再运行 `bun run verify:docnav-workspace`。

## 4. API refinement

- [x] 4.1 将 value kind、Rust value type 和字段级 constraints 聚合为 `FieldValidation<T>`，支持由真实 Rust enum metadata 声明 string enum 字段并拒绝空 enum allowed set，不引入任意闭包式兜底校验。
- [x] 4.2 新增 `FieldDefSet` 汇总 build，并输出 typed `extract_without_default` function、`extract_with_static_defaults` function、`validate_without_default` function、`validate_with_static_defaults` function、same-shaped typed values object、value kind view、typed default values object 和 schema metadata view，不暴露动态 identity-string field lookup。
- [x] 4.3 保持 `FieldDefSet` 只输出 typed-field projections，不执行标准参数来源合并、CLI argv parsing、operation binding 或 schema document generation。
- [x] 4.4 简化常见声明路径：`#[derive(FieldDefs)]` 的 Rust struct 负责 typed values object shape，用 `#[field(group)]` 表达嵌套参数对象，用 `T`/`Option<T>` 显式声明 presence policy 和 leaf Rust value type，叶子 `FieldDef::builder(...).path(...)` 一次性定义 identity、structured JSON path、validation 和 static default metadata；删除 namespace prefix 扩展；删除运行时默认值声明扩展；叶子声明不调用 `.build()`，由 definition set build 统一执行字段 build/path/default/declaration presence/range/duplicate identity 校验；build 后的定义对象不可直接作为业务参数对象，`extract_without_default` 返回不套 static defaults 的 typed values object，`extract_with_static_defaults` 返回字段级合并 static defaults 后的同形 typed values object，`default_values()` 返回同 struct shape 的 typed default values object，`validate_without_default` 和 `validate_with_static_defaults` 与对应提取函数共用字段校验错误类型。
- [x] 4.5 新增 `FieldDefs` derive macro，使用显式 `T`/`Option<T>` leaf Rust value type 生成可跨函数签名使用的 definition type、typed builder、typed values type 和 typed default values type，并通过 `FieldDefBuilder<T>` 编译期校验叶子表达式类型；built definition set 提供 `to_builder()`，可复制回同形 typed builder 后静态覆盖 leaf builder 并重新 build；不保留旧的自定义声明 DSL。
