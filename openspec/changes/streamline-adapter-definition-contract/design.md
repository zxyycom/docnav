本 design 说明如何把 linked adapter 扩展面收敛为单一 registry-facing adapter definition/descriptor：adapter 作者在一个 definition/factory 中声明 adapter 事实，core、CLI inspection、navigation resolution、full-read pre-dispatch 和 dispatch 从该 definition 传递和派生使用。实施和归档前，主规范与当前二进制状态仍以 `docs/`、代码和测试为准。

## Context

Docnav 的稳定导航链路是 `outline -> ref -> read`。当前主规范已经把 owner 边界拆清楚：`docnav` core 拥有 CLI、static registry、输出模式和错误映射；`docnav-navigation` 拥有 config source loading、adapter selection、selected adapter declarations、typed resolution、request construction 和 dispatch；格式 adapter 拥有格式识别、解析、ref、分页结果和格式语义结果。

现有实现和规范已经避免了 parser/ref/pagination 级别的细粒度 hook，但 adapter 作者仍需要在多个函数和 helper 中配置 manifest、native options、operation support 和 full-read support/content/cost/facts，并在 handler 内从 generic options bag 取值。当前 `Adapter` surface 包括 metadata/support methods、`outline/read/find/info` handlers，以及 `unstructured_full_read`、`measure_unstructured_full_read_cost`、`unstructured_full_read_facts` 这一组非结构化全文读取接口。这个 change 的设计目标是收紧 adapter authoring surface，使 adapter-owned facts 只在 definition 中声明一次，再由 registry/navigation/dispatch 传递使用；core/navigation/adapter 的长期 owner 边界保持现有分工。

## Goals / Scope Boundaries

**Goals:**

- 为 built-in linked adapters 定义 registry-facing adapter definition/descriptor 形态，作为 registry 和 navigation 消费的单一入口。
- 让 adapter 作者只维护一个 adapter definition/factory；static registry、CLI native option catalog、adapter inspection、navigation selected declaration registration、full-read pre-dispatch 和 dispatch 都从该 definition 读取同一 fact。
- 保持 `outline`、`read`、`find`、`info` 为必需高层 operation handlers。
- 让 adapter-owned native options 通过 declaration 表达 public contract，通过 navigation 完成来源解析、默认值和基础校验，通过内部 typed handoff/accessor 进入 handler。
- 把当前 non-structured full-read 接口组按 capability group 表达，覆盖 support declaration、content hook、cost measurement hook 和 result facts hook。
- 保持 `protocol-json`、`readable-json`、`readable-view`、ref opacity、pagination semantics 和 adapter implementation source 稳定。

**Scope Boundaries:**

- Adapter definition contract 面向 core release linked adapters；第三方动态插件系统、独立 adapter process runtime 和 service-mode adapter source 属于其它 change。
- Parser、AST、ref parser、pagination algorithm 和 readable rendering 保持 adapter-owned/internal owner。
- 各 adapter 可以保留各自内部 domain model；shared layers 只消费 definition 暴露的 registry-facing facts。
- Markdown ref grammar、Markdown outline/read/find/info 业务语义和现有 document output wrapper 保持当前 owner。
- 实施阶段允许受控过渡适配层；过渡适配层必须由 contract/registry/navigation owner 管理，带明确移除条件，并从新 definition 派生当前 dispatch path。

## Adapter Definition Contract

Adapter definition 是 registry-facing authoring unit，而不是额外 metadata 镜像。一个 definition/factory 必须让以下事实从同一位置可达：

- adapter identity、manifest metadata 和 format descriptors；
- required operation handler handles：`outline`、`read`、`find`、`info`；
- adapter-owned native option declarations，包括 source extraction metadata、defaults、validation facts、operation applicability 和 typed handoff/accessor binding metadata；
- full-read capability group，包括 support declaration、content hook、cost measurement hook 和 result facts hook；
- adapter-private construction state 或 handler context 的绑定方式，只要该状态不泄露为 core-owned parser/ref/pagination contract。

Static registry 只补充 core-owned implementation source 和 registry placement。Registry、CLI native option catalog、adapter inspection、navigation selected declaration registration、full-read pre-dispatch 和 dispatch 从 selected definition 读取 adapter-owned facts。实施阶段的过渡适配层由 shared owner 生成当前 dispatch path，并以删除分散声明入口为完成条件。

Adapter 内部可以按模块、helper 或 private builder 拆分实现，但这些拆分只服务 adapter-private construction。对 registry、core、CLI、navigation 和 dispatch 暴露的 adapter-owned facts 必须通过同一个 definition/factory 汇出；adapter crate 不能把 helper、trait method 或额外 catalog 作为第二个 registry-facing 声明入口。

Internal typed handoff/accessor 是 handler-facing contract。External `OperationArguments.options` 可以作为 protocol-stable request facts 保留；当 adapter declaration 提供 typed binding 时，handler 应消费 prepared typed value/accessor。

## Decisions

### Decision 1: 使用高层 operation handler

决定保留 `outline/read/find/info` 作为默认 adapter layer 的必需扩展点，并把 parser、ref、分页和格式事实留在 adapter 内部。替代方案是开放 parser/ref/pagination/rendering 等细粒度 hook，但这会扩大 public contract 面积，并要求 core/navigation 理解 adapter 私有语义。影响是 adapter 内部仍可自由抽象，shared layers 只验证 operation-level contract。

### Decision 2: 引入集中 adapter definition/descriptor

决定让 static registry 注册一个 registry-facing descriptor/definition，以替代当前从 adapter handle 分散读取 manifest、options、operation support 和 full-read hooks 的方式。descriptor 应聚合 identity、format descriptors、native option declarations、operation handlers 和 capability groups；manifest/probe 仍只描述 capability 和 format support，implementation source 仍由 static registry 记录。过渡适配层从 descriptor 生成当前 dispatch path，并以 descriptor-first path 覆盖 registry、CLI catalog、navigation 和 dispatch 为退出条件。

### Decision 3: Native option 采用 typed handoff/accessor

决定保留 adapter-owned option declarations 作为事实源，仍由 navigation 完成 explicit/project/user/built_in 来源解析、默认值和基础类型/range 校验；handler 消费内部 typed native option values 或 adapter-specific typed accessor。External `OperationArguments.options` 作为 protocol-stable request facts 由 request construction 生成；handler-facing typed option API 由 selected definition 的 binding metadata 决定。

### Decision 4: Full-read 接口组进入 capability group

决定把 non-structured full-read 的 support declaration、content hook、cost measurement hook 和 result facts hook 表达为一个 full-read capability group。Navigation 根据 descriptor 中声明的 capability support 调用对应能力；fallback 和 unsupported behavior 由 navigation/input policy owner 明确。该 group 覆盖当前非结构化阅读接口，而不是只处理未来扩展。

## Risks / Trade-offs

- [Risk] Descriptor 可能只是把现有散点换个位置，而没有降低 adapter 作者负担 -> Mitigation: 实现阶段必须为 Markdown adapter 写出最小 authoring path，并用 tests 覆盖 registry、CLI catalog、declaration registration、full-read pre-dispatch 和 handler typed option consumption 都从同一个 definition 派生。
- [Risk] 过渡适配层滞留为第二套接口 -> Mitigation: 过渡适配层由 shared owner 管理，任务清单记录移除条件；新 adapter authoring path 的验收标准是 adapter crate 只声明 definition。
- [Risk] Typed handoff 可能与现有 protocol `arguments.options` JSON shape 冲突 -> Mitigation: `arguments.options` 作为 request facts 保持 protocol-stable 生成；typed accessor 作为 internal dispatch/handoff 结构处理，只有需要改变 observable shape 时才同步 schema/example。
- [Risk] Capability group 设计过早抽象 -> Mitigation: group 复杂度只服务当前 full-read hook set。
- [Risk] 在途 readable-view adapter hook change 也会触碰 adapter optional capability -> Mitigation: 本 change 明确不定义 readable rendering hook；若两个 change 同时推进，按各自 capability owner 合并 optional capability 命名和 registry exposure。

## Migration Plan

1. 在 `docnav-adapter-contracts` 中引入 adapter definition、operation handler handle、native option typed handoff/accessor binding 和 full-read capability group 类型，并定义受控过渡适配层的 owner 与移除条件。
2. 先让 static registry 接收 definition，并让 adapter inspection、CLI native option catalog 和 navigation selected declaration registration 从同一 definition 读取 adapter-owned facts。
3. 将 Markdown adapter 迁移为 descriptor-first 声明，集中 identity、formats、native options、operation handlers 和 full-read capability，证明 adapter crate 只维护一个 registry-facing declaration。
4. 将 navigation 的 selected adapter declaration registration 改为从 descriptor 读取 declarations，并在 request construction/dispatch 前生成内部 typed native option handoff。
5. 更新 static registry、adapter inspection 和 doctor 检查，确保 implementation source 仍来自 core static registry。
6. 移除分散声明入口前，运行 adapter contract、navigation input resolution、Markdown adapter 和 core smoke 验证。

Recovery 策略是让过渡适配层从单一定义生成当前 dispatch path；若 typed handoff 迁移需要拆分推进，可以先让 definition 继续生成 protocol-stable JSON options，并把 typed accessor 延后到后续小步。恢复路径仍以单一定义为事实源，避免 adapter 作者重新维护多套 declarations。

## Open Questions

无未回答开放问题，可以进入实现阶段。实现前按 `tasks.md` 的实施前确认检查 scope、owner 和 delta 一致性。
