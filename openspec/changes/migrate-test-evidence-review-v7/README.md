# migrate-test-evidence-review-v7

本 change 将 Docnav 测试证据单轨迁移到 test-evidence-review v7 的最小原生测试入口与分主题 case 目录模型。这里的 artifacts 描述目标和执行约束；迁移是否已经发生，只以仓库当前文件、测试证据和 `tasks.md` checkbox 为准。

## 阅读顺序与所有权

1. `proposal.md`：拥有迁移动机、范围、非产品边界和受影响区域。
2. `design.md`：拥有入口粒度、ID 继承、topic、单轨切换、验证集成和回滚取舍。
3. `specs/test-evidence-management/spec.md`：拥有迁移完成后必须满足的可观察契约。
4. `tasks.md`：拥有执行顺序、阻塞门禁和完成进度；不得从本 README 推断任务状态。
5. `migration-map.md`：由任务 1.2 在执行期创建，保存一次性审计、旧系统基线、逐 case 去向、验证结果和回滚记录；它不成为测试行为或长期维护规则的 owner。

## 执行入口

1. 先用 `openspec status` 和 `openspec instructions apply` 读取 OpenSpec 状态。
2. 完成 `tasks.md` 的 1.1 和 1.2，并在 `migration-map.md` 建立审计与映射骨架；两项未完成时不得修改 change 目录外的迁移目标。
3. 完成旧 case 与 runner 原生入口的逐项映射后，才能创建最终 topic/case 目录。
4. 按任务中的显式前置关系推进，每完成一项立即更新 checkbox 和迁移映射。
5. 发现未回答开放问题、缺少 owner 契约或无法确定入口粒度时停止；不得从 marker、文件名或旧 case 数量补造目标。
6. 只有全部任务和严格验证完成后，才把迁移视为完成并评估归档。
