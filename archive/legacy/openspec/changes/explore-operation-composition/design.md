## 一句话核心

operation composition 是一个未来方向：用 core/shared-library 编排现有 document operations，减少重复调用，同时避免提前把候选体验固化成 adapter 协议或新命令。

## 文档状态

本 change 只在 `openspec/changes/explore-operation-composition/` 下形成未审核的未来计划和探索材料，不影响现有其它文档、主规范或实现任务。

## Context


很多体验优化并不要求 adapter 增加能力，而是把已有 operation 按固定模式串起来。例如先 outline 再 read、先 find 再 read、对一组 ref 重复 read。这类逻辑更像接入层编排或 shared helper，而不是格式语义。

## Goals / Non-Goals

Goals:

- 记录 operation composition 的问题空间、候选模式和边界。
- 明确当前阶段是 brainstorming / future plan，不进入实现设计。
- 避免把任一候选命令放到主位置。
- 为后续 implementation change 提供决策问题和验收方向。

Non-Goals:

- 不新增 adapter-level operation。
- 不修改现有 `outline/read/find/info` 行为。
- 不把候选体验排序成 roadmap 优先级。

## Direction

1. 优先把 composition 放在 core/shared-library 层。

   core 拥有 command classification、config source descriptor/path handoff、output mode、error mapping 和 readable output；`docnav-navigation` 拥有 raw config source loading、adapter selection 与 navigation input resolution。shared library helper 可以承载可复用的组合逻辑。格式 adapter 应继续专注单次格式 operation。

2. 优先复用现有 public surface。


3. 先确定 composition 是否属于 core protocol 语义。

   Core document operation 的两条 output path 共享同一个 `ProtocolResponse`。若 composition 在 core 中执行额外 operation 并向用户暴露结果，这些事实必须进入 typed protocol result，再分别由 `protocol-json` 序列化和内置 renderer 投影；若它只是接入层 convenience，则应由 SDK/MCP 等调用方组合基础 operations，不在 core 中引入 renderer-only 业务结果。

4. 把候选体验当作 examples，不当作当前承诺。

   可继续探索的候选包括小文档直接读、多个 ref 一次读取、搜索后自动读取明确结果、围绕 ref 获取上下文等。候选存在是为了帮助识别模式，不代表本 change 要实现或优先实现。

5. 后续实现 change 必须重新定稿 public contract。


## Candidate Patterns

以下候选只用于后续讨论和比较，不代表已选功能、优先级或最终命令形态：

1. 多输入读取：同一 operation 接受多个同类输入，例如多个 ref 或多个 query，由 core/SDK 循环调用现有 operation 并汇总 readable output。
2. 明确结果自动展开：先执行 outline 或 find，当结果足够明确且预算允许时自动推进到 read；不明确时返回原始导航结果。
3. 上下文扩展：给定一个 ref，围绕当前 outline context 读取相邻或相关上下文，减少手动来回读取。
4. continuation recipe：在 readable output 中提供可执行的下一步提示，说明应保留哪些 path、ref、query、options、limit 和 page。
5. composition explain：不读取额外内容，只解释一次组合请求会如何选择 adapter、应用默认值和调用基础 operations。若 explain 成为 core document operation，其 typed result 进入统一 `ProtocolResponse`，`protocol-json` 暴露事实，`readable-view` 由内置 renderer 投影。
6. 批量搜索：一次传入多个 query，按 query 分组返回 find 结果，帮助调用方在术语不确定时减少往返。
7. outline preview：在 outline 结果中附带少量预算内预览，帮助调用方决定下一步 read；是否属于 adapter 展示语义需要后续审计。
8. 预算感知自动停止：组合流程共享总 limit，在预算耗尽时稳定停止并标记 pending 或 continuation。
9. 输入归一化：允许更自然的单值/数组/重复 flag 输入形态，降低接入方在简单批量场景中的参数处理成本。
10. composition dry-run：返回将要执行的 operation plan、顺序和可能的输出 mode，但不读取文档内容。

## Temporary Screening Criteria

后续讨论前先用以下临时标准粗筛候选；该标准不是最终验收规则：

1. 是否主要由现有 `outline`、`read`、`find` 或 `info` 编排完成。
2. 是否能默认放在 core/shared-library 层，而不要求格式 adapter 增加新语义。
3. 是否减少调用方的重复往返、状态记忆或参数拼接。
5. 是否能用 readable output 清楚表达结果、未完成项和 continuation。
6. 是否能复用现有 command、option、输入归一化或 shared helper；只有复用会造成歧义时才考虑新入口。
7. 是否避免污染 raw adapter protocol；需要 protocol 扩展时必须先证明它是长期跨 adapter 语义。
8. 是否有足够小的 spike 或示例可以验证体验收益，而不直接承诺完整实现。

## Risks / Trade-offs

- [Risk] 过早设计具体命令会把探索空间锁死。-> Mitigation: 当前 artifacts 只记录方向和决策问题，不写最终 command contract。
- [Risk] composition 被错误下放到 adapter，造成重复实现。-> Mitigation: 记录默认归属为 core/SDK，adapter 只在需要格式语义时参与。
- [Risk] 过度追求快捷入口会破坏 `outline -> ref -> read` 的清晰模型。-> Mitigation: 后续实现必须说明它如何复用或补充基础链路。
- [Risk] composition 业务事实只存在于 renderer，导致两条 output path 语义分叉。-> Mitigation: 后续 change 必须二选一：core-owned composition 定义统一 typed protocol result；接入层 convenience 只组合既有 operations，不修改 core document output。

## Follow-Up Plan

1. 收集真实使用流程中重复出现的 operation 序列。
3. 选择一个足够小的候选做后续 implementation change。后续 implementation change 必须声明 composition owner；core-owned composition 同步定义 `ProtocolResponse` result、`protocol-json` schema/example、`readable-view` renderer mapping 和两种 public document output mode。
4. 在后续 change 中再更新主规范、schema、examples 和测试。

## Open Questions

- 哪些组合属于“使用体验 convenience”，哪些应成为长期 public contract？
- Shared helper 应支持哪些组合，哪些只属于 core CLI？
- composition 的 continuation 是统一模型，还是每个候选单独设计？
