本 proposal 是 `create-universal-cli-config-crate` 的未审核临时说明：从 Docnav 现有 typed-fields 与 parameter-resolution 经验中抽象出可作为子仓库复用的 Rust CLI/config 解析底层 crate；当前文档只存在于 `openspec/changes/create-universal-cli-config-crate/`，不影响现有其它文档或主规范。

## Why

稍复杂的 Rust CLI 通常同时需要 CLI flag、环境变量、配置文件和默认值，但现有工具多把这些来源拆开处理，导致项目需要手写字段声明、来源优先级、合并规则、错误归因和最终 struct materialization。

Docnav 已经在 `typed-fields` 与 `parameter-resolution` 中沉淀了字段契约、来源投影、默认值、校验和来源优先级的雏形；将这些能力抽象为通用 crate，可以让 Docnav 与其它 Rust CLI 共享同一套底层解析体验。

## What Changes

- 新增通用 Rust CLI/config resolution 的 OpenSpec 能力，覆盖 typed option contract、source projection、multi-source resolution、merge strategy、provenance trace 和 final materialization。
- 设计从 Docnav 当前 `typed-fields` 与 `parameter-resolution` 抽象可复用核心的迁移边界，保留 Docnav 专属 adapter、operation、protocol、diagnostic code 和 output ownership 在 Docnav 内。
- 定义 CLI flag、env var、config file、dynamic/static defaults、custom source 的统一来源模型，并支持 deterministic priority 与 per-field merge strategy。
- 定义 `explain` / trace 类输出所需的来源证据，便于用户理解最终配置值来自哪里、覆盖了什么。
- 将新 crate 设计为可子仓库化的独立边界：先在当前 workspace 中形成可验证 crate，再按审计结果迁移或镜像到独立 repo、submodule 或 subtree，供其它 Rust CLI 项目复用。
- 非目标：本 change 不改变 Docnav 的 `outline -> ref -> read` 协议、不改变现有 adapter contract、不引入新的文档导航行为、不把 Docnav 的 operation/adapter 语义塞进通用库核心。

## Capabilities

### New Capabilities

- `cli-config-resolution`: 通用 Rust CLI/config 解析底层能力，拥有字段契约投影、来源抽取、优先级合并、来源追踪、诊断事实和最终 typed materialization 的长期规范。

### Modified Capabilities

- 无。`typed-fields` 与 `navigation-input-resolution` 是本 change 的输入和集成影响范围，但当前主要求不在本 proposal 中直接修改；若实现审计发现必须改变其可观察 requirement，应先更新本 change 的 capability 列表并补充对应 delta spec。

## Impact

- 影响代码边界：`crates/shared/typed-fields`、`crates/shared/typed-fields-macros`、`crates/shared/parameter-resolution`、`crates/shared/cli-args`、`crates/docnav/src/cli/parser/native_options.rs`、`crates/shared/navigation/src/parameters/**`。
- 新增或重组 crate 边界：候选 crate 名包括 `cli-config-resolution` / `typed-config-resolution`，并可配套 `*-clap`、`*-serde` feature 或 companion crate；最终命名由实现前审计确认。
- 依赖影响：核心 crate 应尽量保持低依赖；`clap`、`serde_json`、`toml`、`figment` 类集成必须通过 feature 或 adapter 层隔离，避免核心库绑定某个 CLI/config 框架。
- 验证影响：需要新增 crate-level unit tests、Docnav compatibility tests、merge strategy tests、source provenance tests、derive macro compile tests，以及后续子仓库化的 package/release 验证。
- 发布影响：实现完成前不得发布外部 artifact；子仓库化方式、包名和 crate API 稳定性需通过实现前审计确认。
