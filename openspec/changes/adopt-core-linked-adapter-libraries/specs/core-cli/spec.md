本 core-cli delta 定义 `docnav` core 在默认 document operation path 中如何选择、检查和调用随 core release 交付的 adapter-layer workspace crates。

## ADDED Requirements

### Requirement: Core release 内置 adapter-layer workspace crates 必须成为默认 document operation implementation 来源
`docnav` MUST use adapter-layer workspace crates shipped with the current core release as default document operation adapter implementations. The default release MUST include all built-in adapters without using feature gates to select the default adapter set. Project config、user config and CLI input MAY select an adapter id only when that id resolves to an adapter implementation registered in the current core release static adapter registry. Project/user config、installed packages、external executables、command paths and historical adapter artifact records MUST NOT provide default document operation implementation. The adapter layer MUST remain a code and contract boundary, not a separate default distribution boundary.

#### Scenario: 列出 core release 内置 adapter libraries
- **WHEN** 调用方执行 `docnav adapter list`
- **THEN** 输出只包含当前 core release static adapter registry 中的 adapter id、version、supported formats、extensions、content types 和 capabilities

#### Scenario: 默认发行物包含 adapter implementation
- **WHEN** 构建默认 `docnav` 发行物
- **THEN** 所有内置 adapter-layer workspace crates 随 `docnav` core release artifact 交付
- **THEN** 默认发行物不需要启用额外 feature 才能获得内置 adapter set
- **THEN** 默认发行物可直接执行已支持格式的 document operation

#### Scenario: Historical adapter config does not provide implementation
- **WHEN** `<project-root>/.docnav/adapters.json` 存在并包含 adapter command path
- **AND** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 不把 adapter command path 当作 implementation source
- **THEN** `docnav` 只从当前 core release static adapter registry 选择候选

#### Scenario: 声明式 adapter 只解析内置 adapter id
- **WHEN** 调用方通过 `--adapter custom-local-adapter` 声明 adapter id
- **AND** 该 id 不存在于当前 core release static adapter registry
- **THEN** `docnav` 返回 adapter selection diagnostic
- **THEN** diagnostic details 包含 declared adapter id、selection source 和 reason

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

### Requirement: adapter 选择必须区分声明式 adapter 和自动发现
`docnav` MUST first honor a declared adapter id from `--adapter` or `defaults.adapter`. Declared adapter failure MUST return an adapter selection diagnostic with the declared source and candidate failure stage. When no declared adapter id exists, `docnav` MAY infer an adapter from the current core release static adapter registry metadata and use the same candidate evaluation rules to select, continue traversal, or report discovery failure. Adapter 评估 MUST 以内置 adapter metadata、当前契约语义、capability 支持和 adapter-owned support check 结果为准。

#### Scenario: 显式 adapter 不存在后返回诊断
- **WHEN** 调用方传入 `--adapter custom-local-adapter` 但当前 core release static adapter registry 中不存在该 adapter id
- **THEN** `docnav` 返回 adapter selection diagnostic
- **THEN** 错误 details 包含 adapter id、selection_source、stage 和 reason
- **THEN** `docnav` 不把显式 adapter failure 转为 automatic discovery success path

#### Scenario: support check 有效不支持后继续
- **WHEN** 候选内置 adapter support check 返回符合当前契约语义的 `supported: false`
- **THEN** `docnav` 保留 `PROBE_UNSUPPORTED` 或等价稳定候选证据
- **THEN** `docnav` 继续 static adapter registry 遍历

#### Scenario: 未声明 adapter 时先 core 推断
- **WHEN** 调用方没有传入 `--adapter`
- **AND** 配置没有指定 `defaults.adapter`
- **THEN** `docnav` 使用当前 core release static adapter registry metadata 的格式信息推断一个预选 adapter id
- **THEN** `docnav` 先校验该预选 adapter

#### Scenario: 自动推断 adapter metadata 当前契约不一致后继续
- **WHEN** 自动推断 adapter metadata 缺少 `docnav` 当前 CLI 选择 adapter 所需字段
- **THEN** `docnav` 保留候选证据
- **THEN** `docnav` 继续 static adapter registry 遍历

#### Scenario: static registry 遍历候选当前契约不一致后继续
- **WHEN** static adapter registry 遍历中的候选 adapter metadata 或 support check 输出字段缺失、类型不符或语义校验失败
- **THEN** `docnav` 保留候选证据
- **THEN** `docnav` 继续 static adapter registry 遍历
- **THEN** 若后续候选成功，前序候选失败只保留为 internal discovery state，不进入 success output

#### Scenario: 所有阶段失败
- **WHEN** 没有 core release static adapter registry 中的 adapter 能校验目标文档
- **THEN** `docnav` 返回 `FORMAT_UNKNOWN`
- **THEN** 错误 details 包含候选摘要
- **THEN** 候选摘要是 JSON 数组
- **THEN** 每条候选摘要只包含 adapter_id、stage 和 reason

## REMOVED Requirements

### Requirement: 临时 adapter 记录必须足以启动 adapter
**Reason**: 默认 document operation adapter source 是当前 core release static adapter registry 中的 adapter-layer workspace crate；项目文件中的 adapter 启动记录不能提供 core release 内置 implementation。

**Migration**: Document operation adapter selection uses the static adapter registry included in the current core release. 需要支持的格式应作为 `docnav` core release 内置 adapter-layer workspace crate 进入发行物；开发期新增 adapter 的默认路径是修改 workspace crate、注册到 core static registry 并重新编译。
