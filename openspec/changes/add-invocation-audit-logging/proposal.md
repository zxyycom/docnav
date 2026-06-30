本 proposal 只在 `openspec/changes/add-invocation-audit-logging/` 下形成未审核临时文档，目标是为 Docnav 核心调用链引入默认元数据级调用日志和可选协议追踪，为简单调用记录与后续审计改进提供基础。

## Why

Docnav 当前只有协议 `request_id` 和验证/烟测日志，缺少运行时调用记录；维护者难以在不重跑命令的情况下复盘一次核心 CLI 到 adapter `invoke` 的请求、响应状态、耗时和失败边界。

需要先定义一个有限、安全、可审计的日志能力，避免后续为了调试临时把完整 document content、protocol payload 或 adapter stderr 写入不稳定位置，破坏 stdout/stderr 契约或泄露敏感输入。

## What Changes

- 新增运行时调用日志能力，记录 `docnav` 文档操作到 adapter `invoke` 的元数据级事件，包括 `request_id`、operation、adapter id、路径显示策略、page/limit、耗时、退出状态、响应大小、错误分类和是否成功。
- 定义日志格式首选 JSON Lines / NDJSON，一行一个事件，带 `schema_version`、timestamp、event name 和稳定字段，便于后续本地审计、CI artifact、问题复现和工具消费。
- 定义默认日志策略为 metadata-only；完整 protocol request/response payload 只能作为显式 opt-in trace，必须有截断、脱敏和大小限制。
- 明确日志不能写入 document output stdout，不能污染 `protocol-json` stdout；日志位置、开关和失败处理由实现阶段在主规范中固化。
- 为后续审核改进预留扩展点，例如更细粒度 event、采样、日志轮转、CI artifact 汇总、审计报告或 trace replay，但首期只交付简单调用记录。
- 非目标：不改变原始协议 envelope，不给 protocol response 增加日志字段，不要求 adapter 自己实现长期日志系统，不把 smoke/verify 日志替换成运行时调用日志。

## Capabilities

### New Capabilities

- `invocation-logging`: 定义 Docnav 运行时调用日志的格式、默认记录范围、trace 边界、输出隔离、安全约束和审计用途。

### Modified Capabilities

- 无。本 change 创建新的运行时调用日志 capability；现有 `code-quality-observability` 继续只拥有源码质量观测，`adapter-protocol` 和 `core-cli` 的既有 requirement 不在提案阶段直接修改。

## Impact

- Affected executable: `docnav` 核心 CLI 的文档操作执行路径，尤其是 adapter selection 后的 `invoke` 调用、响应校验和错误映射边界。
- Affected adapter surface: adapter `invoke` 仍只通过 stdin/stdout/stderr 通信；首期日志由 core 记录 adapter process 调用结果，不要求 adapter 改变协议输出。
- Affected output contracts: `readable-view`、`readable-json` 和 `protocol-json` stdout 必须保持现有语义；日志必须使用独立文件或显式诊断通道。
- Affected validation: 需要覆盖日志开启/关闭、metadata-only 默认、raw trace opt-in、stdout purity、日志字段 shape、截断/脱敏和失败不影响主调用结果。
- Dependency impact: 首期应先评估手写 JSONL writer 与 Rust `tracing`/`tracing-subscriber` 的取舍；是否引入日志库必须经过依赖和输出通道审计。
