**一句话核心：在不引入动态注册或预备性抽象的前提下，交付一个真实可用的内置 JSON adapter，以第二种格式检验现有静态 adapter 架构；本文是仅位于本 change 目录下的未审核临时提案，不影响现有主规范或其它 change。**

## Why

当前静态 linked-adapter 架构只被 Markdown 一个实现使用，许多“通用性”判断仍然只有单一样本。JSON 是常见、结构明确且与 Markdown heading 模型显著不同的输入；在 Markdown CLI Beta 已公开并形成继续扩展的明确产品决定后，用一个范围受限的 JSON adapter 走完整个 `probe -> outline/find -> ref -> read/info` 链路，可以同时交付真实产品能力，并暴露实际扩展摩擦，而不是继续为假设中的格式提前抽象。

## What Changes

- 将已发布的 Markdown CLI Beta 和用户对“继续第二格式”的明确确认作为实现启动门禁；若 Beta 没有足够产品理由继续扩展，本 change 保持草案状态。
- 新增内置 workspace crate `docnav-json`，通过既有 `AdapterDefinition` factory 和 core static registry 随 `docnav` 二进制链接交付。
- 支持 `.json` UTF-8 文档的 probe，以及 JSON-owned outline、read、find、info、分页、成本和 unstructured full-read 行为。
- 使用 adapter-owned、非空且 canonical 的 `json:<RFC 6901 JSON Pointer>` ref；shared/core 层继续把 ref 当作 opaque string 原样传递。
- JSON outline 以确定性的树前序生成扁平条目；JSON read 返回指定值的规范化 JSON；find 在 canonical pointer 和 scalar text 中做 literal search，并返回可继续 read 的 node ref。
- JSON adapter 首期只消费现有 common closed operation input，不新增 caller-configurable 参数；core parameter catalog 和 `StandardInputBinding` inventory 保持不变。
- 增加 JSON adapter 主规范、fixtures、owner tests、core/release smoke、case ledger 和覆盖材料，验证自动选择、显式选择、ref roundtrip、错误分类、分页和发布制品中的 linked behavior。
- 非目标：本 change 不增加动态 adapter 安装/注册/discovery protocol，不允许 adapter 贡献 public 参数，不为 YAML/TOML 或第三种格式预建框架，不重写共享协议、output contract 或 Markdown adapter，也不顺手进行与第二 adapter 落地无关的结构优化。

## Capabilities

### New Capabilities

- `json-adapter`: 定义 JSON probe、结构遍历、ref grammar、outline/read/find/info、分页、成本、full-read、错误和验证边界。

### Modified Capabilities

- `release-artifacts`: 将发布制品 smoke 从仅证明 linked Markdown behavior，扩展为同时证明 linked JSON adapter 可由 package 中的同一个 `docnav` 可执行文件自动选择并执行。

## Impact

- 新增 `crates/adapters/json/`，并更新 workspace dependencies、`crates/docnav` dependency 与 `crates/docnav/src/registry.rs` 的静态注册表。
- 新增 `docs/adapters/json.md`，并更新文档导航、测试策略、case ledger、覆盖矩阵和相应 fixtures。
- 复用现有 `serde_json`、adapter contracts、protocol、text-cost、pagination 与 output 路径；不引入 runtime plugin dependency 或新的公共参数面。
- 影响 core CLI smoke、release package smoke、adapter list/doctor 的既有多-adapter 预期，但不改变公共 protocol result shape 或 shared ref contract。
