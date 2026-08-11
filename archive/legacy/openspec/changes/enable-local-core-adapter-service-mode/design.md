本 design 只记录 core service 性能方向；当前内容不授权 adapter service、external executable fallback 或动态 adapter source。

## Context

`adopt-core-linked-adapter-libraries` 使 adapter layer 作为 core release workspace crate 编译并通过 static registry 接入。service mode 如需存在，应优化 core CLI 高频调用成本，而不是定义新的 adapter runtime。

## Goals / Non-Goals

**Goals:**

- 探索 core-local service 对 startup cost、project/config loading 和 static registry metadata 的缓存收益。
- 定义缓存失效和 doctor/status 可观察边界。
- 保持 document success output 和 protocol/readable failure projection 不变。

**Non-Goals:**

- 不实现 adapter service loop。
- 不把 service 作为 adapter implementation source。
- 不 fallback 到 adapter `invoke` 或外部 executable。
- 不把 internal service protocol 加入 public `docnav-protocol` schema。

## Decisions

1. Service 只能缓存 core-owned state。
   - Project context、config state、static registry metadata 可以被缓存。
   - Adapter parser output、ref semantics 和 native option behavior 仍由 adapter owner 决定，缓存策略不得绕过 adapter-owned invalidation。

2. Service 调用同一 static registry adapter handle。
   - 无论 service 是否启用，document operation 都使用 current core release adapter layer。
   - service failure 不改变 adapter selection semantics。

3. Output contract 不变。
   - service status 可以进入 doctor/status 或内部诊断。
   - document success stdout 仍只包含 documented payload。

## Risks / Trade-offs

- [Risk] 缓存失效错误导致 stale reads。→ Mitigation: 首轮只缓存 core-owned metadata，并为 document file content 保持 per-request adapter parsing。
- [Risk] service 状态污染输出。→ Mitigation: success payload 不包含 service status，status 只在 doctor/context 输出中出现。

## Open Questions

- 是否需要 foreground `docnav service run`，还是只记录为后续性能方向。
- 缓存 profile 和基准指标需要由后续性能 change 固定。
