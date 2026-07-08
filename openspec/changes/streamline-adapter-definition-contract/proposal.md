本 proposal 目标是收敛 linked adapter 扩展面，把分散配置整理为 registry-facing descriptor、高层 operation handler、内部 typed native option handoff/accessor 和 capability group；当前 change 只在 `openspec/changes/streamline-adapter-definition-contract/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

当前 adapter contract 已经倾向高层 operation handler，但 adapter 作者仍需要在多个位置配置 identity、manifest、native option processing 和 capability hooks，并在 handler 内从 generic options bag 重新取值和防御校验。这个分散触点会提高新 adapter 接入成本，也会削弱“navigation 负责 input resolution，adapter 负责格式语义”的边界可读性。

## What Changes

- 将 adapter 对 core/navigation 暴露的扩展面收敛为一个 registry-facing adapter definition/descriptor，统一承载 identity、manifest metadata、format descriptors、native option declarations、capability declarations 和 operation handlers。
- 保持必需文档操作为高层 `outline`、`read`、`find`、`info` handler，不新增 parser、ref、pagination、rendering 等细粒度 hook。
- 将 native option 的声明与消费改为内部 typed handoff/accessor 方向：adapter 仍声明 option contract，navigation 仍负责来源解析与基础校验，handler 不再通过 untyped JSON bag 重做基础类型/range 校验。
- 将可选 hook 按 capability group 表达，例如 non-structured full-read capability 内部聚合 content、cost measurement 和 result facts 能力，避免平铺 hook 继续扩张。
- 保持 `ref`、format parsing、navigation strategy、pagination result 和 adapter-owned facts 由 adapter 拥有；core/navigation 只做 routing、selected adapter declaration registration、request construction 和 dispatch。
- 非目标：本 change 不引入第三方动态插件系统、不改变 adapter implementation source、不改变 `protocol-json` / `readable-json` / `readable-view` 的 document success payload、不改变现有 Markdown ref grammar。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `adapter-contract`: 修改 adapter library interface 的组织方式，规定 registry-facing descriptor、高层 operation handler、内部 typed native option handoff/accessor 和 capability group 的 contract 边界。
- `navigation-input-resolution`: 修改 selected adapter declaration registration 与 request construction/dispatch 的描述，规定 navigation 消费 adapter definition 中的 declarations 并向 handler 交付 typed native option values。

## Impact

- 影响 `docnav-adapter-contracts` 的 adapter trait/facade、native option declaration wrapper 和 optional capability 表达。
- 影响 `docnav-navigation` 的 selected adapter declaration registration、typed option extraction、request construction/dispatch 的内部 handoff 形状。
- 影响 `docnav` static registry 对 built-in adapter 的注册方式，以及 adapter inspection/doctor 中展示的 metadata 来源。
- 影响 `docnav-markdown` 的 adapter definition 声明、native option 消费方式和 full-read capability 暴露方式。
- 实现阶段默认保持 observable protocol/readable/schema/example 行为不变，并通过 regression/smoke 证明；若最终需要改变可观察 output 或 schema，必须同步更新对应 docs、schemas、examples 和 tests。
