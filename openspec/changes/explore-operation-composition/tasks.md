## 一句话核心

先把 operation composition 作为未来方向探索清楚；只有后续具体 change 才决定命令、字段、schema 和实现任务。

## 文档状态

本 change 只在 `openspec/changes/explore-operation-composition/` 下形成未审核的未来计划和探索材料，不影响现有其它文档、主规范或实现任务。

## 1. 探索边界

- [ ] 1.1 审计 proposal、design、spec 和 tasks，确认它们只表达 brainstorming / future plan，不包含实现承诺。

## 2. 候选模式

- [ ] 2.1 收集候选组合场景，例如小文档入口、多 ref 读取、find 后 read、ref 周边上下文读取。
- [ ] 2.3 补充候选池，至少覆盖多输入读取、明确结果自动展开、上下文扩展、continuation recipe、composition explain、批量搜索、outline preview、预算感知自动停止、输入归一化和 dry-run。
- [ ] 2.4 不为候选排序，不选择主方案；只记录足够后续比较的信息。

## 3. 临时筛选标准

- [ ] 3.1 维护一组临时筛选标准，用于后续讨论前粗筛候选。
- [ ] 3.2 筛选标准必须覆盖：是否组合现有 operation、是否默认归属 core/SDK、是否减少往返或状态管理、是否保持 ref opaque、是否能表达 continuation、是否优先复用现有 surface、是否避免污染 raw protocol、是否能用小 spike 验证。
- [ ] 3.3 明确该标准不是最终验收规则；候选进入实现前必须重新定稿 public contract。

## 4. 后续决策问题

- [ ] 4.2 判断哪些问题必须在实现前定稿，哪些可以通过 spike 或小范围实验确认。
- [ ] 4.3 明确后续实现不得直接复用本 change 的探索文本作为最终 contract；后续 implementation change 必须按 `replace-text-with-readable-view` 的最终 typed readable shape 和 renderer config 定稿 public contract。

## 5. 验证

- [ ] 5.1 运行 `openspec validate "explore-operation-composition" --type change --strict --no-interactive`。
- [ ] 5.2 搜索 active changes，确认旧的具体命令方案或过早实现承诺没有残留在本 change 相关引用中。
