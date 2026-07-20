# release-artifacts Specification

## Purpose
定义 Docnav 发布制品的统一目录、文件集合、清单与校验规则，以及本地预验收和 CI/CD 正式生成之间的职责边界，确保核心 CLI 可逐文件校验并直接运行。Linked adapter libraries 随核心 CLI 编译交付，不作为独立 package 文件验收。
## Requirements
### Requirement: Docnav 发布制品目录必须包含版本号和目标平台
Docnav 发布制品 MUST 写入统一目录结构 `artifacts/docnav/v<version>/<target>/package/`。`<version>` MUST 来自 Cargo workspace package version 并带 `v` 前缀，`<target>` MUST 使用 Rust target triple。

#### Scenario: 生成 Windows 目标平台的制品目录
- **WHEN** workspace 版本为 `0.1.0` 且 target 为 `x86_64-pc-windows-msvc`
- **THEN** 最终制品位于 `artifacts/docnav/v0.1.0/x86_64-pc-windows-msvc/package/`
- **THEN** 版本目录和 target 目录不直接放置最终可执行文件

### Requirement: package 必须直接包含核心 CLI
`package/` MUST 直接包含目标平台的 `docnav` 核心 CLI。Linked adapter libraries MUST be compiled into the core CLI release artifact and MUST NOT be packaged as separate adapter executables. 制品生成脚本 MUST 使用 Cargo release profile 构建核心 CLI，显式 `--target <triple>` MUST 传给 Cargo 构建和制品目录。制品生成脚本 MUST NOT 生成 `.zip`、`.tar.gz` 或其它归档包作为 Docnav 发布制品。

#### Scenario: 生成首期 Windows 发布制品
- **WHEN** 制品生成脚本为 `x86_64-pc-windows-msvc` 成功执行
- **THEN** `package/` 直接包含 `docnav.exe`
- **THEN** `package/` 不包含 `docnav-markdown.exe`
- **THEN** Markdown adapter behavior 通过 package 中的 `docnav.exe` core CLI document operation 验收
- **THEN** `docnav.exe` 来自该 target 的 Cargo release profile 构建结果
- **THEN** `package/` 不包含封装这些文件的 Docnav 归档包

### Requirement: 制品清单与校验和必须逐文件描述制品
制品生成脚本 MUST 在 `package/` 中生成 `manifest.json` 和 `SHA256SUMS.txt`。该 `manifest.json` 是 release artifact manifest，MUST 和 adapter manifest 契约分离，并 MUST NOT 复用 `docs/schemas/manifest.schema.json` 的 adapter manifest 语义。`manifest.json` MUST 记录 `schema_version: 1`、`product: "docnav"`、`version`、`target`、`generated_at`、`git_commit`、`source_dirty`、`producer` 和 `files`。`producer.kind` MUST 为 `local` 或 `github-actions`；CI 生成时 MUST 记录 workflow、run id 和 run attempt。每个 `files` 条目 MUST 记录相对路径、`core` 组件类型、字节大小和小写十六进制 SHA-256。`SHA256SUMS.txt` MUST 覆盖 `docnav` 可执行文件和 `manifest.json`，MUST 按相对路径升序使用 `<lowercase-sha256>  <relative-path>` 格式，且 MUST NOT 包含自身。可执行文件的 hash MUST 与 `manifest.json` 一致。

#### Scenario: 审计 package 文件
- **WHEN** 制品生成脚本成功执行
- **THEN** `manifest.json` 描述 `docnav` 核心 CLI 文件
- **THEN** 核心 CLI 的 component 为 `core`
- **THEN** `docnav` 可执行文件的实际大小和 SHA-256 与 `manifest.json` 一致
- **THEN** `SHA256SUMS.txt` 可校验 `docnav` 可执行文件和 `manifest.json`

### Requirement: 发布制品验证必须直接运行 package 原文件
发布制品验证脚本 MUST 从 `artifacts/docnav/v<version>/<target>/package/manifest.json` 定位核心 CLI，并 MUST 在校验文件集合、大小和校验和后直接运行该文件。验收对象 MUST 来自该 `package/` 目录；验证脚本 MUST NOT 使用 Cargo `target/`、`.log`、临时目录、硬编码旧路径或归档包解压结果中的可执行文件替代验收对象。

#### Scenario: 发布制品 smoke 验证
- **WHEN** 执行发布制品 smoke
- **THEN** 脚本读取统一 `package/` 目录中的 `manifest.json`
- **THEN** 脚本直接运行 `manifest.json` 指向的 `docnav`
- **THEN** linked Markdown adapter behavior 通过该 `docnav` 的 document operation 验收
- **THEN** 被验收对象来自 `package/` 目录

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
发布制品目录结构 MUST NOT 改变 `docnav --output protocol-json`、默认 readable-view 输出或 `readable-json` 的字段语义。

#### Scenario: 从 package 执行 protocol-json
- **WHEN** 调用方运行 package 中的 `docnav --output protocol-json`
- **THEN** stdout 仍输出完整原始协议 envelope
- **THEN** 制品目录信息不进入协议字段

### Requirement: Beta release baseline 必须由当前实现和既有验证共同确定
Beta release candidate MUST 对应一个干净 commit，并 MUST 以该 commit 上的当前主规范、实现、测试和发布资料作为完整 baseline。Candidate MUST 通过现有 full workspace verification、文档/OpenSpec validation、CLI smoke，以及两个支持 target 的 canonical package build、verify 和 smoke。

若上述现有验证全部通过，当前实现 MUST 按现状进入打包和发布阶段。若验证失败，任何实现修复 MUST 绑定失败对应的既有 owner contract，并限制为恢复该承诺所需的最小改动。

#### Scenario: 当前实现可以直接进入打包
- **WHEN** 同一 clean commit 通过全部既有 workspace 和 package checks
- **THEN** 该 commit 成为 Beta release baseline
- **THEN** release work 直接进入版本、文档、package 和 publish 阶段

#### Scenario: 既有验证报告真实阻塞
- **WHEN** 任一既有 required/full、docs/OpenSpec、CLI smoke 或 package check 失败
- **THEN** release progression 停止
- **THEN** 修复绑定对应 owner contract 和失败证据
- **THEN** 完整 baseline verification 在修复后重新运行

### Requirement: Beta public files 必须是已验证 canonical package binary 的副本
对每个支持 target，release tooling MUST 从已通过 canonical package verify/smoke 的 core binary 派生一个 target-qualified public binary 和对应 `.sha256`。Public staging MUST 只复制、改名和计算 checksum，不得调用 Cargo 重建 binary、改变 canonical `package/` 文件集合，或修改 release manifest/`SHA256SUMS.txt`。

Linux public binary MUST 命名为 `docnav-v<version>-x86_64-unknown-linux-gnu`；Windows public binary MUST 命名为 `docnav-v<version>-x86_64-pc-windows-msvc.exe`。Public binary MUST 与 manifest 指向的 package binary 逐字节相同；checksum MUST 使用 public filename 和小写十六进制 SHA-256。

#### Scenario: 从已验证 package 派生 public files
- **WHEN** 某个支持 target 的 canonical package 已通过 verify 和 smoke
- **THEN** tooling 生成该 target 的 public binary 与 checksum
- **THEN** public binary bytes 与 canonical package binary 完全相同
- **THEN** canonical package、manifest 和 package checksum 文件保持不变

#### Scenario: 拒绝无有效 package 证据的派生
- **WHEN** package 缺失、未通过验证，或 version/target/path/hash 与 manifest 不一致
- **THEN** public staging 失败
- **THEN** 不留下可被 publish 接受的完整 public file set

### Requirement: Beta prerelease 必须晋级同一 clean CI candidate
公开 Beta MUST 使用包含 SemVer prerelease 部分的 Cargo workspace version；首个版本 MUST 为 `0.1.0-beta.1`，tag MUST 为 `v0.1.0-beta.1`。Promotion MUST 只消费同一次 clean GitHub Actions run 为 exact Linux/Windows target 集合生成的 canonical packages 和 public files。

每个 manifest 的 version、target、`git_commit`、`source_dirty` 和 producer metadata MUST 与当前 tag、commit 和 workflow run 一致。只有 publish job MAY 获得 `contents: write`；手动验证 run、任一验证失败、tag/version/commit 不一致、target 缺失或 existing release 存在时 MUST NOT 创建或修改 GitHub release。

Public prerelease 的 workflow-uploaded assets MUST 恰好为两个 target-qualified binaries 和两个 checksums。Publish MUST 创建新的 prerelease，并 MUST NOT overwrite、clobber 或原位替换 existing release assets。

#### Scenario: 发布首个 Beta
- **WHEN** `v0.1.0-beta.1` 指向通过完整 baseline verification 的 clean commit
- **AND** 当前 workflow 生成并验证两个 target 的 canonical packages 与 public files
- **AND** 该 tag 尚无 GitHub release
- **THEN** publish job 创建新的 `v0.1.0-beta.1` prerelease
- **THEN** prerelease 只附加四个预期 public files

#### Scenario: 非发布 run 只完成打包验收
- **WHEN** release workflow 由非 tag 的手动验证入口触发
- **THEN** workflow 完成同一 build、verify、smoke、staging 和聚合检查
- **THEN** publish job 不运行
- **THEN** GitHub release 状态不变化

#### Scenario: Promotion evidence 不完整
- **WHEN** baseline verification 未完成
- **OR** tag/version/commit、target 集合、producer、dirty state 或 hash 不一致
- **OR** 该 tag 已存在 GitHub release
- **THEN** promotion 失败
- **THEN** public release/assets 不被创建或修改

### Requirement: Beta 发布资料与复核必须对应实际制品
README 和 versioned release notes MUST 说明当前 Beta 版本、支持 target、下载文件、checksum 校验、基本启动入口和当前已知发布限制。它们 MUST 准确描述 release baseline，不得加入超出当前实现的承诺。

发布后 MUST 重新下载两个 public binaries 和两个 checksums，核对 exact asset set、checksum、binary version 和基本可执行性。Release evidence MUST 记录 commit、tag、release URL、workflow run、canonical manifest hashes、public asset hashes 和复核结果。

#### Scenario: 发布资料与公开 bytes 一致
- **WHEN** 用户查看 Beta README/release notes 和公开 assets
- **THEN** 文档中的版本、target、filename 和 checksum 步骤与实际 release 一致
- **THEN** 下载后的 binary version 与 release version 一致
- **THEN** 发布资料与当前 release baseline 一致

#### Scenario: 公开制品复核失败
- **WHEN** asset 集合、checksum、binary version 或基本执行检查失败
- **THEN** 该 release 不被记录为完成交付
- **THEN** existing asset 不被原位替换
- **THEN** 修复使用递增 prerelease version 重新走完整流程
