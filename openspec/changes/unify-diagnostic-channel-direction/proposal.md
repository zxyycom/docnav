本 change 根据 `docs/diagnostics.md` 的目标形态记录错误通道迁移方向：运行时问题进入请求内诊断栈，`docnav-diagnostics::DiagnosticCode` 成为唯一机械身份来源，边界 surface 读取记录并投影输出。当前内容只是 `openspec/changes/unify-diagnostic-channel-direction/` 下的未审核临时文档，本次更新只修改该 change 目录内 artifact。

## Why

Docnav 当前同时存在 `StableError`、warning envelope、standard parameter diagnostics、adapter candidate/config source warning、直接 stderr 诊断和 `docs/protocol/error-rules.json` 生成链路。这些入口分别承接错误身份、details 规则和输出映射，导致同一个问题事实在多个 owner 中重复建模。

目标形态需要一个统一事实源：发现问题的模块只把记录压入请求内栈；是否继续、失败、退出或输出，由读取栈的边界层决定。实施本 change 时执行 full migration，旧错误和 warning 事实源必须被删除或替换为 `DiagnosticCode` projection。

## What Changes

- 错误通道语义：每个 top-level `docnav` command、adapter direct command 或 adapter `invoke` request 都有自己的请求内栈；栈不跨进程、不跨独立请求。
- 记录语义：warning、error、fatal、跳过原因、候选失败和无法继续的上下文都进入同一通道；记录问题不等于立即失败。
- 身份语义：push 时由栈分配 `DiagnosticId`；记录的机械身份来自 `DiagnosticCode`；错误规则和警告规则从 `DiagnosticCode` 规则集合派生。
- Code 结构：`docnav-diagnostics` 按用途或 family 手动维护小 enum，并由顶层 `DiagnosticCode` 聚合为唯一机械身份域。
- Details 结构：每个 `DiagnosticCode` 拥有唯一 canonical details object；调用方按该结构构造记录，输出者把该结构映射为 protocol details、readable warning details、stderr 文案或 exit behavior。
- 依赖方向：`docnav-protocol`、`docnav-output`、`docnav-adapter-sdk`、core 和 `docnav-standard-parameters` 直接依赖 `docnav-diagnostics` 使用 code、details 和 projection 规则。
- Surface 分工：CLI、protocol surface、readable output、adapter direct CLI 和 adapter `invoke` handler 在边界读取错误通道记录，并按各自 owner 文档投影。
- Validation source：删除 `docs/protocol/error-rules.json` 及其生成源路径；protocol/readable schema、examples、fixtures、validator 和 tests 消费 diagnostics-owned projection，不再拥有 code/details 规则。
- Full migration：现有 `StableError`、`StableErrorCode`、独立 warning identity、standard parameter diagnostics 和直接 stderr 事实源全部迁移到错误通道记录。

## Capabilities

### New Capabilities

本 change 不新增长期 capability。

### Modified Capabilities

- `docnav-contracts`: 记录统一错误通道、`DiagnosticCode` 事实源、canonical details 和跨 surface 投影策略。

## Impact

- 代码影响：`docnav-diagnostics`、`docnav-protocol`、`docnav-output`、`docnav` core runtime/output、`docnav-adapter-sdk`、`docnav-standard-parameters` 和非 document command 错误路径。
- 生成链路影响：`scripts/generate-error-rules.ts`、`crates/docnav-protocol/src/generated/error_rules.rs`、validator generated rules 和 `protocol-response.schema.json` 的 error details 校验必须改为从 diagnostics-owned rules 产生或校验。
- 验证材料影响：相关主规范、schema、examples、fixtures 和 consumer tests 必须与 surface projection 一起更新。
- 实现前置：先完成 tasks 中的阻塞级审计，确认本 change 的 proposal、design、specs 和 tasks 都围绕 `docs/diagnostics.md` 的目标语义展开。
