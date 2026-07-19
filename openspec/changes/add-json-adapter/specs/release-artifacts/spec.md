**一句话核心：让 canonical package smoke 同时证明第二个静态 linked JSON adapter，而不改变单一 core executable 的发布形状；本文件是仅位于本 change 目录下的未审核临时 delta，不影响现有主规范或其它 change。**

## ADDED Requirements

### Requirement: 发布包 smoke 必须覆盖每个内置格式的 linked behavior
发布制品 smoke MUST 从 canonical `package/manifest.json` 定位并直接运行同一个 `docnav` 可执行文件，为当前 release static registry 中每个内置格式保留一个代表性 document operation roundtrip。包含 JSON adapter 的 release MUST 至少验证 Markdown 与 JSON 的 automatic selection、outline 返回 ref、该 ref 原样进入 read，以及 `adapter list` 同时报告两个 `core_static` definition。该验证 MUST NOT 要求或寻找独立 adapter executable。

#### Scenario: 验证含 Markdown 与 JSON 的 package
- **WHEN** package 中的 `docnav` 已通过文件集合、size 和 checksum 校验
- **THEN** smoke 直接运行该 package binary 导航一个 Markdown fixture
- **THEN** smoke 直接运行同一 binary 导航一个 JSON fixture
- **THEN** 两条路径都从 outline 取得实际 ref 并成功 read
- **THEN** `adapter list` 报告 `docnav-markdown` 和 `docnav-json` 的 implementation source 都是 `core_static`
