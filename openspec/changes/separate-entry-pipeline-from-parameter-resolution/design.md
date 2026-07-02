本 design 记录 core entry pipeline 与 parameter source resolution 的边界；adapter direct CLI 和 adapter `invoke` 不再是默认 surface。

## Context

`adopt-core-linked-adapter-libraries` 将 adapter implementation source 收敛为 current core release static registry。当前 change 保留“入口生命周期”和“参数来源解析”分层，但必须将 adapter owner 边界表达为 library handle/protocol request execution，而非 SDK direct CLI 或 subprocess invoke。

本 change 使用以下术语分层：

- 标准入口管线：core CLI lifecycle owner，负责 command 分类、配置读取决策、handler dispatch 和 output/error projection。
- 入口参数来源解析：参数来源 owner，负责 direct input view、project/user config source 和 defaults 的合并、校验与 handoff。
- adapter implementation source boundary：只有 static registry 中的 adapter library handle 可以执行 document operation。
- adapter native option sources：adapter owner 明确声明的 native option 输入源；未声明的 public input 不得被 resolver 当作隐式 passthrough 消化。
- 标准参数身份：跨 CLI/config/protocol argument 复用的参数 identity，不代表入口生命周期。

## Goals / Non-Goals

**Goals:**

- 定义 core 标准入口管线和 document operation request construction 边界。
- 将标准参数解析描述为入口参数来源解析，并保持 raw argv/protocol arguments 不可变。
- 明确参数来源解析不能提供 adapter implementation source。
- 为 docs、OpenSpec specs、Rust crate/module/type 命名和测试断言提供迁移路径。

**Non-Goals:**

- 不改变 `outline -> ref -> read` 导航模型。
- 不改变 protocol response/result shape、readable output shape 或 adapter ref ownership。
- 不把 help、version、manifest metadata、probe、adapter inspection 或 config 命令纳入 document output mode。
- 不恢复 adapter direct CLI、adapter `invoke` 或 dynamic adapter management。

## Decisions

### Decision 1: 标准入口管线拥有生命周期，参数来源解析只拥有来源合并与校验

Core CLI 先分类入口，再决定是否调用参数来源解析。Help、config/init/doctor/version 和 `adapter list` 保留各自 output/error owner；只有 document operations 和明确的 document-context inspection 路径可进入参数来源解析。

### Decision 2: 参数来源解析不得提供 implementation source

Adapter id 选择、static registry lookup 和 adapter library handle dispatch 由 core routing/navigation owner 处理。参数来源解析只产出 typed values、source info、diagnostics 和 owner-scoped native option sources。

### Decision 3: 原始输入保持不可变，解析结果是 derived values

参数来源解析只读取入口 owner 构造的 input view 或 loaded config source，不得修改原始 argv、protocol envelope 或 `arguments`。Request construction 只能消费 typed runtime values、source info 和 owner 明确保留的 passthrough。

### Decision 4: adapter native options 必须显式来源化

入口 owner 只把已注册标准参数、已注册 config path 和 adapter owner 明确声明的 native option source descriptors 提供给入口参数来源解析。未映射 public input 必须作为 owner-boundary handoff 返回。

## Risks / Trade-offs

- [Risk] 新旧术语并存会短期增加阅读成本。→ Mitigation: owner docs 使用 static registry/library handle 术语，旧 SDK/invoke 术语只保留为历史背景。
- [Risk] 参数来源解析被误用为 adapter lookup。→ Mitigation: spec 明确 implementation source boundary，tests 覆盖 missing adapter id 和 historical registry ignored。

## Open Questions

无未回答开放问题。
