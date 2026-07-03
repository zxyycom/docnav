# 错误通道

Docnav 的错误通道是请求内诊断记录集合和失败投影事实源。发现问题的组件写入 `DiagnosticRecord`；是否阻断当前流程由发现问题的 owning boundary 决定；如何包装、输出和设置退出行为由对应 surface owner 决定。

本文档关注错误通道的长期语义、记录身份、primary failure 选择和从属失败 details。具体类型字段、函数签名和实现细节以 `docnav-diagnostics` 代码为准。

## 当前语义

错误通道服务于三件事：

- 收集诊断事实：错误、警告、跳过原因、候选失败和无法继续的上下文都可以形成诊断记录。
- 统一失败身份：公共失败必须选定一个 primary `DiagnosticRecord`，由它驱动 protocol、readable 和 CLI 错误投影。
- 保留从属证据：相关 field/config/typed/candidate failures 只作为 primary record 的结构化 details 暴露，不形成 sibling error list。

通道本身只保存记录和规则事实。它不判断 operation 成败，不决定 exit code，不组织用户可见文案，也不替任何 surface 选择输出格式。

## 记录与身份

每条通道记录都有一个请求内 `DiagnosticId`。这个 id 只用于同一次命令或请求里的内部关联、primary 选择和从属证据组织；它不是跨进程、跨运行、持久化或 public caller 可依赖的标识，也不定义读取顺序、阶段截取或回放语义。

记录的机械身份来自 `DiagnosticCode`。同一个 code 可以被不同 surface 投影成 protocol error code、stderr 文案或其它输出字段；反过来，surface 输出字段不能成为通道内部的身份来源。

`DiagnosticCode` 同时承接规则集合。规则描述 code 的类别、默认级别、结构化 details 约束和可投影到哪些 surface。当前 public failure contract 使用 blocking diagnostic rules；实现可以按类别拆分代码结构，但规则来源仍从 `DiagnosticCode` 出发。

Public failure surface 必须选定一个 primary `DiagnosticRecord` 作为本次 failed request 的可见失败。Primary record 的公共字段由 diagnostics owner 统一：`code`、`message`、`owner` 必需；`location`、`received`、`expected`、`guidance` 和 `details` 在记录事实可用时提供。Invalid caller input 在可定位时必须提供 `location`，在可枚举或可描述时提供 `expected`，并提供至少一个可执行 `guidance`。

Protocol error 投影消费 primary record，生成 protocol `error` object；其中 `code`、`owner` 和 required `details` 是机器稳定字段，`message` 与 `guidance` 是 surface 文案。Readable error 消费同一 primary record，但使用阅读层包装和精简文案。Surface 投影字段不能反向成为 `DiagnosticCode` 或 details 规则的来源。

多条相关失败只能作为 primary record 的从属结构化 details 暴露，例如 `field_issues`、`config_issues`、`typed_validation_failures` 或 `candidate_failures`。内部调试记录和 automatic discovery 中的 candidate failures 可以写入同一请求的诊断集合；public failure contract 仍只由一个 primary record 驱动。

一条记录至少表达这些语义：

- 发生了什么问题。
- 问题属于 error、warning 还是 fatal 等级别。
- 这个问题对当前流程的影响，例如拒绝输入、文档失败、adapter boundary 失败或内部失败。
- 可机器处理的 details。
- 问题来源，例如 command、adapter、field、path 或处理阶段。
- 作为 public primary record 时的 owner、location、received、expected 和 guidance。

## 严格输入边界

显式 caller input、present config source 和 selected adapter typed-field 参数必须在 owning boundary 完成结构解析、来源映射和 typed validation。失败时形成 blocking diagnostic；它不是可恢复候选，也不能通过继续尝试其它输入来源来吞掉。

Automatic discovery 的 candidate failure 是 adapter selection 证据，不是宽松输入恢复机制。单个候选失败时，selection owner 可以继续尝试后续 adapter；若后续候选成功，这些候选失败不进入 public success output；若全部候选失败，它们只能作为 primary failure 的从属 `candidate_failures` details 暴露。

Navigation input resolution、adapter selection、protocol envelope、readable wrapping、stdout/stderr 和 exit behavior 的完整规则由各自 owner 文档定义。错误通道只定义这些边界交接时使用的记录身份、公共字段义务和从属 details 语义。

## 写入通道

Docnav runtime 和 public surface 上发现的问题都进入错误通道，包括核心 document operation、protocol request construction、adapter layer dispatch、navigation input resolution、adapter selection、配置命令、初始化、doctor、version 和 help 等路径。

写入通道的模块只描述自己看到的问题、来源、级别、影响和结构化 details。它可以声明该问题对当前 boundary 是 blocking、warning、candidate evidence 还是 internal context，但不负责替外层组织最终 surface 输出。

## 读取与投影

读取通道的是边界层：CLI、protocol surface、readable output 或 adapter layer caller。

这些 surface 可以定义如何展示、过滤或映射通道记录，但不能重新定义 `DiagnosticCode`、`DiagnosticId`、primary record 公共字段、从属 details 结构或 strict caller input 的 blocking 语义。Surface 规则分别由 [原始协议](protocol.md)、[输出模式](output.md)、[CLI](cli.md) 和 [适配器契约](adapter-contract.md) 承接。

Protocol schema、readable schema 和示例可以消费错误通道导出的规则投影做校验；它们不是错误规则或警告规则的来源。

## 验证

修改错误通道或相关 surface 时，验证重点是语义是否还成立：

- `DiagnosticCode` 是错误和警告的机械身份来源，错误规则、警告规则和 primary `DiagnosticRecord` 字段规则从 code 规则集合派生。
- `DiagnosticId` 只在单次命令或请求内用于内部关联，不作为 public、持久化或顺序语义。
- 显式 caller input、present invalid config source 和 selected adapter typed-field validation 失败在 owning boundary 形成 blocking diagnostic。
- Public failure surface 只投影一个 primary record；相关 field/config/typed/candidate failures 保持从属 details。
- Automatic discovery 候选失败只在全部候选失败时进入 primary failure details；候选成功时不进入 success output。
- 受影响的 protocol、readable、adapter output、stderr 和 exit behavior 与对应 owner 文档、schema、examples、fixtures 和 consumer tests 保持一致。
