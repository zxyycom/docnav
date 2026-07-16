## Context

当前 `readable-view` 与 `readable-json` 从同一 serialized readable payload 派生。该 payload 同时是 renderer input、public schema 和 bridge/composition dependency，因此 output owner 实际维护 protocol envelope 与 readable DTO 两套 machine-readable contract。

目标链路为：

```text
completed operation outcome or primary diagnostic
  -> OutputPlan::ProtocolJson
       -> existing protocol envelope -> stdout
  -> OutputPlan::Rendered(RenderStrategy)
       -> complete UTF-8 text -> stdout
```

Core CLI 将省略 output 或 `readable-view` 映射到带内置 renderer 的 `Rendered`；`protocol-json` 映射到 `ProtocolJson`。其它 linked code caller 可以直接构造带自定义 renderer 的 `Rendered`，但不会把该实现注册为 public output mode。

## Ownership

| Concern | Owner | 本 change 的作用 |
| --- | --- | --- |
| Operation success outcome、adapter facts、ref 与 pagination | Navigation + adapter contracts | 保持 typed facts，不增加 presentation responsibility |
| Primary diagnostic identity 与 canonical details | Diagnostics owner | 同一 record 供 protocol serializer 或 renderer 消费 |
| Protocol envelope 与 machine compatibility | Protocol + output owner | `ProtocolJson` 直接序列化既有 contract |
| Renderer selection 与 presentation text | Linked composition caller + selected renderer | Core CLI 注入内置 renderer；direct caller 可注入自定义实现 |
| Path selection、atomic write、render failure 与 channels | `docnav-output` | 编排两个 output paths 并映射 `output_render_failed` |
| CLI output value 与 process exit | Core CLI | 只映射 mode，不暴露 renderer identity |

## Decisions

### Decision 1: Shared output model 使用两个封闭分支

Shared output API 使用：

```text
OutputPlan::ProtocolJson
OutputPlan::Rendered(RenderStrategy)
```

Core CLI 只接受 document output values `readable-view` 与 `protocol-json`。省略 output 或选择 `readable-view` 时，core composition 注入内置 `readable-view` renderer；选择 `protocol-json` 时构造 `ProtocolJson`。Help、version 和尚未形成有效 document output context 的 early failure 继续由 PlainText 或其现有 owner 处理，不构成第三个 document output path。

### Decision 2: Renderer dependency 只由 linked code 提供

每个 `Rendered` plan 在进入 output orchestration 前必须携带一个 renderer function 或 trait value。Core CLI 的 public `readable-view` value 固定对应内置 renderer；其它 linked code caller 直接调用 shared API 时可以提供替代实现，但不复用 `readable-view` 作为自定义格式名称。

CLI、config、environment、manifest、plugin metadata 和 subprocess input 都不携带 renderer implementation 或 strategy id。Adapter definition 继续只描述格式和 operation contract。

### Decision 3: Renderer 直接消费完成的 typed outcome

Renderer contract 为：

```text
RenderStrategy(RenderInput, RenderContext) -> Result<UTF8Text, RenderFailure>
```

`RenderInput` 是一个完成的 operation success outcome 或 primary `DiagnosticRecord`；`RenderContext` 只包含 presentation 所需的 operation 与 selected format/adapter facts。Renderer 可以建立 private helper view，但该 view 不序列化、不发布 schema，也不承担跨进程或跨版本 compatibility。

### Decision 4: Output layer 原子提交 text 并拥有可恢复失败

Renderer 在内存中产生完整 UTF-8 text。成功时 output layer 原样提交该值，不追加 wrapper、block framing、separator 或尾随换行；内置 renderer 自己保持 `readable-view` framing，自定义 renderer 自己定义 presentation contract。

Renderer 返回 `RenderFailure` 时，output layer 产生 output-owned `output_render_failed` boundary diagnostic，保持 stdout 为空，通过 stderr 与 internal failure exit mapping 报告，并且不调用第二个 renderer。Diagnostic details 只包含 bounded render failure facts，不把 renderer-private payload 变成 public contract。

### Decision 5: Protocol、logging 与 adapter payload 绕过 renderer

`ProtocolJson` 直接序列化既有 success/failure envelope，不构造 `RenderInput`。Renderer 只读取 completed outcome，不改写 ref、page、entries、matches、content type、cost、diagnostic code 或 operation status。Invocation logging 继续写入独立 sink，不进入 renderer input/output、protocol fields 或 adapter handler payload。

### Decision 6: `readable-json` hard cutover

实现删除 `readable-json` mode、serializer、public DTO、schema/examples/fixtures/goldens 和 mode-specific validation，不提供 alias、fallback 或 compatibility branch。需要稳定结构化事实的 caller 改用 `protocol-json`；需要自定义 presentation 的 linked caller 直接构造 `Rendered`。

## Acceptance

| Check | Pass condition | Evidence |
| --- | --- | --- |
| CLI mapping | Omitted/`readable-view` 使用内置 renderer；`protocol-json` 绕过 renderer | Core CLI + process tests |
| Linked renderer | Direct caller 可注入自定义 renderer，public inputs 无 implementation identity | Shared output API tests + repository search |
| Protocol isolation | Existing success/failure envelope、ref 与 pagination shape 不变 | Protocol schema/examples + integration tests |
| Atomic rendered output | Success 精确提交 renderer text；returned failure 不产生 partial stdout 或 fallback | Output tests + CLI smoke |
| Diagnostic identity | Primary diagnostic 保持 identity；render failure 使用 `output_render_failed` | Diagnostics + output tests |
| Cutover | Runtime、docs、schema 和 active consumer plans 不再依赖 readable JSON DTO | Filtered search + workspace verification |

## Risks / Trade-offs

- 自定义 renderer 可以省略 ref、continuation 或其它事实：presentation completeness 由 renderer owner 声明，稳定完整事实继续由 `protocol-json` 提供。
- Workspace release 使用 `panic = "abort"`，因此 renderer panic 不属于可恢复 `RenderFailure` contract；linked renderer 是 trusted code，并通过直接 tests 证明正常与 returned-error paths。本 change 不增加 process isolation。
- 轻量结构化 consumer 需要解析更完整的 protocol envelope：这是删除第二套 machine schema 后接受的迁移成本。
- Hard cutover 会让旧 consumer 与回滚版本不兼容：schema、examples、active plans 和 runtime 在同一 change 中迁移；rollback 通过整体 revert 恢复旧 contract。

## Open Questions

无。
