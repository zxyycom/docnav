**一句话核心：用最小公共发布层和可执行 Quick Start 交付 Markdown CLI Beta，所有实现任务必须等待阻塞审计完成；本文件是仅位于本 change 目录下的未审核临时任务清单，不影响现有主规范或其它 change。**

## 0. 实现前阻塞审计

- [ ] 0.1 阻塞审计 proposal、design、`release-artifacts` delta 和本 tasks 是否都围绕“公开可体验的 Markdown CLI Beta”核心句，capability ID 是否复用现有 `release-artifacts` owner；本项和 0.2 未完成前不得执行任何 1.x 及后续实现任务。
- [ ] 0.2 阻塞审计本 change 是否只包含 `openspec/changes/ship-markdown-cli-beta/` 下的未审核临时 artifacts、未修改主规范/实现/其它 change，`## Open Questions` 是否无未回答问题或已收敛歧义，并确认 MCP、service mode、interactive/preview/composition、新格式、遥测和全面评测平台不在实现范围。

## 1. Owner 契约与证明目标

- [ ] 1.1 更新 `docs/testing/release.md`，定义 canonical package、staged public files、public prerelease、验收对象和失败边界。
- [ ] 1.2 按 `docs/testing/case-maintenance.md` 写明 public-file staging、Quick Start acceptance 和发布门禁的“owner 语义 -> 可观察结果”证明目标，并更新 case ledger 与 coverage mapping 计划。
- [ ] 1.3 局部审计 owner 文档与本 change spec 一致，且未改变 CLI/protocol/output/Markdown contract 或把其它 active change 变为依赖。

## 2. Beta 版本与公共文件准备

- [ ] 2.1 将 Cargo workspace version 与相关 package/version 断言更新为 `0.1.0-beta.1`，并用 focused tests 证明 `docnav version`、package path 和 manifest version 一致。
- [ ] 2.2 在 release-package tooling 中增加 target-qualified public binary 与 `.sha256` 的 staging，验证其内容与 canonical package binary 逐字节相同，且不改变 `package/`、manifest 或既有 `SHA256SUMS.txt`。
- [ ] 2.3 为 Linux/Windows public filename、checksum、target/version mismatch、缺失文件、dirty source 和非 GitHub Actions producer 增加脚本级代表测试。

## 3. Quick Start 与 Beta acceptance

- [ ] 3.1 增加一个最小 Markdown acceptance fixture，使 outline/find 返回实际 ref，并让该 ref 原样进入 read；该 fixture 只证明路径可执行，不证明产品效果。
- [ ] 3.2 实现 staged public binary acceptance：直接运行 target-qualified 文件，覆盖 `version`、默认 readable-view、`protocol-json` ref 提取、outline-to-read 和 find-to-read，禁止回退到 Cargo output 或开发 wrapper。
- [ ] 3.3 更新 README，提供 Linux/Windows 下载、checksum、执行准备和与 acceptance 同语义的五分钟 Quick Start。
- [ ] 3.4 在 README 和 prerelease notes owner 中明确 Beta、Markdown-only、支持 target、适用/不适用场景、已知限制、反馈入口、无默认自动遥测和无未经实测效果声明。

## 4. CI prerelease 发布门禁

- [ ] 4.1 在修改 workflow 前核对 GitHub 官方 release/tag/permission 文档，记录所采用官方接口的当前触发、prerelease、asset upload 与 token 权限事实。
- [ ] 4.2 扩展 release workflow，使 matrix job 在 package smoke 后上传 staged public files，聚合 publish job 只消费同一次 run 的完整 Linux/Windows 文件集合并重复核对版本、hash、producer 和 dirty 状态。
- [ ] 4.3 将 `contents: write` 限制到 tag/version 完全匹配的 publish job；branch、PR、普通 dispatch、任一 target/acceptance 失败或非 Beta version 均不得创建或更新 public release。
- [ ] 4.4 为 workflow/job ordering、publish condition、permission、artifact handoff 和 partial failure 增加范围匹配的静态或脚本验证，不通过真实公开发布伪造单元测试。

## 5. 交付验证与发布

- [ ] 5.1 完成 case ledger、coverage mapping、源码 `@case` 标记及 release/CLI smoke proof，避免为等价 platform filename 建重复测试矩阵。
- [ ] 5.2 运行 release-package script tests、docs validation、focused acceptance 和本地 canonical package verify/smoke，并检查 staged public binary hash 与执行结果。
- [ ] 5.3 运行 `bun run verify:docnav-workspace` 与 `openspec validate ship-markdown-cli-beta --type change --strict --no-interactive`，修复本 change 引入的失败。
- [ ] 5.4 用局部 diff 做最小实现审计，删除与公开 Beta 路径无关的抽象、target、教程、功能或 workflow 权限。
- [ ] 5.5 仅在用户明确授权创建外部版本后，创建 `v0.1.0-beta.1` tag 并让干净 CI 发布 prerelease；发布前不得手工上传未验证 binary。
- [ ] 5.6 从公开 prerelease 重新下载两个 target 的 binary/checksum，复核 checksum，并在可用平台人工执行 README Quick Start；若失败则撤下下载入口且不复用该 prerelease version。
