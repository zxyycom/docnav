---
title: 用带单位的输出上限替代分页
status: active
alignment: unaligned
createdAt: 2026-08-12T03:18:35Z
purpose: 让默认调用以统一成本单位限制输出，并由调用方显式选择是否解除限制，而不再维护公开分页位置。
background: 现有分页主要用于防止单次输出失控，但 page、continuation 与 adapter-owned 字符预算增加了跨操作差异和继续读取复杂度。
decision: 移除公开 page 与 continuation，改用显式携带 unit 和 value 的默认输出上限，并提供互斥的 ignore-limit 无界请求。
relations: []
---

## 目的
- 以一个容易理解、默认安全的输出上限控制单次工具结果量级。
- 让 read、outline 和 find 使用同一种预算概念，而不再把安全上限建模为公开页码。

## 背景
- 当前 `limit` 是 adapter-owned 无单位预算；Markdown 与 JSON 的主要分页路径按 Unicode 字符消费预算，而协议 cost 还能同时报告 lines、bytes 和 tokens。
- `page`、`next_page` 和分页前置扫描原本服务于有限输出和继续读取，但输出安全本身只需要一个强制上限。
- 当调用方明确接受完整结果时，继续维护页码和 continuation 不如显式解除普通限制直接。

## 决策
- 采用: 从 public request、result 和 adapter operation contract 移除 `page`、`next_page` 及 continuation 语义；预算耗尽后的后续动作由调用方通过更具体的 ref、更大的 limit 或重新发起无界请求决定。
- 采用: 有界请求使用显式 `{ unit, value }` 预算；规范化后的 limit 始终携带 cost unit，不允许各 adapter 为同一 unit 发明不同计算方法。
- 采用: 默认调用保持有界；public `ignore-limit` 意图与 limit 互斥，并在输入解析后规范化为 `Unbounded`。无界请求返回完整选定结果，同时仍可报告其实际输出成本。
- 采用: 结果明确报告被选定输入是否完整进入本次输出；预算是输出控制事实，不承诺精确等于 raw 或 readable 的最终序列化大小。
- 不采用: 仅为防止单次输出失控而继续保留数字页码、下一页标志或可继续 token。
