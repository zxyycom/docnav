# release-artifacts Specification

## Purpose
定义 Docnav 发布制品的统一目录、文件集合、清单与校验规则，以及本地预验收和 CI/CD 正式生成之间的职责边界，确保核心 CLI 与发布适配器可逐文件校验并直接运行。

## Requirements
### Requirement: Docnav 发布制品目录必须包含版本号和目标平台
Docnav 发布制品 MUST 写入统一目录结构 `artifacts/docnav/v<version>/<target>/package/`。`<version>` MUST 来自 Cargo workspace package version 并带 `v` 前缀，`<target>` MUST 使用 Rust target triple。

#### Scenario: 生成 Windows 目标平台的制品目录
- **WHEN** workspace 版本为 `0.1.0` 且 target 为 `x86_64-pc-windows-msvc`
- **THEN** 最终制品位于 `artifacts/docnav/v0.1.0/x86_64-pc-windows-msvc/package/`
- **THEN** 版本目录和 target 目录不直接放置最终可执行文件

### Requirement: package 必须直接包含核心 CLI 与当前发布适配器
`package/` MUST 直接包含目标平台的 `docnav` 核心 CLI 和显式发布组件集合中的适配器可执行文件。首期发布组件集合 MUST 包含 `docnav` 与 `docnav-markdown`。制品生成脚本 MUST 使用 Cargo release profile 构建发布组件，显式 `--target <triple>` MUST 传给 Cargo 构建和制品目录。制品生成脚本 MUST NOT 生成 `.zip`、`.tar.gz` 或其它归档包作为 Docnav 发布制品。

#### Scenario: 生成首期 Windows 发布制品
- **WHEN** 制品生成脚本为 `x86_64-pc-windows-msvc` 成功执行
- **THEN** `package/` 直接包含 `docnav.exe`
- **THEN** `package/` 直接包含 `docnav-markdown.exe`
- **THEN** 这两个可执行文件来自该 target 的 Cargo release profile 构建结果
- **THEN** `package/` 不包含封装这些文件的 Docnav 归档包

### Requirement: 制品清单与校验和必须逐文件描述制品
制品生成脚本 MUST 在 `package/` 中生成 `manifest.json` 和 `SHA256SUMS.txt`。该 `manifest.json` 是 release artifact manifest，MUST 和 adapter manifest 契约分离，并 MUST NOT 复用 `docs/schemas/manifest.schema.json` 的 adapter manifest 语义。`manifest.json` MUST 记录 `schema_version: 1`、`product: "docnav"`、`version`、`target`、`generated_at`、`git_commit`、`source_dirty`、`producer` 和 `files`。`producer.kind` MUST 为 `local` 或 `github-actions`；CI 生成时 MUST 记录 workflow、run id 和 run attempt。每个 `files` 条目 MUST 记录相对路径、`core|adapter` 组件类型、字节大小和小写十六进制 SHA-256；适配器条目 MUST 记录 adapter id。`SHA256SUMS.txt` MUST 覆盖所有可执行文件和 `manifest.json`，MUST 按相对路径升序使用 `<lowercase-sha256>  <relative-path>` 格式，且 MUST NOT 包含自身。可执行文件的 hash MUST 与 `manifest.json` 一致。

#### Scenario: 审计 package 文件
- **WHEN** 制品生成脚本成功执行
- **THEN** `manifest.json` 分别描述 `docnav` 核心 CLI 与 `docnav-markdown` 适配器文件
- **THEN** 核心 CLI 的 component 为 `core`，Markdown 适配器的 component 为 `adapter` 且 adapter id 为 `docnav-markdown`
- **THEN** 每个可执行文件的实际大小和 SHA-256 与 `manifest.json` 一致
- **THEN** `SHA256SUMS.txt` 可校验所有可执行文件和 `manifest.json`

### Requirement: 发布制品验证必须直接运行 package 原文件
发布制品验证脚本 MUST 从 `artifacts/docnav/v<version>/<target>/package/manifest.json` 定位核心 CLI 与适配器，并 MUST 在校验文件集合、大小和校验和后直接运行这些文件。验收对象 MUST 来自该 `package/` 目录；验证脚本 MUST NOT 使用 Cargo `target/`、`.log`、临时目录、硬编码旧路径或归档包解压结果中的可执行文件替代验收对象。

#### Scenario: 发布制品 smoke 验证
- **WHEN** 执行发布制品 smoke
- **THEN** 脚本读取统一 `package/` 目录中的 `manifest.json`
- **THEN** 脚本直接运行 `manifest.json` 指向的 `docnav` 与 `docnav-markdown`
- **THEN** 被验收对象均来自 `package/` 目录

### Requirement: 开发期 smoke 与发布制品验证必须职责分离
允许保留直接运行 Cargo 构建结果的开发期 smoke，但该入口 MUST 通过名称、脚本文案或 `package.json` 命令明确标识为开发期 smoke。workspace verify MAY 包含开发期 smoke；发布制品验收入口 MUST 只使用统一 `package/` 目录中的文件。

#### Scenario: 保留开发期 smoke
- **WHEN** 仓库保留直接运行 Cargo 构建结果的 smoke 脚本
- **THEN** 该入口明确标识为开发期 smoke
- **THEN** 发布制品验收不复用 Cargo 输出路径作为最终制品来源

### Requirement: 正式制品必须由 CI/CD 生成和保存
仓库 MUST 提交制品生成脚本、验证脚本和 CI/CD 工作流，但 MUST NOT 提交 `artifacts/` 下的生成制品。正式制品 MUST 由 CI/CD 在干净 checkout 中调用仓库脚本生成，通过发布制品验证后按 target 上传保存。正式制品的 `manifest.json` MUST 记录 `source_dirty: false` 和 `producer.kind: "github-actions"`。`source_dirty` MUST 由 Git 状态计算，修改、暂存或未被 ignore 的未跟踪文件 MUST 使其为 `true`，被 ignore 的生成物 MUST NOT 单独使其为 `true`。首期工作流 MUST 在匹配 target 的原生 runner 上覆盖 `x86_64-unknown-linux-gnu`/`ubuntu-latest` 与 `x86_64-pc-windows-msvc`/`windows-latest`。本地脚本输出只用于复现和预验收。

#### Scenario: CI/CD 生成正式制品
- **WHEN** 制品工作流处理某个已配置 target
- **THEN** 工作流调用仓库内制品生成脚本创建统一 `package/` 目录
- **THEN** 工作流在上传前运行发布制品验证
- **THEN** 工作流上传该 target 对应的 `package/` 文件集合
- **THEN** `manifest.json` 记录当前工作流运行信息，并表明源码工作树干净
- **THEN** Git 不跟踪本地或 CI 生成的 `artifacts/` 内容

### Requirement: 统一制品目录不得改变运行时输出协议
发布制品目录结构 MUST NOT 改变 `docnav --output protocol-json`、默认 readable-view 输出、`readable-json`、adapter invoke 协议或 MCP structuredContent 的字段语义。

#### Scenario: 从 package 执行 protocol-json
- **WHEN** 调用方运行 package 中的 `docnav --output protocol-json`
- **THEN** stdout 仍输出完整原始协议 envelope
- **THEN** 制品目录信息不进入协议字段
