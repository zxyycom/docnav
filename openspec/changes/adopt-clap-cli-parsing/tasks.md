## 1. 审计门禁

- [x] 1.1 审计 proposal、design、spec deltas 和 tasks 是否共同表达同一核心决策：本 change 覆盖的 Rust CLI argv 结构使用 `clap`；成功的文档操作入口进入 canonical document operation input 或等价 semantic request；CLI argv 宽松；adapter `invoke` JSON 和 schema 严格；阅读层 warning envelope 迁移是一等契约变更。
- [x] 1.2 确认当前提案阶段只包含 `openspec/changes/adopt-clap-cli-parsing/` 下的临时 OpenSpec artifacts；进入实现前不得混入主规范、代码、schema、示例或其它 change 的未审核改动。
- [x] 1.3 确认 spec deltas 不改变 `outline -> ref -> read` 业务语义、protocol envelope、manifest/probe schema、ref 语义或 adapter 格式解析所有权。
- [x] 1.4 确认 warning 决策已收敛为稳定 warning envelope：稳定 `id` family marker、非空 `reason`、稳定 `effect` 和 `details` 对象。
- [x] 1.5 确认当前 warning family marker 至少固定为 `cli_argv_ignored` 和 `adapter_candidate_failure`；CLI argv token 只作为 `details.tokens` 等 family-specific detail 表达；`adapter_candidate_failure` 在 `details` 中保留 `adapter_id`、`stage`、`code` 和可选 `preselected` 等 candidate family 字段。
- [x] 1.6 确认 CLI argv warning 的 exact token 分组、`reason` 文案和 token 消费顺序不再作为长期稳定契约。
- [x] 1.7 确认 MCP ownership 写窄为：`docnav-mcp` 只把 MCP tool call 映射到核心 `docnav` CLI，并包装 TextContent/structuredContent；它不直接调用 adapter SDK、adapter `invoke` 或 Rust CLI argv parser。
- [x] 1.8 确认命令族矩阵已成为本 change 的固定同步面，并且核心 `docnav adapter list/install/update/remove` 管理命令只在矩阵中记录 owner、边界和非验收状态，不进入本 change 的实现或验收范围。
- [x] 1.9 确认 `clap` 迁移不把 unused known 参数变成 eager typed failure：先确定 command/operation，再只对当前 operation 实际使用的参数做类型、范围和枚举校验。

## 2. 主规范、schema 和示例同步

- [x] 2.1 更新 `docs/cli.md`，说明 `clap` 优先、operation-first 参数校验、canonical document operation input 或等价 semantic request、CLI 成功路径优先、当前 operation 使用参数严格、稳定 warning envelope、help 验收和 `protocol-json` stdout 边界。
- [x] 2.2 更新 `docs/adapter-contract.md`，说明 adapter direct CLI 通过 `clap` 和受控桥接实现宽松 argv；adapter `invoke` stdin JSON 仍严格校验；strict decode 成功后的请求进入 canonical document operation input 或等价 semantic request 和统一 operation handler。
- [x] 2.3 更新 `docs/testing.md`，把测试策略从 exact token 断言改为成功路径、必要失败、help 可用、共享语义归一、稳定 warning envelope、candidate warning family details 和 schema 边界验证。
- [x] 2.4 更新 `docs/schemas/readable-common.schema.json`，表达稳定 warning envelope、`cli_argv_ignored`、`adapter_candidate_failure`、稳定 `effect` 和 family-specific `details` 字段约束。
- [x] 2.5 更新 MCP outputSchema 示例和相关验证材料，确保 `docs/examples/json/mcp-*-tool.json` 与 readable schema 的 warning envelope 保持一致。
- [x] 2.6 复查 `docs/protocol.md`、其它 `docs/schemas/`、`docs/examples/` 和 MCP tool declaration 生成逻辑；只在它们已经描述 readable warning 时同步 envelope，不给 protocol response、manifest 或 probe schema 增加 CLI warning 字段。
- [x] 2.7 在 `docs/cli.md` 和 `docs/testing.md` 固定命令族矩阵，至少覆盖 core document operations、core non-document commands、core adapter management commands、adapter direct document operations、adapter direct machine commands、help commands 和 MCP bridge；每个命令族必须标明 owner、是否进入 canonical document operation input 或等价 semantic request、是否启用宽松 argv、ignored argv 诊断通道、protocol-shaped stdout 边界、help 是否执行业务，以及该命令族是否属于本 change 验收范围。
- [x] 2.8 使用局部 diff 确认文档、schema 和示例改动只覆盖 CLI argv 解析、warning envelope、MCP ownership、help、命令族矩阵和测试策略。

## 3. 依赖和 canonical document operation input

- [x] 3.1 为相关 Rust crate 添加 `clap` workspace dependency，并只启用当前实现需要的功能特性。
- [x] 3.2 为核心 `docnav` CLI 定义 `clap` 命令结构，覆盖 document operations、config、init、doctor、version 和 help；非文档命令只产生类型化命令。核心 `docnav adapter list/install/update/remove` 管理命令不由本 change 新增、迁移或验收，只在命令族矩阵中标明 owner 和边界。
- [x] 3.3 为 adapter direct CLI 定义 `clap` 命令结构，覆盖 `manifest`、`probe`、`invoke`、`outline`、`read`、`find`、`info` 和 help；manifest、probe 和 help 不进入文档 operation input。
- [x] 3.4 定义或收敛 canonical document operation input 或等价 semantic request，覆盖 operation、path、ref、query、page、limit_chars、output、adapter/native options、来源通道和 warning metadata；具体类型名由实现决定，但名称不得暗示它覆盖 config 或其它非文档命令；它必须保持为边界后的内部语义模型，不成为 `docnav-protocol` 公共契约。
- [x] 3.5 将核心 `docnav` document CLI argv、adapter direct CLI document argv 和 adapter `invoke` JSON 在各自传输解析成功后映射到 3.4 的文档操作输入或等价 semantic request。
- [x] 3.6 保证 document operation handler 只通过共享归一层接收 3.5 的输入，不从 CLI 或 invoke 入口复制业务参数解释逻辑。
- [x] 3.7 为未知 argv、多余 positional 和当前 operation 不使用的 CLI 参数实现宽松处理，保留相关原始 token 作为 warning metadata，确保必需语义参数正确时不阻断执行。
- [x] 3.8 对当前 operation 实际使用的参数保持严格校验：缺少必需 path/ref/query、非法 page、非法 limit_chars、非法 output 和非法 native option 必须返回输入错误；当前 operation 不使用的 known 参数不得因其它 operation 的类型、范围或枚举规则提前失败。
- [x] 3.9 保留 adapter `invoke` stdin JSON strict schema 校验；malformed JSON、未知字段、缺失字段或类型错误不得进入 canonical document operation input 或等价 semantic request。

## 4. 核心 `docnav` CLI 迁移

- [x] 4.1 替换或包裹 `crates/docnav/src/cli/parser.rs` 及其子模块中的手写 parser，使核心 CLI 通过 `clap` 或 `clap` builder 识别命令结构，并在 operation 确定后只对实际使用参数产生类型化值。
- [x] 4.2 将核心 document operations 映射到 canonical document operation input 或等价 semantic request，并保持项目根解析、path 规范化、adapter 选择、配置合并、invoke request 构造、输出模式分流和稳定错误映射不变。
- [x] 4.3 保持 readable-json warning envelope 与 schema 一致，使用稳定 `cli_argv_ignored` 和 `adapter_candidate_failure` family marker。
- [x] 4.4 移除测试和实现中对 CLI argv warning exact token 分组、`reason` 文案和 token 消费顺序的稳定依赖。
- [x] 4.5 确认 `docnav --help`、document 子命令 help 和 config 子命令 help 可读，并且 help 不执行 adapter 选择或文档导航业务。
- [x] 4.6 确认 `protocol-json` 成功和错误 stdout 只包含 protocol response envelope，CLI warning 或诊断不污染 stdout schema。
- [x] 4.7 确认 core non-document commands `config get/set/unset/list`、`init`、`doctor` 和 `version` 仍按现有语义执行：它们使用类型化 CLI 命令，但不进入 document operation input、adapter routing 或文档导航业务。

## 5. Adapter SDK 和 Markdown direct CLI 迁移

- [x] 5.1 替换或包裹 `crates/docnav-adapter-sdk/src/direct/args.rs` 中的手写 argv parser，使 direct CLI 通过 `clap` 或 `clap` builder 产生类型化 options。
- [x] 5.2 将 adapter direct CLI document operations 映射到 canonical document operation input 或等价 semantic request，并保持 `run_direct_cli` 的 request 构造、输出模式分流、稳定错误映射和 warning 承载边界。
- [x] 5.3 确认 direct CLI 与 schema-valid `invoke` request 在 `execute_operation` 前共享语义归一和 request 构造路径。
- [x] 5.4 调整 `docnav-markdown` CLI 入口，确保 Markdown native options 通过新的解析结构进入 canonical document operation input 或等价 semantic request，并进入 protocol `arguments.options`。
- [x] 5.5 删除或收敛不再需要的 exact token、细粒度 argv warning family 和 token 消费顺序实现分支；保留 schema-compatible 稳定 warning envelope。
- [x] 5.6 确认 `docnav-markdown --help` 和子命令 help 输出可读参数说明，并且不会执行文档导航业务。

## 6. 自动化测试

- [x] 6.1 更新核心 CLI Rust 单元测试，覆盖 `clap` 类型化命令、canonical document operation input 或等价 semantic request 映射、必需参数缺失、已知使用参数非法、unused 参数宽松、operation-first 参数校验和 help。
- [x] 6.2 更新 adapter SDK Rust 单元测试，覆盖 direct CLI 类型化 options、native options、宽松 argv、operation-first 参数校验、必要失败、有效 invoke 与 direct CLI 的共享语义归一、invoke strict 校验，以及 readable-json 存在 warning 时必须输出顶层 `warnings`。
- [x] 6.3 更新 `scripts/docnav-core-cli-smoke`，覆盖 core CLI unknown flag、多余 positional、unused known 参数、unused known 非法值、unknown 位于 path 前、unknown 位于 `--output` 前、`protocol-json` warning stderr 边界、core help 和 core non-document commands 代表性行为。
- [x] 6.4 更新 `scripts/docnav-markdown-cli-smoke` 的 CLI argument matrix，移除 exact token 分组、`reason` 文案和消费顺序断言，保留稳定 warning family `id`、`effect` 和 `details` 字段断言。
- [x] 6.5 增加 Markdown direct CLI 宽松 argv 成功路径：valid path 加 unknown flag、多余 positional、unknown 位于 path 前、unknown 位于 `--output protocol-json` 前、unused known 参数、unused known 非法值和 `--output readable-json` warning 必须性。
- [x] 6.6 增加 help 用例，验证 `docnav --help`、核心子命令 help、`docnav-markdown --help` 或 adapter 子命令 help 暴露命令和关键参数。
- [x] 6.7 保留必要失败用例，覆盖缺 path、缺 ref、缺 query、非法 page、非法 limit_chars、非法 output、非法 max_heading_level、missing file、invalid ref、non-UTF-8 和 malformed invoke JSON。
- [x] 6.8 验证 readable-json 和 MCP structuredContent 通过更新后的 readable schema；存在 warning 时断言稳定 warning envelope、当前 family marker、稳定 effect 和 family-specific details。
- [x] 6.9 验证 adapter candidate failure warning 使用 `id: "adapter_candidate_failure"`，并在 `details` 中保留 `adapter_id`、`stage`、`code` 和可选 `preselected` 等 candidate family 字段。
- [x] 6.10 验证命令族矩阵中的每个本 change 验收命令族至少有文档或测试覆盖；adapter 管理命令族只验证矩阵 owner 和非验收状态。新增或修改 command family 时必须同步矩阵、help 验收和对应 stdout/stderr 边界测试。
- [x] 6.11 验证 `protocol-json`、manifest、probe 和 adapter invoke stdout 不因 CLI warning 增加非 schema 字段，并验证 invoke 传输层错误不会进入 canonical document operation input 或等价 semantic request。
- [x] 6.12 增加 core non-document commands 代表性验收：`config get/set/unset/list` 成功和关键失败、`init` 幂等、`doctor` 成功或失败 exit code、`version` 输出，以及这些命令的 stdout/stderr 边界。

## 7. 验证和收尾

- [x] 7.1 运行相关 Rust 格式化和静态检查。
- [x] 7.2 运行核心 `docnav`、adapter SDK 与 Markdown adapter 的局部测试。
- [x] 7.3 运行 `pnpm run smoke:docnav-core`，确认核心 CLI smoke 通过。
- [x] 7.4 运行 `pnpm run smoke:docnav-markdown`，确认黑盒 CLI smoke 通过。
- [x] 7.5 若实现跨 Rust、文档、OpenSpec、schema、示例或输出层边界，运行 `pnpm run verify:docnav-workspace`。
- [x] 7.6 使用局部 diff 确认实现、文档、schema、示例和测试改动只覆盖本 change 范围。
- [x] 7.7 更新任务勾选状态，并在最终说明中记录验证命令、结果和未解决风险。
