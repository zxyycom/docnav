本 proposal 说明 Docnav 默认 document operation adapter path 改为 core release 内置 adapter-layer workspace crates 的原因、范围、边界和影响。

## Why

### 之前的决策

Docnav 之前把 adapter 设计成独立运行时制品，希望减少 core 包体积，并允许 adapter 在 core 之外安装、注册和更新。

这个取舍服务的是制品拆分和运行时扩展能力，但它把 adapter 从“格式能力层”推成了“需要 core 管理生命周期、发现机制、通信协议和兼容性的运行时制品”。

### 已造成的影响

独立运行时 adapter 制品带来的主要问题是复杂度收益不成比例。Core 要承担制品生命周期、能力发现、进程通信、版本兼容和失败恢复等管理面；adapter SDK 也被迫承接通信协议、CLI surface、manifest/schema、diagnostic、兼容策略和测试约束。

这些成本没有对应到 Docnav 当前最重要的产品收益。Docnav 的核心价值是稳定地完成 `outline -> ref -> read`，不是提供运行时插件平台。对当前项目和 adapter 开发者而言，新增或调整 adapter 后重新编译 `docnav` core 并不是高成本操作；相比之下，动态注册要求 core 和 adapter SDK 长期维护更复杂的制品边界，降低 adapter 开发体验、调试效率和交付速度。

### 现在的发现

Docnav 默认路径更需要实现简单、调用直接、边界清晰和开发效率稳定，而不是最大化运行时扩展能力。包体积不是此阶段的主导约束；为减少少量 adapter code 而引入完整运行时制品和 SDK 体系，收益和复杂度不匹配。

“内置 adapter”不等于把格式语义并入 core。真正要调整的是默认发布边界和默认执行来源：adapter 不再拥有独立的默认包体或运行时制品，而是作为 adapter-layer workspace crate 随 `docnav` core release 交付，并通过 core 的静态 registry 接入。Adapter layer 仍然是单独的代码和契约边界，继续拥有格式 parser、ref、navigation strategy、pagination 和 native option。

### 当前决定

本 change 接受一次明确的 breaking architecture correction：

- 默认 document operation implementation source 改为当前 core release 内置 adapter-layer workspace crates。
- 默认 release 包含全部内置 adapter；默认 adapter set 不通过 feature gate 裁剪。
- Core 维护一个统一静态 adapter registry，作为 adapter id、metadata、capabilities 和 adapter layer implementation 的 compile/package-time 事实源。
- `docnav adapter list` 保留为内置 adapter inspection；动态注册和制品管理命令从默认 CLI surface 删除。
- Adapter direct CLI / `invoke` 不作为默认 surface 保留。
- 历史 adapter 注册文件、artifact records、command path registry 和相关验证材料从默认路径移除。
- Adapter layer 不并入 core 业务模块，仍负责格式语义和 adapter-owned contract。

现有 adapter SDK 的职责从外部 runtime adapter SDK 转向 adapter support、adapter interface definition 或 core orchestration 支撑；具体拆分和最小 adapter interface 由 `design.md` 保留为实现前设计问题。

## What Changes

- **BREAKING**: `docnav` document operation adapter implementation 由 core release 内置 adapter-layer workspace crates 提供。
- **BREAKING**: 默认文档操作不再把独立 adapter package、外部 executable、command path 或历史 adapter artifact record 当作 implementation source。
- **BREAKING**: `adapter install`、`adapter register`、`adapter update` 和 `adapter remove` 不再是默认有效 CLI commands。
- Core 使用统一静态 adapter registry；adapter selection、adapter inspection 和 `doctor` 都以该 registry 为候选来源。
- Adapter selection 和 discovery 只评估 static registry metadata、显式输入和 adapter-owned support check。
- `init`、docs、schema/examples、fixtures 和 tests 不再创建或依赖历史 adapter registration 材料。
- Adapter ownership 继续保留：格式 adapter 仍拥有格式识别、parser、导航策略、ref 生成/解析、分页业务结果和格式原生 option 语义。
- 相关在途 change 必须按 core release 内置 adapter-layer workspace crates 和 static registry 方向审计后再继续。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `core-cli`: 改写 `docnav` adapter selection、adapter inspection、doctor、dynamic adapter management command surface 和 document operation 启动边界。
- `docnav-contracts`: 改写长期职责边界，把默认 adapter implementation source 定义为 core release 内置 adapter-layer workspace crates，并保留 adapter-owned parsing/ref/navigation 的 library boundary contract。

## Impact

- Affected executable: `docnav` core CLI。
- Affected adapter surfaces: adapter selection、adapter inspection、dynamic adapter management command removal、doctor/health reporting、document operation adapter layer dispatch。
- Affected shared contracts: adapter ownership、ref opacity、diagnostic projection、protocol/readable output projection。
- Affected OpenSpec work: `implement-docnav-adapter-management` 需要改为删除动态注册/制品管理并保留内置 adapter inspection；`enable-local-core-adapter-service-mode` 需要重写为 core service 性能与缓存问题；`separate-entry-pipeline-from-parameter-resolution` 需要确认入口分类继续使用 core release static adapter registry。
- Non-goal: 本 change 不把格式 parser、ref grammar 或格式原生 navigation 语义上移到 core。
- Non-goal: 本 change 不新增远程插件市场、外部 adapter SDK/runtime model 或默认开发者插件入口。
