# add-text-cost-calculator-helper

为通用 text cost calculator helper 建立 change，并把 Markdown adapter 作为首个接入方。

本 change 的核心是提供最小 text cost calculator helper：helper 由一组同签名函数组成，每个函数只接收纯文本并返回对应 protocol-compatible `Measurement`。函数自己定义 unit，helper 根据 text 计算 value；首期必交付 `lines` / `bytes` 文本成本和基于 `tiktoken-rs` `o200k_base` 的 token cost。使用者不限定为 adapter，但 helper 不替调用方选择函数集合、scope、排序或展示策略；adapter 需要 cost 时自己直接调用所需函数并组合结果。helper 输出基于 current raw protocol `cost.measurements[]` shape，不单独决定或改变 raw protocol `cost` 字段结构，也不改变 readable output payload shape。
