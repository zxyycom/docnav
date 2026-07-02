本 proposal 已按 `adopt-core-linked-adapter-libraries` 更新：入口分类和参数来源解析必须以 core release static adapter registry 为 adapter implementation source boundary。

## Why

标准参数仍需要和入口生命周期拆开：help、config/init/doctor/version、adapter inspection 和 document operations 不应共享同一 document operation lifecycle。新的默认边界删除 adapter direct CLI 和 adapter `invoke` surface，因此本 change 不再迁移 adapter SDK，而是确保 core CLI、protocol request construction 和 adapter library dispatch 之间的 raw input/derived values 边界清晰。

## What Changes

- 引入标准入口管线：core CLI 先完成 command family、document operation、help、config/init/doctor/version 和 `adapter list` 分类。
- 将标准参数解析描述为入口参数来源解析：只接收入口 owner 提供的 direct input view、配置来源和默认值，产出 typed runtime values、source info、diagnostic handoff 和 owner-scoped native option handoff。
- 明确 adapter implementation source boundary：document operation 只能选择 core release static registry 中的 adapter id/library handle；参数来源解析不得提供 executable、command path、artifact record 或 implementation source。
- 明确不可变输入规则：参数来源解析不得修改原始 CLI argv tokens、protocol request envelope 或 request `arguments`；后续 request construction 只能消费 derived semantic values 和 owner 明确保留的 passthrough。
- 非目标：不恢复 adapter direct CLI、adapter `invoke` 或动态 adapter management surface。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `core-cli`: 定义 core 标准入口管线，明确 document、config/init/doctor/version/help 和 `adapter list` 的入口分类、配置读取和参数来源解析边界。
- `adapter-protocol`: 收敛为 adapter library protocol request execution boundary，删除 adapter direct CLI/invoke 作为默认入口的要求。
- `standard-parameter-resolution`: 将能力语义从“标准参数解析”改为“入口参数来源解析”，保留字段身份、来源优先级、typed validation、explicit adapter native option sources、handoff 和 operation argument binding。
- `standard-parameter-adoption`: 更新迁移约束，要求 core CLI 和 `docnav-navigation` 消费入口参数来源解析结果，同时保持入口 owner 边界和原始输入不可变。

## Impact

- Affected docs/specs: `docs/architecture.md`, `docs/cli.md`, `docs/standard-parameters.md`, `docs/adapter-contract.md`, `docs/protocol.md`, `docs/testing.md`, and related OpenSpec specs.
- Affected Rust crates: `docnav`, `docnav-navigation`, `docnav-adapter-contracts`, `docnav-standard-parameters`, `docnav-typed-fields`, and tests that assert source-resolution or request construction behavior.
- Affected public surfaces: CLI help wording, docs terminology, input/error ownership descriptions, schema/example validation references, and adapter library contract documentation.
