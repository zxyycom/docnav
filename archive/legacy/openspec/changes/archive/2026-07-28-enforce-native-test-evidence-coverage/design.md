# Design: 当前树驱动的测试评估

## Context

测试评估有两个不同问题：

1. 当前仓库到底有哪些可独立报告的原生测试入口，是否有漏项、悬空或重复。
2. 这些测试是否对稳定 owner contract 提供了可信且有信息增量的证明。

第一类问题可以由源码结构与 runner 报告确定性核对；第二类问题需要读取 owner、
测试实现和可观察信号后判断。把两者压进每入口一份手写文档，会同时削弱完整性和
信息密度。

本体系的主要消费者是项目内 AI agent。AI 日常获得稳定测试文档、项目 skill、
当前源码、runner 输出和可查询投影；它应能直接恢复入口宇宙、变化范围、Claim
质量门槛和验证动作。旧账本、迁移映射和历史编号不属于输入。

## 使用契约与术语

权威关系固定为：

1. 当前源码和 runner 报告拥有入口存在性与 runner 身份。
2. Evidence Claim Markdown 拥有少量长期测试语义。
3. Claim topic 表拥有当前受控分类。
4. machine inventory 和 query index 是可删除重建的当前投影。

术语：

- **NativeTestEntry**：runner 能稳定独立选择或报告，并拥有一项完整测试意图的
  最小原生节点。
- **machine case**：一个 NativeTestEntry 的确定性机器投影；case 与 Entry 一一
  对应，不另建手写文件。
- **Evidence Claim**：不能从 owner requirement 加 Entry 名称直接恢复、且能改善
  后续审查的长期判断。
- **supported runner profile**：纳入 required 门禁的 Cargo target、Bun surface
  和 smoke root 及其固定枚举方式。
- **baseline**：一次当前 inventory 快照，只用于比较两个当前状态，不是历史兼容层。

## Goals / Non-Goals

**Goals:**

- 完整当前树中每个受支持原生入口恰好形成一个 machine case。
- 静态候选、runner 报告与 committed inventory 双向闭合。
- 让 AI 能快速定位新增、删除、重命名候选、实现变化和 Claim 陈旧。
- 只保留 owner-backed、有可观察信号且有信息增量的 Claim。
- required 验证离线、仓库内、确定性且诊断可机器读取。

**Non-Goals:**

- 不自动判断产品实现变化后测试是否充分。
- 不从测试名、AST、断言或 owner 文档自动生成 Claim 语义。
- 不要求每个测试拥有 Claim、topic 或手写说明。
- 不提供旧测试账本的兼容读取、迁移映射、编号连续性或恢复协议。
- 不改变 Docnav 产品运行时或 release file set。

## Decisions

### Decision 1: 完整当前树是唯一入口宇宙

版本化 supported runner profile 列出 Cargo test targets、Bun test surfaces 和
smoke task roots。每次严格检查都读取最终当前树，不以 Git diff、人工抽样或已有
inventory 缩小发现范围。

lint、类型检查、schema、生成物一致性、CI job、fixture、helper、hook、断言和
测试步骤不是原生测试入口。

### Decision 2: 静态结构与 runner 报告双向闭合

每个 runner adapter 同时产生静态候选与 runtime entries，并归一到同一身份。严格
检查至少区分：

- `static-only`
- `runtime-only`
- `duplicate-entry`
- `unsupported-entry-shape`

ast-grep 只负责语法结构候选和规则回归，不推断宏展开、类型关系或运行时组合。
无法稳定归一的动态形态必须显式失败，不能静默排除。

### Decision 3: Entry 本身就是 machine case

共享 `NativeTestEntry` 包含 `entryKey`、`runner`、`target`、`selector`、
`sourcePath`、`sourceRange` 和 `sourceFingerprint`。每个闭合 Entry 恰好生成一个
case；聚合容器和内部步骤不生成 case。

Committed inventory 保留两个当前用途：让 required check 检出未同步的入口变化，
以及让 AI 在不重新执行 runner 时做快速查询和 Git diff 审查。它不能反向创建
入口，也不拥有长期语义。

### Decision 4: Claim 只保存信息增量

Evidence Claim 必须同时满足：

- `ownerRef` 精确定位当前 requirement。
- `statement` 表达稳定契约，而不是测试名或实现步骤。
- `observations` 描述调用方可判断的结果。
- `supportedBy` 至少引用一个当前 `entryKey`。
- 内容能改善后续 AI 审查。

普通 Entry 可以没有 Claim。Claim ID 按稳定语义命名，不编码旧目录或旧 case ID。
topic 只在至少有一个当前 Claim 使用时保留。测试名已经完整表达的简单证明不建立
Claim。

### Decision 5: 结构变化由机器定位，充分性由 AI 判断

相对显式 baseline 的报告提供新增、删除、rename candidate 和
`implementation-changed`；index 对 owner section 或 `supportedBy` 变化报告
`claim-stale`。这些信号只缩小 AI 阅读范围。

AI 仍需读取行为 owner、变化测试和产品影响面，判断证明目标、可观察性、可靠性、
证据独立性与维护价值。同步 fingerprint 不能替代该判断。

### Decision 6: 查询投影可重建且只读回退

统一 index 从 inventory、topic 与 Claims 生成，并支持按 `entryKey`、runner、
target、sourcePath、Claim ID、topic、ownerRef 和文本进行有界查询。索引缺失或
陈旧时，`list` / `show` 可以返回带 warning 的内存投影，但不得隐式写回；严格
`check` 必须失败。

### Decision 7: 项目 wrapper 与通用 skill 分责

`scripts/test-evidence/` 拥有 supported runner profile、ast-grep rules、runner
调用、归一、闭合和 inventory 生成。项目级 `test-evidence-review` skill 拥有
通用 Entry/Claim/index 契约、审查顺序和完成标准。

`validate:docs -- cases` 只从项目 wrapper 进入。发现、runner、inventory、Claim 和
index 失败保持不同 origin 与退出状态。

### Decision 8: 开发期 ast-grep 与产品运行时隔离

仓库精确锁定 `@ast-grep/cli`，只允许 `scripts/test-evidence/ast-grep.ts` 调用。
rules、CLI 和 skill 属于开发验证链，不进入 canonical release；产品期
`docnav-code` 仍遵守自己的进程内 Rust crate 边界。

## Risks / Trade-offs

- ast-grep 规则可能漏掉 alias、宏或动态注册。静态/runtime 双向集合与 unsupported
  规则把漏项转成阻断诊断。
- runner 枚举受 feature、target 或环境影响。supported runner profile 固定命令、
  target 和必要环境。
- source fingerprint 可能产生格式噪声。adapter 对入口声明做规范化，并把
  fingerprint 仅用作审查信号。
- committed inventory/index 增加生成物体积。它们换取当前状态的快速查询和 Git
  审查；两者明确为可删除重建投影，不再复制手写语义。
- AI 可以机械同步 stale 制品。严格检查只证明结构；skill 与变更验收仍要求读取
  owner 和测试内容，不宣称自动证明充分性。

## Implementation Plan

1. 固定 runner profile、ast-grep toolchain、静态规则和正反例。
2. 实现 Cargo、Bun、smoke adapter 与 static/runtime 闭合。
3. 实现确定性 inventory、诊断和显式 baseline 变化报告。
4. 实现 Claim schema、topic、query index、stale 检测与审查 skill。
5. 从当前 owner 与测试实现建立高信息 Claim，删除模板叙述和未使用 topic。
6. 将 docs validator、workspace verifier、稳定文档和 active changes 切到单一当前
   链路。
7. 运行 rule、catalog、docs、OpenSpec、release boundary 和 workspace 验证。

## Open Questions

无未回答开放问题。实现只对当前测试树和当前 Claim 负责。
