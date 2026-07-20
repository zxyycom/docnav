本 change 的目标是新增一个直接链接 ast-grep Rust crates 的多语言代码 adapter，通过 `outline -> ref -> read` 提供有限、可继续的代码结构化阅读。

本文是仅位于 `openspec/changes/add-ast-grep-code-adapter/` 的未审核临时 `code-adapter` delta spec，不修改或替代现有主规范、其它文档或其它 change。

## ADDED Requirements

### Requirement: Linked multi-format code adapter

`docnav-code` MUST 以 adapter id `docnav-code` 暴露一个 registry-facing `AdapterDefinition` factory，并由 core static registry 链接进同一个 `docnav` executable。Manifest MUST 声明以下 format、extension 和 content type mapping：

- `rust`：`.rs`，`text/x-rust`
- `typescript`：`.ts`、`.mts`、`.cts`，`text/typescript`
- `tsx`：`.tsx`，`text/typescript`
- `javascript`：`.js`、`.jsx`、`.mjs`、`.cjs`，`text/javascript`
- `python`：`.py`、`.pyi`，`text/x-python`

Probe MUST 只依据该 closed extension mapping 返回具体 format；未知 extension MUST 返回 `supported: false`。Definition MUST 实现固定 probe、outline、read、find 和 info strategy interface，且 MUST NOT 声明 caller-configurable 参数。

#### Scenario: Automatic Rust selection

- **WHEN** navigation 对 `.rs` 文档执行 automatic adapter discovery
- **THEN** `docnav-code` probe 返回 `supported: true` 和 format `rust`
- **THEN** navigation 选择 static-linked `docnav-code`

#### Scenario: Explicit adapter rejects unsupported extension

- **WHEN** caller 显式选择 `docnav-code` 处理不在 mapping 中的 extension
- **THEN** probe 返回 `supported: false`
- **THEN** 现有 declared-selection error mapping 处理该失败

#### Scenario: Adapter inspection lists one multi-format definition

- **WHEN** 用户运行 `docnav adapter list`
- **THEN** 结果包含 implementation source 为 `core_static` 的 `docnav-code`
- **THEN** 同一个 manifest 列出五个 format descriptors
- **THEN** 发布包不包含独立 code adapter executable

### Requirement: In-process ast-grep ownership boundary

Code adapter MUST 直接使用精确锁定且互相兼容的 `ast-grep-core`、`ast-grep-language` 和 `ast-grep-outline` Rust crates。它 MUST 使用对应 `SupportLang`、`DEFAULT_OUTLINE_RULES` 和 `CombinedExtractors` 在当前进程中解析并提取 outline，MUST NOT 发现、启动或依赖外部 `ast-grep` executable。ast-grep 类型、AST kind、rule id、YAML shape、pattern syntax 和原始依赖错误 MUST NOT 进入 shared protocol、closed operation input 或 code ref。

#### Scenario: Outline without external ast-grep

- **WHEN** package 运行环境无法发现或执行外部 `ast-grep`
- **THEN** 支持格式的 outline 仍由 linked Rust crates 成功执行
- **THEN** operation 不创建 ast-grep subprocess 或解析 ast-grep CLI JSON

#### Scenario: Dependency failure stays private

- **WHEN** 内置 outline rules 无法为所选语言初始化
- **THEN** adapter 返回稳定的 adapter diagnostic
- **THEN** protocol error fields 不暴露 ast-grep Rust type、rule serialization shape 或 backtrace

### Requirement: Deterministic code outline

Code outline MUST 将 ast-grep top-level items 和 direct members 转换为 owned Docnav entries。Top-level items MUST 按 source start、source end、kind、name 确定性排序；每个 item MUST 后接其按 source range 排序的 direct members；相同 ref 的重复提取 MUST 只保留第一项。Import item 的 `kind` MUST 为 `import`；其它 symbol type MUST 由 adapter 映射为稳定 snake_case kind，未知 future category MUST 映射为 `other`，且 raw `ast_kind` MUST NOT 输出。

每个 entry MUST 包含非空 label、完整 code ref 和 one-based line location。`line_start` MUST 是 start byte 所在行；`line_end` MUST 是 exclusive end byte 之前最后一个 byte 所在行，empty range 时 MUST 等于 `line_start`。Label MUST 使用 trim 后的 symbol name；空 name MUST fallback 为 `<kind>@<line_start>`。非空 signature MUST 折叠 whitespace 为单行 summary；summary MUST 最多包含 240 个 Unicode scalar，截断时最后一个 scalar MUST 为 `…`。Item metadata MUST 包含 `role: "item"` 和 `exported`；member metadata MUST 包含 `role: "member"`、`public` 和 item 的 `parent_ref`。Outline MUST 使用现有 limit/page contract 对完整确定性 sequence 分页。

如果没有提取到 symbol，outline MUST 返回唯一的 `kind: "file"` fallback entry，其 range 覆盖整个 source；空文件的 fallback range MUST 为 `0..0`。

#### Scenario: Class or struct with members

- **WHEN** 支持语言文档包含一个带 direct fields 和 methods 的 class、struct、interface 或等价 item
- **THEN** outline 先返回 parent item，再按 source order 返回 direct members
- **THEN** member metadata 的 `parent_ref` 等于 parent entry ref
- **THEN** 每个 ref 都能原样提交给 read

#### Scenario: Stable public mapping

- **WHEN** ast-grep 提取一个带 name、signature、symbol type、source range 和 raw AST kind 的 item
- **THEN** entry 使用 adapter-owned kind、one-based line location 和有界 summary
- **THEN** result 不包含 raw AST kind、rule id 或借用型 ast-grep object

#### Scenario: Empty or symbol-free source

- **WHEN** 一个支持格式的文档为空或没有内置 rules 可提取的 symbol
- **THEN** outline 返回唯一 file fallback entry
- **THEN** 该 entry ref 可由 read 读取完整 source，包括空 content

#### Scenario: Recoverable syntax error

- **WHEN** tree-sitter 能为包含语法错误的不完整源码生成 partial tree
- **THEN** outline 返回仍可确定提取的 symbols
- **THEN** 如果没有 symbol 则返回 file fallback，而不是伪造语义节点

### Requirement: Code ref grammar and snapshot validation

Code adapter MUST 生成和解析非空 ref `code:v1:<format>:<start-byte>:<end-byte>:<sha256>`。`format` MUST 是 code adapter 支持的 format id；offset MUST 是 `0` 或首位非零的十进制 UTF-8 byte offset，并满足 `start <= end`；`sha256` MUST 是所选原始 source bytes 的 64 位小写十六进制 SHA-256。

缺少或错误的 prefix/version/field、未知 format、非法数字、非 canonical 数字或非法 hash MUST 返回 `REF_INVALID`。Grammar 合法但 format 与当前文档不一致、range 越界、offset 不是 UTF-8 boundary 或摘要与当前 source slice 不一致 MUST 返回 `REF_NOT_FOUND`。该 grammar MUST NOT 产生 `REF_AMBIGUOUS`，也 MUST NOT 承诺 ref 在源码修改后继续有效。

#### Scenario: Unchanged-source roundtrip

- **WHEN** outline 为一个 Unicode source symbol 生成 code ref
- **AND** caller 使用相同 path 和未修改 source 将该 ref 原样提交给 read
- **THEN** adapter 验证 byte boundaries 和 SHA-256
- **THEN** read 选择与 outline 相同的 source region

#### Scenario: Malformed code ref

- **WHEN** read 收到带非 canonical offset 或非 64 位小写十六进制 digest 的 code ref
- **THEN** adapter 返回 `REF_INVALID`

#### Scenario: Stale code ref

- **WHEN** code ref grammar 合法但所选 source bytes 已发生变化
- **THEN** digest validation 失败
- **THEN** adapter 返回 `REF_NOT_FOUND` 而不读取同一旧 range 中的新内容

### Requirement: Exact source read

Code read MUST 返回 ref 选中 range 的原始 UTF-8 source slice，不得格式化、去除注释、补充 surrounding context 或重建 AST text。Read result MUST 保留输入 ref，使用当前 format 的 manifest content type，对分页前完整 slice 计算支持的 text cost，并按现有 Unicode-safe text pagination 返回 content 和下一页。

#### Scenario: Read a symbol body

- **WHEN** caller read 一个 function、class、struct 或 member 的 outline ref
- **THEN** content 等于该 ref byte range 对应的原始 source
- **THEN** whitespace、comments 和字符串内容保持不变
- **THEN** content type 与 probe 选择的 format mapping 一致

#### Scenario: Paginated Unicode source

- **WHEN** 选中 source region 超过 limit 且包含多字节 Unicode 字符
- **THEN** read 在 Unicode 字符边界分页
- **THEN** cost 描述分页前的完整 source region
- **THEN** page 可继续直到 region 读取完成

### Requirement: Literal symbol find

Code find MUST 拒绝空 query，并在 outline 的完整 normalized symbol sequence 上对原始 symbol name 和 signature 执行大小写敏感 literal search。一个 symbol 无论 name 和 signature 命中多少次都 MUST 最多返回一次；matches MUST 保持 outline 顺序，并 MUST 复用 outline entry 的 ref、kind、location、summary 和 metadata。Pattern-like 字符 MUST 按普通文本处理，find MUST NOT 解释 ast-grep pattern syntax。

#### Scenario: Find by symbol name

- **WHEN** query literal 命中多个 symbol name
- **THEN** find 按 outline 顺序返回每个 symbol 一次
- **THEN** 每个 match ref 都能原样传给 read

#### Scenario: Find by signature

- **WHEN** query 未命中 label 但命中某个 symbol 的原始 signature
- **THEN** find 返回该 symbol 的 normalized entry
- **THEN** 输出 summary 仍遵守包含结尾 `…` 在内的 240 Unicode scalar 上限

#### Scenario: Pattern syntax remains literal

- **WHEN** query 包含 `$A`、`$$$ARGS` 或其它 ast-grep pattern-like text
- **THEN** find 只执行 literal comparison
- **THEN** adapter 不编译或执行 caller-provided ast-grep rule

### Requirement: Code document info

Code info MUST 返回当前 format 的 content type、UTF-8 encoding、原文件 byte size、adapter id `docnav-code` 和 probe 选择的 format id。Metadata 中 `symbol_count` MUST 是去重后的 normalized symbol 数量且不包含 file fallback；`outline_entry_count` MUST 是分页前实际 deterministic outline sequence 的 entry 数量并包含 fallback。Info MUST NOT 暴露 ast-grep version、AST kind、rule id、parse tree 或 dependency debug output。

#### Scenario: Info for supported source

- **WHEN** info 针对支持格式的 UTF-8 source 执行
- **THEN** document facts、adapter id 和 format 与 probe/read 一致
- **THEN** symbol count 与同一 source 的未分页 normalized extraction 一致

#### Scenario: Info for symbol-free source

- **WHEN** info 针对没有可提取 symbol 的 source 执行
- **THEN** `symbol_count` 为 `0`
- **THEN** `outline_entry_count` 为 `1`，对应 file fallback

### Requirement: Compatibility and verification ownership

Code adapter MUST 使用现有 closed operation inputs 和 operation result types，MUST NOT 新增 CLI、env、config 或 protocol input，也 MUST NOT 修改共享 ref pass-through、protocol envelope 或 readable/protocol output wrapper。Code adapter 主文档、fixtures、protocol examples、adapter tests、case ledger、coverage mapping、core CLI smoke 和 release package smoke MUST 覆盖每个首批 SupportLang mapping、deterministic outline、members、fallback、ref roundtrip/staleness、find、info、Unicode pagination、automatic/explicit selection 和 linked binary behavior。

#### Scenario: Existing consumers remain compatible

- **WHEN** code adapter 被加入 static registry
- **THEN** 现有 Markdown、JSON（若已合并）和 shared protocol consumer 继续使用原字段、ref opacity、output mode 和 error mapping
- **THEN** code adapter 不要求 consumer 识别 ast-grep 类型或 code ref grammar

#### Scenario: Cross-layer verification

- **WHEN** code adapter owner checks 和 workspace verification 运行
- **THEN** adapter tests 证明格式语义和依赖边界
- **THEN** core/release tests 证明 selection、protocol projection 和同一 binary linked behavior
- **THEN** 测试不依赖外部 ast-grep executable
