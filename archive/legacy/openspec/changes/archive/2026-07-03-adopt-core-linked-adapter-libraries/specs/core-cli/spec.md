本 core-cli delta 只定义 `docnav` core 在默认 document operation path 中的 adapter implementation source、内置 adapter inspection 和默认管理命令边界。它不重写完整 adapter selection algorithm。

## ADDED Requirements

### Requirement: Core release 内置 adapter-layer workspace crates 必须成为默认 document operation implementation 来源
`docnav` MUST use adapter-layer workspace crates shipped with the current core release as default document operation adapter implementations. The default release MUST include all built-in adapters without using feature gates to select the default adapter set. CLI input and effective project/user config MAY declare an adapter id only through `--adapter` or `defaults.adapter`, and that id MUST resolve to an implementation registered in the current core release static adapter registry. Registry entries MUST expose source-level static descriptors containing adapter identity、native option registry entries and operation handler bindings. Project/user config、installed packages、external executables、command paths and historical adapter artifact records MUST NOT provide default document operation implementation. The adapter layer MUST remain a code and contract boundary, not a separate default distribution boundary.

#### Scenario: 默认发行物包含 adapter implementation
- **WHEN** 构建默认 `docnav` 发行物
- **THEN** 所有内置 adapter-layer workspace crates 随 `docnav` core release artifact 交付
- **THEN** 默认发行物不需要启用额外 feature 才能获得内置 adapter set
- **THEN** 默认发行物可直接执行已支持格式的 document operation

#### Scenario: Static descriptor supplies operation bindings
- **WHEN** core registry resolves the built-in Markdown adapter
- **THEN** the registry entry exposes a static descriptor with Markdown identity, native option registry entries and handler bindings
- **THEN** navigation input resolution can merge, validate/extract and hand off final native option values for linked dispatch
- **THEN** selected Markdown typed-field declarations validate consumed option support, type and range semantics before handler dispatch

#### Scenario: Core passes absolute path to linked adapter
- **WHEN** caller executes `docnav outline docs/guide.md` from a project subdirectory
- **THEN** `docnav` resolves the document path against cwd/project context to an absolute path
- **THEN** `docnav-navigation` and the linked adapter handler receive the absolute path
- **THEN** adapter IO does not depend on process cwd

#### Scenario: Historical adapter config does not provide implementation
- **WHEN** `<project-root>/.docnav/adapters.json` 存在并包含 adapter command path
- **AND** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 不把 adapter command path 当作 implementation source
- **THEN** `docnav` 只从当前 core release static adapter registry 选择 adapter implementation

#### Scenario: 声明式 adapter 只解析内置 adapter id
- **WHEN** 调用方通过 `--adapter custom-local-adapter` 声明 adapter id
- **AND** 该 id 不存在于当前 core release static adapter registry
- **THEN** `docnav` 返回 adapter selection diagnostic
- **THEN** `docnav` 不尝试从 installed package、external executable、command path 或 historical artifact record 加载该 adapter

#### Scenario: 列出 core release 内置 adapter libraries
- **WHEN** 调用方执行 `docnav adapter list`
- **THEN** 输出只包含当前 core release static adapter registry 中的 adapter id、version 和 supported formats

#### Scenario: Adapter diagnostics do not define process exit code
- **WHEN** a linked adapter handler returns a structured diagnostic
- **THEN** core/output maps that diagnostic to the selected protocol/readable surface
- **THEN** final process exit code remains owned by `docnav` core CLI
- **THEN** the adapter layer does not expose an exit-code API

## MODIFIED Requirements

### Requirement: 核心管理命令必须提供 MVP 行为
`docnav` MUST 实现 `init`、`doctor`、`version` 和 core release 内置 adapter inspection 的可验证基础行为。核心管理命令 MUST keep adapter inspection tied to adapter implementations registered in the current core release static adapter registry. `docnav` MUST NOT expose `adapter install`、`adapter register`、`adapter update` or `adapter remove` as valid default CLI commands.

#### Scenario: init 幂等创建项目配置
- **WHEN** 调用方执行 `docnav init`
- **THEN** `docnav` 创建 `<project-root>/.docnav/docnav.json`
- **AND** 重复执行不会破坏已有配置

#### Scenario: version 输出 crate 版本
- **WHEN** 调用方执行 `docnav version`
- **THEN** stdout 包含 `docnav` crate version

#### Scenario: doctor 检查配置和 core release 内置 adapter libraries
- **WHEN** 调用方执行 `docnav doctor`
- **THEN** `docnav` 检查项目配置和用户配置
- **THEN** `docnav` 检查当前 core release static adapter registry metadata 和 adapter layer 可用性
- **THEN** 输出包含 checks 数组
- **AND** 存在失败检查项时进程非零退出

#### Scenario: dynamic adapter management commands are removed
- **WHEN** 调用方执行 `docnav adapter install ./target/release/custom-adapter`
- **OR** 调用方执行 `docnav adapter register ./target/release/custom-adapter`
- **OR** 调用方执行 `docnav adapter update custom-adapter`
- **OR** 调用方执行 `docnav adapter remove custom-adapter`
- **THEN** `docnav` 按标准 CLI unknown/unsupported command 行为返回失败
- **THEN** 该命令不会写入 adapter registry、project config 或 user config
