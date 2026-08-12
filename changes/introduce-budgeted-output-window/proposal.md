# Proposal

本 Change 起草一个在标记语义字段上统一计量、裁剪和报告完整性的 Budgeted Output Window。

## Why

Read、structured outline、unstructured outline、find 和 nested auto-read 的结果结构不同，现有 adapters 因此分别实现分页和 cost accounting。强行统一 payload 会破坏业务语义，而在 serializer 后裁剪又会让 raw/readable 包装决定预算并产生无效输出。

## Outcome

不同 operation result 通过类型安全的字段投影暴露预算内容；一个 OutputWindow 在 rendering 前使用统一 CostCalculator 处理 text、sequence 和 nested fields，生成裁剪后的合法语义结果以及独立 OutputReport。Raw 与 readable 渲染同一份受限结果。Token path 复用既有统一 calculator，bounded wrapper 由本 Change 实现和验证。
