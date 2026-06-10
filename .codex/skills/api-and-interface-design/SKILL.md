---
name: api-and-interface-design
description: "设计或审查 Docnav public contract：raw protocol 字段、readable CLI output、adapter contract、ref、pagination 和 continuation 行为、schema/example、error mapping、CLI surface 或 MCP tool mapping。用于 Docnav API/interface design；除非用户明确要求，不用于通用 REST、GraphQL 或 TypeScript。"
---

# API 与接口设计

## 目的

使用本 skill 保持 Docnav 的 public contract 稳定、确定、并且便于 CLI-first 接入方消费。

除非某项行为被明确标记为 private，否则把每个可观察字段、readable output 行、ref、error code、continuation token、schema/example、CLI flag、adapter 行为和 MCP mapping 都视为 contract。

不要把本 skill 用作通用 API 设计。只有当用户明确要求 REST、GraphQL 或 TypeScript interface patterns 时，才读取 `references/web-interface-patterns.md`。

## 公共契约面

在设计或审查变更前，先识别所有受影响的 contract surface：

- Raw protocol：machine-readable request/response 字段、envelope、error shape、pagination metadata，以及 adapter 生成的 ref。
- Readable CLI output：text mode 的排序、label、信息密度、truncation、continuation hint，以及面向人的 error text。
- CLI surface：顶层 `docnav` command、flag、default、output mode、exit behavior，以及 project/config 行为。
- Adapter contract：格式识别、解析归属、outline/read/find/info 语义、直接 adapter CLI output，以及 adapter-owned ref 的生成和解析。
- Ref：只由所属 adapter 生成和解析的 opaque identifier。
- Pagination 与 continuation：page number、limit、truncation indicator、可继续读取、稳定排序，以及确定的 next-step instruction。
- Schema 与 example：用于校验或展示 contract 的 JSON schema 文件、example fixture、golden output 和文档片段。
- Error mapping：adapter/core error、CLI exit behavior、protocol error code、readable message 和 MCP tool error。
- MCP tool mapping：stdio bridge 的 tool name、input schema、output mapping，以及到 `docnav` 的 pass-through 行为。

## 流程

1. 识别归属层。
   - `docnav` 负责 format routing、adapter management、共享 CLI default、output mode、configuration、project init 和 error mapping。
   - `docnav-mcp` 只把 MCP tool call 映射到 `docnav`；不要复制 parsing、routing 或 adapter logic。
   - 每个 adapter 负责本格式识别、解析、navigation strategy、ref、pagination result，以及直接 adapter CLI behavior。

2. 在改代码前定义可观察 contract。
   - 判断变更影响 raw protocol、readable output、直接 adapter CLI、顶层 `docnav`、MCP，还是全部层面。
   - 明确会变成可观察行为的字段、flag、ref、page metadata、error、text label、example 和 validation artifact。
   - 让 raw protocol 与 readable output 在业务语义上保持一致，同时保留各自独立的 transport/output wrapper。

3. 选择 compatibility 路径。
   - 优先新增 optional field、enum value、tool 或 output section，再考虑修改现有 contract。
   - 除非有意做 breaking change，否则保持字段含义、nullability、ordering、default、error code 和 readable label 稳定。
   - 如果 breaking change 不可避免，先写清 migration behavior 和所有受影响的 validation update，再实现。

4. 设计 surface 之间的 mapping。
   - MCP tool input/output 应映射到 CLI/protocol contract，而不是重新实现 routing 或 parsing。
   - 直接 adapter CLI output 可以由 adapter 拥有，但顶层 `docnav` output 和 error mapping 保持集中。
   - 生成的 ref 由 `docnav`、MCP、schema、example 和 test 原样传递。

5. 保持 output deterministic。
   - 对 outline entry、search match、error 和 schema/example field 保持稳定排序。
   - 同一 document、adapter、option 和 page request 的 pagination 必须可复现。
   - 避免暴露 parser internals、filesystem-specific path、不稳定 hash、timestamp 或依赖 timing 的文本。

6. 随 interface 同步更新 validation material。
   - Schema、example、fixture、golden output 和文档变更属于 interface change 的一部分，不是后续 cleanup。
   - Test 应断言 public surface 和 cross-layer mapping，而不只覆盖 helper internals。

## 兼容性规则

- Ref 在所属 adapter 之外都是 opaque。不要要求 caller、MCP 或顶层 `docnav` 解析生成的 ref 结构。
- Raw protocol、readable CLI output、直接 adapter CLI output 和 MCP tool result 可以共享业务语义，但不共享 transport wrapper。
- MCP 是到 `docnav` 的 bridge；它应把 tool call 映射到 CLI/protocol 行为并 pass through result，而不是重建 adapter routing 或 parsing。
- Output mode 必须保持区分：`protocol-json` 保持 machine-readable，readable JSON 保持便于消费，text 保持高密度的人类可读输出。
- 任何 observable interface change 所校验或展示的 schema 和 example artifact，必须在同一个 work item 中更新。
- Error mapping 应从 adapter/core error 到 protocol error code、CLI exit/readable message、MCP tool error 保持一致。
- Pagination 与 continuation 必须对同一 input 和 option 保持 deterministic、resumable、stable。
- Readable output 也是 contract。label、ordering、truncation hint 和 continuation instruction 都要有意设计。

## 审查清单

批准 interface code 或 contract artifact 前使用此清单：

- 已命名受影响 surface：raw protocol、readable CLI output、CLI surface、adapter contract、ref、pagination、schema/example、error mapping 或 MCP mapping。
- `docnav`、`docnav-mcp`、adapter、schema、example 和 docs 之间的 ownership 清楚。
- 除非变更有意 breaking，现有 consumer 仍能使用当前字段、ref、flag、default、error 和 output text。
- 生成的 ref 在所属 adapter 之外仍是 pass-through value。
- Raw protocol 与 readable output 保留各自 wrapper，同时保持语义一致。
- MCP 映射到 `docnav` 行为，不复制 parsing、routing 或 adapter logic。
- 当 observable behavior 变化时，schema、example、fixture 和 golden output 已同步更新。
- Test 和 validation command 覆盖 public contract 以及任何 cross-layer mapping。

## 验证

选择能证明 changed contract 的最小 command set；当变更跨 surface 时，再运行更宽的验证。

- Skill markdown shape：`D:\project\ai\docnav\target\debug\docnav-markdown.exe info .codex\skills\api-and-interface-design\SKILL.md --output text`
- Skill markdown outline：`D:\project\ai\docnav\target\debug\docnav-markdown.exe outline .codex\skills\api-and-interface-design\SKILL.md --output text`
- Schema 或 example 变更：运行变更 artifact 附近记录的 validation command。
- CLI、adapter、MCP、schema 或 example boundary 变更：交付前优先运行 `pnpm run verify:docnav-workspace`。
- Local diff check：确认只改了预期拥有的文件。
