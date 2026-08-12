---
title: 输出 token 成本使用低常数参考分词器
status: active
alignment: unaligned
createdAt: 2026-08-12T03:18:35Z
purpose: 以真实但低开销的统一参考分词结果支持输出预算，而不追求与每个调用模型完全一致。
background: 当前 reference tokenizer 的启动和计算常数过高，纯字节或字符公式虽然更便宜，却不能提供期望的实际分词信号。
decision: 将 tokens 定义为版本固定的 Docnav reference tokenizer 计数，并以资源常数、预算提前停止能力和跨语料相对精度选择实现。
relations: []
---

## 目的
- 为带单位的输出 limit、实际 output cost 和 fast-read threshold 提供同一种 token 计算语义。
- 在保留真实 tokenizer 参与的前提下，显著降低冷启动、CPU、内存和依赖成本。

## 背景
- 当前 `o200k_base` tokenizer 能提供确定计数，但对 Docnav 这种短生命周期 CLI 存在明显高常数。
- Docnav 不能知道所有调用方最终使用的模型，因此与任一模型 tokenizer 的完全 parity 不是可稳定承诺的契约。
- 字节或字符比例只能形成统计估算，无法满足使用真实分词器提供相对精确信号的目标。

## 决策
- 采用: `tokens` 表示一个明确版本的 Docnav reference tokenizer 对目标文本产生的实际 token count；它对具体外部模型只是相对成本信号，不承诺模型 parity。
- 采用: 候选实现必须以同一代表性 English、CJK、mixed、code、Markdown、JSON、emoji 和极端语料比较冷启动、CPU、peak RSS、package/transitive 成本及相对计数偏差。
- 采用: calculator 必须支持或能够低成本实现预算式 prefix measurement 和合法文本截止边界；达到阈值后无需继续处理剩余输入。
- 采用: 具体 tokenizer、版本和 dependency 由可复现证据选择并固定，后续更换视为可观察 cost 语义变化。
- 不采用: 用 `bytes / n`、字符权重或其它不执行 tokenizer 的公式作为 `tokens` 正式实现。
- 不采用: 为匹配每个可能的调用模型维护多套 tokenizer 或动态模型选择。
