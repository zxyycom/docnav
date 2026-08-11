# Design

该设计把 MCP 限制为核心 CLI 的格式无关接入层，并从单次 protocol-json 调用投影两种 MCP content。

## Context

Core CLI 已拥有 pathname routing、adapter selection、config、operation input、protocol envelope 和错误映射。MCP bridge 是新的调用入口，不是新的文档业务 owner。当前产品决策要求先稳定 adapter lifecycle、find、protocol 和 output；恢复时必须按届时 Current contract 复核 tool schema。

## Goals / Non-Goals

**Goals**

- 提供 npm 可安装的 `docnav-mcp` bin 和 stdio transport。
- 暴露 outline、read、find、info 四个格式无关 tools，并把参数直接映射到核心 CLI。
- 从单次 protocol-json stdout 生成 schema-checked `structuredContent` 和精简 `TextContent`。
- 把 output schemas 随包内联或打包，支持离线 tool discovery。

**Non-Goals**

- 不解析文档、不识别或调用 adapter、不解释 ref、不拥有 config/routing/error semantics。
- 不解析默认 human output，不复制 Rust readable-view framing，不暴露完整 protocol envelope。
- 不在本 Change 中实现 adapter 管理、core service 或新的 document operation。

## Decisions

1. 每个 document tool 通过子进程调用 `docnav <operation> ... --output protocol-json`；核心 CLI 继续拥有 adapter 选择、配置与错误映射。恢复时以届时 CLI owner 重审精确 spelling，当前计划映射为：

   | MCP tool | Core operation | Tool input passed to CLI |
   | --- | --- | --- |
   | `document_outline` | `docnav outline` | `path`、`page`、`limit_chars`、可选 `adapter` |
   | `document_read` | `docnav read` | `path`、原样 `ref`、`page`、`limit_chars`、可选 `adapter` |
   | `document_find` | `docnav find` | `path`、`query`、`page`、`limit_chars`、可选 `adapter` |
   | `document_info` | `docnav info` | `path`、可选 `adapter`；不传 `page` 或 `limit_chars` |

   Bridge 只形成 argv；typed validation、source resolution 和 operation semantics 仍由核心 CLI 决定。
2. Bridge 校验 stdout 的 protocol response。成功时 `structuredContent` 从对应 operation result 投影，并按届时 Current result shape 处理 optional branch（例如 outline kind 或 auto-read）；它不包含 `protocol_version`、`request_id`、`operation`、`ok` 或完整 envelope。每个 tool schema 只复述其消费的 operation-result facts，并由生成或双向 fixtures 保持可追溯。
3. `TextContent` 消费与 `structuredContent` 相同的 result/error facts；不读取默认 readable output，也不为展示再次调用 CLI。
4. 可选 MCP `adapter` 原样映射为 `docnav --adapter`。Bridge 不解释 id；explicit lookup、automatic routing、selected validation 和 no-fallback 都留在 core。
5. Failure content 至少保留 protocol error 的 code、message、owner，并在存在时保留 guidance/details；不泄露完整 envelope 或不稳定内部 cause。
6. 子进程成功退出时 stderr 中的 owner-scoped status 不自动升级为 MCP error；最终成功/失败同时尊重退出码和 stdout protocol `ok`。
7. 每个 tool 拥有精简 MCP output schema；schema 从项目稳定 contract 生成或用同步验证防漂移，并随 package 离线可用。
8. 本 design 是实施期间唯一承载 change-local Target 的载体，并登记以下 owner delta：新增 MCP bridge/package owner；在 CLI/protocol/output/schema/examples 中登记实际成立的 bridge 映射与消费边界；在 testing/release 中登记 stdio、离线 package 和真实 CLI handoff 证据。实现期间稳定 owner 只提供 Current 基线，不提前写入 Target。只有 mapping、content/error、stdio package 和真实 CLI 行为证据通过后，才把已成立的 delta 同步为 Current，并再次验证 design、owner、package 与证据一致。

## Risks / Trade-offs

- 每次 tool call 启动子进程有固定开销：v1 优先保持单一业务 owner；只有测量证明需要时再处理 service/caching。
- Protocol 或 tool schema 漂移：恢复时重基线，并用生成或双向 fixtures 证明映射一致。
- 两种 content 可能分叉：两者必须消费同一 normalized result/error facts，测试同时断言 shape 与文本。
- stderr status 容易被误判：用明确进程结果规则和 fixture 覆盖成功 stderr、protocol failure 与 malformed stdout。
- Node/MCP SDK 版本会变化：实现时选择当前受支持版本，但不得改变本设计的 tool、stdio 和 schema 边界。

## Open Questions

无改变目标或 contract 的未决问题。具体 MCP SDK 版本属于恢复实施时的依赖选择，不得用来重新定义 tools 或核心职责。
