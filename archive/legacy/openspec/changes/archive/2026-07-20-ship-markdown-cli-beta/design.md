## Context

当前仓库已经具备主规范、实现、自动化验证、Linux/Windows canonical package 生成与 package smoke。缺口不在运行时能力，而在发布收口：当前状态尚未被作为一个明确版本完整验收，现有 CI artifact 也尚未晋级为稳定的公共 Beta 下载。

因此，本 change 先证明当前 commit 按现有 owner contract 和验证入口是完整的，再对同一 commit 生成 package、派生公开文件并发布。

## Release Baseline

Beta release baseline 由同一干净 commit 上的四类材料共同组成：

| 材料 | Owner / 入口 | 完成标准 |
| --- | --- | --- |
| 当前规范与实现 | `docs/`、当前源码 | Current 声明与实现一致 |
| 当前验证证据 | `bun run verify:docnav-workspace` | 全部 required/full checks 通过 |
| Canonical packages | 现有 release-package build/verify/smoke | Linux/Windows package 均通过 |
| 发布资料 | README、versioned release notes、CI run | 版本、commit、assets 和说明一致 |

当前实现整体作为 release baseline。既有 workspace verification 和 package smoke 已负责从各 owner 边界提供证明；本 change 只检查这些证据是否全部完成。

## Goals / Non-Goals

**Goals:**

- 收齐当前规范、实现、测试和发布资料，形成一个没有已知阻塞的 Beta commit。
- 复用现有验证链证明当前实现整体可交付。
- 复用现有 package 形状生成 Linux/Windows 正式制品。
- 以最小 public-file 和 CI promotion 层发布 `0.1.0-beta.1`。
- 保留可追溯的版本、commit、producer、hash 和发布结果。

**Non-Goals:**

- 发布收尾不改变当前产品行为。
- 尚未实现的计划工作不进入 release baseline。

## Decisions

### Decision 1: 现有验证结果决定是否需要修复

实现开始时先在目标 clean commit 上运行现有 required/full workspace verification、文档/OpenSpec validation、CLI smoke 和 release-package checks，形成 release baseline。

- 全部通过：当前实现按现状进入版本、文档和 package/release 工作。
- 出现失败：先定位失败对应的 owner contract，只做使当前承诺重新成立的最小修复，并复用或补充该 owner 所需证据。
- 修复范围以当前失败和明文契约缺口为界。

这样让“收尾”具有可观察出口，并把实现修改严格绑定到实际失败。

### Decision 2: Canonical package 和既有 smoke 是打包验收边界

`artifacts/docnav/v<version>/<target>/package/` 继续是唯一 canonical release package。Build、manifest、checksum、producer metadata、package verification 和 CLI smoke 全部复用现有实现与 owner contract。

README 只补齐当前 Beta 的获取、校验、启动入口，并链接现有 CLI/help/规范。实现正确性继续由既有验证链证明，release work 只补充制品层证据。

### Decision 3: Public files 只解决公开下载命名

GitHub prerelease 不能直接附加目录，且两个 target 的 package binary 同名。每个 native matrix job 在 canonical package verify/smoke 通过后，仅执行一次机械派生：

- Linux：复制为 `docnav-v<version>-x86_64-unknown-linux-gnu`
- Windows：复制为 `docnav-v<version>-x86_64-pc-windows-msvc.exe`
- 为每个 public binary 生成同名 `.sha256`

Public binary 必须与 package binary 逐字节相同。这个 staging 不重建 binary，也不改变 canonical package。CI 内部继续保存 canonical package 作为 provenance evidence；GitHub prerelease 只发布两个 target-qualified binaries 和两个 checksums。

### Decision 4: 版本、CI provenance 与 tag 共同控制发布

Docnav 产品版本继续来自 Cargo workspace metadata。首个 Beta 为 `0.1.0-beta.1`，公开 tag 为 `v0.1.0-beta.1`。

Release workflow 保留手动构建/验收入口，并增加匹配 Beta tag 的 publish 路径：

1. Native matrix 从干净 checkout 构建并验证 canonical packages。
2. Matrix 从已验证 package 派生 public files。
3. 聚合 job 核对 exact target 集合、workspace/tag version、tag commit、clean source、current workflow producer 和 hashes。
4. 只有 publish job 获得 job-level `contents: write`；其它 jobs 保持只读。
5. Publish job 拒绝 existing release，并以一次 `gh release create --verify-tag --prerelease --latest=false` 发布已验证文件和版本化 notes。

GitHub Actions 支持用 tag filter 区分 tag push，并允许在 job 级收紧 `GITHUB_TOKEN` 权限：[Workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)。`gh release create` 支持验证既有 tag、prerelease、notes file 和多 asset 上传：[GitHub CLI manual](https://cli.github.com/manual/gh_release_create)。

### Decision 5: 发布复核只检查制品身份与可执行性

发布后重新下载四个 public assets，复核文件名、checksum、binary version 和基本可执行性。Canonical package smoke 和 workspace verification 已经证明同一 bytes 的当前行为。

最终 `delivery-notes.md` 记录 commit、tag、release URL、workflow run、canonical manifest hashes、public asset hashes 和验证结果。发布前失败可以修复后重跑；已经公开的 asset 不原位覆盖，修复使用递增 prerelease version。

## Risks / Trade-offs

- **现有验证可能暴露真实问题。** 只按失败对应 owner 做最小修复，并保留失败与验证证据。
- **README 或主规范可能与当前实现状态不一致。** 按 `docs/navigation.md` 的状态语义修正事实，不借机改变实现目标。
- **Public file 是 canonical binary 的改名副本。** 用 byte equality 与 checksum 证明身份，canonical package 仍保留完整 manifest/provenance。
- **Release job 需要写权限。** 写权限只存在于满足完整 `needs` 和 tag/version 条件的 publish job。
- **首个公开 binary 没有签名或平台公证。** Release notes 诚实说明当前交付边界；签名策略不阻塞本次现状打包。

## Rollout And Recovery

1. 在目标 clean commit 上完成现状审计和全量验证，修复唯一有证据的阻塞项。
2. 更新 Beta version、README、release owner docs 和 versioned release notes。
3. 本地及 CI 生成并验证 Linux/Windows canonical packages 与 public copies。
4. 先用非发布入口完成完整 CI 演练，确认不会创建 release。
5. 只有用户明确授权后创建并推送 Beta tag，由同一 workflow 发布 prerelease。
6. 下载公开 assets 完成 checksum/version 复核并记录 delivery notes；失败时不覆盖已发布文件，以递增版本重新发布。
