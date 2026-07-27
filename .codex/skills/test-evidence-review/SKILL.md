---
name: test-evidence-review
description: >-
  在新增、修改、删除或审查测试实现，查询原生测试入口或 Evidence Claim，或处理
  测试 rename、split、merge 与证据陈旧性时使用。先要求项目 runner profile
  对静态入口、运行时入口和 machine inventory 做全树闭合，再审查测试的契约、
  可观察信号、可靠性与 Claim 信息增量。仅运行既有测试、只修改被测对象或处理
  lint、类型检查、schema、构建等工程校验时不使用。
---

# Test Evidence Review

## 目标

把两类责任分开：

1. **NativeTestEntry** 是 runner 能稳定独立报告或选择的当前原生测试入口。项目
   wrapper 从源码和 runner 报告发现入口，并为每个入口生成一个 machine case。
2. **Evidence Claim** 是不能从测试名或 AST 机械恢复的长期判断，记录精确行为
   owner、契约陈述、可观察结果和支持它的当前入口。

普通内部测试允许没有 Claim。Claim 必须有至少一个当前入口。不得为每个入口生成
重复的 `Contract` / `Proves` 模板，也不得把 AST、测试名或实现断言改写成看似有
语义的 Claim。

## 适用边界

项目 wrapper 拥有 supported runner profile、ast-grep 规则、runner 调用、入口
归一、闭合检查和 inventory 生成。本 skill 只拥有通用 Entry/Claim/index 契约、
查询模型、证据审查和完成标准。

以下对象不是 NativeTestEntry：测试文件、suite、package script、runner、CI job、
lint、类型检查、fixture、helper、hook、mock、断言和测试步骤。聚合节点包含可分别
报告的更小节点时，machine case 对应更小节点。

需要字段、路径、诊断或 CLI 细节时读取
[证据目录契约](references/evidence-contract.md)。

## 审查顺序

1. 读取项目测试约定、行为 owner、目标测试和当前 diff。
2. 运行项目 wrapper 的全树检查。静态入口、运行时入口和 committed inventory
   必须双向闭合；Git diff 或局部清单不能替代这一步。
3. 查询相关 Entry 与 Claim。变更报告只用于缩小审查范围，不代表未报告入口已经
   证明充分。
4. 对新增或变化的测试判断：
   - **契约背景**：对应哪个稳定 requirement 或边界。
   - **证明信号**：失败是否能指向该契约失效。
   - **可观察性**：是否断言调用方可见的返回值、错误、状态、交互或资源结果。
   - **可靠性**：fixture、mock、时序、随机性和环境是否稳定。
   - **证据独立性**：预期值是否独立于被测实现。
   - **维护价值**：证明增量是否值得运行与维护成本。
5. 只有长期语义无法由 owner 加 Entry 直接恢复、且对后续审查有信息增量时，才新增
   或更新 Claim。
6. 同步 inventory/index，运行目标测试和项目严格检查。

## Claim 门槛

Claim 必须同时满足：

1. `ownerRef` 精确定位当前行为 owner 中的 requirement。
2. `statement` 陈述稳定契约，而不是测试名称、实现步骤或通用模板。
3. `observations` 描述失败时能判断的调用方可观察结果。
4. `supportedBy` 只引用当前 inventory 的 `entryKey`，至少一项。
5. topic 来自受控表，Claim ID 按稳定语义命名且全局唯一；topic 只在至少一个
   当前 Claim 使用它时保留。

以下内容不建立 Claim：

- “测试稳定契约”“结果可观察”等无信息模板。
- 只复述函数名、测试名、AST match 或实现分支。
- 仅对 fixture、mock、helper 或内部步骤成立的陈述。
- 已由精确 owner requirement 和 Entry 名称充分表达、不会改善后续判断的重复说明。

## 结构变化

- **rename candidate**：先确认行为语义是否连续；保留 Claim ID，更新
  `supportedBy`，不要把 `entryKey` 当长期身份。
- **split**：按各入口实际证明信号分配旧 Claim；只有独立长期判断才拆 Claim。
- **merge**：合并入口不自动合并 Claim；分别判断每个长期判断是否仍成立。
- **删除**：没有当前入口支持的 Claim 必须删除、重写或重新关联，不能悬空。
- **implementation-changed / claim-stale**：读取 owner 与测试正文重新判断；不能仅
  通过更新 fingerprint 消除审查。

## 查询

先用项目 wrapper 的 `topics` 或有界 `list` 缩小范围，再用 `show` 展开单个
Entry 或 Claim。查询可按 `entryKey`、runner、target、sourcePath、Claim ID、
精确 topic、ownerRef 和文本过滤。

索引缺失或陈旧时，只读查询可以使用带 warning 的内存投影，但不得写回。严格
`check` 必须要求 committed inventory 和 index 都与当前树一致。

## 完成标准

1. supported profile 的静态入口、运行时入口与 machine inventory 完全闭合；所有
   unsupported、duplicate、missing、orphan 或 stale 诊断已处理。
2. 每个当前 Entry 恰好有一个 machine case；普通 Entry 可以没有 Claim。
3. 每个 Claim 通过 owner、topic、信息增量、观察信号和当前 Entry 引用检查。
4. rename/split/merge/delete 的 Claim 连续性已明确审查，没有靠机械生成补语义。
5. 目标测试、项目 wrapper 严格检查和范围匹配的工作区验证已经通过，或阻塞边界已
   明确说明。
