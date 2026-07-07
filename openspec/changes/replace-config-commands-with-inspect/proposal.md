本 proposal 定义 `replace-config-commands-with-inspect` change 的目标、范围和影响：用单一只读 `docnav config inspect` 替换旧 config 子命令，并让配置读取校验与配置检查复用 owner-provided config metadata。

当前 change 只在 `openspec/changes/replace-config-commands-with-inspect/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

`docnav config get|set|unset|list` 和 core config file read/update 当前用手写分支、serde shape check 和 adapter registry key lookup 校验配置；这会让 core config 命令复制 navigation、typed-fields 与 adapter native option declaration 已经拥有的 value kind、range、enum、nullability、结构 shape 和 source mapping 事实。

这种重复校验已经导致裸 `options.<key>` 写入只检查 key 是否注册而不校验值类型或范围，且无法稳定区分不同 adapter 的同名 native option，容易让无效配置先落盘，随后在 navigation input resolution 阶段才失败。继续保留 `set` / `unset` 会把 CLI token decoding、局部 JSON patch、数组/对象编辑和 canonical write 变成 core CLI 长期负担，偏离“配置文件由用户编辑、Docnav 负责校验和解释”的边界。

## What Changes

- 让 config source 的 key 发现、JSON value validation 和 source inspection 复用 owner-provided config metadata，而不是在 core config 命令里重新推导字段语义。
- 为 core-owned runtime config 字段（如 `invocation_log.*`）、navigation-owned config 字段（如 `defaults.*` 和 `outline.*`）以及 adapter-id namespaced `AdapterOptionSpec` 声明建立可聚合的 config metadata 查询边界。
- 移除 `docnav config set`、`docnav config unset`、`docnav config get` 和 `docnav config list` 作为长期 surface，只保留一个只读 `docnav config inspect` 命令。
- `docnav config inspect` 必须展示所有配置来源的实际状态，包括 selected path、origin、存在性、load state、JSON/config validation issue 和可解析的 source summary。
- `docnav config inspect` 的 surface 聚焦 source inspection：展示 selected sources、load state、validation diagnostics 和可解析 source summary；adapter-id namespaced values 作为配置来源字段接受校验。
- 扩展 adapter native option 配置路径为 `options.<adapter-id>.<option-key>`；路径中的 adapter id 直接使用现有 adapter registry id，不新增 alias 或二级命名。
- 裸 `options.<option-key>` 不做迁移、兼容读取或特殊诊断；它只是普通 unknown/invalid config path。
- 扩展 typed-fields 的 config metadata projection，使它能表达并校验 config source 中的 scalar、array、object 和 nested structure，而不只覆盖 leaf path。
- 保持 adapter option ownership：adapter 继续通过 `AdapterOptionSpec` 暴露 native option declaration；core 和 navigation 只消费 declaration metadata，不复制 adapter-owned value kind、range、default 或 operation applicability。
- 同步配置读取、只读配置检查、navigation input resolution、diagnostic/error mapping、docs、tests 和验证材料，证明同一字段 metadata 驱动同一类校验结果。
- 非目标：不改变 raw protocol envelope，不改变 adapter handler payload，不把 `options.<adapter-id>.*` 改成 core-owned namespace，不保留 config 写入命令，不兼容旧的裸 `options.<key>`，不新增数组/对象编辑 DSL。

## Capabilities

### New Capabilities

- 无。该 change 修改既有 config、navigation input resolution、adapter option declaration 和 typed-field 事实源边界，不创建新的长期 capability。

### Modified Capabilities

- `core-cli`: 收缩 `docnav config` 为单一只读 inspect surface，更新 source inspection、adapter-id option path 和错误行为要求。
- `navigation-input-resolution`: 更新 config source shape、typed-field aggregation、adapter-id native option metadata 复用和 unknown/invalid option diagnostic 要求。
- `typed-fields`: 更新 `FieldDefSet` / metadata helper 对 config-source validation、nested structure lookup、compound shape validation 和 canonical value extraction 的支持边界。
- `adapter-contract`: 更新 `AdapterOptionSpec` native option declaration 作为 config validation 事实源的消费规则和 adapter-id namespace 边界。

## Impact

- Affected executable: `docnav` core CLI 的 `config` 命令族和 config file read/validation flow。
- Affected shared libraries: `docnav-navigation`、`docnav-typed-fields`、`docnav-parameter-resolution` 和 `docnav-adapter-contracts` 的 field metadata、source processing 和 diagnostic handoff 边界。
- Affected adapter surface: linked adapters 继续只声明 native option facts；不要求 adapter handler 接收 raw option value 或重复基础类型/range 校验。
- Affected diagnostics: unsupported key、unknown adapter id、adapter-local declaration conflict、invalid config value、invalid object/array/shape 和 selected adapter option validation 必须携带 source/key/adapter 信息，并保持 machine-readable error shape 稳定。
- Affected validation: 需要覆盖 source inspection、core config fields、outline mode config fields、registered native options、adapter-id option namespace、nested config shape failures、config file read failures、navigation resolution parity，以及被触及的 config schema/example validation materials。
