本 tasks 只在 `openspec/changes/replace-pmd-cpd-with-jscpd/` 下形成未审核临时文档，执行前必须先完成 PMD CPD 到 jscpd 迁移方案审计门禁。

## 1. 阻塞级审计门禁

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“用 jscpd 替换 PMD CPD duplicate-code scanner，同时保持 repository quality observability 归一化输出稳定”这一核心目标。
- [ ] 1.2 审计 capability ID 是否正确复用 `repository-quality-observability`，且没有创建一次性、同义或过宽 capability。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/replace-pmd-cpd-with-jscpd/` 下的未审核临时 artifacts，且没有修改现有主 specs、docs、schemas、examples、测试或实现代码。
- [ ] 1.4 审计 design 中的外部工具依据是否来自官方 jscpd 文档和 npm registry，并确认 `## Open Questions` 没有未回答问题或歧义。
- [ ] 1.5 在 1.1-1.4 全部完成前，不得执行任何实现任务、主规范更新、文档更新、测试更新、CI 更新或代码改动。

## 2. 依赖与工具链确认

- [ ] 2.1 用官方 jscpd 文档和 npm registry 确认当前可用版本、`jscpd` package、`jscpd` command、JSON reporter、`--min-tokens`、output directory/config 行为，并记录最终实现采用的版本。
- [ ] 2.2 通过 `pnpm` 将 `jscpd` 加入 devDependency 和 lockfile，确保仓库脚本不依赖全局 `jscpd`、`cpd`、PMD 或 Java。
- [ ] 2.3 更新 CI quality tools setup，删除 Java/PMD 下载与 `PMD_VERSION` 环境变量，并加入本地 dependency 提供的 `jscpd --version` smoke。

## 3. Scanner Wrapper 迁移

- [ ] 3.1 将 quality config、schema types、cache identity 和 tool config 中的 duplicate-code scanner 字段从 PMD CPD 迁移到 jscpd，并保留 per-code-area minimum token policy。
- [ ] 3.2 实现 jscpd tool availability check，区分 dependency missing、binary unavailable、non-zero execution、empty successful scan 和 parse failure。
- [ ] 3.3 实现 jscpd JSON parser，将 raw clone records 归一化为 `DuplicateCodeFragment`，覆盖 token count、line count、路径归一化、start/end line、多 location 和 code area 标注。
- [ ] 3.4 保留 code-area scan planning、bounded parallel cache miss task、fingerprint cache identity、current/baseline scan kind 和 changed-scope annotation。
- [ ] 3.5 将 current revision 和 baseline revision duplicate-code orchestration 从 PMD CPD wrapper 切换到 jscpd wrapper。

## 4. Warnings、Reports 和文档同步

- [ ] 4.1 将 duplicate-code warning identity 迁移为 jscpd source/rule，更新 accepted warning 配置，并用 jscpd scan 结果校准可接受 warning 的匹配条件。
- [ ] 4.2 更新 Markdown quality report、terminal output、raw artifact names、quality scan help text 和 quick/full profile 文案中的 PMD CPD 表述。
- [ ] 4.3 更新 `docs/testing.md`、`docs/tooling.md`、`docs/testing/cases.md` 和 repository-quality-observability 主规范归档目标中的 duplicate-code 工具表述。

## 5. PMD CPD 清理

- [ ] 5.1 删除 PMD CPD scanner、XML parser、process-result 特判、PMD version parser 和对应 PMD-specific tests。
- [ ] 5.2 删除 CI 中的 Java/PMD setup、PMD zip 下载、`pmd cpd --help` smoke 和相关环境变量。
- [ ] 5.3 确认旧 PMD CPD cache 不会被 jscpd scan 误读为命中，必要时提升 quality cache version 或调整 cache tool identity。

## 6. 验证

- [ ] 6.1 运行 `bun run quality:test`，证明 wrapper parser、tool availability、cache、warnings 和 report tests 全部通过。
- [ ] 6.2 运行 `bun run quality:full-check`，证明 jscpd duplicate-code scan、baseline comparison、warnings 和 artifacts 在 full profile 下可执行。
- [ ] 6.3 运行 `bun run verify:docnav-workspace` 或 `bun run verify:docnav-workspace:full`，证明 workspace verifier 与 quality full profile 集成稳定。
- [ ] 6.4 用局部 diff 确认实现只影响 jscpd duplicate-code scanner 迁移相关代码、tests、docs、CI、dependency files 和本 change artifacts。
