本 design 说明如何把 linked adapter 扩展面收敛为 registry-facing descriptor、高层 operation handler、内部 typed native option handoff/accessor 和 capability group；当前 change 只在 `openspec/changes/streamline-adapter-definition-contract/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 的稳定导航链路是 `outline -> ref -> read`。当前主规范已经把 owner 边界拆清楚：`docnav` core 拥有 CLI、static registry、输出模式和错误映射；`docnav-navigation` 拥有 config source loading、adapter selection、selected adapter declarations、typed resolution、request construction 和 dispatch；格式 adapter 拥有格式识别、解析、ref、分页结果和格式语义结果。

现有实现和规范已经避免了 parser/ref/pagination 级别的细粒度 hook，但 adapter 作者仍需要在多个函数和 helper 中配置 manifest、native options 和 capability facts，并在 handler 内从 generic options bag 取值。这个 change 的设计目标是收紧 adapter authoring surface，而不是重新分配 core/navigation/adapter 的长期 owner。

## Goals / Non-Goals

**Goals:**

- 为 built-in linked adapters 定义 registry-facing adapter definition/descriptor 形态，作为 registry 和 navigation 消费的单一入口。
- 保持 `outline`、`read`、`find`、`info` 为必需高层 operation handlers。
- 让 adapter-owned native options 通过 declaration 表达 public contract，通过 navigation 完成来源解析、默认值和基础校验，通过内部 typed handoff/accessor 进入 handler。
- 把可选 hooks 按 capability group 表达，先覆盖 non-structured full-read hook set。
- 保持 `protocol-json`、`readable-json`、`readable-view`、ref opacity、pagination semantics 和 adapter implementation source 兼容。

**Non-Goals:**

- 不引入第三方动态插件系统、独立 adapter process runtime 或 service-mode adapter source。
- 不把 parser、AST、ref parser、pagination algorithm、readable rendering 等 adapter 内部细节开放成 core-owned hook。
- 不要求所有 adapter 共享同一个内部 domain model。
- 不改变 Markdown ref grammar、Markdown outline/read/find/info 业务语义或现有 document output wrapper。

## Decisions

### Decision 1: 使用高层 operation handler，不放开全细节 hook

决定保留 `outline/read/find/info` 作为默认 adapter layer 的必需扩展点，并把 parser、ref、分页和格式事实留在 adapter 内部。替代方案是开放 parser/ref/pagination/rendering 等细粒度 hook，但这会扩大 public contract 面积，并要求 core/navigation 理解 adapter 私有语义。影响是 adapter 内部仍可自由抽象，shared layers 只验证 operation-level contract。

### Decision 2: 引入集中 adapter definition/descriptor facade

决定让 static registry 注册一个 registry-facing descriptor/facade，而不是从 adapter handle 上分散读取 manifest、options、operation support 和 optional hooks。descriptor 应聚合 identity、format descriptors、native option declarations、operation handlers 和 capability groups；manifest/probe 仍只描述 capability 和 format support，不声明 implementation source。替代方案是继续沿用 trait methods 平铺能力，但 adapter authoring 入口会继续分散，后续能力新增也更容易横向长 hook。

### Decision 3: Native option 采用 typed handoff/accessor

决定保留 adapter-owned option declarations 作为事实源，仍由 navigation 完成 explicit/project/user/built_in 来源解析、默认值和基础类型/range 校验；handler 消费内部 typed native option values 或 adapter-specific typed accessor。替代方案是继续把 `OperationArguments.options` 作为 generic JSON bag 交给 handler；该方案虽然兼容简单，但会让 handler 重复基础校验，并削弱“typed resolution 已完成”的边界。

### Decision 4: Optional hook 进入 capability group

决定把 non-structured full-read 的 content、cost measurement 和 result facts 表达为一个 full-read capability group。navigation 只能根据 descriptor 中声明的 capability support 调用对应能力；fallback 必须由明确 owner 提供。替代方案是继续在 `Adapter` trait 上追加平铺 hook method；该方案短期小，但每新增一个可选能力都会扩大 trait surface。

## Risks / Trade-offs

- [Risk] Descriptor facade 可能只是把现有散点换个位置，而没有降低 adapter 作者负担 -> Mitigation: 实现阶段必须为 Markdown adapter 写出最小 authoring path，并用 tests 覆盖 registry、declaration registration 和 handler typed option consumption。
- [Risk] Typed handoff 可能与现有 protocol `arguments.options` JSON shape 冲突 -> Mitigation: 默认保持 external protocol JSON 兼容；typed accessor 作为 internal dispatch/handoff 结构处理，只有需要改变 observable shape 时才同步 schema/example。
- [Risk] Capability group 设计过早抽象 -> Mitigation: 只先覆盖已有 full-read hook set，不为未来未知能力预留复杂框架。
- [Risk] 在途 readable-view adapter hook change 也会触碰 adapter optional capability -> Mitigation: 本 change 明确不定义 readable rendering hook；若两个 change 同时推进，按各自 capability owner 合并 optional capability 命名和 registry exposure。

## Migration Plan

1. 在 `docnav-adapter-contracts` 中引入 descriptor/facade 与 capability group 类型，同时保留旧 trait method 行为的兼容 shim 或等价过渡路径。
2. 将 Markdown adapter 迁移为 descriptor-first 声明，集中 identity、formats、native options、operation handlers 和 full-read capability。
3. 将 navigation 的 selected adapter declaration registration 改为从 descriptor 读取 declarations，并在 request construction/dispatch 前生成内部 typed native option handoff。
4. 更新 static registry、adapter inspection 和 doctor 检查，确保 implementation source 仍来自 core static registry。
5. 移除或收窄过渡 shim 前，运行 adapter contract、navigation input resolution、Markdown adapter 和 core smoke 验证。

Rollback 策略是保留旧 handler path 到 descriptor facade 的适配层；若 typed handoff 迁移出现兼容问题，可以先让 descriptor 暴露现有 JSON options，同时把 typed accessor 延后到第二步。

## Open Questions

无未回答开放问题，可以进入实现前审计。实现前审计仍需确认本 change 是否只包含未审核临时 artifacts，且没有提前修改主规范或应用代码。
