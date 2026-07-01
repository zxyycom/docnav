本 design 承接 proposal 的方向，说明已确定的 adapter 边界、实现决策、剩余设计问题、实施顺序和风险处理。

## Context

Proposal owns 动机、既有决策影响、当前发现和 high-level decision。本 design 从以下已确定方向开始：默认 document operation adapter implementation 随 `docnav` core release 交付，并通过 static adapter registry 被 `docnav` 调用；adapter layer 仍保留格式语义 ownership。

目标形状：

```text
docnav core
  -> static adapter registry
  -> adapter-layer workspace crates
  -> adapter-owned parse/ref/navigation
  -> protocol/readable/diagnostic projection
```

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
- 默认 release 包含全部内置 adapter；默认 adapter set 不通过 feature gate 裁剪。
- Core 使用一个统一 static adapter registry 注册内置 adapter。
- 默认执行来源不再来自独立 adapter package、外部 executable、command path 或历史 adapter artifact record。
- 动态 adapter registration 和 artifact management commands 从默认 CLI surface 删除。
- Adapter layer 继续作为代码和契约边界存在，并保留 parser、navigation、ref、pagination 和 native option ownership。
- Protocol/readable 输出契约和 diagnostic projection 保持稳定。

**Non-Goals:**

- 不把 Markdown 或其它格式 parser 合并进 core 业务模块。
- 不在本 change 中完成插件市场、远程下载或第三方 adapter SDK/runtime model。
- 不保留 direct adapter CLI 或 adapter `invoke` 作为默认 surface。
- 不把 local service mode 作为 adapter implementation source；service mode 后续只作为 core service 性能与缓存问题讨论。
- 不引入新 protocol output shape 来表达内部 static adapter registry。
- 不把包体积作为本 change 的首要优化目标。

## Decisions

### Decision 1: Adapter implementation source 是 core release 内置 workspace crates

`docnav` 的默认 document operation path 只使用当前 core release 中包含的 adapter-layer workspace crates 作为 implementation source。Adapter crates 是独立 workspace crates，直接作为 core release 的组成部分编译和交付。

默认 release 包含全部内置 adapter。默认 adapter set 不使用 feature gate 裁剪，以避免 release profile、selection、测试矩阵和用户诊断复杂化。

### Decision 2: Static adapter registry 是默认候选事实源

Core 维护一个统一 static adapter registry，注册当前 release 内置 adapter crates 的 adapter id、identity metadata、format metadata、capabilities 和 adapter layer implementation。Adapter selection、inspection 和 doctor 都从这个 registry 获取候选。

该 registry 是 compile/package-time 事实源，不是运行时动态注册表。Adapter crate 需要实现指定 adapter layer interface，并在 core registry 中显式注册其需求和能力。

### Decision 3: Adapter layer boundary 保持独立

Adapter implementation 被 core release 包含并由 `docnav` 调用，但 adapter layer 仍是代码和契约边界。Adapter 仍拥有 parser、format detection、navigation strategy、ref generation/parsing、pagination result 和 native option semantics。

Core 可以调用 adapter layer API，但不能解析 adapter ref、重建格式结构、解释 native option 或合成格式专属 `options`。Ref 在 core 和接入层仍是 opaque pass-through value。

### Decision 4: 动态 adapter management CLI surface 删除

`docnav adapter install/register/update/remove` 等围绕运行时制品和动态注册的命令不进入新默认 CLI surface。`docnav adapter list` 保留为 static registry inspection；`doctor` 检查 static registry、core 配置和 adapter layer 可用性。

`implement-docnav-adapter-management` 需要改写为删除动态注册/制品管理命令，并把剩余范围收敛到 inspection 和 health check。用户要增加 adapter 时，通过修改 workspace crate、注册到 static registry 并重新编译 core，而不是运行时安装或注册。

### Decision 5: Direct adapter CLI / invoke 不作为默认 surface

Adapter layer 不再需要独立 direct CLI 或 `invoke` 来参与默认 document operation。默认路径只要求 adapter layer 实现指定 interface，并通过 core static registry 接入。

现有 adapter direct CLI / invoke 测试不能继续作为默认执行路径的证明。若未来需要 adapter 独立调试工具，应作为非默认开发工具单独设计，不参与默认 SDK 或 CLI contract。

### Decision 6: 历史 adapter registration 材料从默认路径移除

历史 `.docnav/adapters.json`、adapter artifact records、command path registry 和相关示例/fixture 对新默认路径是冗余材料，应从默认配置、docs、schema/examples 和测试中移除。

Document operation、adapter selection、doctor 和 `init` 都不再依赖历史 adapter registration 材料。实现时需要清理创建、读取、校验和示例引用，避免它们继续暗示外部 adapter artifact 是合法来源。

### Decision 7: Local service mode 后置为 core service 性能问题

`enable-local-core-adapter-service-mode` 仍可能有价值，但它的价值来自降低启动成本、复用缓存和改善长会话性能，不来自 adapter implementation source。该 change 应重写为 core service/performance/caching 讨论，并排在本架构修正之后。

本 change 不阻塞未来 core service；但 service 不能重新引入默认动态 adapter 注册或独立 adapter artifact source。

### Decision 8: 现有 SDK 不再是 runtime adapter SDK

现有 SDK 不再作为外部 adapter runtime SDK 继续设计。它应转向 adapter layer 使用的 common utilities、adapter interface definitions，或成为 read/navigation 等命令的集中调配层的一部分。

SDK 的 owner、命名和 API 需要在实现 adapter API 前重新审计。Adapter layer 的理想接口可能进一步收缩到 ref 拆分器、定位器等 building blocks，由集中调配层负责编排 read/navigation；这个拆分保留为剩余设计问题。

## Remaining Design Questions

以下问题仍需在实现 adapter API 前收敛，不能由 tasks 默认假设：

1. **adapter support 与 orchestration 的拆分**：现有 SDK 应改造成 adapter support crate、adapter contract crate、core orchestration layer，还是其中几者的组合？需要确定 owner、crate 命名、public/private API 和测试入口。
2. **最小 adapter layer interface**：adapter crate 最终只实现 ref splitter、locator 和格式支持判断，还是继续实现 outline/read/find/info 等 operation handler？需要在不破坏 adapter-owned 格式语义的前提下，确定 core orchestration 和 adapter building blocks 的边界。
3. **非默认调试入口**：默认不保留 direct CLI / `invoke`；若仍需要本地调试工具，需要单独定义触发方式、输出契约和测试范围，且不能回到默认 adapter SDK 主路径。

Decision gate：1-2 必须在实现 adapter crate/API 前收敛到 design、spec 或 tasks；3 可以作为后续开发体验 change，不阻塞默认 execution source 的边界修正。

## Risks / Trade-offs

- [Risk] 第三方或本地自定义 adapter 独立包扩展路径改变。→ Mitigation: 明确这是默认路径的 breaking change；新增 adapter 的默认开发方式是修改 workspace crate、注册到 core static registry 并重新编译。
- [Risk] 包体积增加。→ Mitigation: 默认 release 包含全部内置 adapter；后续通过压缩、release engineering 或性能优化处理体积，不恢复运行时动态制品管理。
- [Risk] 现有 adapter SDK 抽象不匹配新方案。→ Mitigation: 把 SDK 重定位为 adapter support/interface/orchestration 问题，并在实现 adapter API 前收敛。
- [Risk] 历史 adapter 配置或示例残留导致误导。→ Mitigation: tasks 中清理配置创建、schema/examples、fixtures、docs 和测试引用。
- [Risk] OpenSpec 在途 change 与新方向冲突。→ Mitigation: tasks 中设置阻塞级审计，未完成前不得执行实现任务。

## Implementation Sequence

1. 审计 proposal、design、spec delta 和 tasks，确认本阶段只更新当前 change 目录，尚未提前修改主规范、schema、examples 或代码。
2. 收敛 Remaining Design Questions 中的 adapter support/orchestration 和最小 adapter interface。
3. 审计相关在途 change，标记保留、改写或替代路径。
4. 同步 owner 主规范：`docs/architecture.md`、`docs/cli.md`、`docs/adapter-contract.md`、`docs/testing.md`，以及需要的 schema/example/fixture。
5. 创建或调整 adapter-layer workspace crates，确保默认 release 直接包含全部内置 adapter，且默认 adapter set 不依赖 feature gate。
6. 实现 core static adapter registry，并把 adapter selection、inspection 和 doctor 统一到该 registry。
7. 移除动态 adapter registration/artifact management CLI surface 和历史 adapter registration 材料。
8. 按已收敛的 support/orchestration 设计调整 SDK、adapter interface 和相关测试。
9. 更新测试，覆盖 built-in static registry selection、dynamic management command removal、historical registration material removed、protocol/readable output 不变和 ref opaque pass-through。

Rollback 策略：如实现期间发现 core release 内置 adapter layer 无法满足已实现 adapter 的必要边界，停止实现并回到本 change 更新 design/spec。
