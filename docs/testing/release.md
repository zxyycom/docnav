# 发布包验证

本文记录 release package 的本地预验收和 CI/CD 验证边界。常规开发验证入口见 [测试策略](../testing.md)。

## Release baseline

Beta release candidate 必须对应一个 clean commit，并以该 commit 上的当前主规范、实现、测试和发布资料作为同一 baseline。进入版本、打包或发布阶段前，该 commit 必须通过 full workspace verification、文档/决策/调查/Change Plan validation、core CLI smoke，以及 Linux `x86_64-unknown-linux-gnu` 和 Windows `x86_64-pc-windows-msvc` 的 canonical package build、verify 和 smoke。

Required profile 可用于开发期快速反馈，但不替代上述完整 baseline。任一检查失败都会停止 release progression；修复必须归属失败对应的 owner contract，并限制为恢复当前承诺所需的最小改动。

## 制品形状

正式发布制品由 `bun run package:docnav -- --target <triple>` 生成，落在 `artifacts/docnav/v<version>/<target>/package/`。Linked adapter libraries 编译进 `docnav` 核心 CLI，不作为独立 package 文件验收。

该目录的精确文件集合是：

- 与 target 对应的唯一 core executable：`x86_64-unknown-linux-gnu` 为 `docnav`，
  `x86_64-pc-windows-msvc` 为 `docnav.exe`
- `manifest.json`
- `SHA256SUMS.txt`

仓库脚本不生成 `.zip`、`.tar.gz` 或其它归档包。

`manifest.json` 是 release artifact manifest，不复用 adapter manifest schema。
它必须包含唯一 core executable entry；该 entry 必须直接解析到 `package/` 内的
非符号链接普通文件。发布制品验证先从该清单定位文件集合，以 `lstat` 拒绝
symbolic link 和其它非普通文件，再检查大小和校验和。Package smoke 继续从这个
已验证 entry 解析 binary，而不是按 target 猜测文件名或回退到 `target/`、日志、
临时目录或解压产物。

## 本地预验收

本地预验收通常按下面顺序跑：

```bash
bun run package:docnav
bun run verify:docnav-package
bun run smoke:docnav-package
```

发布包验证和 smoke 命令会自动定位当前 workspace 版本与 host target 对应的 package。使用 `--target <triple>` 选择当前版本的其它 target；使用 `--manifest <path>` 验证显式 package。`bun run info:docnav-package` 可打印自动定位结果。

`package:docnav` 在生成结束时校验文件集合、manifest、普通文件边界、大小和校验和，
但不运行 CLI smoke。`smoke:docnav-package` 使用 manifest 的唯一 core entry，
以同一个 manifest 所指向的 core binary 执行 required adapter inspection，以及
Markdown 与 JSON 的 automatic outline、explicit read 和 outline-ref-read direct CLI
roundtrip；这证明两个 adapter 均 linked 到 canonical core binary，而不是独立
package 文件或按 operation/format 切换 executable。

Package profile 在 Linux 和 Windows 上复用 development core smoke 的同一组
routing、protocol/readable 与 failure assertions，只替换 manifest-resolved binary。
它不建立 target-specific detector、fixture matrix 或 package validator 业务职责。

## Public-file 派生

Public files 是 canonical package 通过 verify 和 smoke 后派生的公开下载副本，不属于 `package/`。Tooling 必须从 `manifest.json` 定位 core binary，只执行复制、target-qualified 改名和 SHA-256 计算：

- Linux：`docnav-v<version>-x86_64-unknown-linux-gnu` 及同名 `.sha256`
- Windows：`docnav-v<version>-x86_64-pc-windows-msvc.exe` 及同名 `.sha256`

Public binary 必须与 manifest 指向的 package binary 逐字节相同；`.sha256` 必须包含小写十六进制 SHA-256 和对应 public filename。派生过程不得调用 Cargo、搜索替代 binary、改变 canonical `package/` 文件集合，或修改 `manifest.json` 和 `SHA256SUMS.txt`。

Package 缺失、尚未通过 verify/smoke，manifest binary 为 symbolic link 或其它
非普通文件，或 version、target、path、hash 与 manifest 不一致时，派生必须失败。
在 public mutation 前发现的失败必须保留既有 public set；mutation 开始后的失败
必须清理本次 partial public files，不能留下可被 promotion 接受的完整 public
file set。

## Prerelease promotion

首个公开 Beta 的 Cargo workspace version 必须为 `0.1.0-beta.1`，tag 必须为 `v0.1.0-beta.1`。Promotion 只消费同一次 clean GitHub Actions run 为上述两个 target 生成并验证的 canonical packages 和 public files；manifest 的 version、target、commit、clean-source 状态和 producer 必须与当前 tag 和 workflow run 一致。

非 tag 的手动验证 run 必须完成相同的 build、verify、smoke、public staging 和聚合检查，但不得创建或修改 GitHub release。只有 publish job 可以获得 `contents: write`；tag/version/commit、target 集合、producer 或 hash 不一致，以及 existing release 已存在时，promotion 必须失败。

新 prerelease 只发布两个 target-qualified binaries 和两个对应 checksums。Canonical packages 继续作为内部 provenance evidence，不改变其既有 build、verify、smoke 或上传归属。

## 发布复核

发布后必须重新下载四个 public assets，复核 exact file set、checksum、binary version 和基本可执行性。Release evidence 必须记录 commit、tag、release URL、workflow run、canonical manifest hashes、public asset hashes 和复核结果；全部通过后才能把该 prerelease 记录为完成交付。

## 失败恢复

发布前失败不得创建或修改 public release/assets。修复后必须重新运行受影响的 focused checks 和完整 release baseline，再从 canonical package 流程继续。

公开后的 asset 集合、checksum、binary version 或基本执行检查失败时，该 release 不得记录为完成交付，也不得原位替换 existing assets；修复必须使用递增 prerelease version 重新走完整 baseline、打包、promotion 和复核流程。
