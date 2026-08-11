# Proposal

本 Draft 保存一个尚未获准规划的交付方向：在不改变 linked adapter 与 invocation-private document 边界的前提下，为高频本地调用增加 core-owned 的可复用执行边界。

## Why

高频 CLI 调用可能重复承担进程启动和 core-owned 状态加载成本，但目前没有证据证明常驻 service 比更小缓存更必要，也没有确认生命周期、失效、错误和分发边界。由于候选方案会实质改变目标与维护面，本 Draft 不能自行选择其中一个，也不承担性能调查或长期决策记录的 owner 职责。

## Outcome

进入 Plan 前必须由用户明确选择唯一的交付目标：要么提供一个能跨 CLI 调用复用 core-owned state 的常驻 local core service，要么不新增常驻 service，改以更小的 cache/startup-path 优化达到届时确认的性能预算。选择后 proposal 只保留所选目标，并补全生命周期、失效、可观察性、分发与验证；两条候选都必须继续复用同一 CLI/operation contract、linked static registry 和 invocation-private adapter document 边界。在完成该选择前，不新增 service、缓存协议、runtime 或分发入口，也不由本 Draft 自动启动性能调查或长期决策记录。
