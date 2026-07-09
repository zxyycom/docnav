本 proposal 定义 `replace-config-commands-with-inspect` change 的目标、范围和影响：用单一只读 `docnav config inspect` 替换旧 config 子命令，把现有 CLI flag owner-map 模式泛化为 owner-provided parameter aggregation metadata，并把 adapter native option 的持久配置路径迁移到 `options.<adapter-id>.<option-key>`。

## 文档重心

本 proposal 只回答“为什么要做、做什么、不做什么、影响哪些 owner”。它不承接设计取舍的完整论证、不定义 capability-level MUST/SHALL，也不安排实现顺序。需要判断实现边界时读 `design.md`；需要验收契约时读 `specs/*/spec.md`；需要推进顺序时读 `tasks.md`。

## Why

`docnav config get|set|unset|list` 和 core config file read/update 当前用手写分支、serde shape check 和 adapter registry key lookup 校验配置；这会让 core config 命令复制 navigation、typed-fields 与 adapter native option declaration 已经拥有的 value kind、range、enum、nullability、结构 shape 和 source mapping 事实。

Docnav 已经在 CLI flag 处理里存在 owner map 思路：不同 owner 声明自己的输入事实，再由共享解析流程汇总。问题不在于没有 owner map，而在于这套 owner-map 只服务 CLI flag 路径，尚未抽象为同时产出 CLI projection 和 config-source projection 的参数汇总边界。

这种重复校验已经导致裸 `options.<key>` 写入只检查 key 是否注册而不校验值类型或范围，且无法稳定区分不同 adapter 的同名 native option，容易让无效配置先落盘，随后在 navigation input resolution 阶段才失败。将持久配置路径迁移到 `options.<adapter-id>.<option-key>` 会触及主规范、schema、examples 和测试材料，但实现上主要是既有 config path 解析、registry lookup 和 hard-coded typed-field path 的替换，不需要改变 adapter handler payload。

继续保留 `set` / `unset` 会把 CLI token decoding、局部 JSON patch、数组/对象编辑和 canonical write 变成 core CLI 长期负担，偏离“配置文件由用户编辑、Docnav 负责校验和解释”的边界。对于 `outline.mode_rules[]`、`outline.auto_full_read.thresholds[]` 这类非结构化全文读取相关数组配置，本 change 先复用现有 owner-specific 校验和参数投影；只有当现有路径不能稳定表达 source inspection / direct config read / navigation resolution 的同一事实时，才对 typed-fields 做最小扩展。

## What Changes

- 将现有 CLI flag owner-map 思路抽象为参数汇总边界，使同一 owner-provided parameter metadata 能产出 CLI/input projection 和 config-source projection。
- 让 config source 的 key 发现、JSON value validation 和 source inspection 复用参数汇总产出的 config-source projection，而不是在 core config 命令里重新推导字段语义。
- 为 core-owned runtime config 字段（如 `invocation_log.*`）、navigation-owned config 字段（如 `defaults.*` 和 `outline.*`）以及 adapter-id namespaced `AdapterOptionSpec` 声明建立可聚合的 parameter metadata 查询边界。
- 移除 `docnav config set`、`docnav config unset`、`docnav config get` 和 `docnav config list` 作为长期 surface，只保留一个只读 `docnav config inspect` 命令。
- `docnav config inspect` 展示 selected project/user config source 的实际状态，包括 scope、path、origin、存在性、load state、JSON/config validation diagnostics 和可解析的 source summary。
- `docnav config inspect` 聚焦 source inspection：adapter-id namespaced values 只作为配置来源字段接受校验，并列出当前输入可解析出的参数事实；它不预演 selected adapter/operation dispatch。
- 将 adapter native option 配置路径迁移为 `options.<adapter-id>.<option-key>`；路径中的 adapter id 直接使用现有 adapter registry id，不新增 alias 或二级命名。这是本 change 的明确迁移子范围，不只是 inspect 输出细节。
- 裸 `options.<option-key>` 不做迁移、兼容读取或特殊诊断；它只是普通 unknown/invalid config path。
- 先审计现有 owner-specific 数组配置校验能否覆盖当前非结构化全文读取相关配置；typed-fields 只在必要时扩展 processing-path projection，用最小 subset 表达当前 config source 所需的 scalar、array、object 和 nested structure facts。
- 保持 adapter option ownership：adapter 继续通过 `AdapterOptionSpec` 暴露 native option declaration；core 和 navigation 只消费 declaration metadata，不复制 adapter-owned value kind、range、default 或 operation applicability。
- 同步配置读取、只读配置检查、navigation input resolution、diagnostic/error mapping、docs、tests 和验证材料，证明同一字段 metadata 驱动同一类校验结果；schema/examples 是验证材料，不成为运行时 schema engine。
- 非目标：不改变 raw protocol envelope，不改变 adapter handler payload，不把 `options.<adapter-id>.*` 改成 core-owned namespace，不保留 config 写入命令，不兼容旧的裸 `options.<key>`，不新增数组/对象编辑 DSL。

## Capabilities

### New Capabilities

- 无。该 change 修改既有 config、navigation input resolution、adapter option declaration 和 typed-field 事实源边界，不创建新的长期 capability。

### Modified Capabilities

- `core-cli`: 收缩 `docnav config` 为单一只读 inspect surface，更新 source inspection、adapter-id option path 和错误行为要求。
- `navigation-input-resolution`: 更新 config source shape、typed-field aggregation、adapter-id native option metadata 复用和 unknown/invalid option diagnostic 要求。
- `typed-fields`: 更新 `FieldDefSet` / metadata helper 对 config-source lookup 和 candidate value validation 的支持边界；compound shape helper 只在现有 owner-specific 数组配置校验不能满足 parity 时作为最小扩展进入实现。
- `adapter-contract`: 更新 `AdapterOptionSpec` native option declaration 作为 config validation 事实源的消费规则和 adapter-id namespace 边界。

## Impact

- Affected executable: `docnav` core CLI 的 `config` 命令族和 config file read/validation flow。
- Affected shared libraries: `docnav-navigation`、`docnav-typed-fields`、`docnav-parameter-resolution` 和 `docnav-adapter-contracts` 的 field metadata、source processing 和 diagnostic handoff 边界。
- Affected adapter surface: linked adapters 继续只声明 native option facts；不要求 adapter handler 接收 raw option value 或重复基础类型/range 校验。
- Affected diagnostics: unsupported key、unknown adapter id、adapter-local declaration conflict、invalid config value、invalid object/array/shape 和 selected adapter option validation 必须携带 source/key/adapter 信息，并保持 machine-readable error shape 稳定。
- Affected validation: 需要覆盖 source inspection、core config fields、outline mode config fields、registered native options、adapter-id option namespace、config file read failures、navigation resolution parity，以及被触及的 config schema/example validation materials；nested config shape failures 先由现有 owner-specific 校验承担，必要时再迁移到 typed-fields helper。
- Affected docs/materials: 后续实现任务需要同步 `docs/cli.md`、`docs/navigation-input-resolution.md`、`docs/adapter-contract.md`、测试策略材料，以及仍引用旧 config command 或旧 config source shape 的 schema/example 材料。
