本 design 记录 core entry pipeline 与 navigation input resolution 的边界；adapter direct CLI 和 adapter `invoke` 不再是默认 surface。

## Context

`adopt-core-linked-adapter-libraries` 将 adapter implementation source 收敛为 current core release static registry。当前 change 保留“入口生命周期”和“navigation input resolution”分层，但必须将 adapter owner 边界表达为 library handle/protocol request execution，而非 SDK direct CLI 或 subprocess invoke。

本 change 使用以下术语分层：

- Core entry pipeline：core CLI lifecycle owner，负责 command 分类、config source descriptor/path handoff、non-navigation handler dispatch 和 output/error projection。
- Navigation input resolution：document operation 来源解析 owner，负责 raw project/user config source loading、direct input view、defaults 的合并、校验与 handoff。
- adapter implementation source boundary：只有 static registry 中的 adapter library handle 可以执行 document operation。
- adapter native option sources：adapter owner 明确声明的 native option 输入源；未声明的 public input 不得被 resolver 当作隐式 passthrough 消化。
- Typed field identity：跨 CLI/config/protocol argument 复用的参数 identity，不代表入口生命周期。

## Goals / Non-Goals

**Goals:**

- 定义 core entry pipeline 和 document operation request construction 边界。
- 将来源解析描述为 navigation input resolution，并保持 raw argv/protocol arguments 不可变。
- 明确 navigation input resolution 不能提供 adapter implementation source。
- 为 docs、OpenSpec specs、Rust crate/module/type 命名和测试断言提供迁移路径。

**Non-Goals:**

- 不改变 `outline -> ref -> read` 导航模型。
- 不改变 protocol response/result shape、readable output shape 或 adapter ref ownership。
- 不把 help、version、manifest metadata、probe、adapter inspection 或 config 命令纳入 document output mode。
- 不恢复 adapter direct CLI、adapter `invoke` 或 dynamic adapter management。

## Decisions

### Decision 1: Core entry pipeline 拥有生命周期，navigation input resolution 拥有来源加载、合并与校验

Core CLI 先分类入口，再决定是否调用 navigation input resolution。Help、config/init/doctor/version 和 `adapter list` 保留各自 output/error owner；只有 document operations 和明确的 document-context inspection 路径可进入 navigation input resolution 或等价只读 helper。

### Decision 2: Navigation input resolution 不得提供 implementation source

Adapter id selection、static registry lookup 和 adapter library handle dispatch 由 `docnav-navigation` 使用 core-supplied registry 处理。Navigation input resolution 只产出 typed values、source info、diagnostics 和 owner-scoped native option sources，不创建新的 implementation source。

### Decision 3: 原始输入保持不可变，解析结果是 derived values

Navigation input resolution 只读取入口 owner 构造的 input view 和自己加载的 raw config source，不得修改原始 argv、protocol envelope 或 `arguments`。Request construction 只能消费 typed runtime values、source info 和 owner 明确保留的 passthrough。

### Decision 4: adapter native options 必须显式来源化

入口 owner 只把 raw navigation command、config source descriptors/paths 和 adapter registry 提供给 `docnav-navigation`。未映射 public input 必须作为 navigation input diagnostic 返回。

## Risks / Trade-offs

- [Risk] 新旧术语并存会短期增加阅读成本。→ Mitigation: owner docs 使用 static registry/library handle 术语，旧 SDK/invoke 术语只保留为历史背景。
- [Risk] navigation input resolution 被误用为新的 adapter implementation source。→ Mitigation: spec 明确 implementation source boundary，tests 覆盖 missing adapter id 和 historical registry ignored。

## Open Questions

无未回答开放问题。
