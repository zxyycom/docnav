本 design 承接 proposal 的方向，说明已确定的 adapter 边界、实现决策、剩余设计问题、实施顺序和风险处理。

## Context

本 change 的核心问题不是安全性、包体积或 adapter selection algorithm，而是默认架构路线的复杂度收益不匹配。独立 adapter 运行时制品要求 core 和 SDK 长期维护动态注册、生命周期管理、跨进程通信、manifest/schema、兼容策略和调试诊断；这些不是 Docnav 当前主要产品收益。

本 design 从以下已确定方向开始：默认 document operation adapter implementation 随 `docnav` core release 交付，并通过 static adapter registry 被 `docnav` 调用；adapter layer 仍保留格式语义 ownership。

目标形状：

```text
docnav core CLI
  -> static adapter registry
  -> adapter-layer workspace crates
  -> adapter-owned format semantics
  -> core protocol/readable/diagnostic projection
```

后续实现中采用 `docnav-navigation` + `docnav-adapter-contracts` 的拆分：`docnav-navigation` 作为内部 document operation orchestration layer，集中调配 `outline/read/find/info`；`docnav-adapter-contracts` 承载 adapter layer interface definitions 和共享 contract types。这个拆分是 SDK 退场后的内部支撑结构，不是本 change 的根因，也不应把 public contract 范围扩大为新的运行时 adapter SDK。

本 change 用四个边界定义“内置但不合并”：

- 发布边界：adapter 不再作为默认独立包体、外部 executable 或运行时制品交付；默认 adapter implementation 是 `docnav` core release artifact 的组成部分。
- 执行来源边界：默认 document operation 只从当前 core release 的 static adapter registry 获取 implementation；项目配置、用户配置、历史 artifact record 或命令路径不能提供 implementation。
- 代码边界：adapter layer 仍是独立职责边界，拥有格式解析、ref、navigation、pagination 和 native option；core 不吸收这些格式语义。
- 公开契约边界：`protocol-json`、readable output、diagnostic projection 和 ref opacity 仍由现有 owner 维护；本 change 不重写输出 shape。

Owner 分工：

- Proposal owns motivation、scope 和 high-level trade-off。
- Spec deltas own observable requirements and scenarios。
- Design owns implementation structure、implementation sequencing、decision gates 和 unresolved risk handling。
- Tasks own execution order and verification gates。

## Goals / Non-Goals

**Goals:**

- 默认 document operation adapter implementation 收敛到随 core release 交付的 adapter-layer workspace crates。
- Adapter-layer library 保持独立 workspace crate，不作为默认独立包体交付。
- 默认 release 包含全部内置 adapter；默认 adapter set 不通过 feature gate 裁剪。
- Core 使用一个统一 static adapter registry 注册内置 adapter identity、metadata、source-level native option registry entries、operation handler bindings 和 implementation handle。
- 默认执行来源不再来自独立 adapter package、外部 executable、command path 或历史 adapter artifact record。
- 动态 adapter registration 和 artifact management commands 从默认 CLI surface 删除。
- Adapter layer 继续作为代码和契约边界存在，并保留 parser、navigation、ref、pagination 和 native option ownership。
- 现有 SDK 退出外部 runtime adapter SDK 定位，并收敛为 `docnav-navigation` 内部 operation orchestration 和 `docnav-adapter-contracts` adapter interface definitions。
- Protocol/readable 输出契约和 diagnostic projection 保持稳定。

**Non-Goals:**

- 不把 Markdown 或其它格式 parser 合并进 core 业务模块。
- 不在本 change 中完成插件市场、远程下载或第三方 adapter SDK/runtime model。
- 不保留 direct adapter CLI 或 adapter `invoke` 作为默认 surface。
- 不把 local service mode 作为 adapter implementation source；service mode 后续只作为 core service 性能与缓存问题讨论。
- 不引入新 protocol output shape 来表达内部 static adapter registry。
- 不重写完整 adapter selection algorithm；本 change 只定义 implementation source boundary。
- 不把包体积作为本 change 的首要优化目标。

## Decisions

### Decision 1: Adapter implementation source 是 core release 内置 workspace crates

`docnav` 的默认 document operation path 只使用当前 core release 中包含的 adapter-layer workspace crates 作为 implementation source。Adapter crates 是独立 workspace crates，直接作为 core release 的组成部分编译和交付，但不再作为默认独立运行时包体。

默认 release 包含全部内置 adapter。默认 adapter set 不使用 feature gate 裁剪，以避免 release profile、selection、测试矩阵和用户诊断复杂化。

### Decision 2: Static adapter registry 是 compile/package-time 事实源

Core 维护一个统一 static adapter registry，注册当前 release 内置 adapter crates 的 adapter id、identity metadata、format metadata、source-level native option registry entries、operation handler bindings 和 adapter layer implementation handle。

该 registry 是 compile/package-time 事实源，不是运行时动态注册表。Adapter crate 需要实现指定 adapter layer interface，并在 core registry 中显式注册自身需求和能力。

### Decision 3: Adapter layer boundary 保持独立

Adapter implementation 被 core release 包含并由 `docnav` 直接作为库调用，但 adapter layer 仍是代码和契约边界。Adapter 仍拥有 parser、format detection、navigation strategy、ref generation/parsing、pagination result 和 native option semantics。

Core 可以调用 adapter layer API，但不能解析 adapter ref、重建格式结构、解释 native option 或合成格式专属 `options`。Native option registry entries 只提供 source 分类、merge metadata 和 handoff 事实；unsupported option、type mismatch 和 range invalid 由消费该 option 的 adapter 产生结构化诊断。Ref 在 core 和接入层仍是 opaque pass-through value。

### Decision 4: 动态 adapter management 和历史 registration 材料删除

`docnav adapter install/register/update/remove` 等围绕运行时制品和动态注册的命令不进入新默认 CLI surface。`docnav adapter list` 保留为 static registry inspection；`doctor` 检查 static registry、core 配置和 adapter layer 可用性。

历史 `.docnav/adapters.json`、adapter artifact records、command path registry 和相关示例/fixture 对新默认路径是冗余材料，应从默认配置、docs、schema/examples 和测试中移除。Document operation、adapter selection、doctor 和 `init` 都不再依赖历史 adapter registration 材料。

### Decision 5: Direct adapter CLI / invoke 不作为默认 surface

Adapter layer 不再需要独立 direct CLI 或 `invoke` 来参与默认 document operation。默认路径只要求 adapter layer 实现指定 interface，并通过 core static registry 接入。

现有 adapter direct CLI / invoke 测试不能继续作为默认执行路径的证明。本 change 不保留非默认 adapter 本地调试入口；adapter 行为通过黑盒 CLI 测试、白盒 adapter/core 测试和 core 调用路径证明。

### Decision 6: 现有 SDK 退出外部 runtime adapter SDK 定位，并拆成 navigation + adapter contracts

现有 SDK 不再作为外部 adapter runtime SDK 继续设计。它不再负责让独立 adapter 包通过动态注册、manifest、命令路径和跨进程协议接入默认 document operation path。

后续内部调配层命名为 `docnav-navigation`，用于集中调配 `outline/read/find/info` 等流程。Adapter interface definitions 和共享 contract types 拆到 `docnav-adapter-contracts`，让 adapter crates 依赖稳定、较小的接口边界，而不是依赖完整 operation orchestration。

默认不新增独立 `docnav-adapter-support` crate。只有在实现中出现跨 adapter 的重复工具且放入 `docnav-adapter-contracts` 会污染 contract boundary 时，才重新评估是否拆出 support crate。

本 change 最终采用 operation-handler granularity：adapter handle 暴露 static descriptor metadata、probe check、source-level native option registry entries，以及 `outline/read/find/info` operation handlers。早期 primitive split（ref splitter、locator、format probe validation、parser/navigation primitives）会要求 `docnav-navigation` 组合格式内部步骤，把 parser/ref/navigation 细节跨过 adapter/core 边界暴露出来；当前实现没有证明这种细分能带来产品收益。Operation handlers 保留 adapter-owned parser、ref、navigation、pagination 和 native option ownership，同时让 `docnav-navigation` 只负责 request construction、handler dispatch 和 operation flow。

### Decision 7: Local service mode 后置为 core service 性能问题

`enable-local-core-adapter-service-mode` 仍可能有价值，但它的价值来自降低启动成本、复用缓存和改善长会话性能，不来自 adapter implementation source。该 change 应重写为 core service/performance/caching 讨论，并排在本架构修正之后。

本 change 不阻塞未来 core service；但 service 不能重新引入默认动态 adapter 注册或独立 adapter artifact source。

## Risks / Trade-offs

- [Risk] 第三方或本地自定义 adapter 独立包扩展路径改变。→ Mitigation: 明确这是默认路径的 breaking change；新增 adapter 的默认开发方式是修改 workspace crate、注册到 core static registry 并重新编译。
- [Risk] 包体积增加。→ Mitigation: 默认 release 包含全部内置 adapter；后续通过压缩、release engineering 或性能优化处理体积，不恢复运行时动态制品管理。
- [Risk] 现有 adapter SDK 抽象不匹配新方案。→ Mitigation: 先移除外部 runtime SDK 定位，再把剩余能力收敛为 `docnav-navigation` 和 `docnav-adapter-contracts`；support crate 不作为默认拆分。
- [Risk] 历史 adapter 配置或示例残留导致误导。→ Mitigation: tasks 中清理配置创建、schema/examples、fixtures、docs 和测试引用。
- [Risk] Selection algorithm 细节继续抢占 change 主线。→ Mitigation: spec delta 只定义 implementation source boundary；阶段排序、诊断字段和候选证据留给相邻 contract 或实现任务处理。
- [Risk] OpenSpec 在途 change 与新方向冲突。→ Mitigation: tasks 中先处置 adapter management、local service mode 和 entry/source resolution 相关 change，再进入实现。

## Implementation Sequence

1. 处置相关在途 change，标记保留、改写或替代路径。
2. 同步 owner 主规范：`docs/architecture.md`、`docs/cli.md`、`docs/adapter-contract.md`、`docs/testing.md`，以及需要的 schema/example/fixture。
3. 创建或调整 adapter-layer workspace crates，确保默认 release 直接包含全部内置 adapter，且默认 adapter set 不依赖 feature gate。
4. 实现 core static adapter registry，并把 adapter implementation source、inspection 和 doctor 统一到该 registry。
5. 移除动态 adapter registration/artifact management CLI surface 和历史 adapter registration 材料。
6. 将现有 SDK 残留能力收敛到 `docnav-navigation` 和 `docnav-adapter-contracts`，采用 operation-handler adapter interface granularity。
7. 更新测试，覆盖 built-in static registry source boundary、dynamic management command removal、historical registration material removed、protocol/readable output 不变和 ref opaque pass-through。

Rollback 策略：如实现期间发现 core release 内置 adapter layer 无法满足已实现 adapter 的必要边界，停止实现并回到本 change 更新 design/spec。
