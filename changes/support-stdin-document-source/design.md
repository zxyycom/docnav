# Design

本 Design 是 `support-stdin-document-source` 的 change-local Draft：它把 pathname routing hint 与文档内容 acquisition 分开，并以 core/navigation-owned document source 将文件和 stdin 交给同一个 adapter-document 生命周期。它不覆盖 Current owner，也不授权实施；未来推进本 Change 的代理必须先关闭本文的 Open Questions，再补全 tasks 并确认 plan。

## Context

- Current [CLI](../../docs/cli.md) 要求每个 document operation 提供 `<path>`；automatic selection 从 lexical pathname 派生 routing hint，selection 后再执行 filesystem-backed path/access normalization。
- Current [Navigation Input Resolution](../../docs/navigation-input-resolution.md) 和 [适配器契约](../../docs/adapter-contract.md) 只为 normalized document path 创建 invocation-private `AdapterDocument`，closed operation input 也携带 `document_path`。
- Current `docnav::run` 已接收一个 `Read` 类型的 stdin，但执行路径尚未消费它。JSON 和 Markdown adapter 都从 path 获取内容。
- Current adapter lifecycle 要求 factory 不做 document I/O，selected behavior 第一次实际需要内容时至多准备一次 private view，并在同一 invocation 的 eligible stages 中复用。
- Current [原始协议](../../docs/protocol.md) 把 `document.path` 定义为 selection 后的 normalized path；stdin 不是 filesystem path，因此进入 plan 前必须明确该字段与非路径 source 的诚实映射。

本文统一使用以下术语：

- **Document source**：一次 invocation 获取完整文档 bytes 的能力；候选变体是 filesystem path 和 stdin。
- **Routing pathname**：只供 automatic adapter selection 使用的 lexical pathname；它不是 document bytes 或 adapter input。
- **Logical identity**：request correspondence 和 diagnostic 指代当前来源时使用的稳定名称；它不能泄露内部临时资源。
- **Prepared view**：selected adapter 从 document bytes 建立并在同一 invocation 内复用的私有 source/model/index 状态。

## Goals / Non-Goals

Goals:

- 为所有采用 Current linked `AdapterDocument` contract 的格式 adapter 提供一致的 stdin document source，而不是 JSON-only capability。
- 使用显式、可预测的 CLI sentinel，保持 automatic pathname routing、explicit adapter selection 和 route-before-document-I/O 的现有边界。
- 保持一次 invocation 至多 acquisition 一次、prepared view 复用、adapter-owned ref 和 raw/readable output 分层。
- 让 source diagnostic 使用稳定、非宿主相关的逻辑名称，不泄露内部临时路径或进程对象。

Non-Goals:

- 不自动侦测是否存在 pipe，也不根据内容 sniff adapter。
- 不在本 Change 中增加 `--stdin-name`、content-type routing、NDJSON、多 document stream 或 streaming parser contract。
- 不提供跨 invocation stdin cache、session、state handle 或自动重放；分页和后续 read invocation 仍由 caller 重新提供兼容内容。
- 不承诺任意 bytes 都能被任意 adapter 接受；selected adapter 继续按自身格式契约校验实际内容。
- 不改变任一 adapter 的 grammar、outline/read/find/info 语义、pagination、ref grammar 或 compatible-view 保证。

## Decisions

以下条目是本 Draft 已选择的 change-local 方向，不是 Current 行为。Open Questions 若改变 request shape、diagnostic identity 或资源 policy，必须先更新这些条目，再确认 plan。

### 1. `-` 是显式 stdin document operand

Document operation 的 positional `<path>` 接受 literal `-` 作为 stdin sentinel。`./-` 等显式 filesystem spelling 仍表示名为 `-` 的真实文件。Core 不因 stdin 非 TTY、pipe 可读或存在 redirected input 而隐式切换来源。

### 2. Document source 是共享 invocation boundary

Core/navigation 建立一个能表达 path-backed 与 stdin-backed acquisition 的 closed document source，并把它交给 selected definition 创建的 invocation-private `AdapterDocument`。Adapter 不读取进程级全局 stdin，也不接收 raw CLI、routing metadata 或可任意查询的 source bag；它只通过该 source boundary 取得本次 document bytes，并继续拥有 decode、parse、model 和 ref facts。

具体 Rust 类型和存储策略属于实施细节，但 source contract 必须显式区分：用于 selection 的 optional routing pathname、用于 diagnostic/request correspondence 的稳定逻辑 identity，以及实际内容 acquisition。不得把临时文件路径伪装成 caller document identity。

### 3. Stdin 第一版只走 explicit adapter selection

Path-backed source 继续使用 Current exact filename / longest suffix automatic routing。Stdin 没有 pathname hint；caller 必须提供例如 `--adapter docnav-json` 或 `--adapter docnav-markdown`。缺少 explicit adapter 时在消费 stdin 前返回 actionable input/selection diagnostic，不尝试读取内容进行格式推断。

示例目标：

```bash
curl -s https://example.test/data.json \
  | docnav outline - --adapter docnav-json

printf '# Title\n\ncontent\n' \
  | docnav outline - --adapter docnav-markdown
```

### 4. Stdin 保持 lazy single-acquisition 与 invocation snapshot

Stdin source 在 selected behavior 第一次确实需要 document access 时读取至 EOF，并为本 invocation 固定同一份 bytes/view。Adapter-owned semantic validation 若按 Current 顺序先于 document access，仍可在不消费 stdin 的情况下失败。首次 acquisition、decode 或 parse failure 被同一 adapter document 观察和复用，不通过重复读取形成隐式 retry。

同一 invocation 的 base operation、eligible auto-read、full-read content/cost/facts hooks 复用该 snapshot。新的 page 或 read command 是新的 invocation；caller 必须重新提供 compatible bytes，mutation outcome 继续服从 Current compatible-view 与 stale-ref 规则。

### 5. Source mechanics 不进入 adapter-owned 语义

Pipe handle、snapshot storage、temporary resource 和 routing absence 不进入 ref、pagination、operation result、readable/protocol response、adapter options、config 或 invocation log content。Source kind 只在 Open Question 1 选定的 request/diagnostic representation 确实需要时出现，不成为 adapter-owned format fact。Document diagnostics 使用该 representation 定义的稳定 logical identity；内部 spool path 若实现采用临时文件，也不得成为 public fact。

## Risks / Trade-offs

- 通用 source boundary 会同时触及 core、navigation、adapter contracts 和全部 linked adapters；当下改动面大于 JSON-local 分支，但避免每个格式重复入口和进程 I/O ownership。
- Stdin 是 one-shot source。跨 invocation pagination/read 依赖 caller 重放相同内容，比可变文件更容易出现 incompatible view，但不应为此引入持久状态。
- 读取到 EOF 的输入可能长期不结束或体积无界。Current adapters 已整体 materialize document view，但 stdin 仍需要明确 cancellation、read failure 和资源上限是否沿用现状或新增 core policy。
- `document.path` 当前是 normalized filesystem path。简单复用 `-` 或 `<stdin>` 会改变该字段语义；新增 tagged source representation 又会扩大 protocol/schema consumer 更新面，必须在确认 plan 前选择其一。

## Open Questions

1. **Source representation**：由 [原始协议](../../docs/protocol.md)、[CLI](../../docs/cli.md)和[适配器契约](../../docs/adapter-contract.md)共同约束。确认 plan 前必须选择：有界扩展 `document.path` 以接受 literal stdin sentinel，或引入明确的 path/stdin document-source variant；同时固定 diagnostic logical identity。关闭证据必须证明 raw request 可独立解释、schema 仍为 closed shape、adapter input 不携带 transport internals，且 normalized filesystem path 的既有含义没有被静默改写。
2. **Acquisition bound and cancellation**：由 core runtime source lifecycle、CLI error mapping 和受影响 adapter 的 whole-document model 共同约束。确认 plan 前必须决定 stdin 是否新增 byte/time bound 或 cancellation policy；若沿用 Current resource model，关闭证据必须覆盖 read failure、EOF、large input、non-terminating producer 和 path/stdin failure parity。
