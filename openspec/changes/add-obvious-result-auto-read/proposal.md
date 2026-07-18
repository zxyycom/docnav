本 proposal 定义 `add-obvious-result-auto-read` 的目标：为 outline 和 find 的唯一明确结果增加确定性的自动 read 组合，减少低歧义阅读流程的一次工具调用；当前 change 只在 `openspec/changes/add-obvious-result-auto-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

`outline -> ref -> read` 和 `find -> ref -> read` 是 Docnav 的稳定基础链路，但当 outline 或 find 只产生一个明确可读 ref 时，第二次 read 往往只是机械步骤。这个 change 从 `explore-operation-composition` 派生出一个足够窄的实现方向：只在可解释、可测试的低歧义场景中由 core 组合现有 operation，不引入运行时智能判断。

## What Changes

- 为 outline 和 find 增加“唯一明确结果自动 read”的确定性组合行为：先执行原 operation，当结果中只有一个可读 ref 且预算允许时，core 使用同一 document context 调用现有 read。
- 自动 read 只适用于明确的单候选结果；零候选、多候选、缺失 ref、预算不足或 read 失败时，必须保留原 operation 结果并清楚表达未展开原因。
- 组合逻辑归属 core 或可复用 shared helper；adapter 继续只负责 outline/find/read 的单次 operation 语义、ref 生成/解析和分页。
- Core 必须在进入 output plan 前把 base result、auto-read 内容、状态、成本和 continuation 表示为同一个 typed `ProtocolResponse::Success` result。
- `protocol-json` 直接序列化这些组合事实；内置 renderer 从同一 result 生成 `readable-view`，不得维护 renderer-only composition payload。
- 非目标：不新增 adapter-level operation，不解释 adapter-owned ref grammar，不在多个候选之间排序或猜测用户意图，不改变普通 read result 语义。

## Capabilities

### New Capabilities

- 无。该 change 修改既有 core CLI 编排和 document result/output contract，不创建新的长期 capability。

### Modified Capabilities

- `core-cli`: 增加核心 CLI 对 outline/find 低歧义结果的自动 read 组合行为要求，并规定该行为不得改变 adapter 单次 operation 职责。
- `output-contract`: 定义 auto-read 的 typed protocol facts，以及 `protocol-json` 与内置 `readable-view` renderer 对同一结果的投影要求。

## Impact

- 影响 `docnav` 核心 CLI document operation 执行编排，尤其是 outline/find 成功结果后的可选 read 调用。
- 影响 `docnav-protocol` / `docnav-output` / `docnav-readable` 的 typed composition result、`protocol-json` schema/example 和 `readable-view` renderer conformance。
- 可能影响 `docs/cli.md`、`docs/output.md`、`docs/examples/`、`docs/schemas/` 和相关 CLI/readable integration tests。
- 不影响 adapter protocol handler、ref ownership、adapter selection、markdown ref grammar 或基础 `ReadResult` shape。
