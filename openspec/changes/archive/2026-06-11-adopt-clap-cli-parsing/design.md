**核心决策：** 本 change 覆盖的 Rust CLI argv 结构由 `clap` 承载，传输层解析成功后的文档操作请求进入 canonical document operation input 或等价 semantic request。CLI argv 只对当前 operation 不使用的额外输入保持宽松。Adapter `invoke` JSON 在归一前保持严格。阅读层 warning envelope 迁移是本 change 的一等契约变更。

## 背景

Docnav 有多个入口会表达同一组文档操作：

- 核心 `docnav` CLI 负责路由、配置、adapter 选择和输出映射。
- Adapter direct CLI 暴露 adapter 操作，并映射格式原生参数。
- Adapter `invoke` 从 stdin 接收 JSON，并输出 protocol-shaped payload。
- `docnav-mcp` 把 MCP tool call 映射到核心 `docnav` CLI。

当前 Rust CLI parser 是手写 token loop。它们同时处理 argv 解析、宽松兼容、warning 构造和 request 构造。这会让 parser 变更风险偏高，也让测试依赖 exact token 分组等实现细节。

`clap` 可以稳定声明命令、option、默认值、枚举和 help。Docnav 仍需要自有的宽松 argv 行为，因为 `clap` 默认解析比“AI 友好成功路径”更严格。本 change 不能把 unused known 参数变成 operation 确定前的 eager typed failure。

## 目标

- 使用 `clap` 作为本 change 覆盖的核心 `docnav` document CLI、已存在 core non-document 命令、adapter direct CLI 和 help 入口的首选 argv 结构解析基础。
- 将固定命令、子命令、共享 flag、默认值、枚举和 help 文本迁移到声明式 CLI 结构。
- 将成功解析的 core document CLI argv、adapter direct document CLI argv 和 adapter `invoke` JSON 映射为 canonical document operation input 或等价 semantic request。
- 文档操作共享语义归一、默认值处理、native option 映射、warning metadata 收集、request 构造和 operation 执行。
- 当前 operation 的必需语义和实际使用参数有效时，CLI argv 中的未知 flag、多余 positional 和未使用参数不阻断成功。
- 保留严格失败：malformed JSON、schema/type/field 错误、缺少必需文档语义、实际使用参数值非法、文档/ref/格式错误和 adapter/protocol 错误。
- 将 readable warning envelope 作为独立契约迁移目标：稳定 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象。
- 固定当前 warning family marker：`cli_argv_ignored` 和 `adapter_candidate_failure`。
- 保持 adapter `invoke` 在进入 canonical document operation input 或等价 semantic request 前严格。
- 为 core CLI 和 adapter direct CLI 提供可审计 help。

## 非目标

- 不改变 `outline -> ref -> read` 行为。
- 不改变 adapter 生成和解析 ref 的 ownership。
- 不给 protocol response、manifest 或 probe schema 增加 CLI warning 字段。
- 不改变 readable operation 字段集合；只收紧 warning item envelope 约束。
- 不把格式解析移入核心 `docnav`。
- 不让 adapter `invoke` request 或 MCP tool arguments 像 argv 一样宽松。
- 不让 `docnav-mcp` 拥有 adapter SDK 解析、adapter `invoke`、格式解析或 Rust argv 解析。
- 不把 CLI warning 的 exact token 分组、`reason` 文案或 token 消费顺序作为稳定契约。
- 不新增、迁移或验收核心 `docnav adapter list/install/update/remove` 管理命令；adapter 管理命令继续由 `implement-docnav-adapter-management` change 拥有。

## 决策

1. **`clap` 承载 argv 结构，Docnav 承载兼容语义。**

   固定命令、子命令、已知 option、默认值、枚举和 help 使用 `clap` 或 `clap` builder API 声明。实现必须先确定 command/operation，再对当前 operation 实际使用的参数做类型、范围和枚举校验。未知和无关 argv 通过 builder 配置、外部参数捕获、预处理或后处理做受控收集。

   推荐实现姿态是：`clap` 负责 argv 结构声明、命令识别和 help；Docnav 在 operation-first 归一层只解析当前 operation 使用的参数值。unknown、extra positional 和 unused known 参数保留为原始 token 并转换为 warning metadata，不在收集层复制业务参数解释、默认值归一或 request 构造逻辑。这样可以降低 parser 审计成本，同时保留 Docnav 的 AI 友好成功路径。继续手写 parser 会保留旧细节但延续高维护成本；完全依赖 `clap` 默认严格行为会拒绝包含无害额外输入的有效调用。

2. **传输解析和文档操作语义分层。**

   Core CLI argv、adapter direct CLI argv 和 adapter `invoke` JSON 先按各自传输规则解析。传输成功后，文档操作进入 canonical document operation input 或等价 semantic request，并共享语义归一、校验、warning metadata 和 operation handler。

   canonical document operation input 或等价 semantic request 是 Rust 边界后的内部语义输入模型，不是 protocol envelope、schema 稳定类型或 MCP 传输模型。它不得把 CLI argv warning、来源通道或 native option bridge 细节提升为 `docnav-protocol` 的公共契约。核心 `docnav` 和 adapter SDK 可以各自实现等价内部类型；验收关注共享语义、request 构造和 operation handler 复用，不要求共享同一个 Rust 类型。

   `docnav-mcp` 不进入这个 Rust/SDK 输入模型。它只把 MCP tool arguments 映射为核心 `docnav` CLI 调用，并验证 MCP 输出包装。

3. **本 change 覆盖的 command family 继承同一模型。**

   本 change 只有在核心 `docnav` document operations、已存在 core non-document 命令、`docnav-markdown`、adapter SDK direct CLI 和 help 入口都使用同一分层模型后才算闭环。后续 Rust 文档操作 CLI 必须进入 canonical document operation input 或等价 semantic request。config、init、doctor、version、help、manifest 和 probe 可以是类型化命令，但不进入文档 operation 管道。核心 `docnav adapter list/install/update/remove` 管理命令不属于本 change 的迁移或验收范围，只在命令族矩阵中标明由 `implement-docnav-adapter-management` 拥有。

4. **命令族矩阵是固定同步面。**

   主规范和测试策略必须维护一份命令族矩阵，按 command family 固定以下决策：argv 解析基础、是否进入 canonical document operation input 或等价 semantic request、是否启用宽松 argv、ignored argv 诊断通道、protocol-shaped stdout 边界和 help 是否执行业务。矩阵至少覆盖：

   - Core document operations：`docnav outline/read/find/info`。
   - Core non-document commands：`config/init/doctor/version`。
   - Adapter direct document operations：`docnav-markdown outline/read/find/info` 和后续 adapter direct document commands。
   - Adapter direct machine commands：`manifest/probe/invoke`。
   - Help commands：root help 和子命令 help。
   - MCP bridge：MCP tool call 到核心 `docnav` CLI 的映射。
   - Core adapter management commands：`docnav adapter list/install/update/remove`，矩阵只记录 owner、是否进入文档 operation input、stdout/stderr 边界和本 change 非验收状态。

   矩阵的目标是防止后续 change 重新扩大或收窄命令族边界。新增 Rust CLI command family 或修改 argv 解析、warning、help、protocol-shaped stdout 行为时，必须先更新矩阵和对应测试。

5. **Warning 稳定性以 envelope 为主。**

   每个 readable `warnings[]` item 包含：

   - `id`：稳定 warning family marker。
   - `reason`：非空人类可读诊断。
   - `effect`：稳定影响或结果，例如 `operation_continued`、`candidate_skipped` 或 `diagnostic_only`。
   - `details`：对象，承载 family-specific 字段。

   当前稳定 family 是 `cli_argv_ignored` 和 `adapter_candidate_failure`。CLI argv warning 可以在 `details.tokens` 中列出相关 argv token；adapter candidate warning 保留 `details.adapter_id`、`details.stage`、`details.code` 和可选 `details.preselected`。CLI argv warning 可以改变 exact token 分组、`reason` 文案和消费顺序，不破坏稳定契约。

6. **实际使用参数严格，未使用 CLI 输入宽松。**

   当前 operation 使用的已知参数缺值、类型非法、范围非法或枚举非法时必须失败。CLI argv 中的未知输入或当前 operation 不使用的参数，只要必需语义有效，就不阻断成功；unused known 参数的值不按其它 operation 的类型、范围或枚举规则校验，而是作为原始 token 进入 warning metadata。每个被忽略的 argv family 必须形成阅读层 warning 或 stderr 诊断。CLI argv warning 的 exact token 分组不稳定，但调用方必须能知道有输入被忽略以及本次 operation 已继续执行。

7. **严格 protocol 传输不等于第二套业务管道。**

   Malformed JSON、未知字段、缺失字段和类型错误在 adapter `invoke` 传输层失败，不进入 canonical document operation input 或等价 semantic request。Schema-valid invoke request 必须使用与 direct CLI 相同的语义归一和 operation handler 路径。

8. **Protocol-shaped stdout 不承载 CLI warning。**

   `protocol-json`、manifest、probe 和 adapter `invoke` stdout 只输出对应 schema payload。CLI warning 进入 stderr，或进入 text、readable-json、MCP structuredContent 等阅读层输出。

9. **Help 是验收面。**

   `docnav --help`、core 子命令 help、`docnav-markdown --help` 和 adapter direct 子命令 help 应列出命令、关键参数、默认值或可选值。Help 命令不读取文档、不选择 adapter、不运行 adapter invoke，也不执行导航操作。

## 风险与取舍

- `clap` 默认可能拒绝 unknown argv，或在 operation 确定前拒绝 unused known 参数的值。缓解：采用 operation-first parse/decode，先确定 command/operation 和 token 归属，再只校验当前 operation 实际使用的参数；测试 unknown 位于 path 前后、unknown 位于 `--output` 前，以及 `info --page nope`、`info --limit-chars nope` 等 unused known 参数场景。
- Invoke strict 边界可能被误解为独立业务路径。缓解：文档和测试说明 strict 只发生在传输校验阶段，有效 request 仍共享 canonical document operation input 或等价 semantic request。
- MCP ownership 可能在实现中被写宽。缓解：`docnav-mcp` 测试只关注 MCP-to-core-CLI 映射和 TextContent/structuredContent 包装。
- Warning family 细节可能与稳定 envelope 混淆。缓解：schema 和测试要求 `id`、`reason`、`effect` 和 `details`，但不要求 CLI token 分组或文案。
- 命令族边界可能在后续 change 中漂移。缓解：把命令族矩阵写入主规范和测试策略，新增或修改 command family 时先更新矩阵。
- 宽松 argv 可能掩盖实际使用参数错误。缓解：当前 operation 使用的 `path`、`ref`、`query`、`page`、`limit_chars`、`output` 和 native options 保持严格。
- 动态 adapter native options 可能不适合 derive。缓解：固定共享参数用 derive 或 builder，native options 通过集中 bridge 进入 protocol `arguments.options`。
- `clap` 增加依赖和编译面。缓解：使用 workspace dependency，只启用必要 features。

## 迁移计划

1. 更新 OpenSpec deltas 和主文档，确立 `clap`、canonical document operation input 或等价 semantic request、宽松 argv、strict invoke、stable warning envelope、MCP ownership 和 help 验收。
2. 添加最小 `clap` workspace dependency/features。
3. 定义或收敛 canonical document operation input 或等价 semantic request，覆盖 operation、path/ref/query/page/limit_chars/output、adapter/native options、来源通道和 warning metadata；参数值校验必须按当前 operation 实际使用集合执行，同时保持它是边界后的内部语义模型，不进入 `docnav-protocol` 公共契约。
4. 在主规范和测试策略中固定命令族矩阵，保持 config、init、doctor、version、manifest、probe、invoke 和 help 在文档 operation input 之外；核心 adapter 管理命令只记录 owner 和边界，不进入本 change 实现或验收范围。
5. 将核心 `docnav` CLI 迁移为类型化 `clap` 命令，并把 document operations 映射到共享 semantic request 路径。
6. 将 adapter SDK direct CLI 迁移为类型化 `clap` 命令，并让有效 direct CLI 与有效 invoke request 共享语义归一。
7. 更新 `docnav-markdown` native option 映射，使其进入 canonical document operation input 或等价 semantic request，并进入 protocol `arguments.options`。
8. 更新 Rust 测试、CLI smoke、Markdown matrix、schema 验证和 workspace 验证。

## 已收敛问题

- 已支持 readable warning 的输出模式继续保留顶层 `warnings` 数组。
- 每个 warning item 使用稳定 envelope：`id`、`reason`、`effect` 和 `details`。
- 当前稳定 warning family marker 是 `cli_argv_ignored` 和 `adapter_candidate_failure`。
- CLI argv warning token 分组、`reason` 文案和 token 消费顺序是实现细节。
- CLI argv 使用 operation-first 校验；unused known 参数以原始 token 进入 warning metadata，不因其它 operation 的类型、范围或枚举规则失败。
- 命令族矩阵是后续 CLI parser、warning、help 和 protocol-shaped stdout 变更的同步面。
- 核心 `docnav`、adapter direct CLI 和后续 Rust CLI argv 入口都在范围内。
- Adapter `invoke` 在进入 canonical document operation input 或等价 semantic request 前拒绝 malformed JSON、未知字段、缺失字段和类型错误。
- 有效 invoke request、有效 direct CLI 文档操作和有效 core CLI 文档操作共享语义归一和 operation handling，或通过等价 semantic request 证明同一语义路径。
- `docnav-mcp` 只作为到核心 `docnav` CLI 的 MCP bridge 和 MCP 输出包装层。
