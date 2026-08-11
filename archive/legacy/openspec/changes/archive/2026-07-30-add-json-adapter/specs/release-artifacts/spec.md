**目标：让 canonical package smoke 从同一个 core executable 证明 Markdown 与静态 linked JSON adapter。**

**适用状态：本 capability delta 定义 `add-json-adapter` 对 release validation 的目标修改，不表示 Current 行为；当前 release 支持仍以 `release-artifacts` owner、脚本和制品证据为准。**

## MODIFIED Requirements

### Requirement: 发布制品验证必须直接运行 package 原文件
发布制品验证脚本 MUST 从 `artifacts/docnav/v<version>/<target>/package/manifest.json` 定位核心 CLI，并 MUST 在校验文件集合、大小和校验和后直接运行该文件；manifest-resolved package path MUST 是 smoke 的唯一 executable identity。发布制品 smoke MUST 为 release static registry 中的每个内置格式保留一个代表性 document operation roundtrip。包含 JSON adapter 的 release MUST 至少验证 Markdown 与 JSON 的 automatic selection、outline 返回实际 ref、该 ref 原样进入 read，以及 `adapter list` 报告 `docnav-markdown` 和 `docnav-json` 都来自 `core_static`。其它内置格式 MUST 按 registry owner 的当前 membership 与 ordering contract 保留代表性验证。JSON adapter MUST 通过 package core `docnav` executable 交付。

#### Scenario: 发布制品 smoke 验证所有内置格式
- **WHEN** package 中的 `docnav` 已通过文件集合、size 和 checksum 校验
- **THEN** smoke 从统一 `package/manifest.json` 定位并直接运行该 `docnav`
- **THEN** smoke 直接运行该 package binary 导航一个 Markdown fixture
- **THEN** smoke 直接运行同一 binary 导航一个 JSON fixture
- **THEN** 两条路径都从 outline 取得实际 ref 并成功 read
- **THEN** `adapter list` 报告 `docnav-markdown` 和 `docnav-json` 的 implementation source 都是 `core_static`
- **THEN** 当时已合并的其它内置格式仍保留代表性 roundtrip
- **THEN** manifest-resolved package `docnav` 是 Markdown 与 JSON smoke 的共同 executable identity
