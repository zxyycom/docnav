本 change 定义 Docnav 协议字段结构化方案，并同步定义 readable 输出如何组织这些结构化事实。

## 背景

当前 raw protocol 中仍有多类机器事实压缩在展示字符串里：

- `limit_chars` 把分页预算命名为字符预算，无法承接 adapter-owned numeric budget。
- `cost` 使用 `7 lines | 0.1 KB` 这类展示文本，调用方无法稳定读取 measurement。
- outline/find/info 的 `display` 同时承载 label、location、summary、encoding、size 等事实。
- protocol/readable error 和 warning 的 `details` 已有结构来源，但 schema 与文档对字段类型、必需/可选字段和投影归属约束不够集中。

这些字段会直接影响 `configure-pagination-defaults` 和 `use-token-based-document-cost`，也会影响后续 adapter、schema、example 和 readable renderer 的稳定契约。

## 变更内容

- Protocol request 硬切换为 canonical `limit`；新 schema、examples、typed arguments 和 renderer input 都使用 `limit`。
- Protocol result 使用结构化 `cost.measurements[]`，readable 输出负责生成成本摘要文本。
- Outline/find/info result 拆出结构化事实字段；readable 输出拥有 `display` 和阅读布局。
- Protocol error details 与 readable warning details 按 diagnostics projection 收紧为 per-code/per-id 结构。
- 同步更新主规范、schema、examples、Rust protocol/diagnostics 类型、core/SDK 参数映射、Markdown adapter 和 readable renderer。

## 能力

### 新增能力

- 无。

### 修改能力

- `docnav-contracts`: 定义 structured protocol fields、diagnostic projections 和 readable output organization 的跨层契约。

## 影响范围

- Protocol request/response schema、examples 和 `crates/docnav-protocol` 类型会发生可观察字段变化。
- Diagnostics projection、protocol error schema、readable warning schema 和示例需要对齐 details shape。
- Core、adapter SDK 和 adapter direct CLI 需要在 CLI/config/invoke budget 输入中使用 `limit`。
- Markdown adapter 需要产出 structured cost、navigation item facts 和 info metadata。
- readable-view/readable-json renderer 需要从 structured facts 生成 display、成本摘要、warnings、errors 和 continuation。
- `configure-pagination-defaults` 依赖本 change 的 `limit` 决策。
- `use-token-based-document-cost` 依赖本 change 的 `cost.measurements[]` shape。
