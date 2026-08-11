## ADDED Requirements

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
