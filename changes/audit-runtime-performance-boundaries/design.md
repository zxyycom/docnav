# Design

审计以有限 workload packet 为输入，依次完成 investigation report、human decision 和 owner handoff；每一步写入自己的权威载体，禁止从单次数字直接跳到优化。

## Context

- 当前没有稳定 owner 同时解释 Docnav 的启动、CPU/I/O、内存、输出、package 和极端输入 runtime evidence。
- 质量观测主要处理仓库静态/工程指标，不应被扩张为产品 runtime performance owner。
- 既有 JSON、长 key/output 和 tokenizer 数字只作为需要复现的 seed observations；缺少命令、build、fixture 或环境时不能成为 Current baseline。

## Goals / Non-Goals

Goals:

- 用有限且可停止的 packet 建立可比较证据。
- 区分 observation、reproducible baseline、approved budget 与 blocking gate。
- 先归因，再由人类决定是否产生长期规则或 owner-specific optimization。

Non-Goals:

- 不建立 benchmark framework、dashboard、通用 producer/sink 或默认 CI gate。
- 不在本 change 选择产品或实现机制。
- 不在 Change 目录保存第二份 audit report，也不让调查报告替代长期决策、当前行为 owner 或实现证据。

## Decisions

### 1. Representative 与 stress workload 分账

Representative 描述正常使用证据；stress/adversarial 用于发现增长和边界失效。两类结果不得互相冒充，也不自动具有相同阻断权重。

### 2. Initial packet 有限且预先停止

按 format、operation、output/page 和 stress shape 选择 required cells，不做全组合；所有 required cells 完成或有证据地标记 unavailable/not-applicable 后停止首轮扩展。

### 3. Investigation report 拥有形成时测量证据

本轮完成后按 `investigation-report` 固定结构写入独立 runtime 主题。每条可比较记录至少包含 binary、command、flags、fixture、output/page/limit/query/ref、build、host、cache、repeats、measurement definition、raw samples 和 noise assumptions；只有复核结论需要且正文不适合完整承载时，才把最小原始材料作为报告随附资源。

### 4. 先归因，再决定 owner

使用 startup/process、input I/O、routing、decode/parse/index、operation traversal、重复准备/composition、cost、pagination/output、serialization/write、memory retention、package/dependency 等 categories；无法证明时保留 `unattributed`。

### 5. Observation 与采用决定分属不同 owner

调查报告中的 observation 默认非阻断。Baseline、数字 budget 和 blocking gate 分别由人类批准，并同时固定 workload、指标、统计、环境、噪声、执行 owner 和更新/移除条件；跨 change 持续有效的批准结果只在用户明确授权维护决策时由 `decision-records` 承接，报告只保留形成时证据。批准方向本身不自动授权执行子代理写决策、Current owner 或后续 Change。

### 6. 优化交还行为 owner

审计只形成证据和候选验收 workload；获准修复交给对应行为 owner。需要持久 Change 时先向用户说明候选，只有用户明确要求才创建或维护，避免性能报告或本计划成为隐藏产品 spec 或自动任务队列。

## Risks / Trade-offs

- Workload 容易无限扩展；用 required cells、明确未测状态和停止规则限制。
- 环境噪声可能制造虚假回归；保存 raw samples、repeats、host/cache/build 条件和不可比标记。
- 总耗时可能诱导预选方案；只有归因证据和 owner review 后才能形成 handoff。
- Investigation report、decision 和 runtime owner 容易重复同一数字；报告保存形成时测量，decision 保存采用理由，owner 只保存当前有效规则并引用必要来源。
- 新 runtime owner 文档可能与 tooling 重叠；只在批准长期规则后创建，并明确产品 runtime 与静态质量边界。

## Open Questions

无未回答的计划结构问题。哪些 measurements 获准成为 baseline、是否建立 budget/gate、哪些 handoff 值得建议后续 Change，是本计划 Implementation 中有明确人类 owner 的硬门禁；对应下游任务必须等待门禁关闭。建议本身不授权创建持久 Change，批准方向也不替代对应载体的写入授权。
