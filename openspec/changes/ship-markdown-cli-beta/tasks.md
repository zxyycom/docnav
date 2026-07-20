## 0. 实施启动门禁

- [x] 0.1 审计 proposal、design、`release-artifacts` delta 和 tasks：主承诺统一为“当前实现收尾、既有验证、打包发布”，capability 复用现有 `release-artifacts` owner。
- [x] 0.2 确认 planning artifacts 完整且没有未回答的开放问题；进入实现前的改动仅位于本 change，后续实现修改由既有验证报告的阻塞决定。

## 1. 当前实现收尾

- [ ] 1.1 在目标 clean commit 上按 `docs/navigation.md` 状态语义审计当前主规范、README、实现和测试证据，列出且只列出会阻塞当前版本交付的真实不一致。
- [ ] 1.2 运行现有 required/full workspace verification、docs/OpenSpec validation、CLI smoke 和 release-package checks，记录 baseline 结果。
- [ ] 1.3 处理 baseline 结果：没有失败时记录 release baseline 已就绪；出现失败时按对应 owner contract 做最小修复，并重新运行受影响的 focused checks 和完整 baseline verification。

## 2. Beta 版本与发布资料

- [ ] 2.1 将 Cargo workspace version 更新为 `0.1.0-beta.1`，刷新 `Cargo.lock`，并确认 Cargo metadata、CLI version、package path 和 manifest version 一致。
- [ ] 2.2 更新 `docs/testing/release.md`，增加 release baseline、public-file 派生、prerelease promotion、发布复核和失败恢复边界，同时保留现有 canonical package owner。
- [ ] 2.3 更新 `README.md`，补齐当前 Beta 的支持 target、下载、checksum、基本启动入口和现有详细文档链接。
- [ ] 2.4 增加 `docs/releases/v0.1.0-beta.1.md` 作为 prerelease notes source，只根据当前实现与验证证据描述版本范围、获取方式和已知发布限制。

## 3. Canonical package 与 public files

- [ ] 3.1 复用现有 release-package build/verify/smoke，为 Linux 与 Windows 生成 canonical packages；不改变 package layout、manifest、`SHA256SUMS.txt` 或 Cargo release profile。
- [ ] 3.2 在现有 release-package 实现归属中增加最小 public-file staging：从 manifest 定位已验证 core binary，复制为 target-qualified filename，并生成对应 `.sha256`。
- [ ] 3.3 验证 public binary 与 package binary 逐字节相同；staging 失败时清理 partial public files，且不得调用 Cargo 或搜索替代 binary。
- [ ] 3.4 为 public filename、checksum、byte equality、exact file set，以及 missing/mismatched package evidence 增加最小脚本级测试。

## 4. CI 打包与 prerelease promotion

- [ ] 4.1 扩展 `.github/workflows/release-package.yml`：保留非发布验证入口，并增加匹配 Beta tag 的 publish 路径；workflow/build jobs 默认 `contents: read`。
- [ ] 4.2 让 native matrix 依次完成 canonical package build、verify/smoke 和 public staging，再上传同一 run 的 package/public evidence。
- [ ] 4.3 增加 read-only aggregate validation，核对 exact target 集合、workspace/tag version、tag commit、clean source、current-run producer 和 package/public hashes。
- [ ] 4.4 增加唯一 `contents: write` publish job：拒绝 existing release，并用 `gh release create --verify-tag --prerelease --latest=false`、versioned notes 和四个已验证 public files 创建新 prerelease。
- [ ] 4.5 为 event condition、job dependencies、job-level permission、artifact handoff、existing-release rejection 和 publish failure 增加范围匹配的静态或脚本验证。

## 5. 最终验证与发布

- [ ] 5.1 运行 release-package script tests、TypeScript checks、docs validation、本地 package verify/smoke 和 public-file verification。
- [ ] 5.2 运行 `bun run verify:docnav-workspace` 与 `openspec validate ship-markdown-cli-beta --type change --strict --no-interactive`。
- [ ] 5.3 用局部 diff 审计最终范围：除有明确失败证据的修复外，只允许版本、发布资料、package/public staging 和 CI promotion 改动。
- [ ] 5.4 在用户明确授权后运行非发布 CI 演练，确认两个 target 完整通过且 GitHub release 状态未变化。
- [ ] 5.5 仅在用户再次明确授权公开版本后创建并推送 `v0.1.0-beta.1` tag；不得手工上传或覆盖未通过当前 workflow 的 assets。
- [ ] 5.6 重新下载四个公开 assets，复核 exact set、checksum、binary version 和基本执行结果，并在 change-local `delivery-notes.md` 记录 commit、tag、release URL、workflow run、hashes 和最终状态。
