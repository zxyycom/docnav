本 change 记录统一 diagnostic channel 的未来方向；当前内容只是 `openspec/changes/unify-diagnostic-channel-direction/` 下的未审核临时文档，不改变现有主规范、schema、示例或实现行为。

## Why

当前项目已经有 `StableError`、稳定 warning envelope、standard parameter diagnostics 和若干直接 stderr 诊断，但它们不是同一个可收集、可传递、可延迟输出的通道。结果是可恢复点可以继续执行，但错误信息可能被拆到 readable payload、protocol stderr 或纯文本 stderr，执行链中也不能在任意时刻统一取出完整诊断集合。

这个 change 用于记录方向：先把内部诊断收集模型统一起来，再由输出层按现有 surface contract 决定如何呈现，避免为了短期统一而直接破坏 `protocol-json`、manifest、probe 或 readable output 的既有边界。

## What Changes

- 引入目标性要求：文档操作、adapter direct CLI、adapter invoke、标准参数解析和输出编排应朝统一 diagnostic event/bag 模型演进，warning 和 error 都先作为结构化事件进入同一内部收集通道。
- 明确兼容路径：第一阶段只统一内部 handoff，不要求 `protocol-json` stdout 增加 `warnings` 或 `diagnostics` 字段，不改变 manifest/probe schema，也不改变 readable output 的 documented shape。
- 明确 surface policy：输出层仍按调用者选择的 surface 决定事件呈现位置；`readable-view`/`readable-json` 可承载可恢复 warning，`protocol-json`、manifest 和 probe 继续保持 stdout 纯净，必要诊断写 stderr。
- 收口直接 stderr 旁路：SDK direct CLI、adapter boundary、invoke decode、JSON/schema/write failure 等路径后续应先生成 diagnostic event，再统一 flush 到 stderr 或转换为 stable error output。
- 保留稳定错误语义：阻断执行的错误仍映射为 `StableError` 和现有 exit code；统一 channel 不把可恢复 warning 升级为失败，也不把 protocol failure envelope 混入 readable wrapper。
- 记录已知不一致：`readable-common.schema.json` 允许 `effect: diagnostic_only`，但 Rust `WarningEffect` 当前没有对应 variant；后续实现前需要决定是补齐 Rust enum，还是从 schema 移除该目标值。

## Capabilities

### New Capabilities

本 change 不新增长期 capability。

### Modified Capabilities

- `docnav-contracts`: 记录统一 diagnostic channel 的目标边界、兼容路径和跨 surface 输出策略。

## Impact

- 影响面包括 `docnav-diagnostics`、`docnav-output`、`docnav` core output/runtime、`docnav-adapter-sdk` direct CLI/invoke/output、`docnav-standard-parameters` diagnostics handoff，以及相关 smoke/assertion。
- 可观察 contract 变更默认不在本 change 第一阶段发生；若未来要求 protocol envelope 直接承载 warnings/diagnostics，必须作为显式 breaking contract 决策，另行更新 docs、schema、examples、fixtures 和 consumer tests。
- 本 change 只保存方向和任务入口；实现前必须完成阻塞级审计，确认是否保持外部兼容还是进入 breaking contract 迁移。
