## Context

当前相关 change 被拆成三层：

- `typed-field-definitions` 拥有字段级 field/path/value metadata、decode/validation、error attribution 和 duplicate identity check。
- `standard-parameter-resolution-core` 消费 typed field metadata，并拥有 CLI/config/invoke 来源映射、合并、source info、passthrough 和 operation binding。
- `typed-json-contract-validation` 消费 typed field metadata，并拥有 manifest/probe/protocol JSON 的 runtime validation policy、schema parity audit 和 error mapping 保持。

本 change 对应 typed-field-definition-core；仓库中的 OpenSpec change ID 保持为 `introduce-typed-field-definitions`。

## Goals / Non-Goals

**Goals:**

- 定义可复用的 typed field identity、structured JSON path 和 value constraint metadata。
- 提供字段级 decode/validation 结果，并把失败归属到 stable field identity 和 JSON path。
- 暴露 schema metadata view，使后续 schema/docs/fixture tooling 可以消费字段事实源。
- 通过 definition set build 拒绝重复 field identity，避免同一 canonical field 被多处重复声明。
- 保持模型只覆盖字段级事实，避免提前绑定标准参数、manifest/probe 或 protocol 的 consumer semantics。

**Non-Goals:**

- 不直接生成完整 JSON Schema 文件，也不决定 schema 文件布局或 `$ref` 策略。
- 不拥有标准参数来源优先级、默认值合并、passthrough、warning 或 operation binding。
- 不拥有 manifest/probe/protocol response 业务规则、stable error category、stdout/stderr placement 或 protocol envelope。
- 不迁移任何当前 runtime validation，也不改变现有 public schema、examples 或 fixtures。

## Core Model

第一版 typed field definition 至少包含以下概念，具体 Rust API 可以在实现阶段按 crate 边界命名：

| Concept | Responsibility |
| --- | --- |
| Field identity | Stable canonical identity，用于 definition set uniqueness、error attribution 和 schema metadata attribution。 |
| Field path | 结构化 JSON path segments；JSON Pointer、dotted path 或 display string 只能由该结构派生。 |
| Field validation | 聚合 value kind 和字段级 constraints，例如 string/integer/number/boolean/array/object、enum、numeric range、string regex 或 string/array length；`FieldValidation::int()` 等构造函数直接声明 runtime value kind 和 Rust value type，integer range 使用 integer bound，number range 使用 floating bound；`FieldValidation::string_enum::<T>()` 可由真实 Rust enum metadata 生成 string enum constraints，空 enum allowed set 在 build 阶段失败。字段 presence 和 JSON null 处理由 derive struct 的 leaf Rust field type 表达。 |
| Default metadata | 表达 static default 是否存在，以及 static default value 是否满足同一 value kind/constraint；static default 由字段 Rust value type 约束后再进入 metadata。动态默认来源和来源合并不属于本层。 |
| Schema metadata view | 从 identity/path/value/constraints/default 派生的字段级 schema facts，不包含完整 schema document writer。 |
| Validation attribution | validation failure 必须携带 field identity、field path 和 machine-readable reason；consumer 决定最终错误分类和文案。 |
| Definition set | 汇总多个 field definitions，统一执行 build-time field validation 和 duplicate identity check，并产出 typed `extract_without_default` function、`extract_with_static_defaults` function、`validate_without_default` function、`validate_with_static_defaults` function、derive struct 形状的 typed values object、value kind view、typed default values object 和 schema metadata view。 |

## Flow

1. Consumer 用 `FieldValidation<T>` 声明 value kind、Rust value type 和字段级 constraints；string enum 字段可以用实现 `FieldStringEnum` 的 Rust enum 作为 `T`，schema metadata 仍暴露 JSON enum values，且空 enum allowed set 不能通过 definition set build。
2. Consumer 通过 `#[derive(FieldDefs)]` 的 Rust struct 汇总多个字段并生成调用侧 typed values object shape；嵌套参数对象用 `#[field(group)]` 的普通 Rust struct 字段表达，例如 `params.defaults.limit_chars`。字段 identity 和输入 JSON path 都由叶子 `FieldDef` 显式声明。叶子字段用 Rust 字段类型声明 presence 和 leaf Rust value type，例如 `limit_chars: Option<i64>` 或 `output: OutputMode`，并由生成代码要求 `#[field(...)]` 表达式类型为 `FieldDefBuilder<i64>` / `FieldDefBuilder<OutputMode>`。
3. Definition set build 使用 canonical identity 检查重复定义；同一 set 内重复 identity 必须失败，即使两处声明的字段语义完全一致。
4. Decoder 按 field path 读取 JSON value，并按 validation 中的 value kind 转换为 typed value。
5. Validator 执行字段级 constraints，并返回 typed value 或 attributed validation failure。
6. Consumer 先 build 字段定义对象，再用 `extract_without_default` 对输入 JSON 做整体字段校验和提取；只有提取函数返回的 derive struct values object 才作为业务参数对象使用，例如 `params.defaults.limit_chars: Option<i64>`。`T` leaf 表示必须取到值，path missing 和 JSON null 都失败；`Option<T>` leaf 表示不保证取到值，path missing 和 JSON null 都提取为 `None`，合法值提取为 `Some(T)`。`extract_without_default` 不套用 static defaults；`extract_with_static_defaults` 以字段为单位用 static default 填补缺失输入，但返回与 `extract_without_default` 相同的 typed values object shape；`default_values()` 返回同一 struct shape 的 typed default values object，缺少 static default 的 leaf 为 `None`。Definition set build 将字段类型中的 `T`/`Option<T>` presence policy 写入字段 metadata；static default 只作为字段 metadata 校验，不改变声明类型。Built definition set 只读；需要复用或静态覆盖字段定义时，consumer 可用 `to_builder()` 取回 typed builder 副本，按 Rust 字段路径替换 leaf builder 后重新 `build()`。需要只检查输入时使用 `validate_without_default` 或 `validate_with_static_defaults`，并与对应提取函数共享同一个 field validation error type。Consumer 也可以读取 value kind/default/schema projections，并映射到自身拥有的 schema 文件、operation envelope、配置来源和 stable diagnostics。

## Decisions

1. 保留 `typed-field-definitions` 作为新 capability ID。
   - Rationale: 现有主 spec owner 分别覆盖标准参数、adapter/protocol、schema 验证材料和输出层，没有一个能独占非标准参数 JSON 与标准参数共享的底层字段模型。
   - Consequence: 归档时需要新增或指定长期 owner 文档；其它主规范只摘要消费边界。

2. Field path 采用结构化 path model，display string 只能派生。
   - Rationale: dotted path、JSON Pointer 和 CLI/config 展示路径有不同 escaping 与 owner 语义；底层模型不能依赖某个展示格式。

3. Typed field 只做字段级 validation。
   - Rationale: required/type/enum/range/regex/length 等字段级约束可复用；operation/result pair、manifest capability policy、source precedence、array item schema 和 object property schema 等语义必须留给 owner。

4. Schema metadata 是 view，不是 schema writer。
   - Rationale: `docs/schemas/` 是验证材料，字段语义 owner 在对应主规范；typed field 可以统一字段 facts，但不能绕过 schema owner 和 example validation。

5. Field identity 在同一 definition set 中必须唯一。
   - Rationale: field identity 是 duplicate detection、error attribution 和 schema metadata attribution 的 canonical key；允许等价重复声明会让调用侧无法判断哪个 struct field declaration path 才是 owner。
   - Consequence: Duplicate identity 在 set build 阶段失败，并暴露 duplicate field identity/path attribution；不提供“语义相同则静默合并”的重复注册规则。

6. 首版实现落在独立 `docnav-typed-fields` crate。
   - Rationale: typed field definition core 会被标准参数和 JSON contract validation 两类 consumer 复用；独立 crate 避免把底层字段模型绑定到 protocol 或标准参数 owner。
   - Consequence: 当前实现不接入任何现有 CLI/config/manifest/probe/protocol runtime path。

7. 字段校验通过 `FieldValidation<T>` 聚合，不开放任意闭包式兜底校验。
   - Rationale: value kind、Rust value type 和 constraints 是同一字段级校验模型；任意闭包难以进入 schema metadata 和 deterministic validation。
   - Consequence: 未来如需 custom validation，必须先设计 stable validator identity、metadata 和 deterministic validation 规则。

8. 字段集合通过 `FieldDefSet` 汇总，并只输出 typed-field projections。
   - Rationale: 多字段 build 可以集中执行 field build validation 和 duplicate identity check，并让 consumer 复用 typed `extract_without_default` function、`extract_with_static_defaults` function、`validate_without_default` function、`validate_with_static_defaults` function、typed values object、value kind/default/schema views。
   - Consequence: `FieldDefSet` 不执行 config source merge、CLI argv parsing、operation binding 或 manifest/probe policy。

9. Field definition derive macro 提供 Rust struct 作为常见声明路径，并以 `FieldDef::builder(...)` 作为叶子字段定义。
   - Rationale: 常见字段定义应像 consumer 想使用的参数对象一样阅读；Rust struct 本身负责 typed values object shape，`#[field(group)]` 表达嵌套对象，叶子字段类型负责 presence policy 和 leaf Rust value type，`FieldDef` 负责 stable canonical identity、结构化输入 JSON path、typed validation 和 default metadata。
   - Consequence: 叶子字段在声明时保持 builder 形态，允许先省略 `.build()`；definition set build 统一执行字段 build validation、missing path validation、default/enum 校验和 duplicate identity check，并把 build failure 归属到 struct field declaration path。build 后的定义对象不能作为业务参数对象直接使用；consumer 必须调用 `extract_without_default` 或 `extract_with_static_defaults` 并使用返回的 values object，例如 `params.defaults.limit_chars`；`default_values()` 返回 typed default values object 而不是裸 JSON；validate functions 只返回同一类字段校验错误，不产出业务对象；动态 identity-string field lookup 不属于 definition set API。
   - Consequence: derive macro 不从 validation expression 反推 Rust type 或 requiredness；声明的 leaf type 与 `FieldDefBuilder<T>` 由 Rust 编译器检查一致，因此 validation helper function 可以作为叶子 builder 的一部分使用。Rust field 中的 `T`/`Option<T>` 是 generated value shape、requiredness 和 null-as-absent 行为的唯一来源；static default metadata 不参与 values object type 推导。
   - Consequence: 生成的 typed builder 保留同一 struct/group shape；built definition set 提供 `to_builder()`，允许 `let mut fields2 = fields.to_builder(); fields2.defaults.limit_chars = fields2.defaults.limit_chars.path(["a", "b"]); let fields2 = fields2.build()?;` 这类静态复用和覆盖。Built definition set 本身保持只读。
   - Consequence: 不提供 namespace prefix、闭包式 object builder、自定义声明 DSL 或隐式 identity/path 派生；需要自定义 JSON path 时在叶子字段上显式调用 `.path([...])`。

10. 运行时默认值来源不属于 typed field core。
   - Rationale: typed field core 只表达字段结构、类型和 static default；cwd/project-root/config-discovery 等 runtime default 属于来源合并层。
   - Consequence: 不提供动态默认值声明 API；`default_values()` 只输出 static defaults，并按 derive struct shape 返回 typed values object，例如 `defaults.limit_chars -> Some(20000)`，不按输入 JSON path 投影。

11. Numeric range constraints 使用显式 open/closed bound。
   - Rationale: 调用点必须能区分端点是否包含；用正负无穷表达单边 range 会污染 schema metadata 和 validation；integer range 不能通过 floating point 比较，否则大整数会丢精度。
   - Consequence: `min(FieldBound::...)` 和 `max(FieldBound::...)` 表达单边约束；`between(lower, upper)` 是同时设置 min/max 的快捷方式；`int()` 的 range bound 类型是 integer，`num()` 的 range bound 类型是 floating number；存在的 floating numeric bound 必须 finite，空 range 在 build 阶段失败。

12. String 和 array 只支持字段级内容约束。
   - Rationale: regex、字符串字符长度和 array 元素数量是字段自身事实；array item schema、object property schema 和跨字段结构语义属于 JSON contract/schema owner。
   - Consequence: string validation 支持 regex 和 character length；array validation 支持 item count length；typed field core 不递归校验 array 内部对象。

## Risks / Trade-offs

| Risk | Mitigation |
| --- | --- |
| 抽象过早导致字段模型不适配具体 consumer。 | 第一版只覆盖 field/path/value/default/schema metadata，不上收来源合并、protocol envelope 或 manifest/probe policy。 |
| schema metadata 与现有 schema 文件漂移。 | 后续 JSON validation 或 schema tooling change 必须增加 parity audit；本 change 不修改 schema 文件。 |
| duplicate identity 检查过严导致调用方不能复用同一 identity 的多个 declaration path。 | 同一 canonical field 在一个 set 内应只有一个 owner declaration；需要复用时提取 shared field builder，而不是重复注册。 |
| 与旧 `unify-standard-parameter-definitions` 范围重叠。 | 本 change 不修改旧 change；相邻 standard-parameter-resolution change 只消费 typed field metadata。 |
| 任意 custom validation 过早进入核心 API。 | 首版不提供闭包兜底；待需要时先补 stable validator identity、metadata 和 deterministic validation 设计。 |

## Open Questions

- Validation attribution 的 machine-readable reason 是否先 crate-private，还是需要为后续 diagnostics 预留稳定枚举。
