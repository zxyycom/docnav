本 proposal 的主承诺是让 adapter 作者只在一个 registry-facing adapter definition/descriptor 中声明 adapter 事实：identity、format metadata、native options、operation handlers 和 full-read capability group。Core、CLI inspection、navigation resolution 和 dispatch 消费并传递这个 definition 派生出的事实，adapter-owned interface 只有一个长期 authoring 入口。实施和归档前，主规范与当前二进制状态仍以 `docs/`、代码和测试为准。

## Why

当前 adapter contract 已经倾向高层 operation handler，但 adapter 作者仍需要在多个位置配置 identity、manifest、native option processing、operation support、full-read support/content/cost/facts hooks，并在 handler 内从 generic options bag 重新取值和防御校验。这个分散触点让同一 adapter fact 同时出现在 adapter trait、static registry、CLI native option catalog、navigation field registration、full-read pre-dispatch hooks 和 handler option lookup 中，提高新 adapter 接入成本，也削弱“navigation 负责 input resolution，adapter 负责格式语义”的边界可读性。

## What Changes

- 将 adapter 对 core/navigation 暴露的 authoring surface 收敛为一个 registry-facing adapter definition/descriptor，统一承载 identity、manifest metadata、format descriptors、native option declarations、capability declarations 和 operation handler handles。
- 规定 static registry、adapter inspection、CLI native option catalog、navigation selected-adapter declaration registration、full-read pre-dispatch 和 dispatch 从该 definition 派生 adapter-owned facts；实施阶段的过渡适配层由 contract/registry/navigation owner 管理，并带移除条件。
- 保持必需文档操作为高层 `outline`、`read`、`find`、`info` handler，不新增 parser、ref、pagination、rendering 等细粒度 hook。
- 将 native option 的声明与消费改为内部 typed handoff/accessor 方向：adapter 在 definition 中声明 option contract，navigation 负责来源解析、默认值和基础校验，handler 接收 adapter-specific typed native option values 或 accessor。
- 将当前 full-read 接口组按 capability group 表达：support declaration、content hook、cost measurement hook 和 result facts hook 由同一个 full-read group 聚合。
- 保持 `ref`、format parsing、navigation strategy、pagination result 和 adapter-owned facts 由 adapter 拥有；core/navigation 只做 routing、selected adapter declaration registration、request construction 和 dispatch。
- 范围边界：本 change 聚焦 core-linked adapter definition contract；第三方动态插件系统、adapter process runtime、adapter implementation source、`protocol-json` / `readable-json` / `readable-view` document success payload 和现有 Markdown ref grammar 保持各自 owner。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `adapter-contract`: 修改 adapter library interface 的组织方式，规定 registry-facing descriptor、高层 operation handler、内部 typed native option handoff/accessor 和 capability group 的 contract 边界。
- `navigation-input-resolution`: 修改 selected adapter declaration registration 与 request construction/dispatch 的描述，规定 navigation 消费 adapter definition 中的 declarations 并向 handler 交付 typed native option values。

## Impact

- 影响 `docnav-adapter-contracts` 的 adapter definition、受控过渡适配层、native option declaration wrapper 和 full-read capability group 表达。
- 影响 `docnav-navigation` 的 selected adapter declaration registration、typed option extraction、request construction/dispatch 的内部 handoff 形状。
- 影响 `docnav` static registry 对 built-in adapter 的注册方式，以及 adapter inspection/doctor 中展示的 metadata 来源。
- 影响 `docnav-markdown` 的 adapter definition 声明、native option 消费方式和 full-read capability 暴露方式。
- 实现阶段默认保持 observable protocol/readable/schema/example 行为稳定，并通过 regression/smoke 证明；若最终需要改变可观察 output 或 schema，必须同步更新对应 docs、schemas、examples 和 tests。

## Success Criteria

- 新 adapter 的 registry-facing 接入路径可以用一个 definition/factory 完成；identity、manifest/formats、native options、required operation handlers 和 full-read capability group 只在 adapter 作者维护的一个长期入口声明。Adapter 内部可以拆 helper/module，但对 registry、core、CLI、navigation 和 dispatch 只暴露这个 definition/factory。
- Static registry 登记 adapter implementation source 和 adapter definition；adapter-owned option semantics、full-read support/content/cost/facts 和 handler handles 都从 definition 传递给 consumers。
- Navigation 在 selected adapter 确定后从同一个 definition 注册 native option declarations，并在 dispatch 前准备 typed native option handoff/accessor。
- Markdown adapter 迁移后，`max_heading_level` 的基础类型、默认值和 range 校验由 declaration/resolution 证明，handler 消费 typed handoff/accessor。
- 现有 document output wrapper、schema/example、Markdown ref、pagination 和 operation result semantics 保持稳定，除非实现阶段明确提出并同步更新对应 owner。
