本 proposal 定义 `add-invocation-audit-logging` change 的目标、范围和影响：为 Docnav 核心调用链引入可通过 CLI/config 显式启用、默认 metadata-only 的运行时调用日志。

## Why

Docnav 当前只有协议 `request_id` 和验证/烟测日志，缺少运行时调用记录；维护者难以在不重跑命令的情况下复盘一次 document operation 从 core handoff 进入 navigation-owned adapter selection、request construction、selected adapter handler dispatch，再到输出投影后的状态、耗时和失败边界。

本 change 定义一个有限、安全、可审计的日志能力，避免为了调试把完整 document content、protocol payload、structured diagnostics 或 debug context 写入不稳定位置，破坏 stdout/stderr 契约或放大日志体积。

## What Changes

- 新增运行时调用日志能力；功能默认不产生新输出或日志副作用，通过 CLI/config 显式启用后记录 `docnav` 文档操作到 selected adapter dispatch outcome 的元数据级事件，包括 `request_id`、operation、selected adapter id、路径显示策略、page/limit、耗时、operation/output status、响应大小摘要、错误分类和是否成功。
- 定义日志格式首选 JSON Lines / NDJSON，一行一个事件，带 `schema_version`、timestamp、event name 和稳定字段，便于本地审计、问题复现和工具消费。
- 定义默认日志策略为 metadata-only；主调用日志不记录完整 `RequestEnvelope`、`ProtocolResponse`、document content 或完整 diagnostic/debug output。
- 定义 document content 记录策略：主调用日志的操作结果事件只记录内容 hash、大小和摘要元数据；content hash 固定使用 SHA-256，事件算法标识为 `sha256`，hash 值使用小写 64 位十六进制；如需审计 hash 对应正文，必须通过单独 CLI/config 选项显式开启 content capture directory，并由主调用日志追加 `content_captured` / `content_capture_failed` 事件记录正文文件的相对路径。
- 明确日志不能写入 document output stdout，不能污染 `protocol-json` stdout；日志开关、sink/path、路径、content capture root path 和 query/ref 摘要策略由实现前的 owner 主规范固化，日志写入失败降级属于本 change 的固定边界。
- 要求为 invocation log JSONL event 建立 JSON Schema 与示例验证材料；schema 覆盖普通调用事件和可选 content capture 事件，正文文件本身不是 JSON schema 校验对象。
- 保持扩展边界：当前交付只定义显式启用的 metadata-only invocation log 和可选 content capture，不定义完整日志系统。
- 非目标：不改变原始协议 envelope，不给 protocol response 增加日志字段，不要求 adapter 自己实现长期日志系统，不把 smoke/verify 日志替换成运行时调用日志。

## Capabilities

### New Capabilities

- `invocation-logging`: 定义 Docnav 运行时调用日志的 CLI/config 显式启用语义、格式、默认记录范围、SHA-256 content hash/capture 边界、输出隔离、安全约束和审计用途。

### Modified Capabilities

- 无。本 delta 创建新的运行时调用日志 capability；现有 `repository-quality-observability` 继续只拥有源码质量观测。实现阶段需要同步对应 owner 主规范，但不在 OpenSpec delta 中把 `adapter-contract`、`protocol-contract`、`output-contract` 或 `core-cli` 改成 invocation logging 的 capability owner。

## Impact

- Affected executable: `docnav` 文档操作执行路径，尤其是 navigation-owned adapter selection 后的 navigation request construction、selected adapter handler dispatch、结果校验和错误映射边界。
- Affected adapter surface: linked adapter handler 继续只返回结构化 result/diagnostic；调用日志由 core 记录 dispatch outcome，不要求 adapter 改变 handler payload、协议输出或自行写审计日志。
- Affected output contracts: `readable-view`、`readable-json` 和 `protocol-json` stdout 必须保持现有语义；日志事件必须使用显式配置的独立 sink，日志失败诊断只能走不会破坏 machine-readable stdout 的有界通道。
- Affected validation: 需要覆盖日志开启/关闭、metadata-only 默认、完整 protocol envelope 不进入日志、SHA-256 content hash/capture、stdout purity、日志字段 schema、截断/脱敏和失败不影响主调用结果。
- Dependency impact: Rust 实现使用 RustCrypto `sha2` crate 计算 SHA-256 content hash；默认使用仓库内 JSONL writer；引入外部日志框架必须经过依赖和输出通道审计。
