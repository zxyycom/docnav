本 design 记录将 Markdown 文档 cost 从单纯文件大小估算改为 token-informed 估算的实现思路；当前只在 `openspec/changes/use-token-based-document-cost/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 的协议和 readable 输出已经包含 `cost` 字段或 display 中的 section cost，但当前示例和实现倾向用行数加 KB 表达。KB 能说明文件传输体积，却不能直接说明 AI 继续读取某个章节会消耗多少上下文。

本 change 只处理 Markdown adapter 生成的可读成本摘要。共享协议仍把 `cost` 当作字符串，`docnav` core 和 readable renderer 仍只透传 adapter 结果。`limit_chars` 仍是分页字符预算，不改为 token budget。

## Goals / Non-Goals

**Goals:**

- 为 Markdown read cost 和 outline section cost 引入 token-informed 估算。
- 记录 Rust `tiktoken` crate 作为首选 tokenizer 实现方向，并在实现前完成依赖审计。
- 保持协议 schema、readable schema 和 core output mapping 的字段 shape 不变。
- 让示例和测试从 `lines | KB` 迁移到包含 token count 的可读 cost 表达。

**Non-Goals:**

- 不把 `limit_chars`、page 或截断规则从字符预算改为 token 预算。
- 不新增机器稳定的 token count 字段。
- 不要求非 Markdown adapter 同步实现 token cost。
- 不承诺 `cost` 文案作为长期机器解析接口。

## Decisions

### Decision 1: token cost 是 adapter-owned readable estimate

Markdown adapter 负责计算和格式化 token-informed cost；`docnav` core、`docnav-output` 和 `docnav-readable` 继续只保留和渲染 adapter 返回的字符串。这样不会扩大 core 对格式内容的理解，也不会改变 protocol/readable schema。

备选方案是新增共享 `token_count` 字段或让 core 统一计算 cost。前者会扩大协议机器契约并要求 schema/example 同步；后者要求 core 读取并解释格式内容，违反当前 adapter 边界。本 change 暂不采用。

### Decision 2: token estimate 不改变 pagination unit

`limit_chars` 继续按 UTF-8 解码后的 Unicode 字符计数，用于分页和截断。Token count 只帮助用户选择要读的章节或判断 read 目标成本，不参与 page 计算。

备选方案是引入 `limit_tokens` 或把 `limit_chars` 重定义为 token budget。该路径会破坏现有 CLI、protocol、config、SDK 和 adapter 测试边界，应该独立设计。

### Decision 3: 使用 `tiktoken` 前必须完成依赖与 encoding 审计

实现任务可以以 Rust `tiktoken` crate 为首选，但在改 `Cargo.toml` 前必须确认 crate 名称、许可、维护状态、离线构建行为、encoding 初始化方式、性能成本和固定 encoding 选择。审计结果需要落到主规范、测试说明或实现注释中，而不是只留在 PR 讨论里。

备选方案是手写近似 tokenizer、继续用 KB、或改用其它 Rust tokenizer crate。手写近似会制造与模型上下文不一致的假精确；继续用 KB 不能解决用户判断成本的问题；其它 crate 只有在 `tiktoken` 审计失败时才作为替代方案。

### Decision 4: cost 文案更新不提供解析兼容承诺

实现后 `cost` 和 outline display 必须包含 token count，但不承诺旧的 `lines | KB` 文案保持不变。测试可以验证 token count 存在、固定 fixture 的稳定结果和字段 shape，不应把可读文案拆成新的机器协议。

## Risks / Trade-offs

- [Risk] Token count 会增加 outline/read 的 CPU 成本 → Mitigation: 实现时优先复用已解析的 section 文本，必要时缓存同一文档解析过程中的 token counts，并用 fixture 级性能观察避免重复全篇 tokenization。
- [Risk] 不同 encoding 会产生不同 token count → Mitigation: 实现前审计固定 encoding，并在文档和测试中记录该选择；不把 token count 声明为跨模型精确值。
- [Risk] 外部 crate 影响离线构建或 release artifact → Mitigation: 在依赖审计中确认 crate 不需要运行时网络访问，并把 dependency/license/build 行为纳入验证。
- [Risk] 用户把可读 cost 当成机器稳定字段解析 → Mitigation: 保持 schema 类型为 string，不新增机器字段，并在主规范中说明 cost 文案不作为机器解析接口。

## Migration Plan

1. 完成阻塞级 change 审计，确认 proposal、design、specs 和 tasks 只描述本 change。
2. 审计 Rust `tiktoken` crate 和 encoding 选择，记录结论。
3. 更新 Markdown adapter cost helper，使 read cost 和 outline section cost 包含 token count。
4. 同步主规范、schema/example 语义说明和 fixture/test expectation。
5. 运行范围匹配的 Rust 测试、schema/example 验证和 workspace 验证。
6. 若依赖审计失败，停止实现并更新本 design，明确替代 tokenizer 或回退为暂不实现。

## Open Questions

无未回答开放问题，可以进入实现前审计。实现前审计必须把 crate 与 encoding 选择固化为可验证结论后，才允许执行代码改动。
