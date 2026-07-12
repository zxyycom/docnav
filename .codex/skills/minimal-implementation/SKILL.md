---
name: minimal-implementation
description: >-
  在开始实现或修改代码之前使用；在 code review、CR 或交付前检查当前 diff
  时再次使用；用户要求审计指定代码范围的过度工程时也使用。用于约束新增
  维护面，并在实现后发现不必要的复杂度。
---

# Minimal Implementation

以最少必要维护面交付当前结果。把用户目标和已确认契约作为当前 scope，选择概念更少、职责更直接、验证更清楚的实现。维护面包括概念、分支、文件、依赖、配置和长期 ownership；代码行数作为结果信号。

## 工作模式

1. **Implement**：选择并应用最小正确方案，完成匹配验证。
2. **Review**：检查当前 diff 或指定改动，报告有证据支撑的复杂度收敛机会。
3. **Audit**：检查用户指定的代码范围，按置信度和维护收益给出排序报告。

用户同时要求实现和 review 时，完成 Implement 和验证，再检查最终 diff。用户要求更大代码范围的复杂度审计时，使用 Audit。

## 决策流程

1. **建立目标**：读取用户目标、相关代码、调用链、可观察契约和验证材料；写清必须交付的行为和最窄证明。
2. **锁定当前 scope**：将明确需求、现有消费者和已确认兼容性责任纳入本次交付；未来能力由真实消费者和可观察条件触发。
3. **依序检查候选方案**：
   1. 复用 codebase 已有的 helper、type、pattern 或责任位置。
   2. 使用覆盖所需行为和 edge cases 的标准库能力。
   3. 使用 OS、browser、database、protocol 或 framework 的原生机制。
   4. 复用完整覆盖需求的已安装依赖。
   5. 引入能显著降低长期维护与 correctness 风险的成熟依赖。
   6. 编写概念最少、表达清楚、符合相邻风格的自定义实现。
4. **应用最小正确门槛**：选择第一个完整通过下一节门槛的候选；同一层级有多个候选时，依次比较契约可靠性、概念数量、长期维护成本和验证清晰度。
5. **修复共同根因**：处理 bug 时先复现并追踪相关调用方，在覆盖受影响路径的最窄共享位置修复共同根因。
6. **处理可选复杂度**：对安全、可逆并保持既有 contract、compatibility、data handling 和状态语义的选择，采用简单默认值；涉及这些语义的决策以用户确认的方向为准。

## 最小正确门槛

候选方案同时满足以下条件时，进入实施或替代建议：

- 完整交付用户当前目标，并保持已确认 contract、compatibility 和 observable behavior。
- 覆盖外部输入验证、数据完整性、security、accessibility、必要 error handling、observability 和已声明的 performance budget。
- 处理由契约和所选机制带来的 edge cases，并采用清楚、常规、可维护的表达。
- 使用与受影响行为匹配的自动化测试、静态检查、可复现操作或其他直接证据完成验证。
- 保留由 deployment、hardware、data distribution 或 operator policy 决定的配置与校准点，并将其作为运行时输入。
- 评估“一种实现、一个调用方、一个导出”时，同步核对公开契约、ownership、test seam、生成流程和兼容性责任。

在同等正确的方案之间，优先删除或复用，优先常规写法，优先更小的长期维护面。简化方案存在明确 ceiling 时，记录 ceiling、可观察触发条件和升级路径。

## Implement

1. 用决策流程选择方案，并用最小正确门槛确认方案。
2. 将修改集中在承接当前行为的责任位置，使新增抽象、配置、文件和依赖与当前目标一一对应。
3. 运行与受影响行为匹配的最窄验证，并确认最终 diff 与当前交付一致。
4. 报告完成结果和验证证据；存在明确 ceiling 时，同时报告可观察触发条件和升级路径。

## Review

1. 确认 diff intent、changed surface 和需要保持的 contract。
2. 追踪调用方、已有能力、dependency behavior、配置生效路径和责任位置，形成有直接依据的 complexity finding。
3. 为每项 finding 给出更小替代方案和需要完成的验证。
4. 用最小正确门槛检查每个替代方案。

## Audit

1. 从用户请求确定审计范围和目标，优先检查维护者直接拥有的源代码、依赖和配置。
2. 建立组件、调用方、消费者、配置生效路径、依赖消费、转发层和平台能力之间的关系。
3. 根据关系证据识别重复实现、未来驱动的灵活性、转发层、依赖收敛和平台能力复用机会。
4. 对每个候选项追踪 contract、历史原因和验证责任；证据仍需补充的候选归入待确认问题。
5. 按置信度和可减少的维护面排序，给出可独立实施和验证的建议。

## Finding 分类与输出

使用以下标签：

- `delete`：调用、契约和验证证据共同支持移除的代码或配置。
- `reuse`：codebase 已有能力可以承接同一语义。
- `stdlib`：标准库完整覆盖自定义实现。
- `native`：平台原生能力完整覆盖依赖或自定义代码。
- `yagni`：把实现时机绑定到真实消费者或可观察触发条件的未来能力。
- `shrink`：保持清晰度和行为的同时减少重复逻辑或维护面。

Review 和 Audit 的 finding 使用：

`<confidence> <path>[:<line>] <tag> — <evidence>; <smallest alternative>; verify: <required evidence>.`

`confidence` 使用 `high`（有直接调用或 contract 证据）和 `medium`（证据充分，实施前完成已写明的检查）。证据仍需补充的候选归入待确认问题。Audit 先按 confidence，再按维护收益排序；收益使用可验证的维护面变化表达。

当前范围已保持精简时，写“当前范围已保持精简，complexity pass 完成。”

## 上游原文

本文件承接合并后的执行规则。比较 Ponytail 原始行为、审计本次提炼或更新固定版本时，读取以下材料。

上游固定为 [DietrichGebert/ponytail@14a0d795](https://github.com/DietrichGebert/ponytail/tree/14a0d79548d4de8fc2de95c1b94bb0de63a739d3)，许可证为 MIT：

- [ponytail-upstream.md](references/ponytail-upstream.md)：原始 implementation mode。
- [ponytail-review-upstream.md](references/ponytail-review-upstream.md)：原始 diff review。
- [ponytail-audit-upstream.md](references/ponytail-audit-upstream.md)：原始 repository audit。
- [ponytail-license.txt](references/ponytail-license.txt)：完整许可证。

`references/` 中的上游文件保持逐字副本。更新时固定新的 commit，并重新进行内容比对。执行时以用户目标、已确认契约和高优先级指令为依据；上游材料用于追溯和更新提炼。

## 完成检查

- 当前 scope、现有消费者和兼容性责任已经明确。
- 当前方案是候选优先级中第一个通过最小正确门槛的方案。
- bug fix 覆盖共同根因和相关调用路径。
- Implement 提供验证证据；Review/Audit finding 提供位置、证据、替代方案和验证要求。
- 简化方案存在明确 ceiling 时，已记录可观察触发条件和升级路径。
