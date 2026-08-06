本 change 的未来目标是新增一个直接链接 ast-grep Rust crates 的多语言代码 adapter，通过 `outline -> ref -> read` 提供有限、可继续的代码结构化阅读。

## 文档状态

- 状态：`product-deferred`，即 code adapter 尚未进入当前实施排序；长期产品方向由 [复杂代码适配器前先扩展简单文档格式](../../../docs/decisions/product-direction/prefer-simple-document-formats-before-code-adapters.md)拥有。
- 当前允许：维护未来 target artifacts、同步已经成为 Current 的共享契约；artifact 完整、routing 可用或实现可行都不构成实施授权。
- 恢复门禁：先明确批准 code adapter 进入当前产品排序，再从届时 Current adapter、routing、ref、output 和 release 基线重新审计；执行顺序见 [tasks](tasks.md)。

## Why

Docnav 当前只能按已有格式结构导航，读取大型源码时仍缺少符号级入口。ast-grep 已提供可直接嵌入 Rust 进程的解析、语言和 outline crates，因此可以复用成熟 parser 与内置规则，同时保持 Docnav 的静态 adapter、opaque ref 和统一输出契约。

## What Changes

- 新增内置 `docnav-code` adapter crate，并由 `docnav` core static registry 链接进同一个发布可执行文件。
- 首批支持 Rust、TypeScript/TSX、JavaScript/JSX 和 Python；一个 adapter definition 通过多个 `formats[]` descriptor 声明这些代码格式。
- 直接依赖并精确锁定兼容版本的 `ast-grep-core`、`ast-grep-language` 和 `ast-grep-outline` Rust crates；代码导航始终在当前 `docnav` 进程内执行。
- 使用 adapter 内部固定的 ast-grep outline 规则提取符号及成员，规范化为现有 Docnav `Entry`，并提供确定性分页、adapter-owned ref、原始源码区域读取、符号查找和基础文档信息。
- 对外继续使用现有 protocol、readable output、closed standard operation input 和共享 ref 契约；ast-grep 类型、AST kind、规则 YAML、pattern syntax 和 crate error 只存在于 adapter 内部。
- 新增代码 adapter 主文档、fixtures、协议示例、测试账本、覆盖映射、core CLI smoke 和 release package smoke。
- 非目标：本 change 不增加调用方自定义 ast-grep rules/patterns，不提供跨文件定义/引用、类型推断、调用图、增量索引或 LSP/SCIP 语义，不新增 CLI/config/env/protocol 参数，也不承诺 ref 跨源码修改保持有效。

## Capabilities

### New Capabilities

- `code-adapter`: 定义 `docnav-code` 的格式识别、ast-grep 进程内集成、outline 映射、ref grammar、read/find/info 行为、错误边界与验证责任。

### Modified Capabilities

- `release-artifacts`: 要求发布包中的同一个 `docnav` executable 包含并验证 code adapter，而不携带或调用独立 ast-grep executable。

## Impact

- 代码：新增 `crates/adapters/code`，并修改 workspace dependency、core static registry、adapter catalog/inspection、相关 fixtures 和验证脚本。
- 公共接口：不新增 protocol 字段、operation、output mode 或用户参数；新增 adapter id `docnav-code`、代码 format descriptors、代码 adapter 私有 ref grammar 和相应可观察导航行为。
- 依赖与制品：发布 binary 静态链接所选语言 parser；实现记录 ast-grep 精确版本、启用的 parser features、license 检查和 release binary size 变化。
- Current JSON 基线：`add-json-adapter` 已归档；实现阶段必须保留现有 JSON linked definition、workspace membership 和 release-smoke 证据，并避免用固定 adapter 数量覆盖现有 registry。
- Routing 基线：本 change 的 target 使用 Current manifest pathname routing；core 在文档 I/O 前按 exact filename 或最长 basename suffix 选择 adapter，matched routing facts 保持私有，选中后的 code adapter 再验证实际 pathname 与文档。恢复时按届时 owner 重新审计，不保留独立 probe contract。
