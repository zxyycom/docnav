# 错误通道

Docnav 的错误通道是运行时发现问题后的统一流向。核心形态是一个进程内、请求内的栈：发现问题的地方只负责把记录压入栈；是否继续、是否失败、如何退出、如何输出，由消费这个栈的调用方决定。

这个文档关注错误通道的长期语义和边界。具体类型字段、函数签名和实现细节以 `docnav-diagnostics` 代码为准。

## 当前语义

错误通道服务于两件事：

- 收集问题：错误、警告、跳过原因、候选失败和无法继续的上下文都进入同一个通道。
- 延迟决策：记录问题不等于立刻失败。发现问题的模块可以继续尝试其它路径；最终 surface 在合适的边界读取通道并决定结果。

通道本身只保存记录。它不判断 operation 成败，不决定 exit code，不组织用户可见文案，也不替任何 surface 选择输出格式。

## 记录与身份

每条通道记录都有一个由栈分配的 `DiagnosticId`。调用方可以保存这个 id，之后按 id 找回对应记录，或以这个 id 为锚点批量取回它之后压入的记录。

记录的机械身份来自 `DiagnosticCode`。同一个 code 可以被不同 surface 投影成 protocol error code、readable warning id、stderr 文案或其它输出字段；反过来，surface 输出字段不能成为通道内部的身份来源。

`DiagnosticCode` 同时承接规则集合。规则描述 code 的类别、默认级别、结构化 details 约束和可投影到哪些 surface。错误规则是这组规则中会投影为错误输出的子集；警告规则是会投影为警告输出的子集。实现可以按类别拆分代码结构，但规则来源仍从 `DiagnosticCode` 出发。

Protocol error 投影消费错误规则，生成 `code`、`details`、`message` 和可选 `guidance`；其中 `code` 和 required `details` 是机器稳定字段，`message` 与 `guidance` 是 surface 文案。Readable error 消费同一错误投影，但使用阅读层包装和精简文案；readable warning 消费警告规则，必须保留稳定 `id`、`effect`、`reason` 和 `details`。这些投影字段不能反向成为 `DiagnosticCode` 或 details 规则的来源。

一条记录至少表达这些语义：

- 发生了什么问题。
- 问题属于 warning、error 还是 fatal 等级别。
- 这个问题对当前流程的影响，例如继续、跳过候选、拒绝输入或中止请求。
- 可机器处理的 details。
- 问题来源，例如 command、adapter、field、path 或处理阶段。

## 栈语义

每个 top-level `docnav` command、adapter direct command 或 adapter `invoke` request 都有自己的错误栈。栈的生命周期不跨进程，也不跨独立请求。

错误栈默认按后进先出读取。调用方需要按插入顺序展示、分组展示或按 surface 规则重排时，在取出记录之后自行反转、过滤或分组。

栈支持三类定位方式：

- 最近记录：按 LIFO 弹出或查看最近压入的问题。
- 阶段标记：在某个阶段开始前打 mark，之后取回该阶段新增的问题。
- 记录锚点：保存某条记录的 id，之后取回该记录之后压入的问题，并可选择是否包含锚点记录本身。

`DiagnosticId` 和 mark 只在创建它们的栈内有效。它们用于同一次命令或请求里的协作，不是跨进程、跨运行或持久化的标识。

## 谁写入通道

Docnav runtime 和 public surface 上发现的问题都进入错误通道，包括核心 document operation、adapter direct CLI、adapter `invoke`、标准参数处理、adapter 选择、配置命令、初始化、doctor、version 和 help 等路径。

写入通道的模块只描述自己看到的问题和影响，不负责替外层组织最终失败。比如一个 adapter 候选失败时可以压入记录并让路由继续尝试其它候选；最后是否展示这些候选失败，由调用方根据结果和输出模式决定。

## 谁读取通道

读取通道的是边界层：CLI、protocol surface、readable output、adapter direct CLI 或 adapter `invoke` handler。

这些 surface 负责把通道记录投影为自己的输出：

- [原始协议](protocol.md) 定义 protocol envelope、schema 和 protocol 错误投影。
- [输出模式](output.md) 定义 readable-view、readable-json、protocol-json 的包装、输出通道和文案。
- [CLI](cli.md) 定义 command surface、非 document 命令行为和 exit behavior。
- [适配器契约](adapter-contract.md) 定义 adapter `manifest`、`probe`、`invoke` 和 direct CLI 的输出行为。

这些文档可以定义如何展示、过滤或映射通道记录，但不重新定义 `DiagnosticCode`、`DiagnosticId`、mark 生命周期或默认 LIFO 语义。

Protocol schema、readable schema 和示例可以消费错误通道导出的规则投影做校验；它们不是错误规则或警告规则的来源。

## 验证

修改错误通道或相关 surface 时，验证重点是语义是否还成立：

- id 只由栈分配，且可用于找回对应记录。
- mark 和记录锚点能准确限定阶段内新增的问题。
- 默认读取顺序是 LIFO；需要插入顺序的 surface 明确反转。
- `DiagnosticCode` 是错误和警告的机械身份来源，错误规则和警告规则从 code 规则集合派生。
- 受影响的 protocol、readable、adapter output、stderr 和 exit behavior 与对应 owner 文档、schema、examples、fixtures 和 consumer tests 保持一致。
