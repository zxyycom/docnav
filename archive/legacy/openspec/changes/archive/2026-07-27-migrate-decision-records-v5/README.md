# migrate-decision-records-v5

本 change 将 Docnav 项目级 decision-records、决策数据和校验入口单轨迁移到 v5 领域与对齐模型，同时保持长期决策 owner 分工。这里的 artifacts 描述目标和执行约束；迁移是否已经发生，只以仓库当前文件、测试证据和 `tasks.md` checkbox 为准。

## 阅读顺序与所有权

1. `proposal.md`：拥有迁移动机、范围、非产品边界和受影响区域。
2. `design.md`：拥有固定上游版本、数据转换、对齐判断、验证集成和回滚取舍。
3. `specs/decision-record-management/spec.md`：拥有迁移完成后必须满足的可观察契约。
4. `tasks.md`：拥有执行顺序、阻塞门禁和完成进度；不得从本 README 推断任务状态。
5. `migration-record.md`：由任务 1.2 在执行期创建，保存一次性审计、迁移基线、对齐证据、验证结果和回滚记录；它不成为长期决策或产品行为 owner。

## 执行入口

1. 先用 `openspec status` 和 `openspec instructions apply` 读取 OpenSpec 状态。
2. 完成 `tasks.md` 的 1.1 和 1.2，并把审计结论写入 `migration-record.md`；两项未完成时不得修改 change 目录外的迁移目标。
3. 按任务中的显式前置关系推进，每完成一项立即更新 checkbox 和执行记录。
4. 发现未回答开放问题、语义无法保真转换或活动决策缺少对齐证据时停止；先修订 change 或保留 `unaligned`，不得自行补造事实。
5. 只有全部任务和严格验证完成后，才把迁移视为完成并评估归档。
