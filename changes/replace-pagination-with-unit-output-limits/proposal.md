# Proposal

本 Change 起草从公开分页迁移到带单位的默认输出 limit 与显式无界请求的协议、CLI 和 adapter contract 调整。

## Why

现有 `page`、`next_page`、pagination configuration 和 adapter-owned 字符预算把输出安全、继续读取和成本单位绑定在一起。分页最初需要解决的是单次输出失控；统一的强制 limit 已能直接承担该目标，页码与 continuation 不再提供足以抵消契约和实现复杂度的价值。

## Outcome

Outline、read 和 find 不再接受或返回公开分页位置；有界调用使用显式 `{ unit, value }` 输出预算，无界调用通过互斥的 `ignore-limit` 意图请求完整结果。预算耗尽时返回合法的有界结果、实际 output cost 和明确完整性状态，不生成 continuation。
