本 design 定义 `add-invocation-audit-logging` 的实现前决策：为 Docnav 核心调用链引入可通过 CLI/config 显式启用、默认 metadata-only 的运行时调用日志。

## Context

Docnav 的文档操作由 core 完成 command classification 和 config source descriptor/path handoff，再由 `docnav-navigation` 加载 config sources、选择 core release static registry 中的 adapter、构造 navigation request、调用 selected adapter handler，并把结构化 result/diagnostic 交给输出投影。当前这条链路只通过内存对象、selected adapter facts 和 protocol `request_id` 关联；验证脚本和 smoke runner 会写 `.log/.../latest.log`，但这些是测试/验证运行日志，不是普通 CLI 调用的运行时审计记录。

本 change 影响 core CLI handoff、navigation-owned selected adapter dispatch 和输出投影之间的运行时观测边界。它必须保持 selected adapter handler payload、`readable-view`、`readable-json` 和 `protocol-json` 的输出契约不变，也必须保持 machine-readable stdout 纯净。

本 OpenSpec 文档只声明计划、边界和验收要求；主规范、schema、示例和实现代码由任务清单同步。

## Goals / Non-Goals

**Goals:**

- 为每次 core CLI 文档操作提供可关联、可过滤、可本地复盘的运行时调用日志事件。
- 保持功能只通过 CLI/config 显式启用；未启用时不得新增 stdout/stderr 输出、protocol 字段、handler payload 或日志文件副作用。
- 记录简单调用元数据，支持维护者回答“执行了什么 operation、用了哪个 adapter、request_id 是什么、是否成功、耗时多少、失败在哪一层”。
- 用 JSON Lines / NDJSON 和 JSON Schema 固定日志事件外形，支持本地审计、问题复现和工具消费。
- 明确正文捕获和安全边界：默认不记录 document content、完整 protocol payload、完整 diagnostic/debug output、环境变量或 secrets；document content 默认只以 SHA-256 hash、大小和摘要元数据进入主调用日志的操作结果事件。
- 明确验证边界：日志功能不能污染 stdout，不能改变 protocol response，不能让日志写入失败改变文档操作的成功/失败语义。

**Non-Goals:**

- 不改变 `RequestEnvelope`、`ProtocolResponse` 或既有 protocol/readable schema。
- 不把日志字段加入 `readable-json`、`protocol-json` 或 adapter handler payload。
- 不要求 linked adapter crate 或 handler 自己实现长期日志系统。
- 不在 OpenSpec delta 中直接确定最终 CLI flag、配置字段或默认日志路径；这些 CLI/config surface 必须在实现前由 owner 主规范和测试固定。
- 不替换现有 workspace verify、smoke 或 code-quality observability 日志。
- 不定义远程日志收集、长期保留策略、日志轮转配额或 merge-blocking 审计策略。

## Owner Placement

- `docs/architecture.md` owns the runtime boundary: core owns invocation logging orchestration, adapters continue to return only typed results or diagnostics, and shared crates must not take ownership of CLI surface, protocol envelope, or output projection.
- `docs/cli.md` owns the user-facing CLI/config enablement surface, invocation log path, content capture root path, path normalization, and exit behavior.
- `docs/navigation-input-resolution.md` owns the navigation-stage metadata that can be recorded: adapter selection outcome, request construction boundary, typed operation arguments, request id availability, and selected adapter dispatch result.
- `docs/output.md` owns stdout/stderr purity and the rule that invocation log events are not document output for `readable-view`, `readable-json`, or `protocol-json`.
- `docs/protocol.md` remains the owner of `RequestEnvelope` and `ProtocolResponse`; invocation log 只记录 request/response correlation 和 bounded status/size metadata，不复制完整 protocol envelope。
- `docs/schemas/` owns the invocation log event schema, including `sha256` content hash fields and optional content capture event variants, as validation material; `docs/examples/` owns examples that prove the documented JSONL event shapes.

## Decisions

### Decision 1: 调用日志必须通过 CLI/config 显式启用

调用日志默认不产生新 stdout/stderr、protocol 字段、handler payload 或日志文件副作用。实现必须先解析 CLI/config 中的显式启用信号、日志 sink/path 和可选 content capture root path，再写入事件；具体 flag、配置字段和默认路径由主规范在实现前固定。

### Decision 2: 使用 JSON Lines 作为日志事件格式

调用日志使用一行一个 JSON event 的 JSON Lines / NDJSON 格式。每条事件包含 `schema_version`、timestamp、event、request_id、operation、adapter_id、result metadata 和 duration 等字段。这样可以用普通文本追加写入，也可以被脚本逐行解析、过滤和归档。

### Decision 3: 默认模式只记录元数据

调用日志的普通模式只记录调用元数据、大小摘要、错误分类、SHA-256 content hash 和截断后的诊断摘要，不写完整 `RequestEnvelope` / `ProtocolResponse`。

### Decision 4: document content 使用 hash 引用和可选 content capture directory

主调用日志不得直接承载完整 document content。涉及正文内容时，操作结果事件记录 hash algorithm、content hash、content type、byte/char size 和必要的 bounded summary。content hash 算法固定为 SHA-256；事件字段使用 `hash_algorithm: "sha256"` 和小写 64 位十六进制 `content_hash`。需要复盘 hash 对应正文时，调用方必须通过独立 CLI/config 选项显式开启 content capture directory；正文文件写入该目录，路径由 owner 主规范固定为日期目录加 SHA-256 hash 文件名，格式为 `<YYYY-MM-DD>/sha256-<content_hash>.content`。

content hash 的输入必须是正文捕获文件实际写入的同一组 bytes；未开启 content capture 时，也必须按“若开启 capture 会写入的正文 bytes”计算 hash，不得在 hash 前执行换行、编码或空白归一化。Rust 实现使用 RustCrypto `sha2` crate 计算 SHA-256；更换 hash 算法需要同步更新主规范、schema、示例和测试，替换实现库需要完成依赖审计并证明可观察 hash 输出不变。

开启 content capture 时，正文文件仍不进入主调用日志；主调用日志追加 `content_captured` 事件，记录 `captured_at`、hash metadata、size metadata 和正文文件的 `relative_path`。content capture event 只用 `relative_path` 表达正文文件位置，不引入额外正文存储配置字段。若正文文件写入失败，主调用日志可以追加 `content_capture_failed` 事件；该失败不得改变原本文档操作的成功/失败语义。

### Decision 5: 日志通道独立于 document output

运行时日志必须写入独立日志文件或显式配置的日志 sink，不得写入 document output stdout。`protocol-json` stdout 继续只输出 protocol-shaped payload；linked adapter handler 继续只通过结构化 result/diagnostic 与 caller boundary 交互。

### Decision 6: 默认使用仓库内 JSONL writer

实现默认使用小型仓库内 JSONL writer，负责事件结构、路径创建、追加写入和失败降级。只有在内部 writer 无法满足跨 crate 调用日志需求时，才引入外部日志框架，并在实现前完成依赖、输出通道和配置行为审计。

### Decision 7: 日志写入失败不改变主调用结果

日志目录不可写、日志序列化失败或日志文件追加失败时，文档操作的成功/失败结果仍由原本 document operation outcome 决定。实现可以在非 protocol stdout 的安全通道输出简短诊断，也可以静默降级；不得因为审计日志失败把成功的 document operation 改成失败。

## Risks / Trade-offs

- [Risk] 元数据日志仍可能放大本地路径、query 或 ref 形状。Mitigation: 实现时定义 path 显示策略、query/ref 记录策略和截断规则，默认记录 bounded value、presence/length/hash/summary，而非无界原始值。
- [Risk] content capture directory 会记录文档正文，若边界不清可能扩大本地数据保留范围。Mitigation: content capture 必须单独显式开启，只捕获 owner-documented 正文内容，并在实现前加入 payload size cap、SHA-256 hash 命名、日期目录和 negative tests。
- [Risk] 日志 schema 过早稳定导致扩展困难。Mitigation: 每条事件带 `schema_version`，新增字段优先 optional，breaking change 通过独立 change 处理。
- [Risk] 启用开关、sink/path、content capture root path 或默认路径在实现时被随意选择，导致 CLI/config surface 与长期 owner 脱节。Mitigation: 实现前必须在 owner 主规范声明 surface、路径语义、默认关闭语义和验证入口。
- [Risk] 日志写入在高频调用下造成性能成本。Mitigation: 本 change 只记录一次调用级事件，避免 per-entry/per-block 细粒度事件；验证中覆盖基本开销和失败降级。
- [Risk] 与现有 smoke/verify `.log` 语义混淆。Mitigation: 文档和任务明确 runtime invocation log 与验证输出是不同记录，不复用测试日志格式作为生产运行日志 contract。

## Implementation Plan

1. 按 Owner Placement 更新对应主规范以声明 runtime invocation logging 的 owner、CLI/config 显式启用语义、格式、开关、日志位置、content capture root 位置、安全边界和验证入口。
2. 补充 JSON Schema、示例和 fixture 验证材料，证明主日志操作事件与 content capture 事件的字段 shape。
3. 在 core CLI document operation dispatch 边界实现 metadata-only JSONL event writer，并确保日志功能关闭或写入失败时不改变现有 CLI 行为。
4. 增加聚焦测试：stdout purity、metadata-only 默认、完整 protocol envelope 不进入日志、SHA-256 content hash/capture、截断/脱敏、日志写入失败降级和 request/outcome `request_id` 关联。
5. 根据改动范围运行局部 Rust 测试、schema/example validation 和 OpenSpec validation；若同时修改 docs、schema、示例或跨 crate 行为，运行 workspace verifier。

Rollback 策略：运行时日志应由显式配置控制；若实现引入问题，可以关闭日志开关或移除 writer 调用，保持原有 protocol 和 output contract 不变。

## Deferred Implementation Decisions

以下细节由 owner 主规范和测试固定，不能只在代码中决定：

- 日志启用 surface：CLI flag、配置字段或组合方式。
- 日志 sink/path：显式路径如何规范化、默认路径是否存在、路径不可写时的诊断通道。
- Content capture surface：是否开启正文捕获目录、root path 如何配置、日期目录和 `sha256-<content_hash>.content` 文件名如何生成、正文文件写入失败如何记录。
- 摘要策略：document path、query、ref、diagnostics 和 content 的 bounded value、presence/length/SHA-256 hash/truncated summary 规则。
- Schema 策略：invocation log event schema、`sha256` content hash fields、content capture event variant、示例文件和验证入口。
- 依赖策略：继续使用内部 JSONL writer，或在单独依赖审计后引入外部日志框架。
