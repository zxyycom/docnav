本 tasks 根据 `docs/diagnostics.md` 的目标形态拆分错误通道迁移路径。主规范、schema、示例和实现行为在对应实施任务中同步更新。

## 1. 实施准备

- [x] 1.1 盘点当前事实源：列出 `StableError`、`StableErrorCode`、`Warning`、`WarningId`、`StandardParameterDiagnostic`、adapter candidate/config source warning、`docs/protocol/error-rules.json` 和直接 stderr 诊断的 owner、输出通道和测试覆盖。
- [x] 1.2 盘点 full migration surface：记录 `protocol-json`、manifest、probe、readable output、stderr、exit behavior、schema、examples、fixtures、consumer tests、generator scripts 和 generated files 的切换范围；投影/details 映射作为内部实现处理，只在 observable field、通道或验证材料变化时同步对应 owner。
- [x] 1.3 确认完成标准：最终状态删除或替换旧 error/warning 事实源，不保留 `docs/protocol/error-rules.json` 或旧 generated required-details 规则作为并行入口。

## 2. 契约同步

- [x] 2.1 将 `docs/diagnostics.md` 作为错误通道 owner 文档，核对 `docs/architecture.md` 的 `docnav-diagnostics` owner 描述与其一致。
- [x] 2.2 更新 `docs/output.md`、`docs/protocol.md`、`docs/cli.md` 和 `docs/adapter-contract.md` 中涉及错误、warning、stderr、exit behavior 和 output shape 的投影说明。
- [x] 2.3 更新 `docs/standard-parameters.md` 中 validation/source-skipped/ignored argv 等诊断交接边界，使标准参数层输出错误通道记录或可直接入栈的数据。
- [x] 2.4 更新 `docs/testing.md`、`docs/testing/case-maintenance.md` 和覆盖矩阵，明确错误通道语义、canonical details、投影、schema/example/fixture 和 consumer tests 的证明目标。
- [x] 2.5 更新受影响 JSON Schema、examples 和 fixtures，使它们校验 surface 投影，不成为 code/details 规则来源。
- [x] 2.6 删除 `docs/protocol/error-rules.json` 的文档入口和生成源说明；若保留 machine-readable 投影 artifact，标注为派生验证材料。

## 3. 诊断模型实现

- [x] 3.1 在 `docnav-diagnostics` 定义 code family enum，并由顶层 `DiagnosticCode` 聚合这些 enum；family 边界按实现可维护性选择，不作为 public contract 或 surface 身份来源。
- [x] 3.2 为每个 `DiagnosticCode` 定义 canonical details object；使用 typed details struct/enum、checked builder 或 code-specific constructor 强制字段完整性和类型约束。
- [x] 3.3 定义记录类型字段：问题事实、severity、`DiagnosticCode`、effect、canonical details、source 和 fatal/recoverable 语义。
- [x] 3.4 为 `DiagnosticCode` 暴露 category、default severity/effect、details rule 和 surface 投影规则；投影规则不拥有文案或 stdout/stderr placement，具体 field 映射由 boundary 内部投影到对应 owner contract。
- [x] 3.5 定义 `DiagnosticId`、mark 和栈 API：push、get by id、mark、peek/pop recent、drain_after(mark)、drain_after_event(id, include_anchor)、snapshot 和 flush-oriented assertions。
- [x] 3.6 明确 stack ordering：内部 pop/drain/snapshot 默认 LIFO；需要正序、分组或 surface-specific ordering 时由调用方显式 reverse 或 group。
- [x] 3.7 让 `docnav-protocol`、`docnav-output`、`docnav-adapter-sdk`、core 和 `docnav-standard-parameters` 直接依赖 `docnav-diagnostics` 使用 `DiagnosticCode`；不得在各 crate 重新声明同义 code enum 或 required details 规则。
- [x] 3.8 删除或重构 `StableError`、`StableErrorCode`、独立 `Warning` fact type、`WarningId` owner 和 `StandardParameterDiagnostic`；保留的 helper 必须从错误通道记录和 `DiagnosticCode` 投影输出。

## 4. 调用方迁移

- [x] 4.1 迁移 core document operation，使 parse/runtime/output 携带 result 或 failure outcome 加 accumulated 错误通道记录。
- [x] 4.2 迁移 adapter routing/probe/candidate selection，使可恢复候选失败和跳过原因先入栈，再由最终 surface 决定是否展示。
- [x] 4.3 迁移 `docnav-output`，由错误通道记录投影 readable warning/error、protocol output 和 stderr flush。
- [x] 4.4 迁移 adapter direct document operation，使 readable/document protocol 输出与 core 共享同一错误通道交接。
- [x] 4.5 收口 adapter SDK 直接 stderr 旁路：direct CLI input error、manifest/probe warning、invoke decode diagnostic、JSON/schema/write failure 和 output write failure。
- [x] 4.6 迁移 `docnav-standard-parameters` diagnostics handoff，使 validation failure、source-skipped warning 和 ignored argv 进入统一错误通道。
- [x] 4.7 迁移非 document 命令（config/init/doctor/version/help 等适用路径），使项目内错误和诊断统一进入错误通道，并由各命令 owner 决定 plain text 或其它输出。
- [x] 4.8 删除 `docs/protocol/error-rules.json` 和依赖它的 generated Rust/TypeScript required-details source；改为从 `docnav-diagnostics` 的 code/detail rules 生成或校验 protocol schema、validator rules 和 constants。
- [x] 4.9 更新 `protocol-response.schema.json`、readable schema 和示例，使 schema 只校验 surface 投影。

## 5. 验证

- [x] 5.1 添加 unit tests，证明可恢复诊断跨 parser/runtime/output 边界后仍可按 id 或 snapshot 取出，且不会由错误通道自身阻断有效 operation。
- [x] 5.2 添加 unit tests，证明 mark 和记录 id 支持批量 drain，`drain_after_event` 可选择是否包含 anchor record，且 drain 不会删除边界之前的记录。
- [x] 5.3 添加 unit tests，证明 pop、drain 和默认 snapshot 返回 LIFO 顺序，调用方显式 reverse 后可得到 insertion order。
- [x] 5.4 添加 `DiagnosticCode` tests，证明每个 family variant 都能通过顶层 code 取得 category、默认 severity/effect、canonical details rule 和投影规则。
- [x] 5.5 添加 details tests，证明每个 code 的 canonical details 缺字段、错类型和多余字段行为符合契约。
- [x] 5.6 添加投影 tests，证明 fatal `DiagnosticCode` 可投影为 protocol/readable error code/details/guidance 和 exit behavior，且栈记录不依赖旧 stable error object。
- [x] 5.7 添加 output tests，证明 `protocol-json`、manifest、probe、readable output 和 stderr 使用错误通道记录作为事实源。
- [x] 5.8 添加 adapter SDK tests，证明 manifest/probe/invoke 诊断从错误通道刷新或投影产生。
- [x] 5.9 添加 readable tests，证明 `readable-view` 和 `readable-json` 从错误通道记录投影 warning/error，并且不会引入目标文档未声明的 `diagnostic_only` effect。
- [x] 5.10 添加 generator/check tests，证明 `docs/protocol/error-rules.json` 已删除，schema/validator/generated constants 不再从它读取，并且 protocol/readable 投影与 `DiagnosticCode` rules 一致。
- [x] 5.11 若实现触及 protocol、schema、examples、output contract 或多个 crate，运行 `bun run verify:docnav-workspace`；否则记录 targeted tests 和未跑全量验证的理由。
