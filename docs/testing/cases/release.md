# release

## Case AUX-RELEASE-ARGS-001: Release package 参数解析保持边界

Owner: `docs/testing/release.md#本地预验收`

Entities:
- `bun|scripts/tools/release-package/args.test.ts|package build target parses supported selectors`
- `bun|scripts/tools/release-package/args.test.ts|package build target rejects invalid selectors`
- `bun|scripts/tools/release-package/args.test.ts|package selection parses supported selectors`
- `bun|scripts/tools/release-package/args.test.ts|package selection rejects invalid selectors`

Proves:
- release package selector 区分 host package default、target triple、manifest path 和 ambiguous selector。
- build target parser 区分 host default、single target 和非法 extra options/path。

## Case AUX-RELEASE-CANDIDATE-001: Release candidate 聚合证据保持同源

Owner: `docs/testing/release.md#prerelease-promotion`

Entities:
- `bun|scripts/tools/release-package/candidate.test.ts|accepts an exact manual-run candidate without modifying its files`
- `bun|scripts/tools/release-package/candidate.test.ts|accepts only the matching workspace tag and tag commit`
- `bun|scripts/tools/release-package/candidate.test.ts|rejects a candidate with a non-exact direct target set`
- `bun|scripts/tools/release-package/candidate.test.ts|rejects a target with a non-exact public file set`
- `bun|scripts/tools/release-package/candidate.test.ts|rejects canonical package and public hash mismatches`
- `bun|scripts/tools/release-package/candidate.test.ts|rejects dirty checkout or manifest evidence`
- `bun|scripts/tools/release-package/candidate.test.ts|rejects package evidence from a different workflow run`
- `bun|scripts/tools/release-package/candidate.test.ts|rejects workspace version and manifest commit mismatches`

Proves:
- 显式 version root 只接受 exact Linux/Windows direct target set；每个 target 的 canonical package、exact public file set、binary bytes 和 checksum 均与 manifest evidence 一致。
- Candidate version、commit、clean-source 和 producer 必须对应当前 workspace 与同一次 GitHub Actions run；tag validation 额外要求 `v<workspace-version>` 指向同一 commit，manual validation 不要求 tag。
- 聚合成功保持 candidate files 不变；任一 target、version、commit、dirty state、producer 或 package/public hash evidence 不一致时失败。

## Case AUX-RELEASE-PUBLIC-001: Public files 从已验证 canonical package 派生

Owner: `docs/testing/release.md#public-file-派生`

Entities:
- `bun|scripts/tools/release-package/public.test.ts|a checksum write failure removes public files created after validation`
- `bun|scripts/tools/release-package/public.test.ts|a missing manifest does not remove an unrelated public directory`
- `bun|scripts/tools/release-package/public.test.ts|mismatched canonical package evidence fails without modifying an existing public set`
- `bun|scripts/tools/release-package/public.test.ts|missing canonical package evidence fails without modifying an existing public set`
- `bun|scripts/tools/release-package/public.test.ts|stages the exact Linux public file set from canonical package evidence`
- `bun|scripts/tools/release-package/public.test.ts|stages the exact Windows public file set from canonical package evidence`

Proves:
- Linux 与 Windows canonical package evidence 分别派生 target-qualified public binary 和 checksum；public binary bytes、checksum filename/hash 和 exact two-file set 与对应 package entry 一致。
- Manifest 或 package evidence 缺失、package binary hash 不一致时，staging 在 public mutation 前失败，并保留既有 public marker/set。
- Package validation 成功后，checksum 写入失败会清理本次 staging 的 public files。

## Case AUX-RELEASE-WORKFLOW-001: Beta release workflow 保持验证与 promotion 门禁

Owner: `docs/testing/release.md#prerelease-promotion`

Entities:
- `bun|scripts/tools/release-package/workflow.test.ts|aggregate validation consumes current-run artifacts for manual and tag inputs`
- `bun|scripts/tools/release-package/workflow.test.ts|native matrix stages one exact package and public artifact per supported target`
- `bun|scripts/tools/release-package/workflow.test.ts|publish is the single writer and creates one new prerelease from four public files`
- `bun|scripts/tools/release-package/workflow.test.ts|release workflow keeps manual validation and gates promotion on Beta tags`

Proves:
- Workflow 保留手动验证入口，只增加 Beta tag push；默认权限为 `contents: read`，唯一写权限属于 tag-only publish job。
- Exact Linux/Windows matrix 按 build、explicit verify、50-command smoke、public staging 的顺序生成唯一 target artifacts；aggregate 只下载当前 run evidence，manual validation 不传 tag，tag validation 传递当前 ref name。
- Publish 依赖 aggregate，拒绝 existing release，并用一次 `gh release create` 把四个动态 version public paths 与 versioned notes 发布为 prerelease；create failure 保持失败。
