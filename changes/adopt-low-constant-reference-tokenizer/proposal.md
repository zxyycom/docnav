# Proposal

本 Change 起草以可复现证据选择并接入低常数 Docnav reference tokenizer，同时定义预算式 token measurement 所需的 calculator 能力。

## Why

当前 tokenizer 为短生命周期 CLI 带来明显启动和计算常数，但纯字节或字符公式又失去真实分词信号。Output limit、实际 cost 和 fast-read probe 需要一个统一、低开销且能在达到预算后停止的 token backend。

## Outcome

在固定 corpus 和资源口径下比较候选，选择并固定一个真实 reference tokenizer、版本和依赖边界，再用它替换当前 token backend。Shared calculator 可以计算完整 text，也可以在给定预算下返回 complete/exceeded 与合法 prefix boundary，并保持非 token unit 不初始化 tokenizer。
