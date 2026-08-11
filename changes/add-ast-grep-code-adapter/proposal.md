# Proposal

本计划在产品方向恢复该工作后，交付一个直接链接 ast-grep Rust crates 的多语言代码 adapter；当前按[复杂代码适配器前先扩展简单文档格式](../../docs/decisions/product-direction/prefer-simple-document-formats-before-code-adapters.md)暂停，而不是退回只有方向的 Draft。

## Why

大型源码也需要通过 `outline -> ref -> read` 做有限、可继续的结构化阅读。现有 adapter contract 可以承载这一能力，但代码解析同时引入多语言 parser、符号语义、依赖体积和替代工具竞争，因此产品顺序明确要求先扩展更简单的文档格式。

## Outcome

产品恢复后，`docnav` 能在同一进程内为 Rust、TypeScript/TSX、JavaScript/JSX 和 Python 源码提供确定性 outline、opaque ref、原文 read、符号 find 和稳定 info；canonical release package 能在没有外部 ast-grep executable 的环境中证明这些行为。

## Scope

- 纳入：一个 linked `docnav-code` adapter、五个 format identity、adapter-private ast-grep 模型、byte-range ref、现有四个 document operations、static registry、owner 文档、测试和 release 验证。
- 不纳入：外部 `ast-grep` CLI、调用方规则或 parser plugin、跨文件索引、定义/引用、调用图、类型推断、edit-stable ref、新 protocol 字段、新 output mode 或通用 parser engine abstraction。
- 当前暂停只阻止实施，不撤销已经形成的计划。恢复时先执行产品排序确认和 Current 重新基线任务。

## Success Criteria

- 五种 format 都能从 automatic 或 explicit adapter selection 完成 `outline -> ref -> read`，并保持 ref 原样传递和稳定错误分类。
- outline、find、read 和 info 的 adapter-owned 映射具有代表性 fixtures、分页、Unicode、空文件、无符号文件和可恢复语法错误证据。
- ast-grep 类型、rule shape 和错误不越过 adapter 私有边界，release package 不依赖外部 executable。
- owner 文档、语义 Case、protocol/readable 验证、Linux/Windows package smoke 和 workspace 验证一致通过。

## Affected Owners

- [架构](../../docs/architecture.md)、[适配器契约](../../docs/adapter-contract.md)和 [Navigation Input Resolution](../../docs/navigation-input-resolution.md)：实施期间作为 linked adapter、manifest routing、static registry 与 invocation-private document lifecycle 的 Current 基线；行为证据成立后再同步实际新增的 Current surface。
- [Ref](../../docs/ref-contract.md)、[原始协议](../../docs/protocol.md)和[输出模式](../../docs/output.md)：实施期间作为 opaque ref 交接、operation result 与两条输出路径的 Current 基线；行为证据成立后再同步实际新增的 Current surface。
- 本 design 登记新增 code adapter owner以及[契约示例](../../docs/examples/contract-examples.md)、[测试策略](../../docs/testing.md)、[语义测试 Case 维护](../../docs/testing/case-maintenance.md)和[发布包验证](../../docs/testing/release.md)的预期 delta；只有实现与行为证据通过后，才把相应 delta 写成 Current。
