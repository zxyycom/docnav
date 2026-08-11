本 design 根据 `docs/diagnostics.md` 的目标形态记录错误通道实现方向：请求内栈保存问题记录、机械身份和 canonical details，边界层负责失败决策和输出投影。主规范、schema、示例和实现行为由实施任务同步更新。

## Context

目标文档已经把错误通道收敛为长期语义边界：

- 错误通道是进程内、请求内的栈；发现问题的地方只负责把记录压入栈。
- 通道本身只保存记录，不判断 operation 成败，不决定 exit code，不组织用户可见文案，也不替任何 surface 选择输出格式。
- 每条记录有栈分配的 `DiagnosticId`，记录的机械身份来自 `DiagnosticCode`。
- `DiagnosticCode` 承接规则集合；错误规则和警告规则是 code 规则集合的投影子集。
- CLI、protocol、readable output、adapter direct CLI 和 adapter `invoke` handler 在边界读取栈，并按各自 owner 文档投影输出。

迁移前的错误和 warning 事实分散在 `StableError`、`StableErrorCode`、warning envelope、standard parameter diagnostics、adapter candidate/config source warning、直接 stderr 诊断和 `docs/protocol/error-rules.json` 生成链路中。迁移目标是把这些事实收敛到 `docnav-diagnostics`，让其它 crate 直接消费 diagnostics-owned code、details 和投影规则。

## Goals / Non-Goals

**Goals:**

- 项目内 warning、可恢复诊断、跳过原因诊断和 fatal context 在进入输出层前都进入同一请求内错误通道。
- `DiagnosticCode` 成为 warning 和 error 的唯一机械身份来源。
- 每个 `DiagnosticCode` 拥有一个 canonical details object，调用方和输出方都以该结构为事实来源。
- `docnav-protocol`、`docnav-output`、`docnav-adapter-sdk`、core 和 `docnav-standard-parameters` 直接依赖 `docnav-diagnostics` 使用 code、details 和投影规则。
- `docs/protocol/error-rules.json` 不再作为规则源，protocol/readable/schema/validator 消费 diagnostics-owned 投影。
- 实现完成后不保留旧 error/warning 事实源的长期兼容层。

**Non-Goals:**

- 本 change 的 design 不直接修改主规范、schema、examples、fixtures 或实现；实施任务按 owner 同步更新受影响材料。
- 本 change 不把 `DiagnosticId`、stack index、mark 或内部 LIFO 顺序暴露为 public protocol/readable ref。
- 错误通道不判断 operation 是否失败；失败、继续和 exit behavior 由调用方或 surface owner 决定。
- 本 change 不改变 adapter-owned ref、format parsing、native option validation 或 document operation 业务语义。
- 本 change 不迁移一次性开发脚本、测试 harness 和辅助工具 stderr 输出。
- 本 change 不新增 `diagnostic_only` 等目标文档未声明的 warning effect；后续有明确行为需求时再单独设计。

## Decisions

### Decision 1: `docnav-diagnostics` owns the diagnostic model

`docnav-diagnostics` 定义 `DiagnosticStack`、记录类型、`DiagnosticId`、mark、`DiagnosticCode`、canonical details 和投影 metadata。该 crate 保持底层依赖，只依赖 serde/serde_json，不依赖 protocol、output、core 或 SDK。

选择这个方向，是因为错误身份和 details 规则需要一个事实源。并行维护 `StableErrorCode`、warning id、standard parameter diagnostic 和 protocol-local JSON 会让同一问题事实在多个 owner 中重复建模。

### Decision 2: `DiagnosticCode` aggregates grouped enums

实现使用手动分组的小 enum 加顶层聚合 enum。family 边界是实现内组织方式，由实现阶段按源码可读性、维护成本和现有 owner 分布选择；可以按 producer domain、问题类别或其它便于维护的方式拆分。family 名称和边界不作为 public contract，也不能成为 protocol、readable、stderr 或 exit behavior 的身份来源。顶层 `DiagnosticCode` 通过 wrapper variant 或 `From` impl 聚合这些 enum，形成唯一机械身份域。

这个结构保留源码可读性，同时禁止各 crate 重新声明同义 code。实现可以在不改变顶层 `DiagnosticCode` 语义的前提下拆分或合并 family；每个 family variant 必须能通过顶层 `DiagnosticCode` 取得 category、默认 severity/effect、canonical details rule 和投影规则。

### Decision 3: Canonical details are owned by code rules

每个 `DiagnosticCode` 必须有一个 canonical details object。调用方 push 记录时提供符合该 code 的 details；实现可以用 typed details enum/struct、checked builder 或 code-specific constructor 强制字段完整性和类型约束。任意 `serde_json::Value` 只能作为已校验后的传输形态。

输出者根据 surface owner contract 映射 canonical details：protocol error 可以投影为 `error.details`，readable warning 可以投影为 warning `details`，stderr 可以格式化为文本，exit behavior 可以只读取 code category。该映射属于内部投影实现，不需要在本 change 逐字段预先规定；只有 observable output、schema、examples、fixtures 或 owner docs 发生变化时，才需要在对应 owner 中明确同步。映射可以筛选、重命名或补充文案，但不能反向定义 code 的 details 结构。

### Decision 4: The stack stores facts, not outcomes

调用点发现问题时创建记录并 push 到栈。若剩余输入仍能形成有效 operation，调用点继续执行；若问题阻断执行，调用点先记录 fatal context，再返回或传播失败结果。错误通道不判断 operation 是否失败，不决定 exit code，也不负责分组、格式化或输出通道。

`DiagnosticId`、mark 和记录锚点只在同一栈生命周期内有效。栈默认按 LIFO 读取；需要插入顺序、分组或 surface-specific ordering 的调用方显式反转或自行组织。

### Decision 5: Boundary surfaces project stack records

读取通道的是边界层：CLI、protocol surface、readable output、adapter direct CLI 或 adapter `invoke` handler。具体 stdout/stderr 通道、JSON shape、用户可见文案和 exit behavior 仍由 `docs/protocol.md`、`docs/output.md`、`docs/cli.md` 和 `docs/adapter-contract.md` 等 owner 文档定义。

Protocol schema、readable schema、examples 和 fixtures 是投影校验材料。它们消费 `DiagnosticCode` 导出的投影，不拥有错误规则或警告规则。实现可以在 boundary 内部完成投影映射；本 change 只要求每个 observable surface 的最终字段、通道、exit behavior 和验证材料与对应 owner contract 一致。

### Decision 6: Protocol error rules JSON is removed

`docs/protocol/error-rules.json` 不再作为源文件保留。由它驱动的生成链路迁移到 diagnostics-owned rules：`docnav-protocol` 直接依赖 `docnav-diagnostics` 使用 `DiagnosticCode` 和 protocol 投影；schema/validator 通过 diagnostics-owned exporter、generated artifact 或 check-only 验证消费 protocol 投影。

删除该文件后，`protocol-response.schema.json` 仍是 protocol surface 的校验材料。其 error code enum 和 details 校验只反映 `DiagnosticCode` 的 protocol 投影。

### Decision 7: Migration is complete, not compatibility-preserving

实现可以分提交推进，但完成状态必须删除旧事实源：`StableError`、`StableErrorCode`、独立 warning fact type、`WarningId` owner、`StandardParameterDiagnostic` 和 direct stderr diagnostic 入口。保留的 helper 必须命名为投影 helper，并从错误通道记录和 `DiagnosticCode` 派生输出。

## Risks / Trade-offs

- 风险：统一记录模型变成过宽的通用对象。缓解：记录只承载问题事实、机械 code、影响、canonical details 和来源；失败判断、分组和格式化留给调用方或 surface owner。
- 风险：迁移同时触及多个 output surface。缓解：实施任务同步更新 owner docs、schema、examples、fixtures 和 consumer tests，并用 workspace verification 覆盖跨 surface mapping。
- 风险：内部 LIFO 顺序不适合某些用户可见输出。缓解：栈默认 LIFO；需要正序、分组或 surface-specific ordering 的调用方显式反转或自行组织。
- 风险：`DiagnosticId` 被误认为跨进程或跨输出的 public ref。缓解：id 只在同一栈生命周期内有效；public protocol/readable output 不暴露 stack id、mark 或 index。
- 风险：`docnav-protocol` 等 crate 依赖 `docnav-diagnostics` 扩大 dependency surface。缓解：`docnav-diagnostics` 保持底层 crate，不依赖 protocol/output/core/SDK；依赖方向仍单向。
- 风险：删除 `error-rules.json` 后 schema 生成失去简单 JSON 输入。缓解：使用 diagnostics-owned exporter 或 check-only 验证生成 protocol/readable 投影，schema 继续是验证材料而不是规则来源。
- 风险：手动分组 enum 和顶层聚合 enum 漏接某个 family。缓解：顶层 `DiagnosticCode` 使用 exhaustive match、`From` impl 和 tests 验证每个 family variant 都有 category、details rule 和投影规则。

## Migration Plan

1. 盘点当前 warning、stable error、standard parameter diagnostic、direct stderr 和 `error-rules.json` 生成链路，按 producer、surface 投影和 details shape 分类。
2. 在 `docnav-diagnostics` 定义 code family enum、顶层 `DiagnosticCode`、typed details、记录类型、`DiagnosticId`、mark 和栈 API。
3. 让 protocol、output、SDK、core 和标准参数 crate 直接依赖 `docnav-diagnostics`，使用 `DiagnosticCode` 和 canonical details 构造或投影记录。
4. 删除 `docs/protocol/error-rules.json` 和基于它的生成链路，改为从 diagnostics-owned code/detail rules 生成或校验 protocol schema、validator rules 和 constants。
5. 迁移 core、adapter direct command、adapter invoke、standard parameters 和非 document commands，使它们以 result/outcome + 错误通道记录交给边界层。
6. 同步更新 owner docs、schema、examples、fixtures 和 consumer tests，明确 protocol/readable/manifest/probe/stderr/exit behavior 如何消费错误通道记录。
7. 删除旧 error/warning 事实源和直接 stderr diagnostic 入口，更新 tests 证明不再依赖 legacy compatibility。
8. 验证 id lookup、mark/event-id drain、LIFO 返回顺序、canonical details 校验、可恢复问题保留和 fatal diagnostic code 投影。

## Implementation Audit

本审计记录迁移起点的事实源和受影响 surface。它不替代 owner docs；当 observable fields、通道或验证材料变化时，由实施任务同步更新对应主规范、schema、examples、fixtures 和 tests。

### Existing fact sources

| 迁移前事实源 | 迁移前 owner | 迁移前通道 / 消费方 | 迁移前覆盖 |
| --- | --- | --- | --- |
| `StableError` / `StableErrorCode` | `crates/docnav-protocol/src/error.rs` 拥有 protocol-visible error object、code enum、category 和 required-details hook。Core、SDK、output 和 adapters 直接构造或消费它。 | `ProtocolResponse::Failure` failure envelope；`docnav-output` readable error 投影；`crates/docnav/src/error.rs` 和 `crates/docnav-adapter-sdk/src/error.rs` 进程退出映射；adapter output contract validation 调用 `validate_required_details`。 | `crates/docnav-protocol/src/tests/basic.rs`、`crates/docnav-output/src/tests.rs`、`crates/docnav-adapter-sdk/src/tests/error.rs`、adapter/core smoke 和 schema/example validators。 |
| `Warning` / `WarningId` | `crates/docnav-diagnostics/src/warning.rs` 拥有 warning details、投影和 warning text / JSON attachment helpers；`crates/docnav-diagnostics/src/warning/id.rs` 拥有 warning id constants 和 id 校验。 | Readable output `warnings` array；readable-view header warning payload；`protocol-json` warning lines on stderr；direct adapter machine-mode warnings on stderr。 | `crates/docnav-diagnostics/src/tests/warning.rs`、`crates/docnav-output/src/tests.rs`、`crates/docnav-adapter-sdk/src/direct/args/tests.rs`、core 和 markdown smoke warning assertions。 |
| `StandardParameterDiagnostic` | `crates/docnav-standard-parameters/src/resolution.rs` 拥有独立的 validation-or-warning diagnostic enum。 | Core 和 SDK consumers 把 validation diagnostics 转换为 `StableError::invalid_request`；config-source diagnostics 携带 standalone `Warning` 值进入 direct CLI output。 | `crates/docnav-standard-parameters/src/tests/*`、`crates/docnav-adapter-sdk/src/direct/args/tests.rs`、markdown config smoke。 |
| Adapter candidate warning | Core routing 拥有 `AdapterSelectionWarning`；`crates/docnav/src/runtime.rs` 把它转换为 `Warning::adapter_candidate_failure`。 | Readable output warning array/header；当 selected surface 必须保持 protocol-shaped stdout pure 时写 stderr。 | Core adapter-selection smoke 和 warning assertion helpers；`docnav-diagnostics` constructor shape test。 |
| Adapter config source warning | `crates/docnav-standard-parameters/src/construction/config.rs` 从 config-source read failures 构造 `Warning::adapter_config_source_skipped`。 | Adapter direct readable output warning array/header；适用时写入 direct machine output stderr。 | Standard-parameter construction/pipeline tests、adapter direct args tests 和 markdown config smoke。 |
| `docs/protocol/error-rules.json` | `docs/protocol/error-rules.json` 与 `scripts/generate-error-rules.ts` 作为 machine-readable source 拥有 required protocol error details。 | 生成 `crates/docnav-protocol/src/generated/error_rules.rs`、`scripts/tools/validators/generated/error/rules.ts` 和 protocol schema error detail branches；validators 导入 generated required-details constants。 | `scripts/generate-error-rules.ts --check` 通过 workspace checks、protocol schema validation 和 protocol example error-details checks 覆盖。 |
| Direct stderr diagnostics | SDK/core boundary code 通过 `emit_diagnostic`、`write_io_error`、direct CLI input errors 和 output write failure handlers 拥有 direct lines。 | manifest/probe schema 或 semantic failures、invoke decode/read failures、adapter boundary failures、direct CLI usage errors 和 output write failures 的 stderr。 | Adapter SDK invoke/error/output tests、markdown invoke-error smoke、core CLI smoke stderr assertions 和 workspace smoke harness checks。 |

### Full migration surface

| Surface | 切换范围 |
| --- | --- |
| `protocol-json` | 用 diagnostic records 和 protocol 投影替换 `StableError` 的事实源地位。`docnav-protocol` 仍拥有 response envelope、request id behavior 和 schema validation helpers，但消费 diagnostics-owned code/details 投影。 |
| Manifest / probe | Manifest 和 probe stdout schema 仍由 protocol/adapter contract 拥有。Manifest/probe schema、semantic、serialization 和 write failure diagnostics 从 direct stderr construction 迁移为 diagnostic records，并由 adapter boundary 刷新或投影。 |
| Readable output | 用 diagnostic-stack 投影 readable warning/error payloads，替换直接输入 `Warning` 和 `StableError` readable mapping。除非同一任务同步修改 owner docs 和 validation material，否则保留当前 observable warning ids/effects/details。 |
| stderr | 用 CLI、adapter direct CLI 和 adapter `invoke` 边界的 stack flush/投影替换 direct warning text 和 `emit_diagnostic` fact construction。Test harness 和 one-off development script stderr 不在范围内。 |
| Exit behavior | 把当前 keyed by `StableErrorCode` category 的映射迁移到 diagnostics-owned 投影/category metadata；`docnav` 和 adapter SDK 仍拥有各自 concrete process exit code enums。 |
| Schema | JSON Schema 保持验证材料角色。`protocol-response.schema.json` 和 readable schemas 只校验 projected surface shape，不定义独立 diagnostic code 或 details rules。 |
| Examples / fixtures | 只在 observable fields 或 channels 改变时更新 protocol error examples、readable examples、manifest/probe examples 和 smoke fixtures。它们仍是 examples 和 fixtures，不是 rule sources。 |
| Consumer tests | 单元和 smoke tests 从断言 legacy constructors/enums 事实源迁移为断言 diagnostic records、stack semantics 和 surface 投影。保留 black-box stdout/stderr/exit behavior assertions。 |
| Generator scripts / generated files | 删除 `docs/protocol/error-rules.json` 和 generated required-details chain 作为规则源。任何保留的 generated artifact 都必须从 `docnav-diagnostics` 投影 metadata 派生，或变成 check-only validation material。 |

### Completion standard

完成后的实现不能保留并行 legacy 事实源。`StableError`、`StableErrorCode`、standalone `Warning`、`WarningId`、`StandardParameterDiagnostic`、direct stderr diagnostic constructors 和 `docs/protocol/error-rules.json` 必须删除，或重命名/重构为消费 `DiagnosticStack` records 与 `DiagnosticCode` metadata 的投影 helper。Generated Rust/TypeScript required-details constants 不能来自已删除的 JSON 文件。Tests 必须证明 diagnostics-owned stack、code/details rules 和投影覆盖此前的 protocol、readable、stderr、exit、schema 和 example surfaces。
