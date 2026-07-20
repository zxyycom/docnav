## Why

Docnav 当前代码、主规范、测试和 canonical release package 链路已经形成可交付基础，但还缺少一次明确的发布收口：确认当前实现与文档一致、清空现有验证失败、冻结 Beta 版本、生成正式 package，并把同一批已验证制品发布为可下载 prerelease。

本 change 要回答的是“当前实现能否直接交付”。当前实现整体进入发布候选，既有契约和验证结果决定它能否继续打包；只有实际失败才触发对应修复。

## What Changes

- 以同一个干净 commit 上的当前代码、主规范、测试和发布资料作为 Beta release baseline，完成状态与证据对齐。
- 运行现有 workspace、文档、OpenSpec、CLI smoke 和 release-package 验证；只处理这些入口实际报告的阻塞问题。
- 将 Cargo workspace 产品版本设置为首个 Beta `0.1.0-beta.1`，更新 README 与版本化 release notes，使其准确描述当前可交付状态和获取方式。
- 复用现有 Linux/Windows canonical package build、verify 和 smoke，再从已验证 binary 派生 target-qualified public binary 与 checksum。
- 由干净 CI 核对 version、target、commit、producer 和 hash 后创建 GitHub prerelease，并记录最终发布证据。

## Success Criteria

- 当前 clean commit 通过既有完整 workspace verification，没有未解决的发布阻塞。
- Linux 与 Windows canonical package 均由现有脚本生成并通过现有 verify/smoke。
- Public binary 与对应 canonical package binary 逐字节相同，checksum 可复核。
- Prerelease tag、Cargo workspace version、manifest commit 和公开 assets 来自同一发布候选。
- 最终实现 diff 只包含收尾、版本、文档、package/release automation，以及由明确失败证据要求的最小修复。

## Scope Boundary

- 当前实现按现状进入 release baseline；发布收尾本身不引入运行时行为变更。
- 尚未进入当前实现的计划性工作不构成本次发布前置条件。
- 只有既有验证实际报告的失败可以触发代码或测试修复。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `release-artifacts`: 在现有 canonical package 契约上增加 Beta release baseline、target-qualified public files、prerelease promotion 和发布证据要求。

## Impact

- 影响 Cargo workspace version、`Cargo.lock`、`README.md`、release owner 文档和版本化 release notes。
- 影响 release-package scripts/tests 与 `.github/workflows/release-package.yml` 的 public-file staging 和 prerelease promotion。
- 既有代码、文档或测试只在现有验证报告真实发布阻塞时进入修改范围；每项修复必须绑定原 owner contract 和失败证据。
