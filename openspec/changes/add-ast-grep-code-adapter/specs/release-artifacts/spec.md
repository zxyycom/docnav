本 change 的目标是新增一个直接链接 ast-grep Rust crates 的多语言代码 adapter，通过 `outline -> ref -> read` 提供有限、可继续的代码结构化阅读。

本文是仅位于 `openspec/changes/add-ast-grep-code-adapter/` 的未审核临时 `release-artifacts` delta spec，不修改或替代现有主规范、其它文档或其它 change。

## ADDED Requirements

### Requirement: Package verifies linked code navigation

包含 `docnav-code` 的发布制品 MUST 把所选 ast-grep language parsers 和 outline implementation 编译进 canonical package 中的同一个 `docnav` executable。Package file set MUST NOT 包含独立 `ast-grep`、`docnav-code` 或 language parser executable。发布制品 smoke MUST 从 `package/manifest.json` 定位并直接运行该 `docnav`，验证 adapter inspection，以及 Rust、TypeScript、TSX、JavaScript 和 Python 各一个代表性 fixture 的 automatic selection、outline ref 和 read roundtrip。Smoke MUST 保留当前 release static registry 中其它 adapters 的代表性验证，并 MUST 在无法发现或执行外部 ast-grep 的环境中成功。

#### Scenario: Canonical package contains linked code behavior

- **WHEN** 包含 code adapter 的 canonical package 已通过文件集合、size 和 checksum 校验
- **THEN** smoke 直接运行 manifest 指向的同一个 `docnav` executable
- **THEN** `adapter list` 报告 implementation source 为 `core_static` 的 `docnav-code`
- **THEN** 五种 code format fixtures 都从 outline 取得实际 ref 并成功 read
- **THEN** package 和执行环境均不需要独立 ast-grep executable

#### Scenario: Other linked adapters are retained

- **WHEN** code adapter 与 Markdown、JSON 或其它已合并 adapter 同时存在于当前 release
- **THEN** release smoke 保留每个现有 adapter 的代表性 roundtrip
- **THEN** registry 验证不使用会覆盖并行 change 的固定 adapter 总数或脆弱顺序假设
