---
name: api-and-interface-design
description: >-
  设计或审查可观察 public contract：machine-readable protocol、readable output、
  adapter/service contract、opaque ref/identifier、pagination/continuation、schema/example、
  error mapping、CLI/API surface 或 tool/bridge mapping。用于 Docnav contract work
  或其它本地工具接口决策；只有用户明确要求时才扩展到 REST、GraphQL 或 TypeScript interface patterns。
---

# API 与接口设计

## 目标

把接口设计写成稳定、可验证、可兼容演进的 public contract。除非某项行为明确标记为 private，否则把可观察字段、readable output 文案、identifier、错误、分页、schema/example、flag、adapter/service 行为和 tool mapping 都当作 contract。

本 skill 不替代具体产品规范。项目专属 ownership、文档入口和验证材料只在对应 scope 内读取。

Docnav document output mode 当前只包含 `readable-view`、`readable-json` 和 `protocol-json`；非文档 help/version 纯文本通道使用 PlainText 或等价明确名称，不并入 document output mode。

## 读取策略

默认只读本文件。按当前任务加载一层 reference：

1. Docnav public contract、CLI-first navigation、adapter/ref ownership 或项目验证材料：读 [docnav-contract-scope.md](references/docnav-contract-scope.md)。
2. 用户明确要求通用 REST、GraphQL 或 TypeScript interface design：读 [web-interface-patterns.md](references/web-interface-patterns.md)。

## 流程

1. 识别 surface。
   - 判断变更影响 machine output、readable output、CLI/API surface、adapter/service contract、identifier/ref、pagination、schema/example、error mapping 还是 bridge/tool mapping。
   - 明确字段、flag、identifier、page metadata、error、readable label、example 和 validation artifact 中哪些会成为可观察行为。

2. 识别 owner。
   - 命名拥有该行为的层：core、adapter/service、bridge/tool、schema/example、docs 或 consumer-facing wrapper。
   - 调用方只映射或传递 owner 暴露的 contract；不要在外层重建 owner 的解析、路由或 identifier 语义。

3. 定义兼容路径。
   - 优先新增 optional field、enum value、tool 或 output section，再考虑修改现有 contract。
   - 除非有意做 breaking change，否则保持字段含义、nullability、ordering、default、error code、identifier opacity 和 readable label 稳定。
   - Breaking change 不可避免时，先写清 migration behavior、affected consumers 和 validation update，再实现。

4. 设计 mapping。
   - Machine-readable output、readable output 和 bridge/tool result 可以共享业务语义，但各自保留 wrapper、schema、pagination envelope 和稳定性承诺。
   - Identifier/ref 由 owning layer 生成和解析；其它层只做存在性、类型和边界检查，并原样传递。

5. 保持 deterministic。
   - 对 entries、matches、errors、schema/example fields 和 readable labels 保持稳定排序。
   - 同一 input、option 和 page request 的 pagination 必须可复现。
   - 避免暴露 parser internals、host-specific path、不稳定 hash、timestamp 或依赖 timing 的文本。

6. 同步 validation material。
   - Schema、example、fixture、golden output、docs 和 tests 是 interface change 的一部分，不是后续 cleanup。
   - Test 应断言 public surface 和 cross-layer mapping，而不只覆盖 helper internals。

## 边界

- Readable output 也是 contract；label、ordering、truncation hint 和 continuation instruction 都要有意设计。
- Machine output 与 readable output 可以语义一致，但不能混用 wrapper 或 validation promise。
- Bridge/tool layer 应保持 thin mapping；复杂解析和领域判断留在 owning implementation。
- Observable interface change 必须同步更新用于校验或展示该行为的 schema、example、fixture、docs 和 tests。
- Compatibility、pagination、error mapping 和 identifier opacity 属于设计输入，不是实现后的修补项。

## 完成检查

批准 interface code 或 contract artifact 前确认：

- 已命名 affected surface 和 owning layer。
- 除非变更有意 breaking，现有 consumer 仍能使用当前字段、identifier/ref、flag、default、error 和 readable output 文案。
- Machine/readable/tool outputs 保留各自 wrapper，同时保持业务语义一致。
- Identifier/ref 在 owning layer 外仍是 opaque pass-through value。
- 当 observable behavior 变化时，schema、example、fixture、docs 和 tests 已同步更新。
- 验证命令覆盖 public contract 和 cross-layer mapping；命令来自当前仓库脚本、规范或相邻测试，而不是硬编码构建产物路径。
