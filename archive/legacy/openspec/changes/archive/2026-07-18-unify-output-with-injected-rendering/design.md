## Context

当前 document pipeline 的完成结果已经是 `ProtocolResponse`。Output layer 收到该结构后，`protocol-json` 直接序列化它，而两个 readable mode 先把其中的 result/error 转成另一套 readable JSON payload。

本 change 删除这套 public readable JSON contract，并把 output flow 收敛为：

```text
ProtocolResponse
  -> OutputPlan::ProtocolJson
       -> protocol JSON
  -> OutputPlan::Rendered(RenderStrategy)
       -> complete UTF-8 text
```

## Ownership

| Concern | Owner | 本 change 的作用 |
| --- | --- | --- |
| `ProtocolResponse`、result/error、ref 与 pagination | Protocol + existing operation owners | 保持结构和语义不变 |
| Output path、renderer invocation 与 document channels | `docnav-output` | 用同一个 response 执行 protocol 或 rendered plan |
| Renderer selection 与 presentation text | Linked caller + selected renderer | Core CLI 注入内置 renderer |
| Public output value 与 process exit | Core CLI | 将 mode 映射为 output plan |

## Decisions

### Decision 1: Shared output model 使用两个封闭分支

Shared output API 使用：

```text
OutputPlan::ProtocolJson
OutputPlan::Rendered(RenderStrategy)
```

Core CLI 只接受 `readable-view` 与 `protocol-json`。省略 output 或选择 `readable-view` 时构造带内置 renderer 的 `Rendered`；选择 `protocol-json` 时构造 `ProtocolJson`。Help、version 和其它 non-document output 保持现有 owner。

### Decision 2: Renderer 由 linked code 提供

每个 `Rendered` plan 在进入 output orchestration 前携带一个 renderer function 或 trait value。Core CLI 固定注入内置 `readable-view` renderer；其它 linked caller 可以直接提供自定义 renderer。

Public input 只选择 output mode，不携带 renderer implementation id。

### Decision 3: Renderer 直接消费 `ProtocolResponse`

Renderer contract 为：

```text
RenderStrategy(&ProtocolResponse) -> Result<String, RenderFailure>
```

`ProtocolResponse` 是 renderer 的完整输入。Success response 提供 typed operation result；failure response 提供现有 protocol error、optional operation 和 request context。Document failure 在进入 output plan 前使用现有 protocol projection 构造成 failure response。

### Decision 4: Renderer 拥有完整文本，output layer 拥有写入

Renderer 在第一次 stdout write 前返回完整 `String`。Output layer 原样写入该值，不追加 wrapper、separator 或换行。

Renderer 返回 `RenderFailure` 时，output layer 返回该错误，stdout 保持为空，并且不调用第二个 renderer。Core CLI 对内置 renderer failure 沿用现有 output failure mapping；本 change 不新增或重命名 stable diagnostic code。

Renderer 成功后的 stdout writer failure 继续作为独立 I/O error，可能发生在部分 bytes 已写入之后。

### Decision 5: 内置 renderer 保持 `readable-view` contract

内置 renderer 从 `ProtocolResponse` 派生现有 readable presentation，并保持 header、block reference、framing、unstructured outline 和 readable error text 的 owner contract。它可以继续使用 private helper value；只有最终 `readable-view` text 是 public readable contract。

### Decision 6: 删除 `readable-json`

实现删除 `readable-json` accepted value、output branch 和 public schema/example/validation surface，不提供 alias 或 fallback。需要稳定结构化输出的 caller 使用 `protocol-json`；需要其它 presentation 的 linked caller 注入 renderer。

## Acceptance

| Check | Pass condition | Evidence |
| --- | --- | --- |
| Unified input | Protocol 与 rendered paths 都消费同一 `ProtocolResponse` contract | Shared output tests |
| CLI mapping | Omitted/`readable-view` 使用内置 renderer；`protocol-json` 绕过 renderer | Core CLI tests |
| Exact text | Rendered stdout 等于 renderer 返回的完整 UTF-8 text | Renderer/output tests |
| Failure boundary | `RenderFailure` 发生在 stdout write 前且不触发 fallback；writer error 保持独立 | Output writer tests |
| Cutover | Runtime 和 Current validation materials 不再提供 `readable-json` | Schema/docs checks + workspace verification |

## Migration

1. 需要稳定结构化结果的 CLI/config consumer 将 `readable-json` 改为 `protocol-json`。
2. 只需要阅读文本的 caller 使用默认输出或 `readable-view`。
3. Readable helper 可以作为内置 renderer 的 private implementation 保留；public readable JSON schema、examples 和 mode-specific validation 被删除。

## Risks / Trade-offs

- Renderer 依赖 stable `ProtocolResponse` contract；这是本 change 选择的统一内部输入。
- 自定义 renderer 可以省略 protocol facts；需要完整稳定事实的 caller 使用 `protocol-json`。
- Renderer 返回完整 `String`，因此 rendered path 的峰值内存随输出文本大小增长。

## Open Questions

无。
