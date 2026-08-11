本 tasks 清单记录 structured protocol fields 与 readable output organization 的交付步骤。

## 1. 字段决策

- [x] 1.1 确认 protocol request/result、readable output、schema/examples 和 Rust protocol/diagnostics 类型中的结构化字段基线。
- [x] 1.2 确认 request budget 硬切换为 canonical `limit`。
- [x] 1.3 确认 raw `cost` 使用 `cost.measurements[]`，readable 输出生成成本摘要。
- [x] 1.4 确认 outline/find/info 在 raw protocol 中暴露事实字段，readable 输出拥有 `display`。
- [x] 1.5 确认 error/warning details 由 diagnostics fact source 投影到 protocol/readable。
- [x] 1.6 确认 `page` 保持 protocol-owned next-page integer or null。

## 2. 规范与验证材料

- [x] 2.1 更新 `docs/protocol.md`：`limit`、cost、navigation item、info metadata、page 和 protocol error projection。
- [x] 2.2 更新 `docs/output.md`：readable display、成本摘要、warnings、errors 和 continuation。
- [x] 2.3 更新 `docs/diagnostics.md`：diagnostic record 到 protocol error、readable error 和 readable warning 的 projection owner。
- [x] 2.4 更新 `docs/standard-parameters.md` 和 `docs/adapter-contract.md`：`limit` 参数来源、校验和 adapter-owned budget unit。
- [x] 2.5 更新 protocol request/response schemas 和 examples。
- [x] 2.6 更新 readable schemas 和 examples。
- [x] 2.7 更新 testing/schema/example 文档，说明 cross-layer mapping 验证。

## 3. 实现

- [x] 3.1 更新 `crates/docnav-protocol` request/result/error 类型、serde decode、schema validation 和 compatibility tests。
- [x] 3.2 更新 `crates/docnav-diagnostics` projection rules 或生成输入，使 error/warning details shape 与 docs/schema 对齐。
- [x] 3.3 更新 core CLI 和 adapter SDK standard parameter registration，使新输出使用 `limit`。
- [x] 3.4 更新 Markdown adapter outline/read/find/info 输出，产出 structured cost、item facts 和 info metadata。
- [x] 3.5 更新 readable renderer，使 display、成本摘要、error 和 warnings 从 structured facts 派生。
- [x] 3.6 更新 CLI smoke、adapter tests、schema/example validation 和 fixture/golden outputs。

## 4. 验证

- [x] 4.1 运行 protocol/readable schema example validation。
- [x] 4.2 运行相关 Rust unit/integration tests。
- [x] 4.3 运行 adapter direct CLI 和 core CLI smoke。
- [x] 4.4 运行 `bun run verify:docnav-workspace`。
