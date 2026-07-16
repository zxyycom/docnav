## Why

本 change 的目标是把 document operation 输出收敛为稳定协议与代码渲染两条路径：`protocol-json` 保持唯一 machine-readable contract，rendered path 接收代码注入的 renderer，默认使用 `readable-view`；当前文档只在 `openspec/changes/unify-output-with-injected-rendering/` 下形成未审核临时文档，不影响现有其它文档或主规范。

当前三模式模型为同一 operation outcome 维护 protocol envelope、readable JSON shape 和 readable view framing，扩大了 schema、投影和验证维护面。渲染应作为 output composition 的可替换 presentation policy，而不是并列的结构化传输契约。

## What Changes

- **BREAKING**：document output 只保留 `ProtocolJson` 与 `Rendered(RenderStrategy)` 两个内部路径；CLI 的 `protocol-json` 选择协议路径，默认或 `readable-view` 选择 rendered path。
- Rendered path 在进入 output orchestration 前获得一个代码函数/trait renderer。Core 默认组合提供内置 `readable-view`，其它 linked code caller 可以注入替代实现。
- Renderer 消费已经完成的 operation success outcome 或 primary `DiagnosticRecord`，返回完整 UTF-8 text；output layer 继续拥有 stdout/stderr、render failure 和 exit mapping。
- `protocol-json` 完全绕过 renderer，继续保留现有 envelope、result/error、ref 和 pagination contract。
- **BREAKING**：删除 `readable-json` output mode、serialized readable DTO 及对应 schema、examples、fixtures、goldens 和验证分支；产品不增加旧 mode 的兼容行为。
- Adapter 继续只拥有格式解析、operation result 和 adapter facts；renderer 注入不进入 adapter definition、manifest、probe 或外部配置 surface。
- 自定义 renderer 只拥有 presentation text。需要稳定结构化事实的调用方使用 `protocol-json`。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `output-contract`: 定义 protocol/rendered 两路径、renderer 函数契约、默认策略、通道和失败边界。
- `core-cli`: 将 document output 选择收敛到 `readable-view` 与 `protocol-json`，并由 core composition 为 rendered path 提供 renderer。
- `diagnostics-contract`: 让同一个 primary diagnostic 进入 protocol serializer 或 renderer，而不产生第三种 public projection。
- `invocation-logging`: 保持 protocol stdout、renderer input/output 和独立日志 sink 互不污染。
- `release-artifacts`: 让 release package 只验收 protocol output 与默认 rendered output 的既有行为。

## Impact

- 影响 `docnav-output`、默认 readable renderer、core CLI output field/composition、primary diagnostic projection 和相关 tests。
- 影响 `docs/output.md`、`docs/cli.md`、对应 schema/example/fixture/golden 索引以及 invocation logging、release smoke 和 workspace validation。
- `derive-document-cli-options-from-fields` 只投影 canonical output field，本 change 只修改该 field 的 owner facts；两者无实施顺序依赖，后合并的一方按当前 declaration 处理普通代码冲突。`interactive-outline-selection`、`implement-docnav-mcp-bridge`、`add-outline-preview-skim-pack`、`add-obvious-result-auto-read` 和 `explore-operation-composition` 含有旧三模式假设，实现前需要重基或暂停。
- `protocol-contract`、adapter operation contract、ref、pagination、routing 和 parsing ownership 保持不变。
