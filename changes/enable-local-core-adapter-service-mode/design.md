# Design

本 Draft 从历史 OpenSpec change 恢复 core-owned 执行复用方向，并在进入 Plan 前保留“常驻 local core service”与“更小的 cache/startup-path 优化”两条待选路径。

## Context

- 当前 `proposal.md` 尚未选择唯一交付目标，也没有性能基线证明常驻 service 是必要的最小方案。
- [历史 OpenSpec design](../../archive/legacy/openspec/changes/enable-local-core-adapter-service-mode/design.md) 已把该方向从 adapter service 改写为 core service 性能问题：复用对象只属于 core-owned state，不提供 adapter implementation source、external executable fallback 或动态 adapter runtime。
- 当前[架构](../../docs/architecture.md)和[适配器契约](../../docs/adapter-contract.md)以 core release 的 static registry 与 linked adapter library handle 执行文档操作；adapter-private document state 只在一次 invocation 内存在。
- 当前[调用内文档准备决策](../../docs/decisions/adapter-document-lifecycle/keep-document-preparation-invocation-private.md)禁止把 private document state 放入公共契约、全局状态注册表或跨调用 cache；它不替本 Change 决定 core-owned project、config 或 registry state 是否值得复用。

## Goals / Non-Goals

**Goals:**

- 恢复历史 change 中仍与当前 owner 一致的 core-owned state、static registry、输出纯度和失效边界。
- 为后续在常驻 local core service 与更小 cache/startup-path 优化之间作出唯一选择提供可审阅起点。
- 无论选择哪条路径，都保持现有 CLI/operation contract、linked static registry 和 invocation-private adapter document 生命周期。

**Non-Goals:**

- 本 Draft 不选择交付路径，不证明性能收益，也不授权实现 service、cache protocol、runtime 或分发入口。
- 不提供 adapter executable discovery、artifact hosting、invoke fallback 或第二个 adapter implementation source。
- 不缓存 adapter parser output、document content、ref 或其它 adapter-private state，也不把内部状态加入 public protocol、readable output 或 caller-visible identifier。

## Decisions

以下边界从历史 change 恢复，并由当前 owner 继续支持：

1. 可复用状态只属于 core owner；document acquisition、decode、parse、private model、ref 和 operation 语义继续由 invocation-private `AdapterDocument` 拥有。
2. service 或优化路径都调用当前 core release 的同一 static registry adapter handle；内部失败不得改变 adapter selection 或引入 fallback source。
3. Document success stdout、`protocol-json` 和 `readable-view` 契约保持不变；内部状态只能进入既有 owner 明确允许的诊断或 doctor/status surface。

以下仍是暂定候选，不是已确认决策：

- **候选 A：常驻 local core service。** 跨 CLI 调用复用安全可失效的 core-owned project、config、registry metadata 或执行上下文，并单独定义生命周期、失效、可观察性和分发边界。
- **候选 B：更小的 cache/startup-path 优化。** 不新增常驻 service，只优化经测量确认的启动或 core-owned state 加载路径，并以届时确认的性能预算约束缓存范围。

进入 Plan 前必须由用户选择一个候选，删除另一个候选的交付义务，再从所选 design 派生 tasks。

## Risks / Trade-offs

- 跨调用复用可能返回陈旧的 project、config 或 registry state；任何候选都需要按实际 owner、输入来源和变更事件定义失效，不能用缓存绕过正常校验。
- 常驻 service 可能以生命周期、并发、故障恢复和分发维护面换取有限收益；更小优化的维护面较低，但可能达不到最终性能预算。
- 内部 service/cache 状态若混入 document success 输出，会破坏已有 public contract；可观察性必须由 doctor/status、诊断或其它既有 owner 承接。
- 历史 OpenSpec 只提供形成时设计输入；凡与当前 owner 冲突的历史内容均不恢复为当前事实。

## Open Questions

- 用户选择候选 A 还是候选 B？
- 需要改善的 cold/hot 路径、基线、目标预算和代表性工作负载是什么？
- 若选择常驻 service，其启动方式、进程生命周期、并发、缓存 profile、失效、故障回退、doctor/status 和分发边界分别是什么？
- 若选择更小优化，实际瓶颈位于哪些 startup/config/project/registry 步骤，允许的缓存持久范围和失效信号是什么？
