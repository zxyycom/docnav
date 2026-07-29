**目标：沿现有静态 linked-adapter 架构交付内置 JSON 导航，并用第二种真实格式验证 adapter boundary。**

**状态：本文描述 candidate target。除 object source order 的 E1 实验外，当前 change 的方向与交付顺序已确认。当前 change 使用既有 generic `readable-view` 走通输出路径，格式专用自定义渲染由相连的后续 change 完成。实施从 `tasks.md` 0.5 关闭后开始；Current 能力仍以 `docs/`、代码和测试为准。**

## Why

当前静态 linked-adapter 架构只有 Markdown 一个真实实现，单一样本不足以区分共享 contract 与 Markdown-specific implementation。JSON 简单、常用、结构明确，使用 object key、array index 和 value tree，能够以可控范围检验 `probe/selection -> outline|find -> ref -> read`、`info`、full-read、registry 和 release path。推进依据是架构边界验证；Beta 使用中尚未观察到 JSON 导航需求。

审批或实施时，从 [design 的 Decision Map](design.md#decision-map) 恢复每项判断的长期 owner、JSON 应用和当前状态；完整决策理由保留在长期记录，目标 contract 保留在 capability delta。

## What Changes

- 新增内置 workspace crate `docnav-json`，通过既有 `AdapterDefinition` factory 和 core static registry 随 `docnav` 二进制链接交付。
- 支持 `.json` UTF-8 文档的 probe，以及 JSON-owned outline、read、find、info、分页、成本和 unstructured full-read 行为。
- 使用 JSON tree path 作为 node identity，并以 adapter-owned、非空、ASCII-safe 且 canonical 的 `json:#<RFC 6901 URI fragment>` 表示；shared/core 层只把 ref 当作 opaque string 原样传递。
- JSON outline 按 E1 选定的 object member 顺序和 array index 做确定性树前序；number identity 使用原始 token；structured read 使用 pinned serializer 的自然结果和两空格布局；find 直接搜索原文并把源码命中映射为可继续 read 的 node ref。
- JSON adapter 消费现有 common closed operation input；max depth 来自 adapter-private 单一硬编码配置源；core parameter catalog 和 `StandardInputBinding` inventory 保持当前契约。
- 增加 JSON adapter 主规范、fixtures、owner tests、core/release smoke、case ledger 和覆盖材料，验证自动选择、显式选择、ref roundtrip、错误分类、分页和发布制品中的 linked behavior。

## Capabilities

### New Capabilities

- `json-adapter`: 定义 JSON probe、结构遍历、ref grammar、outline/read/find/info、分页、成本、full-read、错误和验证边界。

### Modified Capabilities

- `release-artifacts`: 将发布制品 smoke 从仅证明 linked Markdown behavior，扩展为同时证明 linked JSON adapter 可由 package 中的同一个 `docnav` 可执行文件自动选择并执行。

## Impact

- 新增 `crates/adapters/json/`，并更新 workspace dependencies、`crates/docnav` dependency 与 `crates/docnav/src/registry.rs` 的静态注册表。
- 新增 `docs/adapters/json.md`，并更新文档导航、测试策略、case ledger、覆盖矩阵和相应 fixtures。
- 将 Decision Map 中的活动决策分别同步到 `docs/architecture.md`、`docs/adapter-contract.md`、`docs/output.md` 和 JSON adapter 主规范的对应 owner 规则；`unaligned` 表示 Current sources 的同步与事实核对仍待完成。
- 复用现有 `serde_json`、adapter contracts、protocol、text-cost、pagination 与 output 路径；JSON 私有 decode 承接 raw number、duplicate member 和 depth 语义。
- 扩展 core CLI smoke、release package smoke、adapter list 和 doctor 的多-adapter 证据；公共 protocol result shape、shared ref contract 和输入 inventory 保持当前契约。
- Registry 只追加 JSON definition；registry 整体治理继续由既有 owner contract 决定。
- 当前 change 记录 generic `readable-view` 暴露的格式假设；JSON 格式专用自定义渲染作为必需的后续验证阶段继续检验 presentation boundary。
