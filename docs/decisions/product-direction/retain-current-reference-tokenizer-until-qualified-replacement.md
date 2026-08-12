---
title: 保留当前 reference tokenizer，直到可靠替代已具备
status: active
alignment: aligned
createdAt: 2026-08-12T08:50:55Z
purpose: 以既有统一 token calculator 支持当前产品演进，只在可靠候选已经出现且替换仍有收益时重新立项。
background: 当前 backend 已提供统一语义，调查没有发现可靠替代品，候选成熟周期可能超过项目的实际演进窗口。
decision: 保留当前 o200k_base backend；具体候选先通过初步准入且项目仍能获益时，才重新调查和立项。
relations:
  - type: 修订
    target: product-direction/use-low-constant-reference-tokenizer-for-output-cost.md
---

## 目的

- 让现有统一 token calculator 继续支持 OutputWindow、fast-read 和 public output-limit 演进。
- 只有可靠候选已经出现且替换仍有实际收益时，才建立可执行的重新评估入口。

## 背景

- `docnav-text-cost` 已经提供唯一共享 `token_cost`，Markdown 与 JSON 使用同一 `tiktoken-rs / o200k_base / ordinary-text` 语义；项目不缺少统一 token 计算工具。
- [Tokenizer Backend 替代品调查](../../investigations/dependencies/tokenizer-backend-alternatives.md)没有发现同时通过语义、鲁棒性、分发、平台和资源门禁的替代品；最有希望的候选仍需生态成熟、资产处置和跨平台证据，其成熟周期可能超过项目实际需要它的窗口。
- Requested-unit dispatch、bounded prefix、OutputWindow traversal、fast-read admission 和 public contract 是独立于 backend 选择的责任；它们可以基于当前 calculator 推进。

## 决策

- 采用: 保留唯一的 current `tiktoken-rs / o200k_base / ordinary-text` production calculator 和 token 语义；backend identity 不进入 public contract。
- 采用: 当前不维护 tokenizer replacement Change、实施任务或周期性跟踪义务；其它产品与架构 Change 直接使用既有 calculator。
- 采用: 重新调查和立项必须同时满足两个条件：具体候选已经具备初步可信的语义、鲁棒性、资产分发、canonical target、维护和实际收益证据；项目届时仍能从替换中获益。
- 采用: 未来采用候选时直接替换旧 backend 并保持相同 token 语义，不建立多 profile、动态选择、双 backend runtime 或用户可见迁移层。
- 采用: 当前 backend 的具体缺陷按真实影响和行为 owner 单独修复，不把局部 bug 自动升级为 dependency replacement 计划。
- 不采用: 用 bytes、字符权重或其它不执行 tokenizer 的估算公式实现正式 `tokens` unit。
