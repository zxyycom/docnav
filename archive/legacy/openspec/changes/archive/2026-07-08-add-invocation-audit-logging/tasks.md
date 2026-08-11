本 tasks 只在 `openspec/changes/add-invocation-audit-logging/` 下形成 change-stage checklist；实现任务执行前必须先完成 invocation logging 方案审计门禁。

## 1. 阻塞级审计门禁

- [x] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“为 Docnav 核心调用链引入 CLI/config 显式启用、默认 metadata-only 的运行时调用日志”这一核心目标；审计未完成前不得执行任何实现任务。
- [x] 1.2 审计 capability ID 是否只新增 `invocation-logging`，且没有把 change name、`repository-quality-observability` 或过宽 runtime umbrella 当作错误 owner。
- [x] 1.3 审计提案阶段改动是否只修改 `openspec/changes/add-invocation-audit-logging/` 下的 planning documents，且没有修改现有 specs、docs、schemas、examples、测试或实现代码。
- [x] 1.4 审计 `design.md` 是否没有阻塞问题，并确认日志路径、开关名称、content capture、摘要策略、schema 和日志库选型已归入主规范与测试固化流程。
- [x] 1.5 审计安全/体积边界：CLI/config 显式启用、metadata-only 默认、SHA-256 content hash/capture、payload 截断/脱敏、stdout purity、document output/linked handler payload 边界和日志失败降级都已进入 specs。
- [x] 1.6 审计依赖边界：默认使用内部 JSONL writer；若要引入外部日志框架，必须先完成依赖、feature、初始化和输出通道审计。

## 2. 规范同步

- [x] 2.1 按 `design.md` 的 Owner Placement 更新对应主规范，声明 `invocation-logging` 的 owner、运行时日志用途、CLI/config 显式启用语义、JSONL 格式、事件字段和状态语义。
- [x] 2.2 明确日志开关、默认关闭语义、metadata-only 默认模式、日志 sink/path、content capture root path、路径显示策略和 query/ref 摘要策略。
- [x] 2.3 明确 runtime invocation log 与 verify/smoke `.log`、code-quality observability outputs 的边界，避免复用测试日志格式作为运行时 contract。
- [x] 2.4 补充 invocation log event JSON Schema、content capture event variant、example 和 fixture 验证材料，固定 `hash_algorithm: "sha256"`、小写 64 位十六进制 `content_hash` 和 `<YYYY-MM-DD>/sha256-<content_hash>.content` 相对路径，并在 `docs/schemas/json-schema.md` 与 `docs/examples/README.md` 注册 owner/用途。

## 3. Core 实现

- [x] 3.1 在文档操作链路中确定最小插桩点，覆盖 navigation-owned adapter selection、navigation request construction、selected adapter handler dispatch、结果校验和错误映射结果，同时不把 adapter 或 protocol envelope 变成日志 owner。
- [x] 3.2 实现 metadata-only JSONL event writer，支持 schema version、timestamp、event、request id、operation、adapter id、duration、operation/output status metadata、response size、基于 RustCrypto `sha2` crate 的 SHA-256 content hash metadata、content capture event 和 bounded diagnostic summary。
- [x] 3.3 实现日志配置解析和 sink 初始化，保证未启用日志时不产生可观察输出变化、protocol shape 变化、handler payload 变化或日志文件副作用。
- [x] 3.4 实现日志写入失败降级，确保日志目录不可写、序列化失败或 append 失败不改变原本文档操作结果。
- [x] 3.5 实现可选 content capture writer，只有单独 CLI/config 开启且 root path 解析成功时才把正文文件写入日期目录和 `sha256-<content_hash>.content` 文件名，并在主调用日志追加 `content_captured` 或 `content_capture_failed` 事件；主操作结果事件只引用 SHA-256 hash。
- [x] 3.6 如审计决定引入外部日志框架，先完成依赖更新与初始化隔离，再接入 writer；否则保留仓库内 JSONL writer。

## 4. 测试与验证

- [x] 4.1 增加 core 层单元或集成测试，覆盖未启用时没有日志副作用，启用后成功调用写入可解析 JSONL event，并用 `request_id` 关联 request/response。
- [x] 4.2 增加失败路径测试，覆盖 adapter selection failure、linked handler structured diagnostic、protocol/result validation 失败和稳定错误映射摘要。
- [x] 4.3 增加 stdout purity 测试，证明启用日志后 `protocol-json` stdout、`readable-json` stdout、readable-view stdout 和 linked adapter handler payload 不被日志污染。
- [x] 4.4 增加安全/体积测试，证明 metadata-only 默认不记录 full read content、完整 request/response payload、完整 diagnostic/debug output 或无界 query/ref，并记录 `hash_algorithm: "sha256"` 和小写 64 位十六进制 `content_hash`。
- [x] 4.5 增加 content capture 测试，证明未单独开启时不写正文文件，开启后正文文件写入独立 root 下的日期/`sha256-<content_hash>.content` 相对路径，文件名 hash 与正文文件 bytes 的 SHA-256 一致，主操作结果事件只引用 hash，主日志另写 `content_captured` 事件。
- [x] 4.6 增加 schema/example validation，证明 invocation log operation event 和 content capture event variant 都符合对应 JSON Schema。
- [x] 4.7 增加日志写入失败降级测试，证明不可写日志路径或 content capture root path 不会改变原本文档操作的成功/失败语义。
- [x] 4.8 运行受影响 Rust tests、OpenSpec validation、schema/example validation；若同步修改主规范、schema、examples 或跨 crate 行为，运行 `bun run verify:docnav-workspace`。

## 5. 交付审计

- [x] 5.1 用局部 diff 审计实现是否只触及 invocation logging 相关 docs、specs、schemas、examples、tests 和代码，且没有绕过 Owner Placement 直接决定 CLI/config surface。
- [x] 5.2 抽查启用 metadata-only 日志后的实际 JSONL，确认每行可独立解析、字段稳定、诊断有界、通过 schema，且没有 inline document content。
- [x] 5.3 抽查 content capture，确认它只能通过显式配置开启，SHA-256 content hash 可关联，日期/`sha256-<content_hash>.content` 相对路径稳定，正文文件未进入主操作结果事件，截断/脱敏策略实际生效。
- [x] 5.4 记录最终验证命令、结果和任何未覆盖风险，再进入归档或新的审计改进 change。
