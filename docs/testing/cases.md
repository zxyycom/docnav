# 测试用例编号账本

本文记录可审计的测试用例编号、证明目标和源码 `@case` 标记。它不改变现有 smoke task id、测试名称或测试执行语义；现有 JavaScript smoke 报告仍使用 `CORE-*` 和 `MD-*` 任务编号。

## 编号规则

测试用例编号使用 `类别-责任域-证明意图-NNN`：

1. `BB`: 黑盒测试，从真实入口观察用户链路、进程边界或输出边界。
2. `WB`: 白盒测试，从 owner 边界、函数、fixture 或 conformance 入口证明内部语义。
3. `AUX`: 辅助脚本语义守卫，只证明测试、验证或调度语义不会静默漂移。

责任域当前使用 `CORE`、`MD`、`PROTO`、`READABLE`、`SDK`、`WORKSPACE`、`SMOKE`、`PARALLEL`、`QUALITY`、`RELEASE`。新增责任域时先更新本文，再补源码 `@case` 标记。

## 维护与验收

新增或调整 case 时：

1. 在本文新增或更新一个 `### CASE-ID ...` entry，并填写 `Code:` 和 `Proves:`。
2. 在负责该测试语义的源码位置添加 `@case CASE-ID` 标记；黑盒 smoke case 保持现有 smoke task id，不为审计编号重命名。
3. 只登记已有源码标记支撑的 case；需要新增测试语义时，先按 [测试策略](../testing.md) 选择测试层级。
4. 运行 `pnpm run validate:docs cases`，确保本文测试用例编号与源码标记双向一致。

## Black-box Cases

### BB-CORE-LINK-001 Core 原样传递真实 Markdown ref
Existing smoke task: `CORE-LINK-001`
Code: `test/smoke/core/cases/real-markdown.ts`

Proves:
- 真实 `docnav` 进程可以通过 Markdown adapter 完成 `outline -> ref -> read`、`find -> ref -> read` 和 `info` 链路。
- Core 不解析 adapter ref，用户可见 readable JSON 不包含 protocol envelope。

### BB-CORE-REF-001 Adapter ref 错误穿过 Core
Existing smoke task: `CORE-REF-001`
Code: `test/smoke/core/cases/real-markdown.ts`

Proves:
- 被选中 adapter 拒绝的 ref 会从 core 返回稳定 protocol failure。
- `protocol-json` 承载错误时，stderr 不输出 JSON payload。

### BB-CORE-OUTPUT-001 Core 文档输出模式不混层
Existing smoke task: `CORE-OUTPUT-001`
Code: `test/smoke/core/cases/outputs.ts`

Proves:
- `readable-json`、显式/默认 `readable-view` 和 `protocol-json` 通过各自包装表达同一文档结果。
- `readable-view` warning 保留在 stdout，并保持 readable warning schema 有效。

### BB-CORE-ARGS-001 Core 拒绝缺失的 operation 参数
Existing smoke task: `CORE-ARGS-001`
Code: `test/smoke/core/cases/cli-args.ts`

Proves:
- document command 缺少本 operation 拥有的必需参数时返回稳定 input failure。
- 该 smoke case 代表这一类外部 CLI 错误，不枚举所有 parser 组合。

### BB-CORE-CONFIG-001 配置优先级和 path context 可观察
Existing smoke task: `CORE-CONFIG-001`
Code: `test/smoke/core/cases/config-management.ts`

Proves:
- 真实 CLI 边界按文档优先级解析 user、project 和 default config。
- `config list --path` 会报告被选中文档路径对应的 adapter 和 defaults context。

### BB-CORE-SELECT-001 显式 adapter 失败后 fallback 并报告 warning
Existing smoke task: `CORE-SELECT-001`
Code: `test/smoke/core/cases/adapter-selection.ts`

Proves:
- 显式选择的 adapter 失败时不会隐藏 registry fallback。
- `readable-json` 结果携带被拒绝 adapter 的 candidate warning evidence。

### BB-CORE-FAIL-001 Candidate 进程失败保留为发现阶段证据
Existing smoke task: `CORE-FAIL-001`
Code: `test/smoke/core/cases/failures.ts`

Proves:
- candidate discovery 阶段的进程失败被报告为 `FORMAT_UNKNOWN` evidence。
- candidate failure 不会被折叠成 selected adapter invoke failure。

### BB-CORE-INVOKE-001 已选 adapter 进程失败映射为 invoke failure
Existing smoke task: `CORE-INVOKE-001`
Code: `test/smoke/core/cases/failures.ts`

Proves:
- adapter selection 之后的进程失败映射为 `ADAPTER_INVOKE_FAILED`。
- selected invoke failure 与 format discovery failure 保持阶段区分。

### BB-CORE-TOOLS-001 Core 非 document 命令保持可用
Existing smoke task: `CORE-TOOLS-001`
Code: `test/smoke/core/cases/config-management.ts`

Proves:
- `init`、`version`、`doctor` 和 document help 能通过真实 CLI 执行。
- 非 document 命令在 smoke 层保持预期输出和退出行为。

### BB-MD-LINK-001 Markdown 直接 CLI 保持文档链路
Existing smoke task: `MD-LINK-001`
Code: `test/smoke/markdown/cases/outputs.ts`

Proves:
- `docnav-markdown` 在真实进程边界完成 `outline -> ref -> read`、`find -> ref -> read` 和 `info`。
- 直接 CLI 的 `readable-json` 暴露 ref 和 content，不泄漏 protocol envelope。

### BB-MD-OUTPUT-001 Markdown 直接 CLI 输出模式分层
Existing smoke task: `MD-OUTPUT-001`
Code: `test/smoke/markdown/cases/outputs.ts`

Proves:
- 直接 `readable-json`、显式/默认 `readable-view` 和 `protocol-json` read 输出通过不同包装表达等价文档内容。
- 直接 adapter CLI 不把 protocol envelope 字段泄漏到 readable output。

### BB-MD-MACHINE-001 直接 machine 命令保持协议形状
Existing smoke task: `MD-MACHINE-001`
Code: `test/smoke/markdown/cases/machine-commands.ts`

Proves:
- 直接 `manifest`、`probe` 和 valid `invoke` 输出保持 machine-readable 且 schema-valid。
- machine command path 不经过 `readable-view` 包装。

### BB-MD-CORPUS-001 Unicode corpus 分页可重组
Existing smoke task: `MD-CORPUS-001`
Code: `test/smoke/markdown/cases/corpus.ts`

Proves:
- Unicode outline/read 输出在进程边界保持有效。
- 分页 read 可以按 page 继续读取并重组，且不丢失内容。

### BB-MD-ARGS-001 Markdown 直接 CLI 拒绝缺失 operation 参数
Existing smoke task: `MD-ARGS-001`
Code: `test/smoke/markdown/cases/cli-args.ts`

Proves:
- operation-owned 必需参数缺失时，直接 adapter CLI 返回稳定 input failure。
- 该 smoke case 代表这一类外部参数错误，不扩展成 token 组合矩阵。

### BB-MD-WARN-001 Markdown 兼容 warning 保持可观察
Existing smoke task: `MD-WARN-001`
Code: `test/smoke/markdown/cases/cli-args.ts`

Proves:
- document help、`readable-json` warning placement、unused native flag warning 和 `protocol-json` stderr warning 保持区分。
- compatibility warning 不会静默改变命令成功/失败语义。

### BB-MD-ERROR-001 Markdown ref 错误跨输出模式一致映射
Existing smoke task: `MD-ERROR-001`
Code: `test/smoke/markdown/cases/operation-errors.ts`

Proves:
- 同一个 invalid ref 在 `readable-json` 和 `protocol-json` 直接 CLI 输出中一致映射。
- ref error shape 在 adapter process boundary 保持稳定。

### BB-MD-INVOKE-001 Malformed invoke stdin 返回 protocol failure
Existing smoke task: `MD-INVOKE-001`
Code: `test/smoke/markdown/cases/invoke-errors.ts`

Proves:
- malformed `invoke` stdin 返回稳定 protocol error envelope。
- 直接 adapter 进程把 invoke 错误保留在 protocol path，而不是暴露 raw parser failure。

## White-box Cases

### WB-CORE-OUTPUT-001 Core 输出编排保持通道边界
Code: `crates/docnav/src/output.rs`

Proves:
- Core output assembly 分离 protocol JSON、readable JSON、readable view、stdout、stderr 和 exit code 职责。
- 内部编排覆盖 core 文档输出 smoke 中观察到的分支。

### WB-CORE-ARGS-001 Core parser 保持 operation 参数所有权
Code: `crates/docnav/src/cli/parser.rs`

Proves:
- Core argument parsing 不会让 operation-owned 参数被 global compatibility 逻辑静默消费。
- unknown 或 unused token 不能遮蔽 document command 的必需输入。

### WB-CORE-ADAPTER-001 Core 检测 adapter contract 漂移
Code: `crates/docnav/src/contract.rs`

Proves:
- Core 区分 adapter discovery、selection、invoke process 和 malformed adapter output 边界。
- 内部 contract 检查支撑外部 adapter selection 和 invoke failure smoke 证明。

### WB-MD-REF-001 Markdown 重复标题生成唯一可读 ref
Code: `crates/docnav-markdown/tests/adapter.rs`

Proves:
- 重复 heading path 会生成唯一 ref，且每个 ref 都能读取目标 section。
- Markdown ref generation 和 read lookup 仍由 adapter 拥有。

### WB-MD-LINK-001 Markdown outline ref 可通过 read roundtrip
Code: `crates/docnav-markdown/src/markdown.rs`

Proves:
- Markdown navigation 生成的 outline entry ref 可以直接传给 read。
- 本地 parser/formatter 路径支撑直接 CLI 中观察到的 ref handoff。

### WB-MD-REF-002 Markdown ref 错误区分 invalid 和 not-found
Code: `crates/docnav-markdown/tests/adapter.rs`

Proves:
- non-canonical ref 失败为 `REF_INVALID`。
- canonical 但没有匹配 section 的 ref 失败为 `REF_NOT_FOUND`。

### WB-MD-PAGE-001 Markdown read 分页按 Unicode 字符计数
Code: `crates/docnav-markdown/tests/adapter.rs`

Proves:
- Markdown read pagination 按 Unicode 字符计数，不拆分字符。
- page 前进和结束状态可通过返回的 page metadata 观察。

### WB-PROTO-DECODE-001 Protocol request decode 按阶段失败
Code: `crates/docnav-protocol/src/tests.rs`

Proves:
- Protocol request decoding 先运行 schema validation，再进入 typed deserialization。
- schema-invalid、typed-invalid 和 semantic-invalid request 保持可区分。

### WB-PROTO-SCHEMA-001 Protocol fixtures 可解析为共享类型
Code: `crates/docnav-protocol/src/tests.rs`

Proves:
- 已文档化的 protocol fixtures 仍能 deserialize 为共享 protocol types。
- schema/example 材料被实现测试消费，不退化成只存在于 prose 的样例。

### WB-READABLE-VIEW-001 Readable-view conformance vectors 被测试消费
Code: `crates/docnav-readable/tests/conformance_tests.rs`

Proves:
- readable-view conformance fixture set 被测试执行，而不是只在文档中列出。
- renderer framing、block extraction、warning placement 和 extension-field case 由 fixture-driven assertions 覆盖。

### WB-SDK-PAGE-001 共享 adapter paging 一致按字符计数
Code: `crates/docnav-adapter-sdk/src/paging.rs`

Proves:
- SDK paging helper 使用 character count，不使用 byte slice 截断。
- adapter pagination helper 支撑用户可见 continuation 行为。

### WB-SDK-DIRECT-ARGS-001 Direct adapter argv compatibility 不消费必需输入
Code: `crates/docnav-adapter-sdk/src/direct/args/tests.rs`

Proves:
- direct adapter argv compatibility 保持 operation argument ownership。
- unused 或 future flag 可以产生 warning，但不能静默改变 operation 的 required arguments。

### WB-SDK-MACHINE-001 Adapter machine commands 不被 readable 包装
Code: `crates/docnav-adapter-sdk/src/tests/command.rs`

Proves:
- Adapter machine commands 返回 protocol、manifest 或 probe shape，不经过 `readable-view` wrapping。
- SDK command dispatch 保持 machine command boundary。

### WB-SDK-INVOKE-001 Adapter invoke request handling 保持 protocol 所有权
Code: `crates/docnav-adapter-sdk/src/tests/invoke.rs`

Proves:
- SDK invoke 从 stdin 读取 protocol request，并在 invoke path 返回 protocol response。
- request decoding 和 response wrapping 保留在 adapter boundary，不落入 direct readable CLI output。

## Auxiliary Script Cases

### AUX-WORKSPACE-VERIFY-001 Workspace verifier 保持 required gate 语义
Code: `scripts/tools/verify-docnav-workspace.test.ts`

Proves:
- required 和 full verifier profile 保持区分。
- profile membership、check label、arguments、dependencies、mutex 和 output filtering 的变化会被测试暴露。

### AUX-SMOKE-HARNESS-001 Smoke harness 正确记录 task 语义
Code: `test/tools/smoke-harness.test.ts`

Proves:
- independent smoke tasks 可以并发运行，同时 command count 按 report 隔离。
- failed task 和 nested task group 保持预期 audit result shape。

### AUX-PARALLEL-RUNNER-001 Parallel task runner 保持调度契约
Code: `scripts/tools/parallel-task-runner.test.ts`

Proves:
- task normalization、concurrency、mutex serialization、dependency ordering 和 nested task expansion 保持稳定。

### AUX-QUALITY-PARSER-001 Quality tool parsers 保持 fixture 语义
Code: `scripts/tools/quality/tools.test.ts`

Proves:
- quality scan wrapper 仍能解析预期的 scc、Lizard 和 PMD CPD output shape。

### AUX-RELEASE-ARGS-001 Release package 参数解析保持边界
Code: `scripts/tools/release-package/args.test.ts`

Proves:
- release package selector 区分 host package default、target triple、manifest path 和 ambiguous selector。
