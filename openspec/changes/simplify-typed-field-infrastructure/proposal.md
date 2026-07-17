## Why

Typed-fields 与 CLI/config resolution 已提供 typed validation、selected applicability 和 source provenance，但部分通用能力尚未获得生产消费者。需要建立一个渐进优化入口，让维护面按消费者证据逐片收缩，同时保持现有可观察行为。

核心句：采用 B′“薄共享契约 + 受控分散”，每次只处理一个可独立验证、可独立回滚的维护面。

> 状态：本 change 只在当前目录形成未审核临时 artifacts；阻塞审计完成并明确首个 slice 前不修改实现。

## What Changes

- 保留 adapter-owned option facts、typed validation、selected applicability、source priority/provenance，以及既有 protocol、diagnostic 和 output behavior。
- 将共享层限制在多个生产 owner 实际复用的字段语义与 resolution 语义；source-specific extraction 和领域行为留在对应 owner。
- 一次只激活一个 optimization slice，依次完成消费者审计、行为基线、最小实现、验证和 checkpoint。
- 首个 slice 从无生产消费者、无 public behavior 变化且可独立回滚的叶子维护面中选择。

本 change 保持现有 CLI、config、protocol、schema 和 output contract；新的输入类型或复杂 merge 需求由独立 change 承接。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `typed-fields`：明确共享 core 只拥有当前生产消费者共同依赖的字段语义。
- `cli-config-resolution`：明确内部简化必须保持 precedence、validation、fallback 与 provenance。

## Impact

- 候选代码：`crates/shared/typed-fields*` 与 `crates/shared/cli-config-resolution*` 中被当前 slice 选中的范围。
- 保护边界：`navigation`、`protocol`、`adapter-contracts`、`docnav` 和 adapters 的可观察行为。
- 规划材料：proposal、design、delta specs、tasks 和 `type-field-maintenance-report.md`。
- 回滚：每个 slice 独立提交；失败时回退该 slice，不引入 runtime fallback。
