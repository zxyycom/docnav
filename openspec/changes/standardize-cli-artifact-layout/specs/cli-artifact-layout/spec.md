## ADDED Requirements

### Requirement: CLI 发布产物目录必须包含版本号和平台信息
`docnav` CLI 发布产物 MUST 写入统一目录结构 `artifacts/docnav/v<version>/<target>/package/`。`<version>` MUST 来自 Cargo workspace 版本并带 `v` 前缀，`<target>` MUST 使用 Rust target triple，最终打包结果 MUST 位于平台目录下的 `package/` 目录中。

#### Scenario: 生成 Windows 平台发布包
- **WHEN** 打包版本 `0.1.0` 且 target 为 `x86_64-pc-windows-msvc`
- **THEN** 最终打包结果位于 `artifacts/docnav/v0.1.0/x86_64-pc-windows-msvc/package/`
- **THEN** 不在 `artifacts/docnav/v0.1.0/` 或 `artifacts/docnav/v0.1.0/x86_64-pc-windows-msvc/` 直接放置最终 archive

### Requirement: 打包脚本必须生成可审计的 archive、manifest 和 checksum
打包脚本 MUST 在 `package/` 目录中生成最终 archive、`manifest.json` 和 `SHA256SUMS.txt`。archive 文件名 MUST 包含产品名、版本号和 target；`manifest.json` MUST 记录版本、target、archive 文件名、包含的二进制、构建时间、git commit 和 SHA-256。

#### Scenario: 打包脚本完成发布产物
- **WHEN** 打包脚本成功执行
- **THEN** `package/` 目录包含最终 archive
- **THEN** `package/` 目录包含描述该 archive 的 `manifest.json`
- **THEN** `package/` 目录包含可校验 archive 的 `SHA256SUMS.txt`

### Requirement: 发布验证脚本必须从统一打包目录读取产物
验证发布包的测试脚本 MUST 从 `artifacts/docnav/v<version>/<target>/package/manifest.json` 定位 archive，并 MUST 基于该 archive 解包后的 CLI 执行验证。发布验证脚本 MUST NOT 从 Cargo `target/`、`.log`、临时目录或硬编码旧路径查找被验收的 CLI。

#### Scenario: 发布包 smoke 验证
- **WHEN** 执行发布包 smoke 验证脚本
- **THEN** 脚本读取统一 `package/` 目录中的 `manifest.json`
- **THEN** 脚本按 manifest 指向的 archive 解包并运行 CLI
- **THEN** 脚本不直接引用 `target/debug` 或 `target/release` 下的二进制作为被验收对象

### Requirement: 开发期 smoke 与发布包验证必须职责分离
允许保留直接运行 Cargo 输出二进制的开发期 smoke，但该入口 MUST 通过名称、脚本文案或 `package.json` 命令明确标识为开发期验证。workspace 或发布包验收入口 MUST 使用统一打包目录中的产物。

#### Scenario: 保留开发期 smoke
- **WHEN** 仓库保留直接运行 `target/` 二进制的 smoke 脚本
- **THEN** 该脚本入口明确标识为开发期 smoke
- **THEN** 发布包验收入口不复用该路径作为最终产物来源

### Requirement: 统一产物目录不得改变运行时输出协议
发布产物目录结构 MUST NOT 改变 `docnav --output protocol-json`、默认 text 输出、`readable-json`、adapter invoke 协议或 MCP structuredContent 的字段语义。

#### Scenario: 打包后执行 protocol-json
- **WHEN** 调用方运行打包产物中的 `docnav --output protocol-json`
- **THEN** stdout 仍输出完整原始协议 envelope
- **THEN** 目录结构不出现在协议字段中
