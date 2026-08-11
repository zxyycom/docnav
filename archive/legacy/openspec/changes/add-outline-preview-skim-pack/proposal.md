本 proposal 定义 `add-outline-preview-skim-pack` 的目标：让 outline 在预算内附带少量可读预览，使第一屏同时提供结构和正文样本；当前 change 只在 `openspec/changes/add-outline-preview-skim-pack/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

普通 outline 给出结构但不给正文，调用方常常需要逐个 read 才能判断哪些章节值得继续。Skim Pack 从 `explore-operation-composition` 派生出一个低智能、确定性的 preview 方向：保留 outline 的结构判断，同时用预算内短样本减少盲目追读。

## What Changes

- 为 outline 增加预算内 preview 组合行为：先执行原 outline，再对一组确定性选择的 entries 调用现有 read，生成同时包含结构、preview 内容和状态的 typed composition result。
- preview 选择必须使用简单规则，例如 top-level entries、前 N 个 entries、成本阈值或 adapter 已返回的稳定 outline facts；不得依赖运行时智能模型判断文档重点。
- preview 内容必须受总预算约束；预算不足、entry 不可 read、read 失败或分页未完成时，输出必须稳定表达未预览原因和 continuation。
- 组合逻辑归属 core 或可复用 shared helper；adapter 不新增 preview operation，也不需要理解跨章节阅读策略。
- Core 必须在进入 output plan 前把 outline facts、preview content、status 和 continuation 表示为同一个 typed `ProtocolResponse::Success` result；`protocol-json` 序列化这些事实，内置 renderer 从同一结果生成 `readable-view`。
- 非目标：不生成摘要，不重排 outline，不推断用户意图，不把 preview 当作 adapter-owned展示语义，不改变 read/ref/pagination 的基础契约。

## Capabilities

### New Capabilities

- 无。该 change 修改既有 outline result/output contract 和 core 编排行为，不创建新的长期 capability。

### Modified Capabilities

- `core-cli`: 增加核心 CLI 对 outline preview 组合流程的行为要求，包括 deterministic entry selection、预算和错误边界。
- `output-contract`: 定义 outline preview 的 typed protocol facts，以及 `protocol-json` 与内置 `readable-view` renderer 对同一结果的投影要求。

## Impact

- 影响 `docnav` 核心 CLI outline 执行编排，可能在 outline 成功后按规则调用现有 read。
- 影响 `docnav-protocol` / `docnav-output` / `docnav-readable` 的 typed composition result、`protocol-json` schema/example 和 `readable-view` renderer conformance。
- 可能影响 `docs/cli.md`、`docs/output.md`、`docs/examples/`、`docs/schemas/` 和 outline/read integration tests。
- 不影响 adapter protocol handler、ref ownership、adapter selection、markdown ref grammar 或基础 `OutlineResult` / `ReadResult` shape。
