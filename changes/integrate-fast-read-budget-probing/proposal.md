# Proposal

本 Change 起草让 fast-read admission 复用统一 OutputWindow 和 CostCalculator 的 bounded probe，而不再维护独立全文 cost 数据流。

## Why

Fast-read 必须在输出模式确定前判断全文是否足够小，因此不能遵守“只计算最终返回内容”的绝对规则；但当前 adapter-specific full-read measurement hook 会先完整计算 cost，并可能与最终 output accounting 重复。统一 probe 可以在阈值足以判定后停止，并让成功路径复用已取得的测量。

## Outcome

Navigation 用 selected full-read candidate 的标记语义字段运行 bounded probe：输入先结束时选择 unstructured full read，阈值先耗尽时立即回退 structured outline。Probe 和最终输出使用同一个 calculator contract，失败探测不进入 public output cost，成功探测的测量可安全复用。
