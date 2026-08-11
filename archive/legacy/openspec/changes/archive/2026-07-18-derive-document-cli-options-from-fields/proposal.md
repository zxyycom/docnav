## Why

同一个 document named option 目前同时在 canonical field declaration、core Clap argument、native option catalog 和 direct-input bridge 中维护。参数 owner 修改 flag、类型、默认值、约束或 operation applicability 时，仍需跨 core、navigation 与 adapter 同步，CLI shape 与 canonical resolution 因而可能漂移。

本 change 让 owning field declaration 成为 document named option 的唯一 authoring source。CLI registration、help facts、value capture 和 candidate identity 都从该 declaration 派生，使参数修改回到 owner-local 范围。

## Scope

- Field-derived named options：`adapter`、`page`、`limit`、`pagination`、`output` 和 adapter native options。
- Core-owned static inputs：root/subcommand topology、`path`/`ref`/`query` positionals、config path flags、invocation logging、management commands 和 adapter inspection。
- Existing owners：config/source priority、output mode contract、diagnostic projection、protocol/readable shape、ref、pagination 和 adapter format behavior保持不变；field projection 只消费其 canonical facts。
- 同一 document operation 的 registry public flags 本期保持全局唯一；跨 adapter 复用同名 flag 留给后续 change。

## What Changes

1. 为 CLI processing strategy 增加 owner-authored help、value name 和 Boolean input encoding；identity、locator、value kind、constraints、default 与 merge 继续使用 canonical field facts。
2. 让 `cli-config-resolution-clap` 直接从 `FieldDefSet` 生成 Clap arguments、help facts 和 typed/invalid CLI candidates。
3. 由 navigation 分别构造 operation-scoped registry CLI field set 与 selected resolution field set，并用 canonical identity 判断 candidate applicability。
4. Core 用 static command shape 组合 field-derived document options；保留的 lexical preflight 也从同一 projection 获取 locator 与 cardinality。
5. 切换到单一路径后，删除重复的 document option catalog、arg-id derivation、raw remapping、JSON guessing 和 parallel decoder。

## Done When

1. 修改 common 或 adapter-native option 时，只需修改 owning declaration 及其 owner tests。
2. 每个 generated flag 在 parsing 前映射到一个 canonical field identity，typed/invalid candidate 保留该 identity 直到 selected resolution。
3. Help、accepted/default facts、capture 和 validation 都可追溯到同一 declaration；`output` 等 enum field 的具体合法值不在 projection 中重复。
4. Runtime document command 只保留 field-derived option path，owner-documented CLI、config、adapter、diagnostic、output 和 protocol behavior 通过回归验证。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `typed-fields`：保存 framework-neutral CLI processing metadata。
- `cli-config-resolution`：从 canonical `FieldDefSet` 生成 arguments、help facts 和 candidates。
- `adapter-contract`：adapter native option declaration 同时提供 CLI facts、applicability 和 handler binding。
- `core-cli`：组合 static command shape 与 field-derived document options。
- `navigation-input-resolution`：提供 registry/selected field sets 并按 selected declarations 处理 candidates。

## Impact

- `crates/shared/typed-fields/**`
- `crates/shared/cli-config-resolution-clap/**`
- `crates/shared/adapter-contracts/**` 与 built-in adapter declarations
- `crates/shared/navigation/src/parameters/**`
- `crates/docnav/src/cli/**`
- 对应 owner docs、package READMEs、CLI/process tests、case materials 和 workspace verification
