本 design 说明 `add-outline-preview-skim-pack` 的实现边界：用确定性 outline+read 编排给 outline 首屏附带预算内正文样本；当前 change 只在 `openspec/changes/add-outline-preview-skim-pack/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

普通 outline 能稳定提供结构、ref 和分页，但不能告诉调用方每个章节大概写了什么。调用方需要逐个 read 才能判断下一步。Skim Pack 的目标不是摘要或智能推荐，而是在 outline 旁边附带少量按规则选择的原文样本。

## Goals / Non-Goals

**Goals:**

- 为 outline 增加显式的预算内 preview 组合能力。
- 使用确定性 entry selection 规则，优先复用 outline 返回顺序和稳定 item facts。
- 用现有 read pipeline 读取 preview 内容，并在统一 typed result 中表达预览、跳过原因和 continuation。
- 保持 adapter 不新增 preview operation，不要求 adapter 理解跨章节阅读策略。

**Non-Goals:**

- 不生成摘要、不改写原文、不重排 outline。
- 不依赖模型判断章节重要性或用户意图。
- 不修改 raw `OutlineResult` / `ReadResult` shape。
- 不把 preview 规则下放给格式 adapter，除非后续 spec 明确 adapter-owned展示语义。

## Decisions

### Decision 1: preview selection 使用 outline 顺序和预算，而不是智能排序

第一版 Skim Pack 按 outline 返回顺序选择前 N 个带非空 ref 的 entries，并受总 preview budget 限制。若后续发现 top-level facts 能稳定跨 adapter 表达，可以在单独决策中加入 top-level 优先规则。

替代方案是选择“最重要”或“最相关”的章节，但当前执行时没有智能模型，也没有跨格式重要性信号；使用这类规则会让行为难以测试。

### Decision 2: preview 内容来自现有 read pipeline

core 不直接解析文档内容或 ref。每个 preview 都通过现有 read pipeline 获得 content、content_type、cost 和 page，因此 adapter ownership 保持不变。

替代方案是在 outline adapter 中直接塞 excerpt 或 summary；这会把组合体验变成 adapter 展示语义，容易造成格式间重复实现。

### Decision 3: preview facts 进入统一 ProtocolResponse

Skim Pack 虽然服务首屏阅读体验，但 preview selection、追加 read 和 read status 都是 core-owned 业务语义，不能只存在于 renderer。第一版必须定义 typed composition result，包含 base outline facts、preview content、status 和 continuation；`protocol-json` 直接暴露这些事实，内置 renderer 从同一 result 生成 `readable-view`。

Adapter-owned `OutlineResult` / `ReadResult` 保持不变；core 在组合边界构造新的或扩展后的 operation result。该 machine contract 必须在实现前完成 schema、example 和 compatibility 审计。

## Risks / Trade-offs

- [Risk] preview 会让 outline 首屏变长，反而降低结构扫描效率。-> Mitigation: 使用显式 surface 和小预算默认值，并在预算耗尽时稳定停止。
- [Risk] preview 选择前 N 个 entries 可能不是用户最想读的部分。-> Mitigation: 第一版承认它不是智能推荐，只提供确定性样本；用户仍可用 ref/read 精确追读。
- [Risk] 多次 read 增加 latency。-> Mitigation: 限制 preview 数量和总预算；后续实现可先串行，必要时再评估并发。
- [Risk] output shape 与 auto-read composition 重复。-> Mitigation: 复用 typed composition status/fact primitives 和 renderer mapping pattern，但保持两个 change 的 public behavior 独立验收。

## Migration Plan

该 change 是 additive；默认不要求 adapter 迁移。实现阶段应先更新 docs/spec/schema/examples，再实现 outline 后的 preview read 编排，最后覆盖预算、分页、read 失败、无 ref、多 entry，以及两条 output path 对同一 composition facts 的投影测试。

## Open Questions

无未回答开放问题，可以进入实现前审计。实现阶段仍需在 CLI owner 文档中定稿显式 surface 的具体拼写和默认 preview budget。
