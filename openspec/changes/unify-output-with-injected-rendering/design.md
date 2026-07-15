## Context

本 change 的目标是把 document operation 输出重构为 `ProtocolJson` 与 `Rendered(RenderStrategy)` 两条路径，默认 renderer 为 `readable-view`；当前文档只在 `openspec/changes/unify-output-with-injected-rendering/` 下形成未审核临时文档，不影响现有其它文档或主规范。

当前 `readable-view` 与 `readable-json` 共享 serialized readable payload，再分别承担 presentation 与轻量 machine output。这个中间 payload 同时成为 renderer 输入、public schema 和 bridge/composition 依赖，使 output owner 维护两套 machine-readable contract。

本设计把 renderer 归到 output composition：adapter/navigation 先完成业务 outcome，调用方再为 rendered path 提供一个代码函数。`protocol-json` 直接消费 raw protocol envelope；renderer 不参与协议生命周期。

## Goals / Non-Goals

**Goals:**

- 固化 `ProtocolJson` 与 `Rendered(RenderStrategy)` 两路径模型。
- 让每个 rendered invocation 在调用 output orchestration 前获得一个确定的 renderer，默认实现为 `readable-view`。
- 让 renderer 直接消费完成的 typed outcome 或 primary diagnostic，并返回完整 UTF-8 presentation text。
- 保持 protocol、adapter facts、ref、pagination、diagnostic identity 和 process channel ownership 稳定。
- 删除 serialized readable JSON contract 及其验证材料。

**Non-Goals:**

- Renderer implementation 不通过 CLI、配置、环境变量、manifest、plugin 或外部进程选择。
- Adapter definition 不承担 renderer 注册或 presentation ownership。
- 自定义 renderer 不获得 repository-wide layout、字段或内容完整性承诺。
- 本 change 不改变 raw protocol、operation semantics、routing、parsing 或 ref grammar。

## Decisions

### Decision 1: 共享 output model 使用两个封闭分支

共享 output API 使用以下概念模型：

```text
OutputPlan::ProtocolJson
OutputPlan::Rendered(RenderStrategy)
```

Core CLI 仍只接受 document output value `readable-view` 与 `protocol-json`。省略 output 或选择 `readable-view` 时构造 `Rendered`；选择 `protocol-json` 时构造 `ProtocolJson`。PlainText help、version 和尚未形成有效 document output context 的 early failure 继续由各自 owner 处理，不构成第三个 document output 分支。

### Decision 2: Output composition 是 renderer 注入的唯一 owner

构造 `Rendered` 的 linked code caller 必须同时提供 renderer。Core CLI composition 默认注入内置 `readable-view`；其它代码入口可以在调用 shared output API 时提供替代函数。CLI/config 只选择 document output mode，不传递 renderer implementation 或 strategy id。

Adapter 继续返回 structured operation result 或 diagnostic。若需要 format-specific presentation，composition caller 可以注入消费这些结果的 renderer，而无需扩展 adapter definition 或 serialized metadata。该边界保持 adapter 与 output ownership 独立，也消除了多注入源的优先级问题。

### Decision 3: Renderer contract 直接使用完成的 outcome

Renderer 的概念签名为：

```text
RenderStrategy(RenderInput, RenderContext) -> Result<UTF8Text, RenderFailure>
```

`RenderInput` 是一个完成的 operation success outcome 或 primary `DiagnosticRecord`；`RenderContext` 只包含 presentation 所需的 operation 与已选 format/adapter facts。实现可以建立 private helper view，但该 view 不序列化、不发布 schema，也不承担跨进程或跨版本 compatibility。

### Decision 4: Renderer 返回完整文本，output layer 负责提交结果

Renderer 在内存中产生完整 UTF-8 text。成功时 output layer 原样提交该值，不追加 wrapper、block framing、separator 或尾随换行；内置 `readable-view` 自己应用现有 framing contract，自定义 renderer 自己定义 presentation。

Document failure 通过同一个 renderer 投影 primary diagnostic，原 diagnostic 继续决定 exit class。Renderer 自身失败时，output layer 保持 stdout 为空并通过稳定 render diagnostic 与 CLI exit mapping 报告失败。代码显式注入的 renderer 是本次 invocation 的唯一 presentation owner，因此失败后不切换到其它 renderer。

### Decision 5: Protocol、adapter facts 与 presentation 分层保持独立

`ProtocolJson` 直接序列化既有 success/failure envelope，不构造 `RenderInput`。Renderer 只读取 completed outcome，不改写 ref、page、entries、matches、content type、cost、diagnostic code 或 operation status。自定义 renderer 可以选择展示哪些事实；完整稳定事实由 `protocol-json` 提供。

### Decision 6: 删除 readable JSON contract

实现删除 `readable-json` mode、serializer、public DTO、schema/examples/fixtures/goldens 和 mode-specific validation。CLI/config 沿用当前 output field validation，只识别本 change 保留的 mode，不建立旧 mode 的单独分支。

依赖 typed readable JSON 的 active changes 必须在实现前改用 `protocol-json` 或 linked renderer API。该协调在 OpenSpec 层完成，不进入 runtime output contract。

## Risks / Trade-offs

- [Risk] Private render helper 演化成 shadow protocol。→ Mitigation: helper 保持不可序列化，验证聚焦 public stdout 与 function boundary。
- [Risk] 自定义 renderer 省略 ref、continuation 或内容事实。→ Mitigation: 由 renderer owner 声明 presentation contract；稳定完整事实继续由 `protocol-json` 提供。
- [Risk] Renderer failure 把成功 operation 转成 process failure。→ Mitigation: 渲染完成后一次性写 stdout，并覆盖 panic/error、空 stdout 和 exit mapping。
- [Risk] Active changes 继续实现旧三模式。→ Mitigation: 把 conflict rebase 设为实现前阻塞门禁。
- [Trade-off] 轻量结构化 consumer 需要解析较完整的 protocol envelope。→ Accepted: 以单一 machine contract 换取更小的长期 schema 与 mapping surface。

## Migration Plan

1. 完成阻塞审计，并重基或暂停仍依赖三模式或 typed readable JSON 的 active changes。
2. 更新 output、CLI、testing 与 validation owner 文档，固化两路径模型和 code-only renderer injection。
3. 更新 schema、examples、fixtures、goldens 与索引，只保留仍有 owner 的 public contracts。
4. 引入 `OutputPlan`、`RenderStrategy`、`RenderInput`、`RenderContext` 和内置 `readable-view` 实现。
5. 将 core CLI 与其它 linked callers 接到 shared output API，删除 readable JSON 分支。
6. 运行 protocol、renderer、CLI、logging、release 和 workspace verification；回滚时整体恢复旧 contract 与验证材料。

## Open Questions

无未回答开放问题，可以进入实现前审计。
