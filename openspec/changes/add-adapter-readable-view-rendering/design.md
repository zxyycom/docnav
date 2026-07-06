本 design 起草 `readable-view` adapter 可选自定义渲染 hook 与 Markdown adapter 的 md-like 输出方案；当前文档只在 `openspec/changes/add-adapter-readable-view-rendering/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 当前 document output mode 包含 `readable-view`、`readable-json` 和 `protocol-json`。现有 `readable-view` 以 pretty JSON header 和 length-delimited block section 表达阅读结果；它便于字段级校验，但无法表达每个 adapter 对阅读文本的展示偏好。

这次 change 只处理 `readable-view`。`readable-json` 和 `protocol-json` 继续承担机器稳定输出；adapter 的 ref、pagination、operation result、diagnostic 和 routing 语义不变。新的 adapter hook 是通用 presentation hook，不内置原格式 like、省略、ref guidance 或 continuation guidance 语义；这些语义只由具体 adapter 的 readable-view renderer 自己声明。Hook 不读取 argv、stdin、stdout、stderr、cwd 或进程退出状态，只接收已经完成的 document operation 成功结果和输出层提供的 render context。

## Goals / Non-Goals

**Goals:**

- 允许 adapter 可选提供 `readable-view` 文本渲染逻辑，用于把 operation result 投影成 adapter 自己选择的纯文本展示。
- 保留 core generic readable-view fallback，保证未实现 hook 的 adapter 继续可用。
- 明确通用 hook 不要求原格式 like、显式省略、ref guidance 或 continuation guidance；这些是 adapter-specific renderer contract。
- 首先为 Markdown adapter 实现 md-like readable-view，覆盖 outline、read、find 和 info。
- 保持 `readable-json`、`protocol-json`、raw protocol schema、readable JSON schema、ref ownership 和 pagination outcome 稳定。

**Non-Goals:**

- 不把 renderer hook 作为用户配置、项目配置、环境变量或 CLI flag 暴露。
- 不要求其它格式 adapter 在本 change 中实现 md-like、native-like、omission-aware 或 ref-aware readable-view。
- 不要求 adapter-rendered `readable-view` 是合法 Markdown、合法 JSON、原格式 like 或符合原文档业务 schema，除非具体 adapter renderer contract 自己声明。
- 不允许 adapter renderer 接管 stdout/stderr 分流、exit code、error projection、adapter selection 或 operation dispatch。
- 不改变 document output mode 枚举；仍然只有 `readable-view`、`readable-json` 和 `protocol-json`。

## Decisions

### Decision 1: Adapter hook 只返回 text

Adapter readable-view renderer 的概念接口是接收 operation、成功 payload 和 render context，返回一段 UTF-8 text，或返回 unsupported。它不返回 JSON header、block sections、stdout/stderr 写入计划或 exit code。这个决定把自定义展示交给 adapter，同时保留 output layer 对通道、失败投影和 fallback 的所有权。

替代方案是让 adapter 直接接管整个 `readable-view` stdout。该方案会让每个 adapter 重复实现输出模式、错误边界和 channel writing，也会削弱 core 对 document output mode 的统一控制，因此不采用。

### Decision 2: Core 始终保留 generic fallback

`docnav-output` 在 `readable-view` 成功路径上优先尝试 selected adapter 的 renderer hook；当 hook 不存在、返回 unsupported，或发生 renderer-local failure 时，core 使用 generic readable-view fallback 渲染同一个成功结果。Fallback 失败才进入稳定的 readable-view render failure 诊断。

替代方案是 adapter renderer 失败直接导致 document operation 失败。该方案会让体验增强能力影响基本可用性，不符合 hook 的定位，因此不采用。

### Decision 3: Machine outputs 不共享 presentation text

Adapter-rendered readable-view text 不反向写入 protocol result，也不作为 readable-json 的字段来源。`readable-json` 继续序列化 typed readable payload；`protocol-json` 继续序列化 protocol envelope。Adapter renderer 只消费已经生成的成功事实，不生产新的机器事实。

替代方案是让 adapter 返回一个统一 presentation payload，再由 readable-json/readable-view 共用。该方案会把 presentation text 混入机器可读契约，破坏现有分层，因此不采用。

### Decision 4: Hook 本身不定义文本内容语义

Adapter-rendered readable-view 的具体文本语义由 adapter renderer 自己拥有。通用 hook 只规定输入、输出、fallback、side-effect 边界和 machine output 隔离；它不要求原格式 like 输出、省略标记、ref 可发现性、continuation guidance、合法原格式或固定布局。

替代方案是在通用 hook 上固定一组 readable semantics。该方案会把 Markdown 当前想要的 md-like 省略体验误推广到所有 adapter，限制未来 adapter 自定义渲染，因此不采用。

### Decision 5: Markdown 是首个 adapter renderer

Markdown adapter 首期提供 md-like renderer，覆盖 outline、read、find 和 info。Outline 应呈现 Markdown heading 骨架并标出被省略的 sibling/children/content；read 应呈现当前 ref/page 的 Markdown content 并标出 page 前后省略；find 应呈现命中上下文片段并标出非连续片段与更多 match/page；info 应以 Markdown-readable 摘要表达格式事实。

替代方案是先只改 read 或 outline。该方案会让同一 output mode 在不同 operation 间体验分裂，尤其 find 仍然像列表而不是原文上下文，因此不采用。

## Risks / Trade-offs

- [Risk] Adapter renderer 成为第二套协议事实来源 -> Mitigation: renderer input 必须来自已生成 success payload，输出只是一段 text，不得改变 `readable-json` 或 `protocol-json`。
- [Risk] Renderer hook 失败导致成功 operation 变失败 -> Mitigation: hook 缺失、unsupported 或 renderer-local failure 默认 fallback 到 generic readable-view。
- [Risk] 实现者把 Markdown 的 md-like 省略要求误认为通用 hook 要求 -> Mitigation: output-contract 只定义 hook/fallback/machine-output 边界，Markdown 的省略、ref 和 continuation 文案只写在 markdown-adapter。
- [Risk] Markdown 纯文本省略标记被误认为原文内容 -> Mitigation: Markdown docs、tests 和 examples 明确省略标记属于 Markdown readable-view projection semantics。
- [Risk] Markdown md-like output 的 golden 过度绑定具体文案 -> Mitigation: tests 优先断言 operation coverage、ref 可发现性、omission marker、continuation guidance 和关键片段顺序；只对少量用户可见模板做稳定断言。
- [Risk] Hook 被扩展成用户自定义策略接口 -> Mitigation: 本 change 只定义 adapter capability，不暴露 CLI/config/env/plugin-level renderer injection。

## Migration Plan

1. 在 specs 和 owner docs 中先把 `readable-view` 从“必为 JSON+block”迁移为“可由 adapter renderer 返回自定义 text，generic fallback 可继续使用 JSON+block 或后续基础文本布局”。
2. 实现 core render orchestration：成功路径尝试 adapter renderer，缺失或失败时使用 generic fallback；失败路径继续使用 readable error projection。
3. 为 Markdown adapter 增加 md-like renderer，并覆盖 outline、read、find、info 的 CLI smoke/golden。
4. 更新 docs/examples/tests，把 hook 描述为 adapter 可选自定义 text renderer；把 md-like、省略、ref/continuation guidance 限定在 Markdown adapter；机器验证继续使用 `readable-json` 和 `protocol-json`。
5. 回滚时可以禁用 Markdown adapter renderer hook，让 core generic fallback 接管 `readable-view`，不影响 machine outputs。

## Open Questions

无未回答开放问题，可以进入实现前审计。需要在审计中确认通用 hook 文档没有暗示必须 md-like 或 omission-aware，并确认 Markdown 省略标记的具体文案是否足够稳定。
