**一句话核心：保留现有 canonical package 和 CLI 契约，只增加一个受门禁保护的公共 Beta 发布层及可执行 Quick Start；本文是仅位于本 change 目录下的未审核临时设计，不影响现有主规范或其它 change。**

## Context

当前 `release-package.yml` 只接受手动触发，以 Linux/Windows matrix 构建 `package/`、运行 package smoke，并上传 GitHub Actions artifact。这个流程已经证明 release binary、manifest 和 checksum 的工程完整性，但 Actions artifact 不是一个清晰的公开产品入口；README 也没有把下载、准备 binary 和真实 Markdown 导航串成一条可复制路径。

本 change 的产品目标不是宣布 CLI 已经成熟，而是让用户能够在已知范围内真实体验它。现有 `package/` 仍是唯一 canonical release artifact，公开 prerelease 文件只是从已验证 binary 派生的下载形状。Document operation、protocol、output、Markdown adapter 和配置契约均不变化。

## Goals / Non-Goals

**Goals:**

- 发布首个版本化、可公开下载和校验的 Markdown CLI Beta。
- 让 Linux/Windows 用户按 README 在几分钟内走通 `outline -> ref -> read` 和 `find -> ref -> read`。
- 让发布前自动化直接执行 staged public binary，证明 Quick Start 与发布候选一致。
- 公开说明格式、平台、适用场景、限制和反馈边界，不把理论收益写成实测结论。
- 把 Beta 之外的推测性开发从本次交付依赖中移除。

**Non-Goals:**

- 不实现 MCP、service mode、interactive selection、preview/skim、更多 composition 或第二格式。
- 不改变 CLI command/flag、protocol/schema、output、ref、config 或 Markdown behavior。
- 不建设自动遥测、在线服务、完整 benchmark 平台、安装器、包管理仓库或自动更新。
- 不在首个 Beta 扩展 macOS、ARM 或其它 target；后续由实际需求决定。
- 不修改、归档或执行其它 active OpenSpec change。

## Decisions

### Decision 1: Beta 是下一阶段的产品门禁

本 change 只接受完成公开体验路径所需的实现和阻塞修复。`implement-docnav-mcp-bridge`、`interactive-outline-selection`、`add-outline-preview-skim-pack`、`explore-operation-composition` 与 `enable-local-core-adapter-service-mode` 均不构成本 change 的依赖，也不因本 change 自动失效；在 Beta 获得真实反馈前不把它们并入交付。

替代方案是继续并行完善这些能力后再发布。该方案会延后产品证据，并使发布失败难以归因，因此不采用。

### Decision 2: Canonical package 不变，公共文件是可审计副本

`artifacts/docnav/v<version>/<target>/package/`、`manifest.json` 和 `SHA256SUMS.txt` 保持现状。每个 matrix job 先完成现有 package verification 和 smoke，再把其中的 core binary 复制为 target-qualified public filename，并重新生成只覆盖该公共文件的 `.sha256`。

聚合 publish job 下载同一次 run 的 staged public files，核对 target 集合、版本、hash、producer 和 dirty 状态后发布。它不得从 Cargo `target/` 重建、替换或寻找 binary。

考虑过为每个 target 创建 `.zip`/`.tar.gz`。首个 Beta 只有一个可执行文件，归档会增加新的发布格式、解压验证和平台分支；target-qualified binary 加 checksum 已能解决同名冲突，因此不引入归档。

### Decision 3: 首个版本固定为 `0.1.0-beta.1`

实现会把 workspace version 设置为 `0.1.0-beta.1`，公开触发 tag 必须为 `v0.1.0-beta.1`。后续 Beta 使用正常 SemVer prerelease 递增，不覆盖已经公开的旧版本。

发布 workflow 在普通 branch、pull request 或无匹配 tag 时只执行构建/验证，不获得发布权限。只有聚合 publish job 获得最小 `contents: write`，matrix build 保持 `contents: read`。实现阶段使用 GitHub 官方支持的 release API 或 runner 内官方 CLI，并在编码前核对当前官方行为。

### Decision 4: 首期平台范围沿用已验证 matrix

Beta 只承诺：

- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

README 在下载前展示该范围。缺少 macOS/ARM 是已知限制，不在本次顺手扩展，因为新增 runner、签名、权限和平台 smoke 会扩大当前验证问题。

### Decision 5: Quick Start 同时是最小产品验收

README 使用一个短小但真实的 Markdown 样例，给出：

1. 下载 target-qualified binary。
2. 校验 `.sha256`。
3. 在 Linux 设置执行权限，或在 Windows 使用 `.exe`。
4. 运行 `docnav version`。
5. 默认 readable-view 执行一次 outline/find。
6. 用 `protocol-json` 从实际结果取得 ref，再原样执行 read。

仓库 acceptance 复用相同样例内容和命令语义，直接执行 staged public filename。自动化只证明路径可执行和结果契约，不声称用户价值、答案正确率或总体成本改善。

考虑过先写完整教程、网站和多场景 benchmark。它们会让 Beta 再次变成开放式打磨；最小 Quick Start 加诚实限制已经足够开始观察。

### Decision 6: 反馈自愿提供，默认不增加数据收集

Release notes 和 README 指向仓库反馈入口，并提示用户可在自愿且完成脱敏后附带命令、版本、文档类型和既有 invocation log。CLI 不自动启用 invocation logging，不上传内容，也不增加 telemetry。

产品判断优先使用可观察事实：用户能否完成路径、在哪一步失败、哪些错误反复出现。任何 token、延迟或正确率结论都必须来自另行记录的可复现实测，不作为本 change 的发布条件。

### Decision 7: 公共发布不改变进程和协议边界

公开 binary 与 canonical package binary 逐字节相同，因此默认 readable-view、`protocol-json`、stdout/stderr、退出码、adapter selection 和 Markdown ref 行为保持现有 owner contract。发布文件名、GitHub release metadata 和 checksum 不进入任何 document operation output。

## Risks / Trade-offs

- [只支持两个 x86_64 target 会限制早期用户] → 在下载前明确限制；只有出现实际需求后再增加 target。
- [公开 binary 没有签名或平台公证] → 首期提供 SHA-256 和 canonical CI 来源，不宣称签名保证；签名、公证作为独立后续决策。
- [GitHub release 写权限扩大供应链风险] → 仅聚合 publish job 在匹配 tag、完整验证后获得 `contents: write`，其它 job 保持只读。
- [README 示例通过但真实长文档仍有体验问题] → Quick Start 只证明可用入口；真实反馈用于决定后续修复，文档不把 smoke 当产品效果证明。
- [发布失败留下部分资产] → publish job 在上传前完成全部 target 聚合验证；失败版本不复用，修复后发布递增的 Beta 版本。
- [“冻结”被误解为不修问题] → Beta 阻塞、错误信息和真实用户无法完成核心路径的问题仍在范围内；推测性新功能和润色不在范围内。

## Migration Plan

1. 先更新 release owner docs、证明目标和 case/coverage 计划，固定 canonical package、public files、Quick Start 与发布门禁的验收边界。
2. 更新 version、发布脚本、workflow、README 和 staged public asset acceptance，并在普通 CI 中验证 package 与 staged public file，不触发外部发布。
3. 合并后创建与 workspace version 完全一致的 Beta tag，由干净 CI 生成两个 target 并创建 prerelease。
4. 从公开 prerelease 重新下载文件，人工走一次 README Quick Start，记录发布验收结果。
5. 若发布文件有问题，撤下对应 prerelease 的下载入口且不复用版本；修复后递增 prerelease version。Canonical package 和既有 CLI 契约无需回滚。

## Open Questions

无未回答开放问题，可以进入实现前审计。
