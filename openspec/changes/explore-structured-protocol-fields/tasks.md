本 tasks 清单记录 Docnav raw protocol 字段结构化探索入口；当前只在 `openspec/changes/explore-structured-protocol-fields/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“先探索 raw protocol 结构化字段，再确认具体迁移任务”这一核心目标；审计未完成前不得执行实现任务。
- [ ] 1.2 审计 capability ID 是否只复用现有 `docnav-contracts`。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/explore-structured-protocol-fields/` 下的未审核临时 artifacts。

## 2. 字段探索

- [ ] 2.1 审计 `docs/protocol.md`、protocol schemas、examples 和 readable schema 中承载结构化语义的字段。
- [ ] 2.2 分类字段 owner：protocol-owned、adapter-owned、core-owned、readable-only。
- [ ] 2.3 探索 `limit`、`cost`、outline entries、find matches、info result、page 和 error details 的候选 shape。
- [ ] 2.4 明确 raw protocol 与 readable output 的映射原则。

## 3. 收敛与拆分

- [ ] 3.1 给出候选 vNext protocol shape 和兼容策略。
- [ ] 3.2 标出会影响 `configure-pagination-defaults` 与 `use-token-based-document-cost` 的决策。
- [ ] 3.3 将确认后的迁移拆成后续实现 change 或更新现有 change 的具体任务。
