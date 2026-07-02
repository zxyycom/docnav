本 proposal 定义 `adopt-strict-input-boundaries` 的变更目标：将 Docnav 公共输入边界确认为 strict-by-default，并通过可执行诊断引导调用方修复输入。

## Why

Docnav 的 AI 友好性需要落到“失败后可一次修正”。调用方显式输入不符合契约时，CLI parser、标准参数、adapter routing、输出模式、schema 和测试围绕同一个 owner 边界返回可修复错误，长期维护成本更低。

本 change 将 AI 友好性放到错误质量上：调用者显式表达的输入不符合契约时快速失败，系统通过一个稳定、结构化、可操作的 primary `DiagnosticRecord` 降低下一次调用修复成本。目标契约是“合法输入成功、非法输入失败且可修复”。

## What Changes

- **BREAKING**：公共输入边界按 strict-by-default 校验。未知 argv、多余 positional、当前 operation 不支持的 flag、未归属的协议/配置字段和 undeclared native option 在 owner 边界返回输入诊断。
- **BREAKING**：显式用户意图按 owner contract 校验。显式 `--adapter`、显式配置路径、显式 ref/path/operation 参数失败时返回对应 owner diagnostic。
- 自动 discovery、隐式默认配置缺失和内部候选探测属于 owner-declared internal flow；成功结果表达被选中操作的成功 payload，全部候选失败时返回候选失败列表。
- failure surface 使用一个 primary `DiagnosticRecord`，字段名由 diagnostics owner 统一；字段问题列表、config 问题列表和候选失败列表作为该诊断的从属结构化失败信息。
- document success output 承载成功业务 payload 和对应输出模式拥有的结构；失败修复建议进入 failure diagnostic。
- 标准参数层接收有明确 owner 的 core CLI、protocol argument、project config、user config 和 default 输入源；adapter-owned `options` 和 native options 作为 owner-scoped source 进入 generic handoff 和 adapter-side validation。
- core CLI 保留 `clap` 作为 parser/help 的默认实现依赖；严格输入契约由 parser/mapper 直接产出 input diagnostic。
- 非目标：document operation、ref ownership、格式导航策略和当前实现代码由各自 owner change 承接。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `core-cli`: core CLI 输入边界按 strict-by-default 校验，并定义 explicit intent failure 的诊断规则。
- `standard-parameter-resolution`: direct input、config source 和 operation applicability 使用 owner-scoped source 规则；adapter native options 是明确 owner 的输入源。
- `adapter-protocol`: linked adapter handler boundary、protocol request source handling、manifest/probe/descriptor metadata 与 adapter-owned options 使用严格输入责任边界；legacy invoke semantics 只保留为 compatibility。
- `docnav-contracts`: 公共契约原则更新为 invalid caller intent 在 owner 边界失败，定义内部探测、single primary `DiagnosticRecord` 和成功 payload 边界。
- `readable-view-output`: readable/raw 输出投影规则更新为成功输出承载业务 payload，失败输出承载 primary `DiagnosticRecord`。
- `markdown-navigation`: core-linked Markdown adapter、core 配置读取、adapter-owned options 和 core CLI smoke/矩阵测试使用 strict failure 与 primary `DiagnosticRecord` 证明。

## Impact

- 受影响文档与规范：`docs/architecture.md`、`docs/cli.md`、`docs/standard-parameters.md`、`docs/adapter-contract.md`、`docs/protocol.md`、`docs/output.md`、`docs/diagnostics.md`、`docs/ref-contract.md`、`docs/testing.md`、`docs/testing/cases.md`、OpenSpec specs 以及相关 examples/schemas。
- 受影响 Rust crate：`docnav`、`docnav-navigation`、`docnav-standard-parameters`、`docnav-adapter-contracts`、`docnav-markdown`、`docnav-diagnostics`、`docnav-output`、adapter routing 模块，以及覆盖 strict failure、primary `DiagnosticRecord`、owner-scoped native options 和 success payload shape 的 smoke/unit tests。
- 受影响 public surface：core CLI exit behavior、protocol/readable error shape、stdout/stderr channel policy、adapter selection semantics、config source handling、schema examples 和 test case expectations。
- 受影响 active changes：`replace-clap-with-bpaf-frontend` 需要改为 strict CLI 下保留 `clap` 的方向；`separate-entry-pipeline-from-parameter-resolution` 需要改为 owner-scoped native option source；`implement-docnav-mcp-bridge`、`outline-unstructured-full-read`、`enable-local-core-adapter-service-mode` 和 `markdown-document-head-outline-mode` 需要与成功 payload、failure diagnostic、internal-event ownership 和 adapter-owned option 声明保持一致。Track A 负责协调其它 active changes 中涉及 diagnostic、protocol/readable output、config、native option、adapter selection 和 CLI parser/help 的语言。
- 交付方式：按 docs、active OpenSpec coordination、schema/examples、crate implementation 和 verification 多个 owner 轨道并行推进。每个轨道在集成前报告改动文件、验证命令和阻塞点。

## Success Criteria

- 每个 public input owner 都使用同一条失败规则：invalid caller input 在 owner 边界返回一个 primary `DiagnosticRecord`。
- 成功 document output 在 readable 和 protocol-json 投影中只包含成功业务 payload 与该输出模式拥有的结构。
- Adapter discovery 作为内部、未显式声明候选的评估流程保留；全部候选失败时返回候选失败列表。
- 显式 adapter/config/ref/path/operation intent 按 owner contract 校验，并在失败时返回对应诊断。
- Adapter native options 建模为 explicit owner-scoped source。
- Active OpenSpec changes 和 owner docs 的执行指令统一指向 strict parser/mapper、primary `DiagnosticRecord`、owner-scoped native options 和成功 payload 投影。
- OpenSpec spec deltas 覆盖 `docnav-contracts`、`core-cli`、`adapter-protocol`、`standard-parameter-resolution`、`readable-view-output` 和 `markdown-navigation` 中的 strict input boundary、primary `DiagnosticRecord`、owner-scoped native options、success payload projection 和 readable conformance 契约。
