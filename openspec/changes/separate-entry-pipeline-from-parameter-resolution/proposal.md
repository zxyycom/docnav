本 proposal 已按 `adopt-core-linked-adapter-libraries` 更新：入口分类和 navigation input resolution 必须以 core release static adapter registry 为 adapter implementation source boundary。

## Why

Navigation input resolution 需要和入口生命周期拆开：help、config/init/doctor/version、adapter inspection 和 document operations 不应共享同一 document operation lifecycle。新的默认边界删除 adapter direct CLI 和 adapter `invoke` surface，因此本 change 不再迁移 adapter SDK，而是确保 core CLI handoff、navigation request construction 和 selected adapter dispatch 之间的 raw input/derived values 边界清晰。

## What Changes

- 引入入口管线：core CLI 先完成 command family、document operation、help、config/init/doctor/version 和 `adapter list` 分类。
- 将 navigation input resolution 描述为 document operation 的来源解析 owner：接收 raw navigation command、config source descriptors/paths 和 registry，加载 raw config sources，产出 typed runtime values、source info、diagnostic handoff 和 owner-scoped native option handoff。
- 明确 adapter implementation source boundary：document operation 只能选择 core release static registry 中的 adapter id/library handle；navigation input resolution 不得提供 executable、command path、artifact record 或 implementation source。
- 明确不可变输入规则：navigation input resolution 不得修改原始 CLI argv tokens、protocol request envelope 或 request `arguments`；后续 request construction 只能消费 derived semantic values 和 owner 明确保留的 passthrough。
- 非目标：不恢复 adapter direct CLI、adapter `invoke` 或动态 adapter management surface。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `core-cli`: 定义 core 入口管线，明确 document、config/init/doctor/version/help 和 `adapter list` 的入口分类、config source descriptor/path handoff 和 navigation input resolution 调用边界。
- `adapter-protocol`: 收敛为 adapter library protocol request execution boundary，删除 adapter direct CLI/invoke 作为默认入口的要求。
- `navigation-input-resolution`: 将能力语义收敛为 navigation command 的 input resolution，保留字段身份、来源优先级、typed validation/extraction、explicit adapter native option sources、handoff 和 operation argument binding。

## Impact

- Affected docs/specs: `docs/architecture.md`, `docs/cli.md`, `docs/navigation-input-resolution.md`, `docs/adapter-contract.md`, `docs/protocol.md`, `docs/testing.md`, and related OpenSpec specs.
- Affected Rust crates: `docnav`, `docnav-navigation`, `docnav-adapter-contracts`, `docnav-typed-fields`, and tests that assert source-resolution or request construction behavior.
- Affected public surfaces: CLI help wording, docs terminology, input/error ownership descriptions, schema/example validation references, and adapter library contract documentation.
