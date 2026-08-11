# Proposal

本计划落实已确认的 bounded approximate token cost 方向：用证据和人工门禁选择 machine representation、calculator、accuracy、资源与迁移边界，再把 public token cost 限定为 returned content 或当前可见 selection 的低开销估算。

## Why

Token cost 帮助 AI 决定下一步读取，但 exact tokenizer parity 或对完整隐藏 selection 的测量可能比有限结果本身更昂贵。既有方向已把它确认为需要单独修复的有界性能债务；当前计划需要重新建立 Current release baseline，并防止算法、依赖和预算由 agent 或单次 benchmark 暗中决定。

## Outcome

所有 public token-valued measurements 使用获批且可识别的 approximation contract，只估算实际返回内容或当前可见 selection 的 cheap facts；字符分页和 required cost reporting 保持，hidden full serialization/tokenization 被禁止，普通与极端输入通过 accuracy、CPU、RSS、cold-start、platform 和 package evidence。获批 Target 在实施期间由本 Change 的 design Decisions 拥有；稳定 owner、schema 和 examples 只在实现与行为验证成立后同步为 Current。

## Scope

- 决定 machine representation、estimate scope、calibration/error、resource budgets、calculator/dependency、full-read threshold、consumer migration 和 structured-outline admission。
- Ordinary/nested read 与 unstructured full-read outline 只估算 returned content；structured outline 在确定 current-page membership 后才估算 entries。
- Shared helper 只拥有 calculator mechanics，不拥有 adapter selection、pagination、presentation 或 page admission。
- 不承诺 exact tokenizer parity，不让本计划成为其它独立产品 change 的统一前置。

## Success Criteria

- Q1–Q7 有可复现证据和明确人工批准；change-local Target 已进入 design/exact tasks，跨 change 方向在用户明确授权维护决策后进入 decision，已经成立的 Current contract 才进入稳定 owner。
- Public schema/examples/outputs 能区分 approximation scope，existing consumers 有获批 compatibility/migration。
- 实现不为 cost 读取、物化、序列化或 tokenize 未返回内容，且 page admission 不先估算后丢弃。
- 普通与 adversarial corpus 满足获批 accuracy/resource/package/platform budgets；失败按设计重新打开门禁而不放宽标准。
- Protocol、Markdown、JSON handoff、readable output、CLI/package、Semantic Cases 和 workspace verification 通过。

## Affected Owners

- [将 token cost 作为有界性能债务修复](../../docs/decisions/product-direction/repair-token-cost-as-bounded-debt.md)：长期方向和非统一前置约束。
- [原始协议](../../docs/protocol.md)、[输出模式](../../docs/output.md)及 schema/examples：public cost representation、scope 和 readable mapping。
- [Markdown Adapter](../../docs/adapters/markdown.md)及必要时的 [JSON Adapter](../../docs/adapters/json.md)：selection/page membership 与 format-owned measurements。
- Shared cost helper、navigation、tests/benchmarks、Semantic Cases、CLI/package 与 release validation。
