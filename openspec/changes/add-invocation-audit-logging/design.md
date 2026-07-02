本 design 只在 `openspec/changes/add-invocation-audit-logging/` 下形成未审核临时文档，目标是为 Docnav 核心调用链引入默认元数据级调用日志和可选协议追踪，为简单调用记录与后续审计改进提供基础。

## Context

Docnav 的核心文档操作会解析输入和配置、选择 core release static registry 中的 adapter、构造 navigation request、调用 linked adapter handler，并把结构化 result/diagnostic 映射到 protocol/readable output。当前这条链路只通过内存对象、selected adapter facts 和 protocol `request_id` 关联；验证脚本和 smoke runner 会写 `.log/.../latest.log`，但这些是测试/验证运行日志，不是普通 CLI 调用的运行时审计记录。

本 change 影响 core CLI 文档操作、linked adapter dispatch 和输出投影之间的运行时观测边界。它必须保持 linked adapter handler payload、`readable-view`、`readable-json` 和 `protocol-json` 的输出契约不变，也必须保持 machine-readable stdout 纯净。

当前 change 只在 `openspec/changes/add-invocation-audit-logging/` 下形成未审核临时 artifacts，不影响现有其它文档、主规范、schema、示例或实现代码。

## Goals / Non-Goals

**Goals:**

- 为每次 core CLI 文档操作提供可关联、可过滤、可本地复盘的调用日志事件。
- 首期记录简单调用元数据，支持维护者回答“执行了什么 operation、用了哪个 adapter、request_id 是什么、是否成功、耗时多少、失败在哪一层”。
- 用 JSON Lines / NDJSON 固定日志事件外形，为后续审计报告、CI artifact、trace replay 或更细粒度 runtime observability 保留演进空间。
- 明确安全边界：默认不记录 document content、完整 protocol payload、完整 diagnostic/debug output、环境变量或 secrets。
- 明确验证边界：日志功能不能污染 stdout，不能改变 protocol response，不能让日志写入失败改变文档操作的成功/失败语义。

**Non-Goals:**

- 不改变 `RequestEnvelope`、`ProtocolResponse` 或 schema。
- 不把日志字段加入 `readable-json`、`protocol-json` 或 adapter handler payload。
- 不要求 linked adapter crate 或 handler 自己实现长期日志系统。
- 不替换现有 workspace verify、smoke 或 code-quality observability 日志。
- 不在首期定义远程日志收集、长期保留策略、日志轮转配额或 merge-blocking 审计策略。

## Decisions

### Decision 1: 使用 JSON Lines 作为日志事件格式

调用日志使用一行一个 JSON event 的 JSON Lines / NDJSON 格式。每条事件包含 `schema_version`、timestamp、event、request_id、operation、adapter_id、result metadata 和 duration 等字段。这样可以用普通文本追加写入，也可以被脚本逐行解析、过滤和归档。

备选方案是 plain text 或完整 protocol JSON 数组。Plain text 容易变成不可验证文案；JSON 数组不适合持续追加和崩溃后保留部分结果。

### Decision 2: 默认模式只记录元数据，raw trace 必须显式开启

调用日志的普通模式只记录调用元数据、大小摘要、错误分类和截断后的诊断摘要。完整 request/response envelope 只允许在显式 trace 模式下记录，并必须执行大小限制、字段脱敏和长文本截断。

备选方案是默认记录完整 payload。该方案调试方便，但会把 `read.content`、query、document path、diagnostic/debug context 或未来敏感字段固化到日志里，风险过高。

### Decision 3: 日志通道独立于 document output

运行时日志必须写入独立日志文件或显式配置的日志 sink，不得写入 document output stdout。`protocol-json` stdout 继续只输出 protocol-shaped payload；linked adapter handler 继续只通过结构化 result/diagnostic 与 caller boundary 交互。

备选方案是把日志追加到 stderr。stderr 已承载边界诊断和人类可读消息，默认追加结构化日志会增加人类输出噪音，并可能让调用方误把诊断和审计事件混在一起。

### Decision 4: 首期不强制引入日志库

首期实现可以先使用小型仓库内 JSONL writer，负责事件结构、路径创建、追加写入和失败降级。只有在需要 span、level、模块过滤、跨 crate subscriber 或 adapter 统一日志时，才引入 `tracing` / `tracing-subscriber`，并在实现前完成依赖、输出通道和配置行为审计。

备选方案是立即引入日志库。该方案扩展性更好，但会提前扩大依赖和全局初始化面；如果首期只是 core document operation dispatch 记录，手写 writer 足以证明行为并降低引入风险。

### Decision 5: 日志写入失败不改变主调用结果

日志目录不可写、日志序列化失败或日志文件追加失败时，文档操作的成功/失败结果仍由原本 document operation outcome 决定。实现可以在非 protocol stdout 的安全通道输出简短诊断，或在后续审计策略中选择静默降级；不得因为审计日志失败把成功的 document operation 改成失败。

备选方案是把日志失败作为 fatal error。该方案便于发现审计缺口，但会让观测能力反过来影响核心导航链路，不适合首期运行时日志。

## Risks / Trade-offs

- [Risk] 元数据日志仍可能泄露本地路径、query 或 ref 形状。Mitigation: 实现时定义 path 显示策略、query/ref 记录策略和截断规则，默认记录 presence/length/summary 而非完整敏感值。
- [Risk] trace 模式会记录文档正文或过多 structured diagnostic/debug context。Mitigation: trace 必须显式开启，并在实现前加入 payload size cap、field redaction 和 negative tests。
- [Risk] 日志 schema 过早稳定导致后续扩展困难。Mitigation: 每条事件带 `schema_version`，新增字段优先 optional，breaking change 通过后续 change 处理。
- [Risk] 日志写入在高频调用下造成性能成本。Mitigation: 首期记录一次调用级事件，避免 per-entry/per-block 细粒度事件；验证中覆盖基本开销和失败降级。
- [Risk] 与现有 smoke/verify `.log` 语义混淆。Mitigation: 文档和任务明确 runtime invocation log 与验证日志是不同 artifact，不复用测试日志格式作为生产运行日志 contract。

## Migration Plan

1. 先完成阻塞级审计，确认本 change 只创建 `invocation-logging` 临时 artifacts，且不修改现有主规范或实现代码。
2. 审计通过后，更新对应主规范以声明 runtime invocation logging 的 owner、格式、开关、日志位置、安全边界和验证入口。
3. 在 core CLI document operation dispatch 边界实现 metadata-only JSONL event writer，并确保日志功能关闭或写入失败时不改变现有 CLI 行为。
4. 增加聚焦测试：stdout purity、metadata-only 默认、trace opt-in、截断/脱敏、日志写入失败降级和 request/outcome `request_id` 关联。
5. 根据改动范围运行局部 Rust 测试和 OpenSpec validation；若同时修改 docs、schema、示例或跨 crate 行为，运行 workspace verifier。

Rollback 策略：运行时日志应由显式配置控制；若实现引入问题，可以关闭日志开关或移除 writer 调用，保持原有 protocol 和 output contract 不变。

## Open Questions

无未回答开放问题，可以进入实现前审计。日志文件默认路径、具体开关名称和是否引入 `tracing` 的最终结论需要在实现审计任务中用主规范和测试固化。
