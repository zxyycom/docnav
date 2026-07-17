# simplify-typed-field-infrastructure

本 change 为 typed-field 基础设施建立渐进优化入口：保留已被生产使用的共享语义，一次只实施一个可验证、可回滚的优化 slice。

## 阅读顺序

1. `type-field-maintenance-report.md`：调查、内部审计和 A/B 实验形成的决策证据。
2. `proposal.md`：目标、范围和 affected capabilities。
3. `design.md`：责任边界、slice 状态转移和回滚方式。
4. `specs/`：优化过程中必须保持的稳定契约。
5. `tasks.md`：阻塞审计、首个 slice 和验收清单。

当前 artifacts 未审核，首个 slice 尚未选择。`derive-document-cli-options-from-fields` 暂作为历史计划保留，本 change 不直接应用或归档其 deltas。
