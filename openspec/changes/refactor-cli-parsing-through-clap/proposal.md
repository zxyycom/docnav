本 change 的核心目标是让 Clap 与 `cli-config-resolution-clap` 成为 Docnav CLI 结构解析和动态字段解码的唯一业务实现路径；本文是仅位于 `openspec/changes/refactor-cli-parsing-through-clap/` 的未审核临时 proposal，不影响现有主规范或其它文档。

## Why

当前 `docnav` 同时使用 Clap、`docnav-cli-args`、native option catalog 和 navigation 内的字符串/JSON 解码逻辑。同一参数的 flag、argument id、类型和消费规则由多个层重复维护，canonical adapter declaration 没有真正成为 CLI 事实源。

本 change 将 CLI 业务解析收敛为一条路径：通用的 canonical field 投影和类型化解码进入 `cli-config-resolution` 子仓库；Docnav 只保留产品命令结构、operation applicability、adapter 选择、诊断和输出策略。抽取范围以当前真实复用点为限，不新增通用 CLI 框架。

## What Changes

- Core 用一棵 authoritative Clap command tree 解析 root/subcommand、help/version、固定 positional 和 core-owned flags，并从 typed `ArgMatches` 构造 command model。
- `cli-config-resolution-clap` 统一负责动态 native flag 的 short/long 注册、argument id、action/cardinality、typed value parser、冲突检查和 `SourceCandidate` 提取。
- Navigation 按 operation 聚合 static registry 中的 canonical native CLI declarations；adapter 选择后，只让 selected adapter 的 typed candidates 进入 resolution 和 handler handoff。
- 同一 operation 的 native CLI flag 必须全局唯一。重复 locator 是 release-local declaration failure，不推断跨 adapter 语义兼容性。
- Core 通过结构化 Clap error kind/context 映射既有 diagnostic、输出通道和退出码；help/version 保持无业务副作用。
- 删除 `docnav-cli-args`、native option 字符串桥接、`cli_arg_id()` 猜测、post-match 字符串解码和 arbitrary JSON 猜测路径，不保留运行时 fallback。
- 清除本次 CLI 重构覆盖的 8 条 accepted quality warnings，并同步 owner docs、tests 和 case 账本。

## Compatibility

- 命令、flag、默认值、source priority、adapter selection、protocol/readable payload、ref 和格式业务语义保持不变。
- **BREAKING（仅歧义 argv 拼写）**：以 `-` 开头的 string/path value 改用 `--flag=<value>`；例如使用 `--query=--future`，不再接受无法与下一个 flag 区分的 `--query --future`。
- Authoritative parse 失败前仍需识别 document operation 和有效 `--output`，以保持 failure envelope。该逻辑被收窄为 presentation-only raw argv probe，不解析任何业务或 native 参数。

## Non-Goals

- 不把 Docnav command tree、固定 positional、adapter registry、operation applicability、diagnostic code 或输出策略移入通用子仓库。
- 不给 typed-fields 增加 positional/alias DSL、自定义 parser registry、plugin framework 或 compatible-flag inference。
- 不改变 generic native boolean 的现有 `SetTrue` grammar；core-owned `--pagination enabled|disabled` 继续由 core Clap definition 声明。
- 不处理 Markdown 性能问题，也不改 protocol/schema/example shape；这些材料只做一致性审计。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `core-cli`：authoritative Clap command tree、typed core arguments、strict argv、help/version、error projection 和 bounded failure presentation probe。
- `cli-config-resolution`：canonical CLI projection、typed Clap decoding、argument conflict 和 canonical candidate extraction 的通用 owner。
- `navigation-input-resolution`：operation-scoped registry projection、typed CLI source handoff 和 selected-adapter candidate filtering。

## Impact

- 主仓库：`crates/docnav/src/cli/**`、`crates/docnav/src/runtime.rs`、`crates/shared/navigation/src/parameters/**`、adapter native option glue、workspace Cargo metadata，以及删除 `crates/shared/cli-args/`。
- 子仓库：`subrepos/cli-config-resolution/crates/cli-config-resolution-clap/**` 的内部职责拆分、typed parsers、错误和 public tests；优先保持现有 public API，不增加 Docnav-specific abstraction。
- 规范与验证：CLI、architecture、navigation、adapter contract、testing owner docs，case 账本，Rust tests，core CLI smoke，quality check，子仓库独立验证和 OpenSpec strict validation。
