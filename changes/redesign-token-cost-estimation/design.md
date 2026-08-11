# Design

设计先用统一 corpus 和资源证据关闭 Q1–Q7 人工门禁，再以 shared helper 实现获批 estimator，并让每个 public surface 只对已返回或当前可见 facts 计量。

## Context

- 当前 owner 仍可能把 `unit: "tokens"` 解释为 exact selection cost；实施前必须从 release/source 重新核对，不把历史实验当 Current。
- Character budgets 继续决定 pagination；token estimate 是描述 returned result 的 cost fact，不反向成为隐藏 selection work。
- Structured outline、ordinary read、nested read 和 unstructured full-read 的“returned content/visible selection”边界不同，需要逐 surface 定义。
- [将 token cost 作为有界性能债务修复](../../docs/decisions/product-direction/repair-token-cost-as-bounded-debt.md)已确认 bounded approximation 方向，但没有替人类选择 encoding、calculator、dependency 或数字 budgets。

## Goals / Non-Goals

Goals:

- 保留 AI 可用的 token-valued cost，同时让其计算工作受返回事实边界约束。
- 显式表达 estimate、scope 与 unavailable 状态，并提供可复现 error/resource evidence。
- 让 calculator mechanics 可共享而 selection、admission 和 presentation 保持原 owner。

Non-Goals:

- 不提供 exact tokenizer parity 或隐藏全文/selection tokenization。
- 不由 benchmark 自动批准 dependency、coefficients 或 budgets。
- 不改变字符分页为 token pagination，也不协调 find/renderer/service 等独立 changes。

## Decisions

### 1. Token cost 保留但明确为 estimate

Public representation 必须让 consumer 识别 approximation 及其 scope；不得继续暗示 exact tokenizer or complete hidden selection parity。

### 2. 每个 surface 只有一个 bounded meaning

Ordinary/nested read 和 unstructured full-read 只覆盖 returned content；structured outline 只覆盖已经确定属于当前返回页的 visible entries。

### 3. Page admission 先于 entry estimation

Character budget 和 adapter-owned ordering 先确定 returned membership；不能为了决定是否返回而 estimate 一个 entry 后再丢弃其工作。

### 4. Character pagination 保持

Token estimates 是 cost metadata，不替换 limit/page/continuation 的既有字符或 adapter-owned预算。

### 5. Shared helper 只拥有 mechanics

Helper 接受已经选定的 bounded input 并返回获批 estimate；它不打开文档、选择 entries、render output 或决定 pagination。

### 6. Evidence 与人类批准选择 calculator/dependency

同一 corpus 比较候选 encoding/calculator 的 error distribution、under/over-estimation、CPU、RSS、cold start、platform、package 和 worst case。任何 production dependency 单独审核生态、安全、license、MSRV、transitives 和 alternatives。

### 7. Compatibility 通过独立 owner handoff

获批 representation 与 migration 在实施期间由本 Change 的 design Decisions 拥有。实施前只审计 protocol/schema/readable/adapter owners 的 Current delta；实现和行为验证成立后，才把实际 contract 分别同步到稳定 owner、schema 和 examples。其它 changes 只在重叠 Current clauses 时 rebase，不成为互相前置。

### 8. 证据失效会重新打开门禁

若 final calculator validation 违反获批 accuracy/resource/package/platform budget，相关 Q2–Q4 批准失效，依赖任务停止并回到 evidence → approval → synchronization；不得降低标准继续。

## Risks / Trade-offs

- Approximation 可能低估并误导 AI；批准 packet 必须定义 underestimation 与 worst-case 上限，而不只看平均误差。
- 新 dependency 可能使 cold start/package 超过收益；no-new-dependency 始终是候选。
- Existing consumers 可能把 tokens 当 exact；必须选择 compatibility/versioning，不静默改变语义。
- Structured page assembly 容易先算后丢；tests/benchmarks必须证明 admission 顺序。

## Open Questions

以下 Q1–Q7 由 Implementation 1.1–1.4 形成证据，1.5 只能由用户或其指定 product/architecture owner 批准，1.6 固化 Target 并关闭阻断审计。1.6 通过前，owner、schema、测试和 production 修改全部被阻塞：

1. Q1 — 哪个 machine representation 区分 approximation、returned-content、visible-selection 与 unavailable？
2. Q2 — 哪些 reference tokenizer/corpus weighting、accuracy statistics、最大误差/低估和 worst case 构成 acceptance？
3. Q3 — CPU、peak RSS、cold start、package、platform/target、per-entry/page 和 adversarial budgets 是什么？
4. Q4 — 哪个 measured calculator 获批，是否引入经过完整审核的 production dependency？
5. Q5 — 现有 token-valued unstructured-full-read threshold 怎样在不计算未返回内容的情况下保持，或由哪个 owner 迁移？
6. Q6 — Schema/examples 和 existing consumers 的 compatibility/versioning/migration 是什么？
7. Q7 — Structured outline 怎样先确定 current-page membership，再进行 entry estimation 和 accounting？
